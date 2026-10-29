use crate::{Attack, Player, Position};

/// La décision que le moteur attend, et de quel joueur.
///
/// Les actions acceptées pour cette décision sont données par
/// [`crate::Game::legal_actions`]. D'autres décisions s'ajouteront avec les réactions et
/// le draft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decision {
    /// Le joueur choisit sa prochaine action du tour : poser un pion, lancer une attaque
    /// ou finir son tour.
    MainAction {
        /// Le joueur qui doit choisir.
        player: Player,
    },
    /// Le joueur choisit le pion depuis lequel il lance son attaque.
    ChooseLauncher {
        /// Le joueur qui doit choisir.
        player: Player,
        /// L'attaque en cours.
        attack: Attack,
    },
    /// Le joueur choisit la prochaine cible de son attaque.
    ChooseTarget {
        /// Le joueur qui doit choisir.
        player: Player,
        /// L'attaque en cours.
        attack: Attack,
        /// La case du pion lanceur.
        launcher: Position,
    },
}

impl Decision {
    /// Vrai si c'est un choix d'attaque commencée (lanceur ou cible), et non l'action
    /// principale du tour.
    pub fn is_attack_choice(self) -> bool {
        matches!(
            self,
            Decision::ChooseLauncher { .. } | Decision::ChooseTarget { .. }
        )
    }

    /// Le joueur qui doit choisir.
    pub fn player(self) -> Player {
        match self {
            Decision::MainAction { player }
            | Decision::ChooseLauncher { player, .. }
            | Decision::ChooseTarget { player, .. } => player,
        }
    }
}
