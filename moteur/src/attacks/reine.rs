//! La Reine (fiche n°03).
//!
//! - Commune, le Garde royal : le lanceur est protégé pendant 2 tours. Comme toute carte
//!   qui agit sur un de tes pions, elle s'applique au lanceur (carnet, 2026-10-30).
//!
//! La Rare, l'Épique et la Légendaire posent une reine : elles viendront avec la pose (B5).

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{EffectKind, Position, Rarity};

/// La durée de la protection du Garde royal.
const ROYAL_GUARD_TURNS: u32 = 2;

pub(crate) struct Reine;

impl AttackRules for Reine {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common]
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        0..=0
    }

    fn candidate_targets(&self, _context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        Vec::new()
    }

    fn effects(&self, context: &AttackContext, _targets: &[Position]) -> Vec<Effect> {
        vec![Effect::Apply {
            at: context.launcher,
            kind: EffectKind::Protected,
            turns: ROYAL_GUARD_TURNS,
        }]
    }
}
