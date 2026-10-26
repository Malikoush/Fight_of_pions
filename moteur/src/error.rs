use crate::Position;
use crate::config::ConfigError;
use std::fmt;

/// Pourquoi le moteur refuse une configuration ou une action.
///
/// Une action refusée ne modifie jamais la partie. Le message affiché commence par une
/// minuscule, pour qu'un client puisse écrire « Coup refusé : {erreur} ».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameError {
    /// La configuration est incohérente.
    InvalidConfig(ConfigError),
    /// La position est hors du plateau.
    OutOfBounds(Position),
    /// La case n'est pas libre.
    CellOccupied(Position),
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
            GameError::InvalidConfig(problem) => write!(f, "configuration invalide : {}", problem),
            GameError::OutOfBounds(pos) => write!(f, "la case {} est hors du plateau", pos),
            GameError::CellOccupied(pos) => write!(f, "la case {} est déjà occupée", pos),
            GameError::EmptyReserve => write!(f, "la réserve de pions est vide"),
            GameError::NotEnoughActions { needed, available } => write!(
                f,
                "pas assez d'actions : il en faut {}, il en reste {}",
                needed, available
            ),
            GameError::GameOver => write!(f, "la partie est terminée"),
        }
    }
}
