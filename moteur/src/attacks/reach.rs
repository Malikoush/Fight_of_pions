//! Les portées partagées par les familles : où une attaque peut trouver ses cibles. Chaque
//! carte dit la sienne, il n'y a pas de portée par défaut (carnet, 2026-10-30).

use crate::{Board, Direction, Position};

/// Les cases collées à `from` : les 8 autour, diagonales comprises, dans le plateau.
pub(crate) fn around(board: &Board, from: Position) -> impl Iterator<Item = Position> + '_ {
    Direction::ALL
        .iter()
        .filter_map(move |&dir| board.neighbor(from, dir))
}

/// Le regard : les cases en ligne droite depuis `from` (8 directions), jusqu'à `distance`
/// cases, sans pion entre `from` et la case. Dans chaque direction, le regard s'arrête au
/// premier pion, qui en fait partie.
pub(crate) fn in_sight(
    board: &Board,
    from: Position,
    distance: usize,
) -> impl Iterator<Item = Position> + '_ {
    Direction::ALL.iter().flat_map(move |&dir| {
        let mut blocked = false;
        std::iter::successors(board.neighbor(from, dir), move |&pos| {
            board.neighbor(pos, dir)
        })
        .take(distance)
        .take_while(move |&pos| {
            let visible = !blocked;
            blocked = board.owner(pos).is_some();
            visible
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Pion, PionId, Player};

    fn at(row: usize, col: usize) -> Position {
        Position { row, col }
    }

    fn board_with(pions: &[Position]) -> Board {
        let mut board = Board::new(5);
        for &pos in pions {
            board
                .set(pos, Cell::with_pion(Pion::new(PionId::FIRST, Player::Two)))
                .unwrap();
        }
        board
    }

    fn sorted(cells: impl Iterator<Item = Position>) -> Vec<Position> {
        let mut cells: Vec<Position> = cells.collect();
        cells.sort_by_key(|pos| (pos.row, pos.col));
        cells
    }

    #[test]
    fn around_a_corner_there_are_three_cells() {
        let board = board_with(&[]);
        assert_eq!(
            sorted(around(&board, at(0, 0))),
            vec![at(0, 1), at(1, 0), at(1, 1)]
        );
    }

    #[test]
    fn sight_reaches_two_cells_and_stops_at_the_first_pion() {
        // Depuis C3 (2, 2) : un pion en C2 (1, 2) cache C1 ; un pion en E5 (4, 4) est vu.
        let board = board_with(&[at(1, 2), at(0, 2), at(4, 4)]);
        let seen = sorted(in_sight(&board, at(2, 2), 2));
        assert!(seen.contains(&at(1, 2)), "le premier pion est vu");
        assert!(!seen.contains(&at(0, 2)), "le pion derrière est caché");
        assert!(seen.contains(&at(4, 4)), "à 2 cases sans pion entre");
        assert!(seen.contains(&at(2, 0)) && seen.contains(&at(2, 4)));
        // 7 directions × 2 cases + la direction bloquée (1 case) = 15.
        assert_eq!(seen.len(), 15);
    }
}
