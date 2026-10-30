//! L'Interchange (fiche n°06) : le lanceur échange sa case avec celle d'un pion adverse.
//!
//! - Rare : un pion adverse à 1 ou 2 cases en ligne droite, même derrière un autre pion
//!   (carnet, 2026-10-30).
//! - Épique : n'importe quel pion adverse.
//!
//! Le pion adverse peut être protégé (protégé veut dire « ne peut pas être tué ») ou
//! immobilisé : seul son propriétaire ne peut pas le déplacer. La Commune (téléportation
//! avec retour) et la Légendaire viendront avec les retours programmés (B6).

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::in_line;
use super::rules::{AttackContext, AttackRules};
use crate::{Position, Rarity};

/// La portée de la Rare, en cases.
const RARE_REACH: usize = 2;

pub(crate) struct Interchange;

impl AttackRules for Interchange {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Rare, Rarity::Epic]
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        1..=1
    }

    fn candidate_targets(&self, context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        match context.rarity {
            Rarity::Rare => in_line(context.board, context.launcher, RARE_REACH)
                .filter(|&pos| context.is_enemy(pos))
                .collect(),
            // L'Épique ; la Commune et la Légendaire ne sont jamais jouées ici (le catalogue
            // validé n'accepte que les raretés de `rarities`).
            Rarity::Common | Rarity::Epic | Rarity::Legendary => context
                .board
                .positions()
                .filter(|&pos| context.is_enemy(pos))
                .collect(),
        }
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        targets
            .iter()
            .map(|&target| Effect::Swap {
                a: context.launcher,
                b: target,
            })
            .collect()
    }
}
