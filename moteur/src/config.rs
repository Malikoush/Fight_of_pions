use crate::Position;
use crate::error::GameError;
use std::fmt;

/// Tous les réglages chiffrés d'une partie, vérifiés par [`GameConfig::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameConfig {
    /// Nombre de cases par côté du plateau, entre [`GameConfig::MIN_BOARD_SIZE`]
    /// et [`GameConfig::MAX_BOARD_SIZE`].
    pub board_size: usize,
    /// Longueur de la ligne qui fait gagner, entre [`GameConfig::MIN_ALIGN_TO_WIN`]
    /// et `board_size`.
    pub align_to_win: usize,
    /// Pions en réserve pour chaque joueur au début, au moins 1.
    pub pawns_per_player: u32,
    /// Actions du tout premier tour de la partie, au moins 1.
    pub first_turn_actions: u32,
    /// Actions des tours suivants, au moins 1.
    pub actions_per_turn: u32,
    /// Actions non utilisées qu'on peut garder pour le tour suivant.
    pub max_banked_actions: u32,
    /// Actions que coûte la pose d'un pion, au moins 1.
    pub place_cost: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board_size: 10,
            align_to_win: 6,
            pawns_per_player: 15,
            first_turn_actions: 1,
            actions_per_turn: 2,
            max_banked_actions: 2,
            place_cost: 1,
        }
    }
}

/// Ce qui rend une configuration incohérente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigError {
    /// La taille du plateau sort des bornes permises.
    BoardSizeOutOfRange,
    /// La ligne gagnante est trop courte.
    AlignmentTooShort,
    /// La ligne gagnante est plus longue que le plateau.
    AlignmentLongerThanBoard,
    /// Un joueur commencerait sans pion.
    NoPawns,
    /// Un tour ne donnerait aucune action.
    NoActionsPerTurn,
    /// Poser un pion ne coûterait rien : toute la réserve partirait en un tour.
    FreePlacement,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::BoardSizeOutOfRange => write!(
                f,
                "la taille du plateau doit être comprise entre {} et {}",
                GameConfig::MIN_BOARD_SIZE,
                GameConfig::MAX_BOARD_SIZE
            ),
            ConfigError::AlignmentTooShort => write!(
                f,
                "il faut aligner au moins {} pions pour gagner",
                GameConfig::MIN_ALIGN_TO_WIN
            ),
            ConfigError::AlignmentLongerThanBoard => {
                write!(f, "l'alignement est plus long que le plateau")
            }
            ConfigError::NoPawns => write!(f, "chaque joueur doit avoir au moins un pion"),
            ConfigError::NoActionsPerTurn => {
                write!(f, "chaque tour doit donner au moins une action")
            }
            ConfigError::FreePlacement => {
                write!(f, "poser un pion doit coûter au moins une action")
            }
        }
    }
}

impl GameConfig {
    /// Le plus petit plateau accepté.
    pub const MIN_BOARD_SIZE: usize = 2;
    /// Le plus grand plateau accepté : une colonne par lettre, de A à Z.
    pub const MAX_BOARD_SIZE: usize = Position::NAMED_COLUMNS;
    /// La plus courte ligne gagnante acceptée.
    pub const MIN_ALIGN_TO_WIN: usize = 2;

    /// Vérifie que les réglages sont cohérents entre eux.
    pub fn validate(&self) -> Result<(), GameError> {
        self.find_problem()
            .map_or(Ok(()), |problem| Err(GameError::InvalidConfig(problem)))
    }

    fn find_problem(&self) -> Option<ConfigError> {
        if !(Self::MIN_BOARD_SIZE..=Self::MAX_BOARD_SIZE).contains(&self.board_size) {
            return Some(ConfigError::BoardSizeOutOfRange);
        }
        if self.align_to_win < Self::MIN_ALIGN_TO_WIN {
            return Some(ConfigError::AlignmentTooShort);
        }
        if self.align_to_win > self.board_size {
            return Some(ConfigError::AlignmentLongerThanBoard);
        }
        if self.pawns_per_player == 0 {
            return Some(ConfigError::NoPawns);
        }
        if self.first_turn_actions == 0 || self.actions_per_turn == 0 {
            return Some(ConfigError::NoActionsPerTurn);
        }
        if self.place_cost == 0 {
            return Some(ConfigError::FreePlacement);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem(config: GameConfig) -> Option<ConfigError> {
        match config.validate() {
            Ok(()) => None,
            Err(GameError::InvalidConfig(problem)) => Some(problem),
            Err(other) => panic!("erreur inattendue : {other:?}"),
        }
    }

    #[test]
    fn default_config_is_valid() {
        assert_eq!(problem(GameConfig::default()), None);
    }

    #[test]
    fn board_size_must_be_between_2_and_26() {
        let with_size = |board_size| GameConfig {
            board_size,
            align_to_win: 2,
            ..GameConfig::default()
        };
        for size in [0, 1, 27, usize::MAX] {
            assert_eq!(
                problem(with_size(size)),
                Some(ConfigError::BoardSizeOutOfRange)
            );
        }
        assert_eq!(problem(with_size(2)), None);
        assert_eq!(problem(with_size(26)), None);
    }

    #[test]
    fn alignment_must_be_between_2_and_board_size() {
        let with_align = |align_to_win| GameConfig {
            align_to_win,
            ..GameConfig::default()
        };
        assert_eq!(problem(with_align(0)), Some(ConfigError::AlignmentTooShort));
        assert_eq!(problem(with_align(1)), Some(ConfigError::AlignmentTooShort));
        assert_eq!(
            problem(with_align(11)),
            Some(ConfigError::AlignmentLongerThanBoard)
        );
        assert_eq!(problem(with_align(2)), None);
        assert_eq!(problem(with_align(10)), None);
    }

    #[test]
    fn each_player_needs_at_least_one_pawn() {
        let config = GameConfig {
            pawns_per_player: 0,
            ..GameConfig::default()
        };
        assert_eq!(problem(config), Some(ConfigError::NoPawns));
    }

    #[test]
    fn each_turn_needs_at_least_one_action() {
        let first = GameConfig {
            first_turn_actions: 0,
            ..GameConfig::default()
        };
        let later = GameConfig {
            actions_per_turn: 0,
            ..GameConfig::default()
        };
        assert_eq!(problem(first), Some(ConfigError::NoActionsPerTurn));
        assert_eq!(problem(later), Some(ConfigError::NoActionsPerTurn));
    }

    #[test]
    fn placing_cannot_be_free() {
        let config = GameConfig {
            place_cost: 0,
            ..GameConfig::default()
        };
        assert_eq!(problem(config), Some(ConfigError::FreePlacement));
    }

    #[test]
    fn error_message_uses_the_real_bounds() {
        assert_eq!(
            ConfigError::BoardSizeOutOfRange.to_string(),
            "la taille du plateau doit être comprise entre 2 et 26"
        );
    }
}
