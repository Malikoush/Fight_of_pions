use crate::Player;

/// La décision que le moteur attend, et de quel joueur.
///
/// Les actions acceptées pour cette décision sont données par
/// [`crate::Game::legal_actions`]. D'autres décisions s'ajouteront avec les attaques
/// (choix du lanceur, d'une cible, fenêtre de réaction) et le draft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decision {
    /// Le joueur choisit sa prochaine action du tour : poser un pion ou finir son tour.
    MainAction {
        /// Le joueur qui doit choisir.
        player: Player,
    },
}

impl Decision {
    /// Le joueur qui doit choisir.
    pub fn player(self) -> Player {
        match self {
            Decision::MainAction { player } => player,
        }
    }
}
