//! L'horloge de la partie : ce qui dure après une action (effets continus, infirmerie),
//! quand cela commence et quand cela s'arrête. Une seule règle pour tout : une durée de N
//! tours se compte en tours d'un joueur (l'adversaire de celui qui a joué l'effet pour un
//! effet, le propriétaire du pion pour l'infirmerie), le tour en cours ne compte jamais,
//! et elle s'éteint à la fin de son dernier tour (carnet, 2026-10-30).

use super::lasting::{
    ActiveEffect, CardPlay, EffectEnd, EffectId, EffectKind, Patient, PlayId, cancellable_plays_on,
    pion_has,
};
use super::state::Game;
use crate::attacks::both_sides;
use crate::{Cell, Direction, Event, Pion, PionId, Player, Position};

impl Game {
    /// Les effets continus en jeu, dans l'ordre de leur création. Chaque changement est
    /// annoncé par [`Event::EffectStarted`] et [`Event::EffectEnded`].
    pub fn active_effects(&self) -> &[ActiveEffect] {
        &self.effects
    }

    /// Vrai si le pion de cette case a un effet de cette sorte (immobilisé, protégé…). Pour
    /// un pacte, la sorte comprend l'ami exact : voir plutôt [`Game::pact_friend`].
    pub fn has_effect(&self, pos: Position, kind: EffectKind) -> bool {
        pion_has(&self.board, &self.effects, pos, kind)
    }

    /// L'ami du pion de cette case, s'il est lié par un pacte (Meilleurs amis).
    pub fn pact_friend(&self, pos: Position) -> Option<PionId> {
        let pion = self.board.get(pos).and_then(Cell::pion)?;
        self.effects
            .iter()
            .find_map(|effect| effect.friend_of(pion.id()))
    }

    /// Les pions des deux joueurs à l'infirmerie, dans l'ordre de leur arrivée. Chaque
    /// changement est annoncé par [`Event::InfirmaryEntered`] et [`Event::InfirmaryLeft`].
    pub fn infirmary(&self) -> &[Patient] {
        &self.infirmary
    }

    /// Le dernier tour d'une durée de `turns` tours (au moins 1) comptée en tours de
    /// `counted_on`, à partir de son prochain tour : le tour en cours ne compte jamais.
    fn last_turn_of(&self, counted_on: Player, turns: u32) -> u32 {
        let first = if counted_on == self.current_player {
            2
        } else {
            1
        };
        self.turn
            .saturating_add(first)
            .saturating_add(turns.saturating_sub(1).saturating_mul(2))
    }

    /// Donne au pion de cette case un effet de `turns` tours, créé par le joueur courant
    /// avec cette carte. Il se compte en tours de l'adversaire du joueur courant.
    pub(super) fn start_effect(
        &mut self,
        at: Position,
        kind: EffectKind,
        turns: u32,
        play: CardPlay,
        events: &mut Vec<Event>,
    ) {
        let Some(pion) = self.board.get(at).and_then(Cell::pion) else {
            return;
        };
        let last_turn = self.last_turn_of(self.current_player.opponent(), turns);
        self.add_effect(pion.id(), kind, play, EffectEnd::Turn(last_turn), events);
    }

    /// Lie les pions de ces deux cases par un pacte (Meilleurs amis) : chacun porte le sien,
    /// sans durée.
    pub(super) fn bind(
        &mut self,
        a: Position,
        b: Position,
        play: CardPlay,
        events: &mut Vec<Event>,
    ) {
        let pions = (
            self.board.get(a).and_then(Cell::pion),
            self.board.get(b).and_then(Cell::pion),
        );
        let (Some(first), Some(second)) = pions else {
            return;
        };
        let pact = |friend: Pion| EffectKind::Pact {
            friend: friend.id(),
        };
        self.add_effect(first.id(), pact(second), play, EffectEnd::Never, events);
        self.add_effect(second.id(), pact(first), play, EffectEnd::Never, events);
    }

    /// Couronne le pion de cette case (il devient une reine) et lui donne ces gardes, sans
    /// durée : ils s'arrêtent quand la reine meurt, ou un garde quand il sert.
    pub(super) fn crown(
        &mut self,
        at: Position,
        guards: u32,
        play: CardPlay,
        events: &mut Vec<Event>,
    ) {
        let Some(queen) = self.board.get(at).and_then(Cell::pion) else {
            return;
        };
        self.add_effect(
            queen.id(),
            EffectKind::Crown,
            play,
            EffectEnd::Never,
            events,
        );
        for _ in 0..guards {
            self.add_effect(
                queen.id(),
                EffectKind::Guard,
                play,
                EffectEnd::Never,
                events,
            );
        }
    }

