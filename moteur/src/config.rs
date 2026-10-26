use crate::error::GameError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    pub board_size: usize,
    pub align_to_win: usize,
    pub pawns_per_player: u32,
    pub first_turn_actions: u32,
    pub action_per_turn: u32,
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
