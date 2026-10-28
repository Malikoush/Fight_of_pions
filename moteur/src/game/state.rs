use super::arsenal::ArsenalSlot;
use super::attack_flow::Launch;
use crate::{
    Action, Attack, Board, Cell, Decision, EndReason, Event, GameConfig, GameEnd, GameError,
    Outcome, Pion, PionId, Player, Position, rules,
};

/// L'état complet d'une partie, et le seul point d'entrée des actions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    pub(super) config: GameConfig,
    /// La graine de la partie. Rien n'est encore tiré au hasard ; le draft s'en servira.
    pub(super) seed: u64,
    pub(super) board: Board,
    pub(super) current_player: Player,
    pub(super) turn: u32,
    pub(super) actions_left: u32,
    /// Les pions posés par le joueur courant depuis le début de son tour.
    pub(super) placements_this_turn: u32,
    /// L'identité du premier pion créé ce tour : les identités suivant l'ordre des poses,
    /// tout pion d'identité supérieure ou égale est arrivé ce tour.
    pub(super) turn_start_pion_id: PionId,
    pub(super) reserve_pions: [u32; 2],
    pub(super) banked_actions: [u32; 2],
    /// Les attaques de chaque joueur et leur recharge.
    pub(super) arsenals: [Vec<ArsenalSlot>; 2],
    /// L'attaque en cours de choix (lanceur, cibles), s'il y en a une.
    pub(super) pending_attack: Option<PendingAttack>,
    /// `Some` dès que la partie est finie.
    pub(super) end: Option<GameEnd>,
    /// L'identité que recevra le prochain pion créé.
    pub(super) next_pion_id: PionId,
}

/// Une attaque commencée dont les choix ne sont pas finis. Rien n'est modifié tant que le
/// dernier choix n'est pas fait.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct PendingAttack {
    /// L'emplacement de l'attaque dans l'arsenal du joueur courant.
    pub(super) slot: usize,
    /// L'attaque elle-même, retenue au début : rien ne la relit ensuite dans l'arsenal.
    pub(super) attack: Attack,
    /// La case du pion lanceur, une fois choisie.
    pub(super) launcher: Option<Position>,
    /// Les cibles déjà choisies.
    pub(super) targets: Vec<Position>,
}

impl Game {
    /// Les réglages de la partie.
    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    /// La graine de la partie : avec la configuration et les actions jouées, elle suffit
    /// à reconstruire la partie (voir [`crate::GameRecord`]).
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Actions qu'il reste au joueur courant pour ce tour.
    pub fn actions_left(&self) -> u32 {
        self.actions_left
    }

    /// Combien de pions le joueur courant peut encore poser ce tour : autant que ses
    /// actions en paient, que ses poses permises par tour, que sa réserve et que les cases
    /// libres. 0 si la partie est finie.
    pub fn placements_left(&self) -> u32 {
        if self.end.is_some() {
            return 0;
        }
        self.actions_left
            .checked_div(self.config.place_cost)
            .unwrap_or(u32::MAX) // pose gratuite : seules les autres limites comptent
            .min(self.placement_limit_left())
            .min(self.reserve_pions[self.current_player.index()])
            .min(self.free_cells())
    }

    /// Le nombre de cases libres du plateau.
    fn free_cells(&self) -> u32 {
        let free = self
            .board
            .positions()
            .filter(|&pos| self.ensure_empty(pos).is_ok())
            .count();
        u32::try_from(free).unwrap_or(u32::MAX)
    }

    /// Les poses que la limite par tour permet encore ; sans limite, autant qu'on veut.
    fn placement_limit_left(&self) -> u32 {
        self.config.max_placements_per_turn.map_or(u32::MAX, |max| {
            max.saturating_sub(self.placements_this_turn)
        })
    }

    /// Vrai si le pion de cette case ne peut pas encore lancer d'attaque à cause du mal du
    /// débarquement ([`GameConfig::summoning_sickness`]) : il a été posé ce tour.
    pub fn is_summoning_sick(&self, pos: Position) -> bool {
        self.config.summoning_sickness && self.arrived_this_turn(pos)
    }

    /// Vrai si le pion de cette case a été posé ce tour.
    fn arrived_this_turn(&self, pos: Position) -> bool {
        self.board
            .get(pos)
            .and_then(Cell::pion)
            .is_some_and(|pion| pion.id() >= self.turn_start_pion_id)
    }

