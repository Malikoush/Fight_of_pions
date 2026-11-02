//! L'Interchange (fiche n°06) : le lanceur échange sa case avec celle d'un pion adverse.
//!
//! - Commune : le lanceur est téléporté sur n'importe quelle case libre ; au début de ton
//!   tour suivant, il revient sur sa case d'origine (`game/returns.rs`). Pas jouable depuis
//!   un pion qui attend déjà son retour.
//! - Rare : un pion adverse à 1 ou 2 cases en ligne droite, même derrière un autre pion
//!   (carnet, 2026-10-30).
//! - Épique : n'importe quel pion adverse.
//! - Légendaire : le double échange. Le lanceur prend la place d'un pion adverse, n'importe
//!   où, puis un second de tes pions prêts prend celle d'un pion adverse collé au premier
//!   (carnet, 2026-11-02). Choix : le premier pion adverse, ton second pion, puis le second
//!   pion adverse.
//!
//! Le pion adverse peut être protégé (protégé veut dire « ne peut pas être tué ») ou
//! immobilisé : seul son propriétaire ne peut pas le déplacer.

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::{around, in_line};
use super::rules::{AttackContext, AttackRules};
use crate::{Cell, EffectEnd, EffectKind, Position, Rarity};

/// La portée de la Rare, en cases.
const RARE_REACH: usize = 2;

pub(crate) struct Interchange;

impl AttackRules for Interchange {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    /// L'Épique et la Légendaire visent n'importe où ; la Rare vise depuis le lanceur, et la
    /// Commune dépend de lui (pas de second départ avant son retour).
    fn ignores_launcher(&self, rarity: Rarity) -> bool {
        matches!(rarity, Rarity::Epic | Rarity::Legendary)
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Legendary => 3..=3,
            _ => 1..=1,
        }
    }

    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        let board = context.board;
        match context.rarity {
            Rarity::Common if !awaits_return(context) => board
                .positions()
                .filter(|&pos| board.is_free(pos))
                .collect(),
            Rarity::Common => Vec::new(),
            Rarity::Rare => in_line(board, context.launcher, RARE_REACH)
                .filter(|&pos| context.is_enemy(pos))
                .collect(),
            Rarity::Epic => board
                .positions()
                .filter(|&pos| context.is_enemy(pos))
                .collect(),
            Rarity::Legendary => match *chosen {
                // Un pion adverse collé à un autre pion adverse.
                [] => board
                    .positions()
                    .filter(|&pos| {
                        context.is_enemy(pos) && around(board, pos).any(|n| context.is_enemy(n))
                    })
                    .collect(),
                // Ton second pion : prêt, autre que le lanceur.
                [_] => board
                    .positions()
                    .filter(|&pos| {
                        pos != context.launcher
                            && board
                                .get(pos)
                                .and_then(Cell::pion)
                                .is_some_and(|pion| context.can_act(pion))
                    })
                    .collect(),
                // Le second pion adverse, collé au premier.
                [first, _] => around(board, first)
                    .filter(|&pos| context.is_enemy(pos))
                    .collect(),
                _ => Vec::new(),
            },
        }
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let launcher = context.launcher;
        match (context.rarity, targets) {
            (Rarity::Common, &[to]) => vec![
                Effect::Move { from: launcher, to },
                Effect::Attach {
                    at: to,
                    kind: EffectKind::Return { origin: launcher },
                    end: EffectEnd::NextTurn,
                },
            ],
            (Rarity::Legendary, &[first, mine, second]) => vec![
                Effect::Swap {
                    a: launcher,
                    b: first,
                },
                Effect::Swap { a: mine, b: second },
            ],
            (Rarity::Rare | Rarity::Epic, &[target]) => vec![Effect::Swap {
                a: launcher,
                b: target,
            }],
            _ => Vec::new(),
        }
    }
}

/// Vrai si le lanceur attend déjà le retour d'une Commune.
fn awaits_return(context: &AttackContext) -> bool {
    context
        .board
        .get(context.launcher)
        .and_then(Cell::pion)
        .is_some_and(|pion| {
            context.effects.iter().any(|effect| {
                effect.pion() == pion.id() && matches!(effect.kind(), EffectKind::Return { .. })
            })
        })
}
