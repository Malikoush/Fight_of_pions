use crate::config::ConfigError;
use crate::{Attack, Position};
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
    /// Le joueur a déjà posé autant de pions que permis ce tour.
    NoPlacementLeft,
    /// Il ne reste pas assez d'actions ce tour.
    NotEnoughActions {
        /// Coût de l'action demandée.
        needed: u32,
        /// Actions qu'il reste ce tour.
        available: u32,
    },
    /// La partie est terminée.
    GameOver,
    /// Cette action ne répond pas à la décision attendue (par exemple poser un pion
    /// pendant le choix d'une cible).
    UnexpectedAction,
    /// Le joueur n'a pas d'attaque à cet emplacement de son arsenal.
    NoSuchAttack(usize),
    /// Cette attaque est en recharge.
    AttackRecharging(Attack),
    /// Cette attaque n'a aucune cible possible en ce moment.
    NoTarget(Attack),
    /// Cette attaque aurait des cibles, mais seulement depuis des pions posés ce tour, que
    /// le mal du débarquement empêche d'attaquer.
    NoReadyLauncher(Attack),
    /// Ce choix de lanceur ou de cible n'est pas permis.
    InvalidChoice(Position),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::InvalidConfig(problem) => write!(f, "configuration invalide : {}", problem),
            GameError::OutOfBounds(pos) => write!(f, "la case {} est hors du plateau", pos),
            GameError::CellOccupied(pos) => write!(f, "la case {} est déjà occupée", pos),
            GameError::EmptyReserve => write!(f, "la réserve de pions est vide"),
            GameError::NoPlacementLeft => write!(f, "plus aucune pose permise ce tour"),
            GameError::NotEnoughActions { needed, available } => write!(
                f,
                "pas assez d'actions : il en faut {}, il en reste {}",
                needed, available
            ),
            GameError::GameOver => write!(f, "la partie est terminée"),
            GameError::UnexpectedAction => {
                write!(f, "ce n'est pas le moment de cette action")
            }
            GameError::NoSuchAttack(slot) => {
                write!(
                    f,
                    "aucune attaque à l'emplacement {}",
                    slot.saturating_add(1)
                )
            }
            GameError::AttackRecharging(attack) => {
                write!(f, "l'attaque {} est en recharge", attack)
            }
            GameError::NoTarget(attack) => {
                write!(f, "l'attaque {} n'a aucune cible possible", attack)
            }
            GameError::NoReadyLauncher(attack) => write!(
                f,
                "l'attaque {} n'a aucun lanceur prêt : un pion posé ce tour ne peut pas attaquer",
                attack
            ),
            GameError::InvalidChoice(pos) => write!(f, "la case {} ne peut pas être choisie", pos),
        }
    }
}
