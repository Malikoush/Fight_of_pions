use crate::Player;

/// Le contenu d'une case du plateau.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    /// Case libre.
    Empty,
    /// Case occupée par un pion du joueur indiqué.
    Pion(Player),
}

impl Cell {
    /// Le propriétaire du pion présent, ou `None` si la case est libre.
    pub fn owner(self) -> Option<Player> {
        match self {
            Cell::Empty => None,
            Cell::Pion(player) => Some(player),
        }
    }
}
