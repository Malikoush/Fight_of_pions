//! Ce qui dure après une action : les effets continus (statuts…) et l'infirmerie. Les
//! deux suivent la même horloge : une durée se compte en tours de l'adversaire de celui
//! qui a joué l'effet, le tour en cours ne compte jamais, et elle s'éteint à la fin de son
//! dernier tour (carnet de règles, 2026-10-30).

use crate::{Attack, Board, Cell, Pion, PionId, Player, Position};
use std::fmt;

/// L'identité d'un effet continu, donnée dans l'ordre de création à partir de 0 : une même
/// suite d'actions donne toujours les mêmes identités.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectId(u32);

impl EffectId {
    /// L'identité du premier effet créé.
    pub(crate) const FIRST: EffectId = EffectId(0);

    /// L'identité suivante ; resterait bloquée au maximum, impossible dans une partie.
    pub(crate) fn next(self) -> EffectId {
        EffectId(self.0.saturating_add(1))
    }
}

/// « effet 3 », pour les journaux et le débogage.
impl fmt::Display for EffectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "effet {}", self.0)
    }
}

/// Ce que fait un effet continu. D'autres sortes viendront avec les cartes qui en ont
/// besoin (jetons…).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectKind {
    /// Statut immobilisé : le pion ne peut pas lancer d'attaque, donc pas non plus bouger
    /// par une carte de son propriétaire.
    Immobilized,
    /// Statut protégé : le pion ne peut pas être tué.
    Protected,
    /// Le pacte de Meilleurs amis : si ce pion est tué, `friend` venge sa mort. Chacun des
    /// deux amis porte le sien. Sans durée.
    Pact {
        /// L'autre ami.
        friend: PionId,
    },
    /// La couronne : ce pion est une reine. Elle avance d'une case gratuitement à chaque
    /// tour ([`crate::Action::StepQueen`]). Sans durée.
    Crown,
    /// Un garde de la reine : il la sauve de la prochaine attaque qui la tuerait, ou tire
    /// ([`crate::Action::Shoot`]). Chaque garde sert une fois. Sans durée.
    Guard,
}

impl EffectKind {
    /// Vrai pour un statut (immobilisé, protégé) : ce qu'Attila Commune retire.
    pub fn is_status(self) -> bool {
        matches!(self, EffectKind::Immobilized | EffectKind::Protected)
    }
}

/// « immobilisé », « protégé », « lié par un pacte avec #3 », « couronné », « gardé ».
impl fmt::Display for EffectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EffectKind::Immobilized => write!(f, "immobilisé"),
            EffectKind::Protected => write!(f, "protégé"),
            EffectKind::Pact { friend } => write!(f, "lié par un pacte avec {friend}"),
            EffectKind::Crown => write!(f, "couronné"),
            EffectKind::Guard => write!(f, "gardé"),
        }
    }
}

/// Un effet continu : ce qu'une carte laisse en jeu après avoir été jouée. Il vise un pion
/// par son identité, pas par sa case, pour le suivre s'il bouge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveEffect {
    id: EffectId,
    kind: EffectKind,
    pion: PionId,
    creator: Player,
    source: Attack,
    last_turn: Option<u32>,
}

impl ActiveEffect {
    pub(crate) fn new(
        id: EffectId,
        kind: EffectKind,
        pion: PionId,
        creator: Player,
        source: Attack,
        last_turn: Option<u32>,
    ) -> ActiveEffect {
        ActiveEffect {
            id,
            kind,
            pion,
            creator,
            source,
            last_turn,
        }
    }

    /// Son identité.
    pub fn id(self) -> EffectId {
        self.id
    }

    /// Ce qu'il fait.
    pub fn kind(self) -> EffectKind {
        self.kind
    }

    /// Le pion qu'il vise.
    pub fn pion(self) -> PionId {
        self.pion
    }

    /// Le joueur qui l'a créé : Attila ne retire que ceux de l'adversaire.
    pub fn creator(self) -> Player {
        self.creator
    }

    /// La carte qui l'a créé.
    pub fn source(self) -> Attack {
        self.source
    }

    /// Le dernier tour où il agit ([`crate::Game::turn`]) : il s'éteint à la fin de ce tour.
    /// `None` pour un effet sans durée (le pacte), qui ne s'arrête que par un événement.
    pub fn last_turn(self) -> Option<u32> {
        self.last_turn
    }

    /// Vrai si cet effet est un statut sur ce pion, posé par ce joueur : ce qu'Attila
    /// Commune retire.
    pub(crate) fn is_status_on_by(self, pion: PionId, creator: Player) -> bool {
        self.kind.is_status() && self.pion == pion && self.creator == creator
    }

    /// L'autre ami, si cet effet est le pacte de ce pion.
    pub(crate) fn friend_of(self, pion: PionId) -> Option<PionId> {
        match self.kind {
            EffectKind::Pact { friend } if self.pion == pion => Some(friend),
            _ => None,
        }
    }
}

/// Un pion tué qui attend à l'infirmerie de revenir dans la réserve de son propriétaire.
/// L'infirmerie n'est pas un effet continu : aucune carte ne l'annule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Patient {
    pion: Pion,
    last_turn: u32,
}

impl Patient {
    pub(crate) fn new(pion: Pion, last_turn: u32) -> Patient {
        Patient { pion, last_turn }
    }

    /// Le pion blessé, avec son identité et son propriétaire.
    pub fn pion(self) -> Pion {
        self.pion
    }

    /// Le dernier tour qu'il passe à l'infirmerie ([`crate::Game::turn`]) : à la fin de ce
    /// tour, il revient dans la réserve de son propriétaire.
    pub fn last_turn(self) -> u32 {
        self.last_turn
    }
}

/// Vrai si ce pion a un effet de cette sorte : la seule lecture d'un statut.
pub(crate) fn has_effect_of(effects: &[ActiveEffect], pion: PionId, kind: EffectKind) -> bool {
    effects
        .iter()
        .any(|effect| effect.pion == pion && effect.kind == kind)
}

/// La case de la reine de ce joueur, s'il en a une : le pion couronné qui est à lui.
/// Partagé par la partie et par la carte Reine (une seule reine à la fois).
pub(crate) fn queen_of(
    board: &Board,
    effects: &[ActiveEffect],
    player: Player,
) -> Option<Position> {
    effects
        .iter()
        .filter(|effect| effect.kind == EffectKind::Crown)
        .filter_map(|effect| board.position_of(effect.pion))
        .find(|&pos| board.owner(pos) == Some(player))
}

/// Vrai si le pion de cette case a un effet de cette sorte. Partagé par la partie et par
/// les familles d'attaques, pour qu'elles lisent les statuts de la même façon.
pub(crate) fn pion_has(
    board: &Board,
    effects: &[ActiveEffect],
    pos: Position,
    kind: EffectKind,
) -> bool {
    board
        .get(pos)
        .and_then(Cell::pion)
        .is_some_and(|pion| has_effect_of(effects, pion.id(), kind))
}
