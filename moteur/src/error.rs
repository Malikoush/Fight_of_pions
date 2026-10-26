use crate::Position;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    InvalidConfig(&'static str),
    OutOfBounds(Position),
    CellOccupied(Position),
    NotYourPawn(Position),
    IllegalMove { from: Position, to: Position },
    EmptyReserve,
    NotEnoughActions { needed: u32, available: u32 },
    GameOver,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::CellOccupied(pos) => write!(f, "la case {} n'est pas libre", pos),
            GameError::IllegalMove { from, to } => {
                write!(f, "déplacement interdit de {} vers {}", from, to)
            }
            GameError::EmptyReserve => write!(f, "votre réserve de pions est vide"),
            GameError::GameOver => write!(f, "La partie est finie"),
            GameError::InvalidConfig(str) => write!(f, "Config {} est incohérente", str),
            GameError::OutOfBounds(pos) => write!(f, "La case {} est en dehors des limites", pos),
            GameError::NotYourPawn(position) => {
                write!(f, "la case {} n'est pas votre pion", position)
            }
            GameError::NotEnoughActions { needed, available } => write!(
                f,
                "Action épuisé nécessite {} action il en reste {}",
                needed, available
            ),
        }
    }
}
