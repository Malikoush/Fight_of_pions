use crate::Position;

/// Une décision du joueur, envoyée au moteur par [`crate::Game::apply`]. Elle doit
/// répondre à la décision attendue ([`crate::Game::pending_decision`]).
///
/// Son coût en actions dépend de la configuration de la partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    /// Poser un pion de sa réserve sur une case libre ; coûte
    /// [`crate::GameConfig::place_cost`] actions.
    Place(Position),
    /// Commencer l'attaque de cet emplacement de son arsenal. Le lanceur puis les cibles
    /// se choisissent ensuite avec [`Action::Choose`] ; le coût est payé au dernier choix.
    /// Une attaque commencée va toujours au bout : le moteur ne propose jamais une
    /// attaque qui n'a aucune cible possible.
    UseAttack(usize),
    /// Choisir une case : le pion lanceur, puis chaque cible de l'attaque en cours.
    Choose(Position),
    /// Finir son tour, gratuitement ; les actions restantes sont gardées dans la limite
    /// de [`crate::GameConfig::max_banked_actions`].
    EndTurn,
}
