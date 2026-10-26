use crate::{Board, Direction, Player, Position};

fn line_length(board: &Board, pos: Position, axis: Direction) -> usize {
    match board.owner(pos) {
        None => 0,
        Some(player) => {
            1 + count_in_direction(board, pos, axis, player)
                + count_in_direction(board, pos, axis.opposite(), player)
        }
    }
}

fn count_in_direction(board: &Board, start: Position, dir: Direction, player: Player) -> usize {
    let mut count = 0;
    let mut current = start;
    while let Some(next) = board.neighbor(current, dir) {
        if board.owner(next) != Some(player) {
            break;
        }
        count += 1;
        current = next;
    }
    count
}

fn longest_line(board: &Board, pos: Position) -> usize {
    let mut longest = 0;
    for axis in Direction::AXES {
        let length = line_length(board, pos, axis);
        if length > longest {
            longest = length;
        }
    }
    longest
}

/// Vrai si le pion en `pos` fait partie d'une ligne d'au moins `target` pions du même joueur.
pub fn complete_line(board: &Board, pos: Position, target: usize) -> bool {
    longest_line(board, pos) >= target
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Pion, PionId};

    /// Pose un pion pour un test. Plante si la case est hors du plateau.
    /// L'identité du pion n'a pas d'importance pour l'alignement.
    fn put_pion(board: &mut Board, row: usize, col: usize, player: Player) {
        let pion = Pion::new(PionId::FIRST, player);
        board
            .set(Position { row, col }, Cell::with_pion(pion))
            .unwrap();
    }

    #[test]
    fn horizontal_line_of_six() {
        let mut board = Board::new(10);
        for col in 2..8 {
            put_pion(&mut board, 4, col, Player::One);
        }

        assert_eq!(longest_line(&board, Position { row: 4, col: 4 }), 6);
    }

    #[test]
    fn diagonal_line_of_six() {
        let mut board = Board::new(10);
        for i in 1..7 {
            put_pion(&mut board, i, i, Player::One);
        }

        assert_eq!(longest_line(&board, Position { row: 3, col: 3 }), 6);
    }

    #[test]
    fn opponent_pion_cuts_the_line() {
        let mut board = Board::new(10);
        for col in 1..4 {
            put_pion(&mut board, 4, col, Player::One);
        }
        put_pion(&mut board, 4, 4, Player::Two);
        for col in 5..7 {
            put_pion(&mut board, 4, col, Player::One);
        }

        assert_eq!(longest_line(&board, Position { row: 4, col: 2 }), 3);
    }

    #[test]
    fn line_stops_at_the_edge() {
        let mut board = Board::new(10);
        for col in 5..10 {
            put_pion(&mut board, 0, col, Player::One);
        }

        assert_eq!(longest_line(&board, Position { row: 0, col: 9 }), 5);
    }
}
