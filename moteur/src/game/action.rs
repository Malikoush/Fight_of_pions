use crate::Position;

/// Une décision du joueur, envoyée au moteur par [`crate::Game::apply`]. Elle doit
/// répondre à la décision attendue ([`crate::Game::pending_decision`]).
///
/// Son coût en actions dépend de la configuration de la partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    /// Poser un pion de sa réserve sur une case libre ; coûte
    /// [`crate::GameConfig::place_cost`] actions, dans la limite de
    /// [`crate::GameConfig::max_placements_per_turn`] poses par tour.
    Place(Position),
    /// Commencer l'attaque de cet emplacement de son arsenal. Le lanceur puis les cibles
    /// se choisissent ensuite avec [`Action::Choose`] ; le coût est payé quand l'attaque
    /// est jouée.
    /// Une attaque commencée va toujours au bout : le moteur ne propose jamais une
    /// attaque qui n'a aucune cible possible.
    UseAttack(usize),
    /// Choisir une case : le pion lanceur, puis chaque cible de l'attaque en cours.
    /// L'attaque est jouée dès qu'il n'y a plus rien à choisir.
    Choose(Position),
    /// Jouer l'attaque en cours avec les cibles déjà choisies, quand elle en permet plus
    /// (« jusqu'à 2 ») et qu'on n'en veut pas d'autre. Permis seulement si l'attaque est
    /// déjà complète.
    FinishAttack,
    /// Faire avancer sa reine d'une case, vers cette case libre collée à elle : le pas
    /// gratuit de la couronne, une fois par tour. Ce n'est pas une attaque.
    StepQueen(Position),
    /// Faire tirer un garde de sa reine sur ce pion, le premier rencontré en ligne droite
    /// depuis elle : une attaque, une fois par tour, qui coûte
    /// [`crate::GameConfig::guard_shot_cost`] et le garde.
    Shoot(Position),
    /// Finir son tour, gratuitement ; les actions restantes sont gardées dans la limite
    /// de [`crate::GameConfig::max_banked_actions`].
    EndTurn,
}
