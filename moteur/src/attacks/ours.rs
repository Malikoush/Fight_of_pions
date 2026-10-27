//! L'Ours (GDD n°09) : une attaque de corps à corps sur les 8 cases autour du lanceur.
//!
//! - Rare : élimine 1 ennemi adjacent.
//! - Épique : élimine 2 ennemis adjacents.
//!
//! La Commune et la Légendaire repoussent : elles viendront avec le déplacement (B4).

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{Direction, Position, Rarity};

pub(crate) struct Ours;

impl AttackRules for Ours {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Rare, Rarity::Epic]
    }

    fn target_count(&self, rarity: Rarity) -> usize {
        match rarity {
            Rarity::Epic => 2,
            Rarity::Common | Rarity::Rare | Rarity::Legendary => 1,
        }
    }

    /// Les ennemis sur les 8 cases autour du lanceur, pas encore choisis.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        let enemy = context.player.opponent();
        Direction::ALL
            .iter()
            .filter_map(|&dir| context.board.neighbor(context.launcher, dir))
            .filter(|pos| context.board.owner(*pos) == Some(enemy))
            .filter(|pos| !chosen.contains(pos))
            .collect()
    }

    fn effects(&self, _context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        targets.iter().map(|&at| Effect::Kill { at }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Board, Cell, Pion, PionId, Player};

    fn put(board: &mut Board, row: usize, col: usize, player: Player) {
        let pion = Pion::new(PionId::FIRST, player);
        board
            .set(Position { row, col }, Cell::with_pion(pion))
            .unwrap();
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
            player: Player::One,
            launcher: Position { row: 2, col: 2 },
            rarity: Rarity::Rare,
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