    /// Le pion de `at` devient un enfant (Familia), encadré par les pions de `framed_by` :
    /// il est protégé, sans durée, tant qu'ils l'encadrent (voir
    /// [`Game::end_unframed_children`]).
    pub(super) fn make_child(
        &mut self,
        at: Position,
        framed_by: [Position; 2],
        play: CardPlay,
        events: &mut Vec<Event>,
    ) {
        let pion_at = |pos: Position| self.board.get(pos).and_then(Cell::pion).map(Pion::id);
        let (Some(child), Some(first), Some(second)) =
            (pion_at(at), pion_at(framed_by[0]), pion_at(framed_by[1]))
        else {
            return;
        };
        let end = EffectEnd::Framed([first, second]);
        self.add_effect(child, EffectKind::Protected, play, end, events);
    }

    /// La protection d'un enfant tombe pour de bon dès que les deux membres de sa famille ne
    /// l'encadrent plus de part et d'autre, en ligne droite : l'un d'eux est tué, déplacé,
    /// échangé, ou l'enfant lui-même a bougé (carnet, 2026-11-02). Vérifié après chaque
    /// action.
    pub(super) fn end_unframed_children(&mut self, events: &mut Vec<Event>) {
        let broken: Vec<EffectId> = self
            .effects
            .iter()
            .filter(|effect| {
                effect.framed_by().is_some_and(|framed_by| {
                    !self.is_framed(effect.pion(), framed_by, effect.creator())
                })
            })
            .map(|effect| effect.id())
            .collect();
        if !broken.is_empty() {
            self.end_effects(|effect| broken.contains(&effect.id()), events);
        }
    }

    /// Vrai si les pions de `framed_by`, à ce joueur comme `child`, l'encadrent de part et
    /// d'autre en ligne droite.
    fn is_framed(&self, child: PionId, framed_by: [PionId; 2], owner: Player) -> bool {
        let ours = |pion: PionId| {
            self.board
                .position_of(pion)
                .filter(|&pos| self.board.owner(pos) == Some(owner))
        };
        let (Some(at), Some(first), Some(second)) =
            (ours(child), ours(framed_by[0]), ours(framed_by[1]))
        else {
            return false;
        };
        Direction::ALL
            .iter()
            .any(|&dir| both_sides(&self.board, at, dir) == Some((first, second)))
    }

    /// Le nombre de gardes de ce pion.
    pub fn guards_of(&self, pion: PionId) -> usize {
        self.effects
            .iter()
            .filter(|effect| effect.pion() == pion && effect.kind() == EffectKind::Guard)
            .count()
    }

    /// Dépense le plus ancien garde de ce pion. Faux s'il n'en a pas.
    pub(super) fn use_guard(&mut self, pion: PionId, events: &mut Vec<Event>) -> bool {
        let Some(index) = self
            .effects
            .iter()
            .position(|effect| effect.pion() == pion && effect.kind() == EffectKind::Guard)
        else {
            return false;
        };
        let guard = self.effects.remove(index);
        events.push(Event::EffectEnded(guard));
        true
    }

    /// Annule en entier les effets de l'adversaire qui visent le pion de cette case (Attila
    /// Rare et Épique) : pour chaque jeu de carte adverse qui a laissé un effet annulable
    /// sur lui, tous les effets annulables de ce jeu s'arrêtent, sur tous les pions.
    pub(super) fn cancel_enemy_effects(&mut self, at: Position, events: &mut Vec<Event>) {
        let Some(pion) = self.board.get(at).and_then(Cell::pion) else {
            return;
        };
        let enemy = self.current_player.opponent();
        let plays: Vec<PlayId> = cancellable_plays_on(&self.effects, pion.id(), enemy);
        self.end_effects(
            |effect| effect.kind().is_cancellable() && plays.contains(&effect.play()),
            events,
        );
    }

    /// Crée un effet continu du joueur courant sur le pion de cette case, qui finit ainsi,
    /// et l'annonce (retour d'Interchange, jetons du Lièvre).
    pub(super) fn add_effect_at(
        &mut self,
        at: Position,
        kind: EffectKind,
        play: CardPlay,
        end: EffectEnd,
        events: &mut Vec<Event>,
    ) {
        if let Some(pion) = self.board.get(at).and_then(Cell::pion) {
            self.add_effect(pion.id(), kind, play, end, events);
        }
    }

    /// Crée un effet continu du joueur courant sur ce pion, et l'annonce.
    fn add_effect(
        &mut self,
        pion: PionId,
        kind: EffectKind,
        play: CardPlay,
        end: EffectEnd,
        events: &mut Vec<Event>,
    ) {
        let effect = ActiveEffect::new(
            self.next_effect_id,
            kind,
            pion,
            self.current_player,
            play,
            end,
        );
        self.next_effect_id = self.next_effect_id.next();
        self.effects.push(effect);
        events.push(Event::EffectStarted(effect));
    }

