/// Un des deux camps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    /// Le joueur qui commence.
    One,
    /// Le second joueur.
    Two,
}

impl Player {
    /// L'autre joueur.
    pub fn opponent(self) -> Player {
        match self {
            Player::One => Player::Two,
            Player::Two => Player::One,
        }
    }
    /// Indice 0 ou 1, pour ranger une valeur par joueur dans un tableau `[T; 2]`.
    pub fn index(self) -> usize {
        match self {
            Player::One => 0,
            Player::Two => 1,
        }
    }
}
