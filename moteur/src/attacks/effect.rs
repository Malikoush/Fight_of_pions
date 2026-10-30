use crate::{EffectKind, Position};

/// Une opération de base produite par une attaque. Les familles décrivent leurs effets avec
/// ces primitives ; seul le résolveur de la partie les applique, émet les événements et
/// déclenche les conséquences (infirmerie, fin des effets d'un pion tué…). De nouvelles
/// primitives (poser…) s'ajouteront avec les familles qui en ont besoin (catalogue dans
/// l'architecture).
///
/// Les effets s'appliquent dans l'ordre : chaque case désigne le pion qui s'y trouve au
/// moment où l'effet s'applique (après les déplacements et les kills qui le précèdent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Effect {
    /// Tuer le pion de cette case. Les familles ne le produisent jamais sur un pion protégé
    /// (`AttackContext::can_kill`). Un pion protégé atteint sans être visé (survol, charge)
    /// devra survivre : le résolveur le vérifiera avec les familles qui le permettent (B4).
    Kill {
        /// La case du pion tué.
        at: Position,
    },
    /// Donner un effet continu au pion de cette case, pour `turns` tours (au moins 1).
    Apply {
        /// La case du pion visé.
        at: Position,
        /// L'effet donné (immobilisé, protégé).
        kind: EffectKind,
        /// Sa durée, en tours de l'adversaire du joueur qui attaque.
        turns: u32,
    },
    /// Retirer du pion de cette case tous les effets créés par l'adversaire du joueur qui
    /// attaque.
    RemoveEnemyEffects {
        /// La case du pion visé.
        at: Position,
    },
    /// Déplacer le pion de `from` vers la case libre `to`. C'est la famille qui calcule le
    /// trajet (pas, saut…) ; seule l'arrivée compte pour la partie.
    Move {
        /// La case du pion, au moment où l'effet s'applique.
        from: Position,
        /// La case d'arrivée.
        to: Position,
    },
    /// Échanger les pions de deux cases, d'un coup.
    Swap {
        /// La case du premier pion.
        a: Position,
        /// La case du second pion.
        b: Position,
    },
}
