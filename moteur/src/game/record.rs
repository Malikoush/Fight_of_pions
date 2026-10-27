use crate::{Action, Game, GameConfig, GameError};
use std::fmt;

/// Tout ce qu'il faut pour reconstruire une partie : sa configuration, sa graine et la
/// liste des actions acceptées, dans l'ordre.
///
/// Comme le moteur est déterministe, rejouer ces actions redonne exactement la même
/// partie. C'est la base de la sauvegarde, du replay, de l'annulation d'un coup et de la
/// reproduction d'un bug. L'historique vit ici et non dans [`Game`] : deux parties dans la
/// même position restent égales, et une partie reste légère à copier pour les bots.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct GameRecord {
    /// Les réglages de la partie.
    pub config: GameConfig,
    /// La graine de la partie.
    pub seed: u64,
    /// Les actions acceptées, dans l'ordre où elles ont été jouées.
    pub actions: Vec<Action>,
}

/// Pourquoi un [`GameRecord`] n'a pas pu être rejoué.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayError {
    /// La partie n'a pas pu commencer : configuration invalide.
    Start(GameError),
    /// Une action a été refusée pendant la relecture.
    RefusedAction {
        /// La position de l'action refusée dans la liste, à partir de 0.
        index: usize,
        /// Pourquoi elle a été refusée.
        error: GameError,
    },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReplayError::Start(error) => write!(f, "la partie ne peut pas commencer : {}", error),
            ReplayError::RefusedAction { index, error } => {
                write!(f, "l'action n°{} est refusée : {}", index + 1, error)
            }
        }
    }
}

impl GameRecord {
    /// Un enregistrement vide, pour une partie qui commence.
    pub fn new(config: GameConfig, seed: u64) -> GameRecord {
        GameRecord {
            config,
            seed,
            actions: Vec::new(),
        }
    }

    /// Rejoue toutes les actions depuis le début et renvoie la partie obtenue.
    ///
    /// Erreur si la configuration est invalide ou si une action est refusée :
    /// l'enregistrement ne vient alors pas de cette version du moteur, ou a été modifié.
    pub fn replay(&self) -> Result<Game, ReplayError> {
        let mut game = Game::new(self.config.clone(), self.seed).map_err(ReplayError::Start)?;
        for (index, &action) in self.actions.iter().enumerate() {
            game.apply(action)
                .map_err(|error| ReplayError::RefusedAction { index, error })?;
        }
        Ok(game)
    }
}
