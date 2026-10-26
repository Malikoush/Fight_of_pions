use crate::Position;

/// Une décision du joueur courant, envoyée au moteur par [`crate::Game::apply`].
///
/// Son coût en actions dépend de la configuration de la partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Poser un pion de sa réserve sur une case libre ; coûte
    /// [`crate::GameConfig::place_cost`] actions.
    Place(Position),
    /// Finir son tour, gratuitement ; les actions restantes sont gardées dans la limite
    /// de [`crate::GameConfig::max_banked_actions`].
    EndTurn,
}
