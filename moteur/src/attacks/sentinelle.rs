//! La Sentinelle (fiche R2) : une réaction. Quand l'adversaire pose un pion (sa pose
//! normale, ou une carte qui pose), tu poses un pion de ta réserve juste à côté.
//!
//! - Commune : tu poses un pion sur une case libre collée au sien.
//! - Rare : la même pose, et son pion est immobilisé 2 tours.
//!
//! Le lanceur est le pion adverse posé, auquel on répond (carnet, « Réactions ») : une
//! attaque qui en pose plusieurs n'ouvre qu'une fenêtre, et l'on choisit lequel. Le pion
//! posé vient de la réserve ; impossible si elle est vide.

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::around;
use super::rules::{AttackContext, AttackRules, Trigger};
use crate::{EffectKind, Position, Rarity};

/// La durée de l'immobilisation de la Rare.
const RARE_IMMOBILIZED_TURNS: u32 = 2;

pub(crate) struct Sentinelle;

impl AttackRules for Sentinelle {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common, Rarity::Rare]
    }

    fn trigger(&self) -> Option<Trigger> {
        Some(Trigger::Placement)
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        1..=1
    }

    /// Les cases libres collées au pion posé, si la réserve a un pion.
    fn candidate_targets(&self, context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        if context.reserve == 0 {
            return Vec::new();
        }
        around(context.board, context.launcher)
            .filter(|&pos| context.board.is_free(pos))
            .collect()
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let [at] = targets else {
            return Vec::new();
        };
        let mut effects = vec![Effect::Place { at: *at }];
        if context.rarity == Rarity::Rare {
            effects.push(Effect::Apply {
                at: context.launcher,
                kind: EffectKind::Immobilized,
                turns: RARE_IMMOBILIZED_TURNS,
            });
        }
        effects
    }
}
