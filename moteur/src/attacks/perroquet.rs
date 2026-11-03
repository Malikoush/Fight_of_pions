//! Le Perroquet (fiche R4) : une réaction. Quand l'adversaire joue une attaque, une fois
//! son attaque terminée, tu la rejoues aussitôt, dans la même rareté, comme si c'était ta
//! carte : ton lanceur (posé avant ce tour, non immobilisé), tes cibles, ta réserve, tes
//! effets.
//!
//! - Commune : rejoue une attaque adverse de 1 point.
//! - Rare : rejoue une attaque adverse de 2 points ou moins.
//!
//! La limite se lit sur le coût de l'attaque. Seulement les cartes : ni le tir d'un garde,
//! ni une réaction (carnet, « Réactions »). Le Perroquet se paie et se recharge comme une
//! carte ; la partie fait jouer l'attaque copiée à sa place (`Game::played_attack`, pour
//! toute réaction déclenchée par une attaque), ses autres règles ne servent donc jamais.

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules, Trigger};
use crate::{Position, Rarity};

/// Le coût le plus haut d'une attaque que la Commune rejoue.
const COMMON_COST_LIMIT: u32 = 1;
/// Le coût le plus haut d'une attaque que la Rare rejoue.
const RARE_COST_LIMIT: u32 = 2;

pub(crate) struct Perroquet;

impl AttackRules for Perroquet {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common, Rarity::Rare]
    }

    fn trigger(&self) -> Option<Trigger> {
        Some(Trigger::Attack)
    }

    fn attack_cost_limit(&self, rarity: Rarity) -> Option<u32> {
        Some(if rarity == Rarity::Rare {
            RARE_COST_LIMIT
        } else {
            COMMON_COST_LIMIT
        })
    }

    // Jamais appelées : c'est l'attaque copiée qui se joue.
    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        0..=0
    }

    fn candidate_targets(&self, _context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        Vec::new()
    }

    fn effects(&self, _context: &AttackContext, _targets: &[Position]) -> Vec<Effect> {
        Vec::new()
    }
}