    /// Le joueur qui doit jouer.
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    /// Le plateau, en lecture seule.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Pions qu'un joueur peut encore poser.
    pub fn reserve_pions(&self, player: Player) -> u32 {
        self.reserve_pions[player.index()]
    }

    /// Actions qu'un joueur a gardées de son dernier tour : elles s'ajouteront à son
    /// prochain tour. Toujours 0 pour le joueur courant, qui les a déjà reçues.
    pub fn banked_actions(&self, player: Player) -> u32 {
        self.banked_actions[player.index()]
    }

    /// L'arsenal d'un joueur : ses attaques, dans l'ordre de leurs emplacements.
    pub fn arsenal(&self, player: Player) -> &[ArsenalSlot] {
        &self.arsenals[player.index()]
    }

    /// Numéro du tour en cours, à partir de 1 ; chaque joueur qui joue compte pour un tour.
    pub fn turn(&self) -> u32 {
        self.turn
    }

    /// La fin de la partie (résultat et raison), si elle est finie.
    pub fn end(&self) -> Option<GameEnd> {
        self.end
    }

    /// Le vainqueur, si la partie est finie par une victoire.
    pub fn winner(&self) -> Option<Player> {
        self.end.and_then(GameEnd::winner)
    }

    /// La décision attendue maintenant, ou `None` si la partie est finie.
    pub fn pending_decision(&self) -> Option<Decision> {
        if self.end.is_some() {
            return None;
        }
        let player = self.current_player;
        Some(match &self.pending_attack {
            None => Decision::MainAction { player },
            Some(pending) => {
                let attack = pending.attack;
                match pending.launcher {
                    None => Decision::ChooseLauncher { player, attack },
                    Some(launcher) => Decision::ChooseTarget {
                        player,
                        attack,
                        launcher,
                    },
                }
            }
        })
    }

    /// Toutes les actions que [`Game::apply`] accepterait maintenant, dans un ordre
    /// toujours identique : les poses rangée par rangée (A1, B1… puis A2…), les attaques
    /// dans l'ordre de l'arsenal, les choix de case rangée par rangée, puis `EndTurn`.
    /// Vide si la partie est finie.
    ///
    /// La liste est construite en passant chaque action candidate par la validation
    /// elle-même : elle ne peut pas contredire `apply`. Les candidates sont seulement
    /// celles qui répondent à la décision attendue, pour ne pas tester pour rien des
    /// actions forcément refusées (c'est appelé à chaque coup par les bots).
    pub fn legal_actions(&self) -> Vec<Action> {
        if self.end.is_some() {
            return Vec::new();
        }
        let candidates: Vec<Action> = match self.pending_attack {
            None => {
                let slots = 0..self.arsenal(self.current_player).len();
                self.board
                    .positions()
                    .map(Action::Place)
                    .chain(slots.map(Action::UseAttack))
                    .chain([Action::EndTurn])
                    .collect()
            }
            Some(_) => self.board.positions().map(Action::Choose).collect(),
        };
        candidates
            .into_iter()
            .filter(|&action| self.validate(action).is_ok())
            .collect()
    }

    /// Une nouvelle partie avec cette graine. Erreur si la configuration est incohérente.
    ///
    /// Même configuration, même graine et mêmes actions donnent toujours la même partie.
    pub fn new(config: GameConfig, seed: u64) -> Result<Game, GameError> {
        config.validate()?;
        let pions = config.pions_per_player;
        let [first_bank, second_bank] = config.starting_banked_actions;
        let arsenal_of = |player: Player| -> Vec<ArsenalSlot> {
            config.starting_attacks[player.index()]
                .iter()
                .map(|&attack| ArsenalSlot::new(attack))
                .collect()
        };

        Ok(Self {
            seed,
            board: Board::new(config.board_size),
            current_player: Player::One,
            turn: 1,
            // Le premier joueur reçoit tout de suite sa banque de départ, comme on reçoit
            // ses actions gardées au début d'un tour.
            actions_left: config.first_turn_actions.saturating_add(first_bank),
            placements_this_turn: 0,
            turn_start_pion_id: PionId::FIRST,
            reserve_pions: [pions, pions],
            banked_actions: [0, second_bank],
            arsenals: [arsenal_of(Player::One), arsenal_of(Player::Two)],
            pending_attack: None,
            end: None,
            next_pion_id: PionId::FIRST,
            config,
        })
    }

