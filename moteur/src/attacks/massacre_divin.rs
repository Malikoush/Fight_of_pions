//! Le Massacre divin (fiche n°08) : tes prochains kills rapportent.
//!
//! - Commune : ton prochain kill te rend 2 points.
//! - Rare : tes 4 prochains kills te rendent 1 point chacun.
//! - Épique : tes 2 prochains kills envoient le pion au cimetière au lieu de l'infirmerie.
//! - Légendaire : tes 3 prochains kills l'envoient au cimetière.
//!
//! La carte pose une pile de jetons devant toi ; elle se lance, comme Attila, depuis
//! n'importe lequel de tes pions prêts, sans cible. Les piles s'additionnent : chaque kill
//! use un jeton de chacune (carnet, 2026-11-02). Ce que fait un jeton vit dans la partie
//! (`game/tokens.rs`).

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{Position, Rarity, TokenReward};

/// Les jetons de la Commune.
const COMMON_TOKENS: u32 = 1;
/// Les points rendus par le jeton de la Commune.
const COMMON_POINTS: u32 = 2;
/// Les points rendus par chaque jeton de la Rare : 1 seul, pour qu'un kill ne devienne
/// jamais gratuit (fiche).
const RARE_POINTS: u32 = 1;
/// Les jetons de la Rare.
const RARE_TOKENS: u32 = 4;
/// Les jetons de l'Épique, qui envoient au cimetière.
const EPIC_TOKENS: u32 = 2;
/// Les jetons de la Légendaire, qui envoient au cimetière.
const LEGENDARY_TOKENS: u32 = 3;

pub(crate) struct MassacreDivin;

impl AttackRules for MassacreDivin {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    fn ignores_launcher(&self, _rarity: Rarity) -> bool {
        true
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        0..=0
    }

    fn candidate_targets(&self, _context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        Vec::new()
    }

    fn effects(&self, context: &AttackContext, _targets: &[Position]) -> Vec<Effect> {
        let (reward, tokens) = match context.rarity {
            Rarity::Common => (TokenReward::Points(COMMON_POINTS), COMMON_TOKENS),
            Rarity::Rare => (TokenReward::Points(RARE_POINTS), RARE_TOKENS),
            Rarity::Epic => (TokenReward::Graveyard, EPIC_TOKENS),
            Rarity::Legendary => (TokenReward::Graveyard, LEGENDARY_TOKENS),
        };
        vec![Effect::AddTokens { reward, tokens }]
    }
}
