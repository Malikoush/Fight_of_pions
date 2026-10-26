use crate::Player;
use std::fmt;

/// L'identité stable d'un pion : attribuée à sa pose, elle le suit quand il bouge.
///
/// Un effet qui vise un pion (immobilisé, protégé…) retient son identité, pas sa case.
/// Les identités sont données dans l'ordre des poses, à partir de 0 : une même suite
/// d'actions donne toujours les mêmes identités.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PionId(u32);

impl PionId {
    /// L'identité donnée au premier pion posé.
    pub(crate) const FIRST: PionId = PionId(0);

    /// L'identité suivante. Après 4 milliards de poses, elle resterait bloquée au maximum :
    /// impossible dans une partie réelle.
    pub(crate) fn next(self) -> PionId {
        PionId(self.0.saturating_add(1))
    }
}

/// « #3 » : le numéro du pion, pour les journaux et le débogage.
impl fmt::Display for PionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Un pion posé sur le plateau.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pion {
    id: PionId,
    owner: Player,
}

impl Pion {
    pub(crate) fn new(id: PionId, owner: Player) -> Pion {
        Pion { id, owner }
    }

    /// L'identité du pion.
    pub fn id(self) -> PionId {
        self.id
    }

    /// Le joueur à qui appartient le pion.
    pub fn owner(self) -> Player {
        self.owner
    }
}
