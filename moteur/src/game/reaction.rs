//! Les réactions (B8, carnet « Réactions ») : des cartes de l'arsenal jouées pendant le
//! tour adverse, juste après leur déclencheur, et payées avec la banque.
//!
//! Après chaque coup du joueur courant, le moteur regarde ce que le coup a fait (ses
//! événements : poses, morts, déplacements, attaque). S'il déclenche une réaction que
//! l'autre joueur peut jouer maintenant (prête, payable, avec une cible), une **fenêtre**
//! s'ouvre : l'autre joueur réagit ou passe. Sinon, rien ne s'arrête. Une seule fenêtre
//! s'ouvre **avant** un coup : quand une attaque, ses choix faits, vise un de ses pions
//! (Feintaille) ; l'attaque attend alors sa réponse pour s'appliquer.
//!
//! Une réaction se choisit et se résout comme une attaque (`attack_flow.rs`), par le joueur
//! qui réagit ; on ne réagit pas à une réaction, et chacun en joue au plus une par tour
//! adverse.

use super::attack_flow::Resolution;
use super::state::Game;
use crate::attacks::{Launch, Trigger};
use crate::{Attack, Cell, Event, GameError, Player, Position};

/// Une fenêtre de réaction ouverte : ce que le dernier coup du joueur courant a déclenché.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(super) struct ReactionWindow {
    /// Les cases des pions que le joueur courant vient de poser, et qui y sont encore : une
    /// Sentinelle répond à l'un d'eux.
    placed: Vec<Position>,
    /// Les cases où des pions de l'autre joueur viennent de mourir : une Queue du lézard
    /// répond à l'un d'eux.
    deaths: Vec<Position>,
    /// Vrai si le coup a déplacé au moins un pion, de l'un ou l'autre joueur.
    moved: bool,
    /// L'attaque que le coup a jouée, s'il en a joué une : le Perroquet la rejoue.
    attack: Option<Attack>,
    /// Les cases des pions de l'autre joueur que vise l'attaque sur le point de
    /// s'appliquer : une Feintaille en fait esquiver un.
    targeted: Vec<Position>,
    /// L'attaque sur le point de s'appliquer, qui les vise.
    threat: Option<Attack>,
}

impl ReactionWindow {
    /// Vrai si le coup a eu lieu de ce qui déclenche cette réaction.
    fn has(&self, trigger: Trigger) -> bool {
        match trigger {
            Trigger::Placement => !self.placed.is_empty(),
            Trigger::Death => !self.deaths.is_empty(),
            Trigger::Move => self.moved,
            Trigger::Attack => self.attack.is_some(),
            Trigger::Targeted => !self.targeted.is_empty(),
        }
    }

    /// L'attaque adverse à laquelle répond une réaction déclenchée ainsi, s'il y en a une :
    /// sa limite de coût se lit sur elle.
    fn answered_attack(&self, trigger: Trigger) -> Option<Attack> {
        match trigger {
            Trigger::Attack => self.attack,
            Trigger::Targeted => self.threat,
            Trigger::Placement | Trigger::Death | Trigger::Move => None,
        }
    }

    /// Les cases d'où part une réaction qui part de son déclencheur (Sentinelle, Queue du
    /// lézard, Feintaille) ; `None` pour une réaction qui part d'un pion du joueur.
    fn trigger_cells(&self, trigger: Trigger) -> Option<&[Position]> {
        match trigger {
            Trigger::Placement => Some(&self.placed),
            Trigger::Death => Some(&self.deaths),
            Trigger::Targeted => Some(&self.targeted),
            Trigger::Move | Trigger::Attack => None,
        }
    }
}

impl Game {
    /// Le joueur qui décide maintenant : celui qui réagit pendant une fenêtre de réaction,
    /// sinon le joueur courant. C'est lui qui joue la carte en cours, la paie, et crée ses
    /// effets.
    pub(super) fn acting_player(&self) -> Player {
        if self.reaction_window.is_some() {
            self.current_player.opponent()
        } else {
            self.current_player
        }
    }

    /// Ouvre une fenêtre de réaction si le coup qui vient d'être joué (ses `events`)
    /// déclenche une réaction que l'autre joueur peut jouer maintenant. Vrai si elle est
    /// ouverte : la suite du tour attend sa réponse.
    pub(super) fn open_reaction_window(&mut self, events: &[Event]) -> bool {
        let current = self.current_player;
        let mut window = ReactionWindow::default();
        for event in events {
            match *event {
                Event::PionPlaced { pion, at }
                    if pion.owner() == current
                        && self.board.get(at).and_then(Cell::pion) == Some(pion) =>
                {
                    window.placed.push(at);
                }
                Event::PionKilled { pion, at, .. }
                    if pion.owner() != current && !window.deaths.contains(&at) =>
                {
                    window.deaths.push(at);
                }
                Event::PionMoved { .. } | Event::PionsSwapped { .. } => window.moved = true,
                Event::AttackUsed { player, attack, .. } if player == current => {
                    window.attack = Some(attack);
                }
                _ => {}
            }
        }
        self.open_if_playable(window)
    }

