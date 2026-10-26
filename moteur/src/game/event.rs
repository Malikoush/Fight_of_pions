use crate::{Player, Position};

/// Ce que le moteur annonce après une action acceptée.
///
/// Les clients mettent leur affichage à jour à partir de ces événements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Un pion a été posé.
    PawnPlaced {
        /// Le propriétaire du pion.
        player: Player,
        /// La case où il a été posé.
        at: Position,
    },
    /// Le tour est fini.
    TurnEnded {
        /// Le joueur qui joue maintenant.
        next: Player,
    },
    /// La partie est gagnée.
    GameWon {
        /// Le joueur qui a aligné assez de pions.
        winner: Player,
    },
}
