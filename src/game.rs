
use crate::action::Action;
use crate::board::Cell::{self, Pion};

use crate::rules;
use crate::{
    board::Board, config::GameConfig, error::GameError, player::Player, position::Position,
};

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,                      
    TurnEnded { next: Player }, 
    Victory(Player), 
}

impl Game {
       pub fn actions_left(&self) -> u32 {
        self.actions_left
    }

    pub fn current_player(&self) -> Player {
        self.current_player
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn reserve_pions(&self,player:Player)-> u32 {
        self.reserve_pions[player.index()]
    }

    pub fn turn(&self) -> u32 {
    self.turn
}

pub fn winner(&self) -> Option<Player> {
    self.winner
}

    pub fn new(config: GameConfig) -> Result<Game, GameError> {
        config.validate()?;
        let board = Board::new(config.board_size);
        let pawns = config.pawns_per_player;
        let first_actions = config.first_turn_actions;

        Ok(Self {
            config,
            board,
            current_player: Player::One,
            turn: 1,
            actions_left: first_actions,
            reserve_pions: [pawns, pawns],
            banked_actions: [0, 0],
            winner: None,
        })
    }

    fn ensure_owned(&self, pos: Position) -> Result<(), GameError> {
        match self.board.get(pos) {
            None => Err(GameError::OutOfBounds(pos)),
            Some(cell) if cell.owner() == Some(self.current_player) => Ok(()),
            Some(_) => Err(GameError::NotYourPawn(pos)),
        }
    }

    fn ensure_empty(&self, pos: Position) -> Result<(), GameError> {
        match self.board.get(pos) {
            None => Err(GameError::OutOfBounds(pos)),
            Some(cell) if cell == Cell::Empty => Ok(()),
            Some(_) => Err(GameError::CellOccupied(pos)),
        }
    }

    fn start_turn(&mut self) {
    let base = if self.turn == 1 {
        self.config.first_turn_actions
    } else {
        self.config.action_per_turn
    };
    let i = self.current_player.index();
    self.actions_left = base + self.banked_actions[i];
    self.banked_actions[i] = 0;
}

fn end_turn(&mut self) {
    let i = self.current_player.index();
    self.banked_actions[i] = self.actions_left.min(self.config.max_banked_actions);
    self.current_player = self.current_player.opponent();
    self.turn += 1;
    self.start_turn();
}

    fn execute(&mut self, action: Action) -> Result<Outcome, GameError> {
    self.actions_left -= action.cost();
    match action {
        Action::Place(pos) => {
            self.board.set(pos, Pion(self.current_player))?;
            self.reserve_pions[self.current_player.index()] -= 1;

            if rules::complete_line(&self.board, pos, self.config.align_to_win) {
                self.winner = Some(self.current_player);
                return Ok(Outcome::Victory(self.current_player));
            }
            if self.actions_left == 0 {
                self.end_turn();
                return Ok(Outcome::TurnEnded { next: self.current_player });
            }
            Ok(Outcome::Continue)
        }
        Action::EndTurn => {
            self.end_turn();
            Ok(Outcome::TurnEnded { next: self.current_player })
        }
    }
}

    fn validate(&self,action: Action) -> Result<(),GameError> {
        if self.winner.is_some(){
            return Err(GameError::GameOver);
        }
        if self.actions_left < action.cost(){
            return Err(GameError::NotEnoughActions { needed: action.cost(), available: self.actions_left})
        }
        match action {
            Action::EndTurn => Ok(()),
            Action::Place(pos) => self.validate_place(pos)
        }
    }

    fn validate_place(&self,pos: Position) -> Result<(),GameError>{
        self.ensure_empty(pos)?;
        if self.reserve_pions[self.current_player.index()]==0{
            return Err(GameError::EmptyReserve);
        }
        Ok(())

    }

    pub fn apply(&mut self, action: Action) -> Result<Outcome, GameError> {
        self.validate(action)?; // lit l'état, ne modifie rien
        self.execute(action) // modifie l'état, ne vérifie plus
    }
}
