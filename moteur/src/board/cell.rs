use crate::{Pion, Player};

/// Le contenu d'une case du plateau.
///
/// C'est une structure en couches, et non un choix exclusif : un pion, et sous lui, la
/// tombe d'un pion tué (le Médecin) ; une tombe ne bloque pas la case. Le mur n'est pas
/// une couche : une case de mur n'est pas sur le plateau ([`crate::Board::is_wall`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub(super) pion: Option<Pion>,
    pub(super) tomb: Option<Pion>,
}

impl Cell {
    /// Une case sans rien dessus.
    pub const EMPTY: Cell = Cell {
        pion: None,
        tomb: None,
    };

    /// Le pion présent, s'il y en a un.
    pub fn pion(self) -> Option<Pion> {
        self.pion
    }

    /// Le propriétaire du pion présent, ou `None` si la case n'a pas de pion.
    pub fn owner(self) -> Option<Player> {
        self.pion.map(Pion::owner)
    }

    /// Le pion tué dont la tombe est sur cette case, s'il y en a une. Elle disparaît quand
    /// ce pion sort de l'infirmerie.
    pub fn tomb(self) -> Option<Pion> {
        self.tomb
    }
}