    /// Ouvre une fenêtre avant que cette attaque du joueur courant ne s'applique, si elle
    /// vise (`targets` : ses cases choisies) un pion de l'autre joueur et qu'il peut
    /// l'esquiver. Vrai si elle est ouverte : l'attaque attend sa réponse.
    pub(super) fn open_targeted_window(
        &mut self,
        resolution: &Resolution,
        targets: &[Position],
    ) -> bool {
        let reactor = self.current_player.opponent();
        let mut targeted: Vec<Position> = targets
            .iter()
            .copied()
            .filter(|&pos| self.board.owner(pos) == Some(reactor))
            .collect();
        targeted.dedup();
        self.open_if_playable(ReactionWindow {
            targeted,
            threat: Some(resolution.card),
            ..ReactionWindow::default()
        })
    }

    /// Ouvre cette fenêtre si elle déclenche quelque chose et qu'une réaction y est
    /// vraiment jouable : la même validation que celle de l'action. Vrai si elle est
    /// ouverte.
    fn open_if_playable(&mut self, window: ReactionWindow) -> bool {
        let triggers = [
            Trigger::Placement,
            Trigger::Death,
            Trigger::Move,
            Trigger::Attack,
            Trigger::Targeted,
        ];
        if self.reacted_this_turn || !triggers.into_iter().any(|trigger| window.has(trigger)) {
            return false;
        }
        self.reaction_window = Some(window);
        let reactor = self.acting_player();
        let can_react =
            (0..self.arsenal(reactor).len()).any(|slot| self.validate_use_attack(slot).is_ok());
        if !can_react {
            self.reaction_window = None;
        }
        can_react
    }

    /// Une attaque pendant son tour, une réaction déclenchée pendant le tour adverse, et
    /// face à une attaque qui ne coûte pas plus que sa limite (Feintaille Commune,
    /// Perroquet).
    pub(super) fn ensure_right_moment(&self, card: Attack) -> Result<(), GameError> {
        let rules = card.kind.rules();
        let (window, trigger) = match (&self.reaction_window, rules.trigger()) {
            (None, None) => return Ok(()),
            (Some(_), None) => return Err(GameError::NotAReaction(card)),
            (None, Some(_)) => return Err(GameError::NotTriggered(card)),
            (Some(window), Some(trigger)) => (window, trigger),
        };
        let within_limit = match (
            window.answered_attack(trigger),
            rules.attack_cost_limit(card.rarity),
        ) {
            (Some(attack), Some(limit)) => self.attack_stats(attack).cost <= limit,
            _ => true,
        };
        if window.has(trigger) && within_limit {
            Ok(())
        } else {
            Err(GameError::NotTriggered(card))
        }
    }

    /// Vrai si la réaction part de son déclencheur (le pion posé pour une Sentinelle, la
    /// case du pion mort pour une Queue du lézard, ton pion visé pour une Feintaille) et que
    /// cette case en est un ; ton pion visé doit en plus pouvoir agir `when`. `None` si
    /// elle part d'un pion du joueur, comme une attaque.
    pub(super) fn is_trigger_launcher(
        &self,
        player: Player,
        attack: Attack,
        pos: Position,
        when: Launch,
    ) -> Option<bool> {
        let trigger = attack.kind.rules().trigger()?;
        let window = self.reaction_window.as_ref()?;
        let cells = window.trigger_cells(trigger)?;
        Some(
            cells.contains(&pos)
                && (trigger != Trigger::Targeted || self.can_launch(player, pos, when)),
        )
    }

    /// L'attaque que cette carte fait jouer : elle-même, ou pour un Perroquet l'attaque
    /// adverse qu'il rejoue. Erreur si le Perroquet n'a rien à rejouer.
    pub(super) fn played_attack(&self, card: Attack) -> Result<Attack, GameError> {
        if card.kind.rules().trigger() != Some(Trigger::Attack) {
            return Ok(card);
        }
        self.reaction_window
            .as_ref()
            .and_then(|window| window.attack)
            .ok_or(GameError::NotTriggered(card))
    }

    /// Passer : la fenêtre se ferme, et le tour adverse reprend où il en était (l'attaque
    /// qui attendait s'applique).
    pub(super) fn execute_pass(&mut self, events: &mut Vec<Event>) {
        self.reaction_window = None;
        self.resume_turn(events);
    }

    /// Ce qui suit une réaction jouée par `reactor` : la fenêtre se ferme, il ne réagira
    /// plus ce tour ; une ligne que la réaction lui a donnée (`had_a_line` : il n'en avait
    /// pas avant) gagnera un tour plus tard. Puis, comme après toute action, la partie peut
    /// finir, et le tour adverse reprend.
    pub(super) fn after_reaction(
        &mut self,
        reactor: Player,
        had_a_line: bool,
        events: &mut Vec<Event>,
    ) {
        self.reaction_window = None;
        self.reacted_this_turn = true;
        if !had_a_line && self.has_winning_line(reactor) {
            self.late_lines[reactor.index()] = true;
        }
        if self.settle_board(events) {
            self.paused_attack = None;
            return;
        }
        self.resume_turn(events);
    }

    /// Le tour adverse reprend après une fenêtre : l'attaque qui attendait s'applique,
    /// sinon le tour finit s'il n'y a plus rien à faire.
    fn resume_turn(&mut self, events: &mut Vec<Event>) {
        match self.paused_attack.take() {
            Some(resolution) => self.apply_resolution(resolution, events),
            None => self.end_turn_if_done(events),
        }
    }
}
