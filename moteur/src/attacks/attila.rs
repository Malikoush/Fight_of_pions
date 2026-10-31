//! Attila (fiche n°04) : le contre des effets en cours, sur tout le plateau.
//!
//! - Commune : retire d'un pion tous les statuts placés par l'adversaire (immobilisé,
//!   protégé), quelle que soit la carte qui les a posés. Le pion peut être à toi (libérer
//!   un pion immobilisé) ou à l'adversaire (lui ôter sa protection).
//!
//! La Rare, l'Épique et la Légendaire annulent des effets entiers : elles viendront avec
//! les jetons et l'annulation (B6).

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{Cell, Position, Rarity};

pub(crate) struct Attila;

impl AttackRules for Attila {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common]
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        1..=1
    }

    /// Les pions, n'importe où, qui portent un statut posé par l'adversaire.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        let enemy = context.player.opponent();
        context
            .board
            .positions()
            .filter(|pos| !chosen.contains(pos))
            .filter(|&pos| {
                context
                    .board
                    .get(pos)
                    .and_then(Cell::pion)
                    .is_some_and(|pion| {
                        context
                            .effects
                            .iter()
                            .any(|effect| effect.is_status_on_by(pion.id(), enemy))
                    })
            })
            .collect()
    }

    fn effects(&self, _context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        targets
            .iter()
            .map(|&at| Effect::RemoveEnemyStatuses { at })
            .collect()
    }
}