    /// Applique une action du joueur courant et renvoie les événements produits.
    ///
    /// Une action refusée renvoie une erreur et ne modifie rien.
    pub fn apply(&mut self, action: Action) -> Result<Vec<Event>, GameError> {
        self.validate(action)?; // lit l'état, ne modifie rien
        Ok(self.execute(action)) // modifie l'état, ne vérifie plus
    }

    fn validate(&self, action: Action) -> Result<(), GameError> {
        if self.end.is_some() {
            return Err(GameError::GameOver);
        }
        match (&self.pending_attack, action) {
            (None, Action::Place(pos)) => self.validate_place(pos),
            (None, Action::UseAttack(slot)) => self.validate_use_attack(slot),
            (None, Action::EndTurn) => Ok(()),
            (Some(pending), Action::Choose(pos)) => self.validate_choice(pending, pos),
            _ => Err(GameError::UnexpectedAction),
        }
    }

    /// Vérifie qu'il reste assez d'actions pour payer ce coût.
    pub(super) fn ensure_actions(&self, needed: u32) -> Result<(), GameError> {
        if self.actions_left < needed {
            return Err(GameError::NotEnoughActions {
                needed,
                available: self.actions_left,
            });
        }
        Ok(())
    }

    fn validate_place(&self, pos: Position) -> Result<(), GameError> {
        self.ensure_actions(self.config.place_cost)?;
        if self.placement_limit_left() == 0 {
            return Err(GameError::NoPlacementLeft);
        }
        self.ensure_empty(pos)?;
        if self.reserve_pions[self.current_player.index()] == 0 {
            return Err(GameError::EmptyReserve);
        }
        Ok(())
    }

    fn ensure_empty(&self, pos: Position) -> Result<(), GameError> {
        match self.board.get(pos).map(Cell::pion) {
            None => Err(GameError::OutOfBounds(pos)),
            Some(None) => Ok(()),
            Some(Some(_)) => Err(GameError::CellOccupied(pos)),
        }
    }

    /// Exécute une action déjà validée. Ne peut pas échouer.
    fn execute(&mut self, action: Action) -> Vec<Event> {
        let mut events = Vec::new();
        match action {
            Action::Place(pos) => self.execute_place(pos, &mut events),
            Action::UseAttack(slot) => self.start_attack(slot),
            Action::Choose(pos) => self.execute_choice(pos, &mut events),
            // Finir son tour ne change ni le plateau ni les réserves : la partie ne peut
            // pas devenir figée ici, seule la limite de tours peut l'arrêter.
            Action::EndTurn => self.finish_turn(&mut events),
        }
        events
    }

    fn execute_place(&mut self, pos: Position, events: &mut Vec<Event>) {
        let player = self.current_player;
        let pion = Pion::new(self.next_pion_id, player);
        // La position a été validée. En debug, un écart futur entre `validate` et `execute`
        // est signalé tout de suite ; en release, le plateau est modifié en premier, donc
        // si `set` refusait quand même, rien d'autre n'aurait encore changé.
        debug_assert!(self.board.contains(pos), "pose validée hors du plateau");
        if self.board.set(pos, Cell::with_pion(pion)).is_err() {
            return;
        }
        self.next_pion_id = self.next_pion_id.next();
        self.placements_this_turn = self.placements_this_turn.saturating_add(1);
        // Les soustractions ne peuvent pas passer sous zéro : `validate` a vérifié les deux.
        self.actions_left = self.actions_left.saturating_sub(self.config.place_cost);
        let reserve = &mut self.reserve_pions[player.index()];
        *reserve = reserve.saturating_sub(1);
        events.push(Event::PionPlaced { pion, at: pos });
        events.push(self.reserve_changed(player));
        // Une pose gratuite ne change pas les compteurs d'actions : rien à annoncer.
        if self.config.place_cost > 0 {
            events.push(self.actions_changed(player));
        }

        if rules::complete_line(&self.board, pos, self.config.align_to_win) {
            self.finish_game(Outcome::Won(player), EndReason::Alignment, events);
        } else {
            self.after_action(events);
        }
    }

    /// Ce qui suit toute action qui a modifié le plateau sans aligner : la partie peut se
    /// figer, ou le tour se terminer quand il ne reste ni action ni pose.
    pub(super) fn after_action(&mut self, events: &mut Vec<Event>) {
        if self.is_frozen() {
            self.finish_game(self.outcome_without_alignment(), EndReason::Frozen, events);
        } else if self.actions_left == 0 && self.placements_left() == 0 {
            self.finish_turn(events);
        }
    }

