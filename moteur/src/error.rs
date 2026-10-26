use crate::Position;
use std::fmt;

/// Pourquoi le moteur refuse une configuration ou une action.
///
/// Une action refusée ne modifie jamais la partie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    /// La configuration est incohérente.
    InvalidConfig(&'static str),
    /// La position est hors du plateau.
    OutOfBounds(Position),
    /// La case n'est pas libre.
    CellOccupied(Position),
    /// La case ne contient pas un pion du joueur courant.
    NotYourPawn(Position),
    /// Ce déplacement n'est pas autorisé.
    IllegalMove {
        /// Case de départ.
        from: Position,
        /// Case d'arrivée demandée.
        to: Position,
    },
    /// Le joueur n'a plus de pion en réserve.
    EmptyReserve,
    /// Il ne reste pas assez d'actions ce tour.
    NotEnoughActions {
        /// Coût de l'action demandée.
        needed: u32,
        /// Actions qu'il reste ce tour.
        available: u32,
    },
    /// La partie est terminée.
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
