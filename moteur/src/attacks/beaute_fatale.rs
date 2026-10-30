//! La Beauté fatale (fiche n°01) : le charme fige les pions adverses.
//!
//! - Commune : immobilise 1 pion adverse pendant 2 tours.
//! - Rare : immobilise 2 pions adverses pendant 2 tours.
//!
//! Elle vise par le **regard** : en ligne droite, jusqu'à 2 cases, sans pion entre les deux.
//! **Charme fatal** : un pion visé déjà immobilisé (par n'importe quelle carte) est tué ;
//! s'il est protégé, il est seulement immobilisé à nouveau.
//!
//! L'Épique et la Légendaire séduisent (tuer puis poser) : elles viendront avec la pose (B5).

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::in_sight;
use super::rules::{AttackContext, AttackRules};
use crate::{EffectKind, Position, Rarity};

/// La portée du regard, en cases.
const SIGHT: usize = 2;
/// La durée de l'immobilisation.
const CHARM_TURNS: u32 = 2;

pub(crate) struct BeauteFatale;

impl AttackRules for BeauteFatale {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common, Rarity::Rare]
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Rare => 2..=2,
            Rarity::Common | Rarity::Epic | Rarity::Legendary => 1..=1,
        }
    }

    /// Les pions adverses dans le regard du lanceur, pas encore choisis.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        in_sight(context.board, context.launcher, SIGHT)
            .filter(|&pos| context.is_enemy(pos) && !chosen.contains(&pos))
            .collect()
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        targets
            .iter()
            .map(|&at| {
                let fatal = context.has(at, EffectKind::Immobilized) && context.can_kill(at);
                if fatal {
                    Effect::Kill { at }
                } else {
                    Effect::Apply {
                        at,
                        kind: EffectKind::Immobilized,
                        turns: CHARM_TURNS,
                    }
                }
            })
            .collect()
    }
}
