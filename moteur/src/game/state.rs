use crate::{
    Action, Board, Cell, Decision, EndReason, Event, GameConfig, GameEnd, GameError, Outcome, Pion,
    PionId, Player, Position, rules,
};

/// L'état complet d'une partie, et le seul point d'entrée des actions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    config: GameConfig,
    /// La graine de la partie. Rien n'est encore tiré au hasard ; le draft s'en servira.
    seed: u64,
    board: Board,
    current_player: Player,
    turn: u32,
    actions_left: u32,
    reserve_pions: [u32; 2],
    banked_actions: [u32; 2],
    /// `Some` dès que la partie est finie.
    end: Option<GameEnd>,
    /// L'identité que recevra le prochain pion créé.
    next_pion_id: PionId,
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
        match self.end {
            Some(_) => None,
            None => Some(Decision::MainAction {
                player: self.current_player,
            }),
        }
    }

    /// Toutes les actions que [`Game::apply`] accepterait maintenant, dans un ordre
    /// toujours identique : les poses rangée par rangée (A1, B1… puis A2…), puis
    /// `EndTurn` en dernier. Vide si la partie est finie.
    ///
    /// La liste est construite en passant chaque action candidate par la validation
    /// elle-même : elle ne peut pas contredire `apply`.
    pub fn legal_actions(&self) -> Vec<Action> {
        self.board
            .positions()
            .map(Action::Place)
            .chain([Action::EndTurn])
            .filter(|&action| self.validate(action).is_ok())
            .collect()
    }

    /// Une nouvelle partie avec cette graine. Erreur si la configuration est incohérente.
    ///
    /// Même configuration, même graine et mêmes actions donnent toujours la même partie.
    pub fn new(config: GameConfig, seed: u64) -> Result<Game, GameError> {
        config.validate()?;
        let pions = config.pions_per_player;

        Ok(Self {
            config,
            seed,
            board: Board::new(config.board_size),
            current_player: Player::One,
            turn: 1,
            actions_left: config.first_turn_actions,
            reserve_pions: [pions, pions],
            banked_actions: [0, 0],
            end: None,
            next_pion_id: PionId::FIRST,
        })
    }

    /// Applique une action du joueur courant et renvoie les événements produits.
    ///
    /// Une action refusée renvoie une erreur et ne modifie rien.
    pub fn apply(&mut self, action: Action) -> Result<Vec<Event>, GameError> {
        self.validate(action)?; // lit l'état, ne modifie rien
        Ok(self.execute(action)) // modifie l'état, ne vérifie plus
    }

    /// Le nombre d'actions que coûte une décision.
    fn cost(&self, action: Action) -> u32 {
        match action {
            Action::Place(_) => self.config.place_cost,
            Action::EndTurn => 0,
        }
    }

    fn validate(&self, action: Action) -> Result<(), GameError> {
        if self.end.is_some() {
            return Err(GameError::GameOver);
        }
        let needed = self.cost(action);
        if self.actions_left < needed {
            return Err(GameError::NotEnoughActions {
                needed,
                available: self.actions_left,
            });
        }
        match action {
            Action::EndTurn => Ok(()),
            Action::Place(pos) => self.validate_place(pos),
        }
    }

    fn validate_place(&self, pos: Position) -> Result<(), GameError> {
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
        // Les soustractions ne peuvent pas passer sous zéro : `validate` a vérifié les deux.
        self.actions_left = self.actions_left.saturating_sub(self.config.place_cost);
        let reserve = &mut self.reserve_pions[player.index()];
        *reserve = reserve.saturating_sub(1);
        events.push(Event::PionPlaced { pion, at: pos });
        events.push(self.actions_changed(player));

        if rules::complete_line(&self.board, pos, self.config.align_to_win) {
            self.finish_game(Outcome::Won(player), EndReason::Alignment, events);
        } else if self.is_frozen() {
            self.finish_game(self.outcome_without_alignment(), EndReason::Frozen, events);
        } else if self.actions_left == 0 {
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
        self.start_turn();
        if ending_counters_change {
            events.push(self.actions_changed(ending));
        }
        events.push(Event::TurnEnded {
            next: self.current_player,
        });
        // Le joueur qui commence reçoit toujours au moins une action : son compteur change.
        events.push(self.actions_changed(self.current_player));
    }

    /// L'annonce des compteurs d'actions d'un joueur, tels qu'ils sont maintenant.
    fn actions_changed(&self, player: Player) -> Event {
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

    /// Prépare le tour du nouveau joueur courant. Le tout premier tour de la partie est
    /// réglé dans [`Game::new`] : ici, c'est toujours un tour ordinaire.
    fn start_turn(&mut self) {
        let i = self.current_player.index();
        self.actions_left = self
            .config
            .actions_per_turn
            .saturating_add(self.banked_actions[i]);
        self.banked_actions[i] = 0;
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

    /// Vrai si ce joueur pourra encore agir, maintenant ou après avoir économisé ses
    /// actions : la configuration garantit qu'une pose est toujours atteignable.
    fn can_ever_act(&self, player: Player) -> bool {
        let has_pions = self.reserve_pions[player.index()] > 0;
        has_pions
            && self
                .board
                .positions()
                .any(|pos| self.ensure_empty(pos).is_ok())
    }
}
