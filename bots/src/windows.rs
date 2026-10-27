use fight_of_pions::{Board, Direction, Player, Position};
use std::iter;

/// Toutes les fenêtres d'un plateau : chaque suite de `length` cases consécutives sur un
/// axe d'alignement, où `length` est la longueur de la ligne gagnante.
///
/// Une ligne gagnante couvre forcément une fenêtre entière d'un même joueur. Une fenêtre
/// où l'adversaire a un pion ne peut donc plus servir : c'est l'unité de mesure des bots,
/// qui lisent le plateau sans connaître les règles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Windows {
    windows: Vec<Vec<Position>>,
}

/// Ce que contient une fenêtre, vu par un joueur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowContent {
    /// Les pions du joueur.
    pub own: usize,
    /// Les pions de son adversaire.
    pub opponent: usize,
    /// Les cases libres, dans l'ordre de la fenêtre.
    pub empty: Vec<Position>,
}

impl Windows {
    /// Les fenêtres de ce plateau pour une ligne gagnante de `length` cases, dans un ordre
    /// toujours identique.
    pub fn new(board: &Board, length: usize) -> Windows {
        let windows = board
            .positions()
            .flat_map(|start| {
                Direction::AXES
                    .iter()
                    .filter_map(move |&axis| window_from(board, start, axis, length))
            })
            .collect();
        Windows { windows }
    }

    /// Les fenêtres, chacune comme la liste de ses cases.
    pub fn iter(&self) -> impl Iterator<Item = &[Position]> {
        self.windows.iter().map(Vec::as_slice)
    }

    /// Les fenêtres que `player` finirait en posant au plus `placements` pions : sans
    /// pion adverse, et avec au plus `placements` cases libres.
    pub fn finishable(
        &self,
        board: &Board,
        player: Player,
        placements: usize,
    ) -> Vec<WindowContent> {
        // Une fenêtre pleine d'un même joueur aurait déjà fini la partie : sans pose
        // possible, aucune ne peut être finie.
        if placements == 0 {
            return Vec::new();
        }
        self.iter()
            .map(|window| WindowContent::of(board, window, player))
            .filter(|content| content.opponent == 0 && content.empty.len() <= placements)
            .collect()
    }
}

impl WindowContent {
    /// Le contenu de cette fenêtre, vu par `player`.
    pub fn of(board: &Board, window: &[Position], player: Player) -> WindowContent {
        let mut content = WindowContent {
            own: 0,
            opponent: 0,
            empty: Vec::new(),
        };
        for &pos in window {
            match board.owner(pos) {
                Some(owner) if owner == player => content.own += 1,
                Some(_) => content.opponent += 1,
                None => content.empty.push(pos),
            }
        }
        content
    }
}

/// La fenêtre de `length` cases qui part de `start` dans la direction `axis`, si elle
/// tient dans le plateau.
fn window_from(
    board: &Board,
    start: Position,
    axis: Direction,
    length: usize,
) -> Option<Vec<Position>> {
    let cells: Vec<Position> = iter::successors(Some(start), |&pos| board.neighbor(pos, axis))
        .take(length)
        .collect();
    (cells.len() == length).then_some(cells)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fight_of_pions::{Action, Game, GameConfig};

    fn game_with(board_size: usize, align_to_win: usize) -> Game {
        let config = GameConfig {
            board_size,
            align_to_win,
            ..GameConfig::default()
        };
        Game::new(config, 0).unwrap()
    }

    #[test]
    fn a_10x10_board_has_150_windows_of_6() {
        // Par axe : 10 rangées × 5 départs, autant de colonnes, et 5 × 5 par diagonale.
        let game = game_with(10, 6);
        assert_eq!(Windows::new(game.board(), 6).iter().count(), 150);
    }

    #[test]
    fn a_3x3_board_has_the_8_tic_tac_toe_lines() {
        let game = game_with(3, 3);
        let windows = Windows::new(game.board(), 3);
        assert_eq!(windows.iter().count(), 8);
        assert!(windows.iter().all(|window| window.len() == 3));
    }

    #[test]
    fn content_counts_each_side_and_lists_the_empty_cells() {
        let mut game = game_with(3, 3);
        let at = |row, col| Position { row, col };
        game.apply(Action::Place(at(0, 0))).unwrap(); // joueur 1
        game.apply(Action::Place(at(0, 1))).unwrap(); // joueur 2

        let content = WindowContent::of(game.board(), &[at(0, 0), at(0, 1), at(0, 2)], Player::One);
        assert_eq!(
            content,
            WindowContent {
                own: 1,
                opponent: 1,
                empty: vec![at(0, 2)],
            }
        );
    }

    #[test]
    fn finishable_windows_have_no_enemy_and_few_empty_cells() {
        let mut game = game_with(3, 3);
        let at = |row, col| Position { row, col };
        game.apply(Action::Place(at(0, 0))).unwrap(); // joueur 1
        game.apply(Action::Place(at(1, 1))).unwrap(); // joueur 2, au centre

        let windows = Windows::new(game.board(), 3);
        // Avec 2 poses, le joueur 1 finirait la première rangée ou la première colonne ;
        // le centre lui ferme la diagonale, et les lignes vides demandent 3 poses.
        let open_from_a1 = |empty| WindowContent {
            own: 1,
            opponent: 0,
            empty,
        };
        assert_eq!(
            windows.finishable(game.board(), Player::One, 2),
            vec![
                open_from_a1(vec![at(0, 1), at(0, 2)]),
                open_from_a1(vec![at(1, 0), at(2, 0)]),
            ]
        );
        assert!(windows.finishable(game.board(), Player::One, 1).is_empty());
    }
}
