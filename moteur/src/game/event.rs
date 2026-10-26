use crate::{GameEnd, Pion, Player, Position};

/// Ce que le moteur annonce après une action acceptée.
///
/// Les clients mettent leur affichage à jour à partir de ces événements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    /// Un pion a été posé.
    PionPlaced {
        /// Le pion posé, avec son identité et son propriétaire.
        pion: Pion,
        /// La case où il a été posé.
        at: Position,
    },
    /// Le tour est fini.
    TurnEnded {
        /// Le joueur qui joue maintenant.
        next: Player,
    },
    /// La partie est finie : victoire ou match nul, avec sa raison.
    GameEnded(GameEnd),
}