    /// Arrête les effets qui remplissent cette condition, en annonçant chacun.
    pub(super) fn end_effects(
        &mut self,
        ends: impl Fn(&ActiveEffect) -> bool,
        events: &mut Vec<Event>,
    ) {
        let (ended, kept): (Vec<ActiveEffect>, Vec<ActiveEffect>) =
            std::mem::take(&mut self.effects)
                .into_iter()
                .partition(|effect| ends(effect));
        self.effects = kept;
        events.extend(ended.into_iter().map(Event::EffectEnded));
    }

    /// Les effets visant ce pion s'arrêtent : il vient de quitter le plateau.
    pub(super) fn end_effects_on(&mut self, pion: PionId, events: &mut Vec<Event>) {
        self.end_effects(|effect| effect.pion() == pion, events);
    }

    /// Un pion tué va à l'infirmerie pour [`crate::GameConfig::infirmary_turns`] tours de
    /// son propriétaire (carnet, 2026-10-30), ou directement dans la réserve si cette durée
    /// est 0. Un pion du joueur courant (tué par une vengeance, par exemple) ne compte pas
    /// le tour en cours.
    ///
    /// À l'infirmerie, il laisse sa tombe sur la case `at` où il est mort ; elle remplace
    /// la tombe qui s'y trouvait (une case porte au plus une tombe, carnet, 2026-10-30).
    pub(super) fn send_to_infirmary(&mut self, pion: Pion, at: Position, events: &mut Vec<Event>) {
        let turns = self.config.infirmary_turns;
        if turns == 0 {
            self.return_to_reserve(pion.owner(), events);
            return;
        }
        let last_turn = self.last_turn_of(pion.owner(), turns);
        let patient = Patient::new(pion, last_turn);
        self.infirmary.push(patient);
        events.push(Event::InfirmaryEntered(patient));
        if let Some(replaced) = self.board.set_tomb(at, Some(pion)) {
            events.push(Event::TombRemoved { pion: replaced, at });
        }
        events.push(Event::TombAdded { pion, at });
    }

    /// Ce pion sort de l'infirmerie (vers la réserve ou, ressuscité, vers le plateau) : sa
    /// tombe s'efface, s'il en a encore une. Faux s'il n'y était pas.
    pub(super) fn leave_infirmary(&mut self, pion: PionId, events: &mut Vec<Event>) -> bool {
        let index = self
            .infirmary
            .iter()
            .position(|patient| patient.pion().id() == pion);
        let Some(index) = index else {
            return false;
        };
        let patient = self.infirmary.remove(index);
        events.push(Event::InfirmaryLeft(patient));
        if let Some(at) = self.board.tomb_of(pion) {
            self.board.set_tomb(at, None);
            events.push(Event::TombRemoved {
                pion: patient.pion(),
                at,
            });
        }
        true
    }

    fn return_to_reserve(&mut self, owner: Player, events: &mut Vec<Event>) {
        let reserve = &mut self.reserve_pions[owner.index()];
        *reserve = reserve.saturating_add(1);
        events.push(self.reserve_changed(owner));
    }

    /// À la fin du tour en cours : les effets dont c'était le dernier tour s'arrêtent, et
    /// les pions dont c'était le dernier tour à l'infirmerie reviennent dans la réserve.
    /// Vrai si quelque chose a changé.
    pub(super) fn end_what_expires(&mut self, events: &mut Vec<Event>) -> bool {
        let turn = self.turn;
        let before = events.len();
        self.end_effects(
            |effect| effect.last_turn().is_some_and(|last| last <= turn),
            events,
        );
        let leaving: Vec<Pion> = self
            .infirmary
            .iter()
            .filter(|patient| patient.last_turn() <= turn)
            .map(|patient| patient.pion())
            .collect();
        for pion in leaving {
            if self.leave_infirmary(pion.id(), events) {
                self.return_to_reserve(pion.owner(), events);
            }
        }
        events.len() > before
    }

    /// Vrai si quelque chose est programmé pour changer la partie plus tard : un effet
    /// continu qui s'arrêtera à un tour donné, un retour d'Interchange, un pion qui sortira
    /// de l'infirmerie. Un effet sans durée (le pacte) ne change rien tout seul : il ne
    /// compte pas, ni des jetons de pas, qui attendent leur joueur.
    pub(super) fn something_is_scheduled(&self) -> bool {
        self.effects.iter().any(|effect| effect.is_scheduled()) || !self.infirmary.is_empty()
    }
}
