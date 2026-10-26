use crate::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Pion(Player),
}

impl Cell {
    pub fn owner(self) -> Option<Player> {
        match self {
            Cell::Empty => None,
            Cell::Pion(player) => Some(player),
        }
    }
}
