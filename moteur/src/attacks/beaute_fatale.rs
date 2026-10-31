//! La Beauté fatale (fiche n°01) : le charme fige les pions adverses, puis les séduit.
//!
//! - Commune : immobilise 1 pion adverse pendant 2 tours.
//! - Rare : immobilise 2 pions adverses pendant 2 tours.
//! - Épique : séduit 1 pion adverse.
//! - Légendaire : séduit 1 pion adverse n'importe où sur le plateau ; le nouveau pion est
//!   protégé jusqu'au prochain tour du joueur.
//!
//! La Commune, la Rare et l'Épique visent par le **regard** : en ligne droite, jusqu'à 2
//! cases, sans pion entre les deux.
//! **Charme fatal** : un pion visé par la Commune ou la Rare, déjà immobilisé (par n'importe
//! quelle carte), est tué ; s'il est protégé, il est seulement immobilisé à nouveau.
//! **Séduction** : le pion visé est tué, et un pion de ta réserve est posé à sa place
//! (carnet, 2026-10-30 : c'est un kill, pas un changement de camp). Impossible si ta
//! réserve est vide ou si la cible est protégée.

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::in_sight;
use super::rules::{AttackContext, AttackRules};
use crate::{EffectKind, Position, Rarity};

/// La portée du regard, en cases.
const SIGHT: usize = 2;
/// La durée de l'immobilisation.
const CHARM_TURNS: u32 = 2;
/// La protection du pion posé par la Légendaire : jusqu'au prochain tour du joueur, donc
/// pendant 1 tour de l'adversaire.
const SEDUCED_GUARD_TURNS: u32 = 1;

pub(crate) struct BeauteFatale;

impl AttackRules for BeauteFatale {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    /// Seule la Légendaire vise n'importe où ; les autres, par le regard du lanceur.
    fn ignores_launcher(&self, rarity: Rarity) -> bool {
        rarity == Rarity::Legendary
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Rare => 2..=2,
            Rarity::Common | Rarity::Epic | Rarity::Legendary => 1..=1,
        }
    }

    /// Les pions adverses visables, pas encore choisis : dans le regard du lanceur, ou
    /// n'importe où pour la Légendaire. La séduction ne vise qu'un pion qu'elle peut tuer,
    /// et seulement si ta réserve a un pion à poser.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        let seduces = matches!(context.rarity, Rarity::Epic | Rarity::Legendary);
        if seduces && context.reserve == 0 {
            return Vec::new();
        }
        let visible = |pos: &Position| {
            let fits = if seduces {
                context.can_kill(*pos)
            } else {
                context.is_enemy(*pos)
            };
            fits && !chosen.contains(pos)
        };
        if context.rarity == Rarity::Legendary {
            context.board.positions().filter(visible).collect()
        } else {
            in_sight(context.board, context.launcher, SIGHT)
                .filter(visible)
                .collect()
        }
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        targets
            .iter()
            .flat_map(|&at| match context.rarity {
                Rarity::Common | Rarity::Rare => vec![charm(context, at)],
                Rarity::Epic => vec![Effect::Kill { at }, Effect::Place { at }],
                Rarity::Legendary => vec![
                    Effect::Kill { at },
                    Effect::Place { at },
                    Effect::Apply {
                        at,
                        kind: EffectKind::Protected,
                        turns: SEDUCED_GUARD_TURNS,
                    },
                ],
            })
            .collect()
    }
}

/// Le charme de la Commune et de la Rare : immobiliser, ou tuer un pion déjà immobilisé
/// (charme fatal), sauf s'il est protégé.
fn charm(context: &AttackContext, at: Position) -> Effect {
    if context.has(at, EffectKind::Immobilized) && context.can_kill(at) {
        Effect::Kill { at }
    } else {
        Effect::Apply {
            at,
            kind: EffectKind::Immobilized,
            turns: CHARM_TURNS,
        }
    }
}
