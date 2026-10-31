//! La Reine (fiche n°03).
//!
//! - Commune, le Garde royal : le lanceur est protégé pendant 2 tours. Comme toute carte
//!   qui agit sur un de tes pions, elle s'applique au lanceur (carnet, 2026-10-30).
//! - Rare : pose une reine.
//! - Épique : pose une reine avec 1 garde.
//! - Légendaire : pose une reine avec 2 gardes.
//!
//! La **reine** est un pion de ta réserve, posé n'importe où, en plus de ta pose normale,
//! avec une couronne. Une seule reine à la fois (carnet, 2026-10-31) : la carte n'est pas
//! jouable tant que ta reine est en vie, ni si ta réserve est vide. Son pas gratuit et le
//! tir de ses gardes sont des actions à part (`game/queen.rs`) ; le bouclier de ses gardes
//! agit dans le kill (`game/kill.rs`).

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::game::queen_of;
use crate::{EffectKind, Position, Rarity};

/// La durée de la protection du Garde royal.
const ROYAL_GUARD_TURNS: u32 = 2;

pub(crate) struct Reine;

/// Les gardes de la reine posée par cette rareté.
fn guards(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common | Rarity::Rare => 0,
        Rarity::Epic => 1,
        Rarity::Legendary => 2,
    }
}

impl AttackRules for Reine {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    /// Le Garde royal protège son lanceur ; la reine se pose n'importe où.
    fn ignores_launcher(&self, rarity: Rarity) -> bool {
        rarity != Rarity::Common
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Common => 0..=0,
            // La case de la reine.
            Rarity::Rare | Rarity::Epic | Rarity::Legendary => 1..=1,
        }
    }

    /// Les cases libres, n'importe où, si ta réserve a un pion et que tu n'as pas de reine.
    fn candidate_targets(&self, context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        if context.rarity == Rarity::Common || context.reserve == 0 || has_a_queen(context) {
            return Vec::new();
        }
        context
            .board
            .positions()
            .filter(|&pos| context.board.is_free(pos))
            .collect()
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        if context.rarity == Rarity::Common {
            return vec![Effect::Apply {
                at: context.launcher,
                kind: EffectKind::Protected,
                turns: ROYAL_GUARD_TURNS,
            }];
        }
        targets
            .iter()
            .flat_map(|&at| {
                [
                    Effect::Place { at },
                    Effect::Crown {
                        at,
                        guards: guards(context.rarity),
                    },
                ]
            })
            .collect()
    }
}

/// Vrai si le joueur qui attaque a déjà une reine sur le plateau.
fn has_a_queen(context: &AttackContext) -> bool {
    queen_of(context.board, context.effects, context.player).is_some()
}
