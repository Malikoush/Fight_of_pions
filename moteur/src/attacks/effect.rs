use crate::Position;

/// Une opération de base produite par une attaque. Les familles décrivent leurs effets avec
/// ces primitives ; seul le résolveur de la partie les applique, émet les événements et
/// déclenche les conséquences (retour en réserve…). De nouvelles primitives (déplacer,
/// poser, changer de camp…) s'ajouteront avec les familles qui en ont besoin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Effect {
    /// Tuer le pion de cette case : il retourne dans la réserve de son propriétaire.
    Kill {
        /// La case du pion tué.
        at: Position,
    },
}
