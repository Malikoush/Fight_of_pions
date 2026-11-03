//! La Feintaille (fiche R1) : une réaction. Quand une attaque adverse vise un de tes pions,
//! juste avant qu'elle ne s'applique, ton pion visé esquive : il se déplace d'une case vers
//! une case libre collée.
//!
//! - Commune : contre une attaque de 2 points ou moins.
//! - Rare : contre n'importe quelle attaque.
//!
//! « Visé » : choisi comme cible de l'attaque (carnet, « Réactions »). Le lanceur est le
//! pion visé, qui doit pouvoir agir (non immobilisé, posé avant ce tour). L'attaque
//! s'applique ensuite sur les cases choisies : le pion n'y est plus, elle ne le touche pas ;
//! les autres pions visés sont touchés normalement.

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::around;
use super::rules::{AttackContext, AttackRules, Trigger};
use crate::{Position, Rarity};

/// Le coût le plus haut d'une attaque que la Commune esquive.
const COMMON_COST_LIMIT: u32 = 2;

pub(crate) struct Feintaille;

impl AttackRules for Feintaille {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common, Rarity::Rare]
    }

    fn trigger(&self) -> Option<Trigger> {
        Some(Trigger::Targeted)
    }

    fn attack_cost_limit(&self, rarity: Rarity) -> Option<u32> {
        (rarity == Rarity::Common).then_some(COMMON_COST_LIMIT)
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        1..=1
    }

    /// Les cases libres collées au pion visé.
    fn candidate_targets(&self, context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        around(context.board, context.launcher)
            .filter(|&pos| context.board.is_free(pos))
            .collect()
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let [to] = targets else {
            return Vec::new();
        };
        vec![Effect::Move {
            from: context.launcher,
            to: *to,
        }]
    }
}
