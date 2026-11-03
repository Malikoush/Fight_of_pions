//! La Queue du lézard (fiche R3) : une réaction. Quand un de tes pions meurt (infirmerie
//! ou cimetière, quelle que soit l'attaque), tu reposes tout de suite à côté.
//!
//! - Commune : tu poses 1 pion sur une case libre collée à la case du pion mort.
//! - Rare : jusqu'à 2 pions, sur des cases libres collées à la case du pion mort.
//!
//! Jamais sur la case du mort elle-même. Le lanceur est cette case (carnet,
//! « Réactions ») : si plusieurs de tes pions meurent dans le même coup, tu choisis autour
//! duquel tu poses. Les pions viennent de la réserve ; impossible si elle est vide.

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::around;
use super::rules::{AttackContext, AttackRules, Trigger};
use crate::{Position, Rarity};

pub(crate) struct QueueDuLezard;

/// Le plus de pions posés dans cette rareté.
fn most(rarity: Rarity) -> usize {
    if rarity == Rarity::Rare { 2 } else { 1 }
}

impl AttackRules for QueueDuLezard {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common, Rarity::Rare]
    }

    fn trigger(&self) -> Option<Trigger> {
        Some(Trigger::Death)
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        1..=most(rarity)
    }

    /// Les cases libres collées à la case du pion mort, pas encore choisies, tant que la
    /// réserve a un pion de plus.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        if usize::try_from(context.reserve).unwrap_or(usize::MAX) <= chosen.len() {
            return Vec::new();
        }
        around(context.board, context.launcher)
            .filter(|&pos| context.board.is_free(pos) && !chosen.contains(&pos))
            .collect()
    }

    fn effects(&self, _context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        targets.iter().map(|&at| Effect::Place { at }).collect()
    }
}