    /// Termine le tour du joueur courant : la partie s'arrête si c'était le dernier tour
    /// permis, sinon l'adversaire commence le sien.
    fn finish_turn(&mut self, events: &mut Vec<Event>) {
        if self.turn >= self.config.max_turns {
            self.finish_game(
                self.outcome_without_alignment(),
                EndReason::TurnLimit,
                events,
            );
            return;
        }
        let ending = self.current_player;
        // Les compteurs du joueur sortant ne changent que s'il lui restait des actions :
        // elles passent alors dans sa réserve. Sinon, ils sont déjà annoncés à (0, 0).
        let ending_counters_change = self.actions_left > 0;
        self.banked_actions[ending.index()] = self.actions_left.min(self.config.max_banked_actions);
        self.current_player = ending.opponent();
        self.turn = self.turn.saturating_add(1);
        if ending_counters_change {
            events.push(self.actions_changed(ending));
        }
        events.push(Event::TurnEnded {
            next: self.current_player,
        });
        self.start_turn(events);
    }

    /// Prépare le tour du nouveau joueur courant : ses actions et ses poses, et la
    /// recharge de ses attaques. Le tout premier tour de la partie est réglé dans
    /// [`Game::new`] : ici, c'est toujours un tour ordinaire.
    fn start_turn(&mut self, events: &mut Vec<Event>) {
        let player = self.current_player;
        let i = player.index();
        self.placements_this_turn = 0;
        self.turn_start_pion_id = self.next_pion_id;
        self.actions_left = self
            .config
            .actions_per_turn
            .saturating_add(self.banked_actions[i]);
        self.banked_actions[i] = 0;
        // Le joueur qui commence reçoit toujours au moins une action : son compteur change.
        events.push(self.actions_changed(player));

        for (slot, emplacement) in self.arsenals[i].iter_mut().enumerate() {
            let left = emplacement.cooldown_left();
            if left > 0 {
                emplacement.set_cooldown(left - 1);
                events.push(Event::CooldownChanged {
                    player,
                    slot,
                    turns_left: left - 1,
                });
            }
        }
    }

    /// L'annonce des compteurs d'actions d'un joueur, tels qu'ils sont maintenant.
    pub(super) fn actions_changed(&self, player: Player) -> Event {
        let actions_left = if player == self.current_player {
            self.actions_left
        } else {
            0
        };
        Event::ActionsChanged {
            player,
            actions_left,
            banked: self.banked_actions[player.index()],
        }
    }

    /// L'annonce de la réserve de pions d'un joueur, telle qu'elle est maintenant.
    pub(super) fn reserve_changed(&self, player: Player) -> Event {
        Event::ReserveChanged {
            player,
            pions: self.reserve_pions[player.index()],
        }
    }

    fn finish_game(&mut self, outcome: Outcome, reason: EndReason, events: &mut Vec<Event>) {
        let end = GameEnd { outcome, reason };
        self.end = Some(end);
        events.push(Event::GameEnded(end));
    }

    /// Le résultat quand la partie s'arrête sans alignement. Pour l'instant toujours le
    /// nul ; un départage (plus longue ligne…) pourra le remplacer ici seulement.
    fn outcome_without_alignment(&self) -> Outcome {
        Outcome::Draw
    }

    /// Vrai si plus aucun joueur ne pourra jamais agir autrement qu'en finissant son tour,
    /// et que rien de programmé ne peut changer la situation (rien ne l'est encore).
    fn is_frozen(&self) -> bool {
        !self.can_ever_act(Player::One) && !self.can_ever_act(Player::Two)
    }

    /// Vrai si ce joueur pourra encore agir, maintenant ou plus tard : poser un pion, ou
    /// lancer une attaque qui a une cible, même si elle est en recharge ou qu'il doit
    /// d'abord économiser ses actions (la configuration garantit que tout coût est
    /// atteignable).
    fn can_ever_act(&self, player: Player) -> bool {
        let can_place = self.reserve_pions[player.index()] > 0 && self.free_cells() > 0;
        can_place
            || self
                .arsenal(player)
                .iter()
                .any(|slot| self.attack_has_a_target(player, slot.attack(), Launch::Someday))
    }
}
