//! L'Ours (fiche n°09) : le corps à corps, sur les 8 cases collées au lanceur.
//!
//! - Commune, le Grognement : les pions adverses collés au lanceur sont immobilisés 1 tour.
//! - Rare : tue 1 pion adverse collé au lanceur.
//! - Épique : tue jusqu'à 2 pions adverses collés au lanceur.
//! - Légendaire, le Rugissement : tue jusqu'à 2 pions adverses collés au lanceur ; les
//!   autres pions adverses collés à lui sont immobilisés 2 tours.
//!
//! Un pion protégé ne peut pas être choisi pour être tué ; il est immobilisé comme les autres.

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::around;
use super::rules::{AttackContext, AttackRules};
use crate::{EffectKind, Position, Rarity};

/// Durée de l'immobilisation du Grognement : la seule d'1 tour, car il fige toute la zone.
const GROWL_TURNS: u32 = 1;
/// Durée de l'immobilisation du Rugissement.
const ROAR_TURNS: u32 = 2;

pub(crate) struct Ours;

impl AttackRules for Ours {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Common => 0..=0,
            Rarity::Rare => 1..=1,
            Rarity::Epic => 1..=2,
            // Tuer « jusqu'à 2 » : même aucun, si l'on veut seulement immobiliser.
            Rarity::Legendary => 0..=2,
        }
    }

    /// Les pions adverses collés au lanceur qu'on peut tuer, pas encore choisis.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        around(context.board, context.launcher)
            .filter(|&pos| context.can_kill(pos) && !chosen.contains(&pos))
            .collect()
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let kills = targets.iter().map(|&at| Effect::Kill { at });
        let immobilize = |turns| {
            around(context.board, context.launcher)
                .filter(|pos| context.is_enemy(*pos) && !targets.contains(pos))
                .map(move |at| Effect::Apply {
                    at,
                    kind: EffectKind::Immobilized,
                    turns,
                })
        };
        match context.rarity {
            Rarity::Common => immobilize(GROWL_TURNS).collect(),
            Rarity::Rare | Rarity::Epic => kills.collect(),
            Rarity::Legendary => kills.chain(immobilize(ROAR_TURNS)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attacks::Launch;
    use crate::{Board, Pion, PionId, Player};

    fn put(board: &mut Board, row: usize, col: usize, player: Player) {
        let pion = Pion::new(PionId::FIRST, player);
        board.place_pion(Position { row, col }, pion).unwrap();
    }

    #[test]
    fn targets_are_the_enemies_on_the_eight_surrounding_cells() {
        let mut board = Board::new(5);
        put(&mut board, 2, 2, Player::One); // le lanceur
        put(&mut board, 1, 1, Player::Two); // diagonale : visée
        put(&mut board, 2, 3, Player::Two); // à côté : visée
        put(&mut board, 3, 2, Player::One); // allié : jamais visé
        put(&mut board, 0, 0, Player::Two); // trop loin

        let context = AttackContext {
            board: &board,
            effects: &[],
            token_piles: &[],
            player: Player::One,
            launcher: Position { row: 2, col: 2 },
            rarity: Rarity::Rare,
            launch: Launch::Now,
            sick_from: None,
            reserve: 0,
        };
        let mut targets = Ours.candidate_targets(&context, &[]);
        targets.sort_by_key(|pos| (pos.row, pos.col));
        assert_eq!(
            targets,
            vec![Position { row: 1, col: 1 }, Position { row: 2, col: 3 }]
        );

        let after_first = Ours.candidate_targets(&context, &[Position { row: 1, col: 1 }]);
        assert_eq!(after_first, vec![Position { row: 2, col: 3 }]);
    }
}
