use crate::Position;

/// Une décision du joueur courant, envoyée au moteur par [`crate::Game::apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Poser un pion de sa réserve sur une case libre.
    Place(Position),
    /// Finir son tour ; les actions restantes sont gardées dans la limite autorisée.
    EndTurn,
}

impl Action {
    /// Nombre d'actions que coûte cette décision.
    pub fn cost(&self) -> u32 {
        match self {
            Action::EndTurn => 0,
            Action::Place(_) => 1,
        }
    }
}
