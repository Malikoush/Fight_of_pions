//! Le Hibou (fiche R5) : une réaction. Quand l'adversaire déplace un pion, le sien ou un
//! des tiens (saut, poussée, charge, échange, téléportation, Lièvre, pas de la reine), tes
//! pions avancent d'un pas.
//!
//! - Commune : 1 de tes pions avance d'1 pas.
//! - Rare : jusqu'à 2 de tes pions, différents, avancent d'1 pas chacun.
//!
//! Un pas va vers une case libre collée (8 directions), comme le Lièvre, dont il reprend
//! les trajets : chaque pion est un lanceur, posé avant ce tour et non immobilisé (carnet,
//! « Réactions »).

use std::ops::RangeInclusive;

use super::Effect;
use super::lievre::Steps;
use super::rules::{AttackContext, AttackRules, Targets, Trigger};
use crate::{Position, Rarity};

pub(crate) struct Hibou;

/// Combien de pions avancent, d'un pas chacun.
fn limits(rarity: Rarity) -> Steps {
    let pions = if rarity == Rarity::Rare { 2 } else { 1 };
    Steps { pions, steps: 1 }
}

impl AttackRules for Hibou {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common, Rarity::Rare]
    }

    fn trigger(&self) -> Option<Trigger> {
        Some(Trigger::Move)
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        limits(rarity).target_count()
    }

    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        limits(context.rarity).candidate_targets(context, chosen)
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        limits(context.rarity).effects(context, targets)
    }

    fn can_stop(&self, context: &AttackContext, chosen: &Targets) -> bool {
        limits(context.rarity).can_stop(context, chosen)
    }
}
