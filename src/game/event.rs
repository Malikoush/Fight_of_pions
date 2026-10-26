use crate::{Player, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    PawnPlaced { player: Player, at: Position },
    TurnEnded { next: Player },
    GameWon { winner: Player },
}
