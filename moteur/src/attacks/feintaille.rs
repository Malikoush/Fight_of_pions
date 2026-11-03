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
//! les autres pions visés sont touchés normalement. Le pion n'esquive pas vers une case
//! où l'attaque va arriver (l'arrivée d'un saut, la case où elle pose) : carnet,
//! 2026-11-03.

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::free_around;
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

    /// Les cases libres collées au pion visé, hors de celles où l'attaque va arriver.
    fn candidate_targets(&self, context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        free_around(context.board, context.launcher)
            .filter(|pos| !context.reserved.contains(pos))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attacks::Launch;
    use crate::{Board, Pion, PionId, Player};

    #[test]
    fn the_dodge_avoids_the_cells_where_the_attack_arrives() {
        let mut board = Board::new(3);
        let launcher = Position { row: 1, col: 1 };
        board
            .place_pion(launcher, Pion::new(PionId::FIRST, Player::Two))
            .unwrap();
        let reserved = [Position { row: 0, col: 0 }, Position { row: 2, col: 2 }];
        let context = AttackContext {
            board: &board,
            effects: &[],
            token_piles: &[],
            player: Player::Two,
            launcher,
            rarity: Rarity::Rare,
            launch: Launch::Now,
            sick_from: None,
            reserve: 0,
            may_create_effects: true,
            reserved: &reserved,
        };
        let cells = Feintaille.candidate_targets(&context, &[]);
        assert_eq!(cells.len(), 6);
        assert!(reserved.iter().all(|pos| !cells.contains(pos)));
    }
}
