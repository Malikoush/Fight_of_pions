use crate::{Pion, Player};

/// Le contenu d'une case du plateau.
///
/// C'est une structure en couches, et non un choix exclusif : d'autres couches
/// (tombe, trou…) s'ajouteront à côté du pion quand une attaque en aura besoin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pion: Option<Pion>,
}

impl Cell {
    /// Une case sans rien dessus.
    pub const EMPTY: Cell = Cell { pion: None };

    /// Une case occupée par ce pion.
    pub(crate) fn with_pion(pion: Pion) -> Cell {
        Cell { pion: Some(pion) }
    }

    /// Le pion présent, s'il y en a un.
    pub fn pion(self) -> Option<Pion> {
        self.pion
    }

    /// Le propriétaire du pion présent, ou `None` si la case n'a pas de pion.
    pub fn owner(self) -> Option<Player> {
        self.pion.map(Pion::owner)
    }
}
