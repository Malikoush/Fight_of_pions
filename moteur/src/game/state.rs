use crate::{
    Action, Board,
    Cell::{self, Pion},
    Event, GameConfig, GameError, Player, Position, rules,
};

/// L'état complet d'une partie, et le seul point d'entrée des actions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    config: GameConfig,
    board: Board,
    current_player: Player,
    turn: u32,
    actions_left: u32,
    reserve_pions: [u32; 2],
    banked_actions: [u32; 2],
    winner: Option<Player>,
}

impl Game {
    /// Les réglages de la partie.
    pub fn config(&self) -> &GameConfig {
        &self.config
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

    /// Numéro du tour en cours, à partir de 1 ; chaque joueur qui joue compte pour un tour.
    pub fn turn(&self) -> u32 {
        self.turn
    }

    /// Le vainqueur, si la partie est finie.
    pub fn winner(&self) -> Option<Player> {
        self.winner
    }

    /// Une nouvelle partie. Erreur si la configuration est incohérente.
    pub fn new(config: GameConfig) -> Result<Game, GameError> {
        config.validate()?;
        let pawns = config.pawns_per_player;

        Ok(Self {
            config,
            board: Board::new(config.board_size),
            current_player: Player::One,
            turn: 1,
            actions_left: config.first_turn_actions,
            reserve_pions: [pawns, pawns],
            banked_actions: [0, 0],
            winner: None,
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
        if self.winner.is_some() {
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
        match self.board.get(pos) {
            None => Err(GameError::OutOfBounds(pos)),
            Some(Cell::Empty) => Ok(()),
            Some(_) => Err(GameError::CellOccupied(pos)),
        }
    }

    /// Exécute une action déjà validée. Ne peut pas échouer.
    fn execute(&mut self, action: Action) -> Vec<Event> {
        match action {
            Action::Place(pos) => self.execute_place(pos),
            Action::EndTurn => {
                self.end_turn();
                vec![Event::TurnEnded {
                    next: self.current_player,
                }]
            }
        }
    }

    fn execute_place(&mut self, pos: Position) -> Vec<Event> {
        let player = self.current_player;
        // La position a été validée. En debug, un écart futur entre `validate` et `execute`
        // est signalé tout de suite ; en release, le plateau est modifié en premier, donc
        // si `set` refusait quand même, rien d'autre n'aurait encore changé.
        debug_assert!(self.board.contains(pos), "pose validée hors du plateau");
        if self.board.set(pos, Pion(player)).is_err() {
            return Vec::new();
        }
        // Les soustractions ne peuvent pas passer sous zéro : `validate` a vérifié les deux.
        self.actions_left = self.actions_left.saturating_sub(self.config.place_cost);
        let reserve = &mut self.reserve_pions[player.index()];
        *reserve = reserve.saturating_sub(1);

        let mut events = vec![Event::PawnPlaced { player, at: pos }];
        if rules::complete_line(&self.board, pos, self.config.align_to_win) {
            self.winner = Some(player);
            events.push(Event::GameWon { winner: player });
        } else if self.actions_left == 0 {
            self.end_turn();
            events.push(Event::TurnEnded {
                next: self.current_player,
            });
        }
        events
    }

    fn end_turn(&mut self) {
        let i = self.current_player.index();
        self.banked_actions[i] = self.actions_left.min(self.config.max_banked_actions);
        self.current_player = self.current_player.opponent();
        self.turn = self.turn.saturating_add(1);
        self.start_turn();
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
}
