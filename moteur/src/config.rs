use crate::error::GameError;

/// Tous les réglages chiffrés d'une partie, vérifiés par [`GameConfig::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    /// Nombre de cases par côté du plateau.
    pub board_size: usize,
    /// Longueur de la ligne qui fait gagner.
    pub align_to_win: usize,
    /// Pions en réserve pour chaque joueur au début.
    pub pawns_per_player: u32,
    /// Actions du tout premier tour de la partie.
    pub first_turn_actions: u32,
    /// Actions des tours suivants.
    pub action_per_turn: u32,
    /// Actions non utilisées qu'on peut garder pour le tour suivant.
    pub max_banked_actions: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board_size: 10,
            align_to_win: 6,
            pawns_per_player: 15,
            action_per_turn: 2,
            first_turn_actions: 1,
            max_banked_actions: 2,
        }
    }
}

impl GameConfig {
    /// Vérifie que les réglages sont cohérents entre eux.
    pub fn validate(&self) -> Result<(), GameError> {
        if self.align_to_win > self.board_size {
            return Err(GameError::InvalidConfig(
                "l'alignement est plus long que le plateau",
            ));
        }
        //autres vérifications
        Ok(())
    }
}
