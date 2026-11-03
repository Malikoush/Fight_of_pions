use crate::{Attack, DraftStep, Player, Position};

/// La décision que le moteur attend, et de quel joueur.
///
/// Les actions acceptées pour cette décision sont données par
/// [`crate::Game::legal_actions`]. D'autres décisions s'ajouteront avec le draft.
///
/// Pendant le tour adverse, juste après un déclencheur, c'est l'autre joueur qui décide :
/// il réagit ([`Decision::React`]), puis choisit le lanceur et les cibles de sa réaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Decision {
    /// Avant la partie, un choix du draft ([`crate::Action::Draft`]) : une carte pour sa
    /// cache, à bannir ou à prendre. Le joueur est celui de l'étape.
    Draft {
        /// L'étape du draft.
        step: DraftStep,
    },
    /// Le joueur choisit sa prochaine action du tour : poser un pion, lancer une attaque,
    /// faire avancer sa reine ou tirer avec un de ses gardes, ou finir son tour.
    MainAction {
        /// Le joueur qui doit choisir.
        player: Player,
    },
    /// Le joueur peut répondre au coup adverse par une de ses réactions
    /// ([`crate::Action::UseAttack`]), ou passer ([`crate::Action::Pass`]). Le moteur ne le
    /// demande que s'il a une réaction prête, payable avec sa banque et utilisable.
    React {
        /// Le joueur qui peut réagir, celui qui ne joue pas ce tour.
        player: Player,
    },
    /// Le joueur choisit le pion depuis lequel il lance son attaque (pour une Sentinelle :
    /// le pion adverse posé auquel elle répond).
    ChooseLauncher {
        /// Le joueur qui doit choisir.
        player: Player,
        /// L'attaque en cours.
        attack: Attack,
    },
    /// Le joueur choisit la prochaine cible de son attaque (une case, ou les jetons de
    /// l'adversaire : [`crate::Action::ChooseEnemyTokens`]), ou la joue avec les cibles déjà
    /// choisies ([`crate::Action::FinishAttack`]) si elle en permet moins.
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
            Decision::Draft { step } => step.player(),
            Decision::MainAction { player }
            | Decision::React { player }
            | Decision::ChooseLauncher { player, .. }
            | Decision::ChooseTarget { player, .. } => player,
        }
    }
}
