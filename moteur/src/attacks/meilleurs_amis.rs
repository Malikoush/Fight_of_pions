//! Meilleurs amis (fiche n°12) : deux pions liés par un pacte ; si l'un est tué, l'autre
//! le venge.
//!
//! - Épique (version unique) : pose 2 pions de ta réserve n'importe où, liés par le pacte.
//!
//! Impossible avec moins de 2 pions en réserve : un pacte demande deux amis. Les deux pions
//! comptent comme posés ce tour et n'utilisent pas la pose normale. La vengeance a lieu à
//! la fin de l'attaque qui tue un des amis, dans le résolveur (`game/kill.rs`). Rejouer la
//! carte crée un autre pacte, indépendant (carnet, 2026-10-30).

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{Position, Rarity};

/// Le nombre d'amis liés par un pacte.
const FRIENDS: usize = 2;

pub(crate) struct MeilleursAmis;

impl AttackRules for MeilleursAmis {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Epic]
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        FRIENDS..=FRIENDS
    }

    /// Les cases libres, n'importe où, pas encore choisies, si la réserve a assez de pions.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        if usize::try_from(context.reserve).unwrap_or(usize::MAX) < FRIENDS {
            return Vec::new();
        }
        context
            .board
            .positions()
            .filter(|&pos| context.board.is_free(pos) && !chosen.contains(&pos))
            .collect()
    }

    fn effects(&self, _context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let [a, b] = targets else {
            return Vec::new();
        };
        vec![
            Effect::Place { at: *a },
            Effect::Place { at: *b },
            Effect::Bind { a: *a, b: *b },
        ]
    }
}
