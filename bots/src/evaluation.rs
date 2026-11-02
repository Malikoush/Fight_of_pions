//! L'évaluation : ce que vaut une partie pas encore finie, pour chaque joueur. Le MCTS
//! s'en sert pour juger les positions au bout de sa recherche, et pour choisir les coups
//! qu'il essaie d'abord.
//!
//! Elle ne lit aucune règle : seulement le plateau, par les fenêtres, et les compteurs que
//! le moteur expose. Chaque critère a un poids ([`Weights`]) : on ne devine pas les bons,
//! on les règlera dans l'arène en faisant jouer des poids différents les uns contre les
//! autres (cible : l'arène ne compare pas encore deux jeux de poids).
//!
//! Les points sont des entiers ; les chances de gagner n'utilisent ensuite que +, −, × et
//! ÷ en flottant, exacts selon la norme IEEE : le résultat est le même sur toutes les
//! machines, et une partie entre bots se rejoue à l'identique d'un ordinateur à l'autre.

use crate::Windows;
use fight_of_pions::{Board, Game, Player, Position};

/// Les poids des critères de l'évaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Weights {
    /// Ce qu'une fenêtre ouverte rapporte à son seul occupant : cette base, élevée à la
    /// puissance du nombre de ses pions dedans. Une fenêtre où les deux joueurs ont un pion
    /// ne rapporte rien : aucune ligne ne peut s'y faire tant qu'on n'en retire pas un.
    pub growth: u64,
    /// La prime d'une **double menace** : au moins deux fenêtres à qui il ne manque qu'un
    /// pion, sur deux cases différentes. Comptée en fenêtres « à qui il manque un pion » :
    /// avec 4, une double menace vaut quatre fois une telle fenêtre en plus.
    pub double_threat: u64,
    /// Ce que vaut une action disponible : gardée en banque, ou restante dans le tour en
    /// cours. Les actions servent aux attaques et aux bonus de la partie.
    pub action: u64,
    /// L'avance en points qui vaut 3 chances sur 4 de gagner. Plus elle est grande, plus
    /// l'évaluation reste prudente face à une avance.
    pub scale: u64,
}

impl Weights {
    /// Les poids de départ, choisis pour un plateau de 10 × 10 où il faut aligner 6 pions :
    /// un point de départ, à régler dans l'arène.
    pub const DEFAULT: Weights = Weights {
        growth: 5,
        double_threat: 4,
        action: 100,
        scale: 2000,
    };

    /// Les chances de gagner de `player`, entre 0 et 1 : exactes si la partie est finie
    /// (1, 0, ou 1/2 pour un nul), estimées d'après ses points et ceux de l'adversaire
    /// sinon.
    pub fn win_chance(&self, game: &Game, windows: &Windows, player: Player) -> f64 {
        if game.end().is_some() {
            return match game.winner() {
                Some(winner) if winner == player => 1.0,
                Some(_) => 0.0,
                None => 0.5,
            };
        }
        let points = self.points(game, windows);
        let lead = points[player.index()] as f64 - points[player.opponent().index()] as f64;
        // Une avance égale à `scale` donne 3/4 ; les chances tendent vers 0 ou 1 sans les
        // atteindre : seule une partie finie est sûre.
        0.5 + 0.5 * lead / (lead.abs() + self.scale as f64).max(1.0)
    }

    /// Les points de chaque joueur, rangés par [`Player::index`] : ses fenêtres ouvertes,
    /// sa double menace s'il en a une, et ses actions disponibles.
    pub fn points(&self, game: &Game, windows: &Windows) -> [u64; 2] {
        let board = game.board();
        let length = u32::try_from(game.config().align_to_win).unwrap_or(u32::MAX);
        let mut points = [0_u64; 2];
        // Les cases qui finiraient une fenêtre de chaque joueur en un seul pion.
        let mut finishing_cells: [Vec<Position>; 2] = [Vec::new(), Vec::new()];
        for window in windows.iter() {
            let pions = pions_in(board, window);
            for player in [Player::One, Player::Two] {
                let own = pions[player.index()];
                let opponent = pions[player.opponent().index()];
                let total = &mut points[player.index()];
                *total = total.saturating_add(self.window_points(own, opponent));
                if opponent == 0 && own.saturating_add(1) == length {
                    let cells = &mut finishing_cells[player.index()];
                    for &pos in window {
                        if board.owner(pos).is_none() && !cells.contains(&pos) {
                            cells.push(pos);
                        }
                    }
                }
            }
        }
        let missing_one = self.growth.saturating_pow(length.saturating_sub(1));
        for player in [Player::One, Player::Two] {
            let total = &mut points[player.index()];
            if finishing_cells[player.index()].len() >= 2 {
                *total = total.saturating_add(self.double_threat.saturating_mul(missing_one));
            }
            let actions = u64::from(available_actions(game, player));
            *total = total.saturating_add(self.action.saturating_mul(actions));
        }
        points
    }

    /// Ce que `player` gagnerait à poser sur chaque case, d'après ses fenêtres : ce qu'il
    /// ajoute aux siennes, plus ce qu'il retire à l'adversaire en entrant dans les
    /// siennes. Un seul passage sur les fenêtres, pour trier toutes les poses à la fois
    /// sans les jouer.
    pub fn placement_gains(
        &self,
        board: &Board,
        windows: &Windows,
        player: Player,
    ) -> PlacementGains {
        let size = board.size();
        let mut gains = PlacementGains {
            size,
            gains: vec![0_u64; size * size],
        };
        for window in windows.iter() {
            let pions = pions_in(board, window);
            let own = pions[player.index()];
            let opponent = pions[player.opponent().index()];
            let built = self
                .window_points(own.saturating_add(1), opponent)
                .saturating_sub(self.window_points(own, opponent));
            let blocked = self.window_points(opponent, own);
            let gain = built.saturating_add(blocked);
            for &pos in window {
                if board.owner(pos).is_none()
                    && let Some(cell) = gains.cell_mut(pos)
                {
                    *cell = cell.saturating_add(gain);
                }
            }
        }
        gains
    }

    /// Ce que rapporte une fenêtre à un joueur qui y a `own` pions, face à `opponent` pions
    /// adverses.
    fn window_points(&self, own: u32, opponent: u32) -> u64 {
        if own == 0 || opponent > 0 {
            0
        } else {
            self.growth.saturating_pow(own)
        }
    }
}

/// Ce qu'une pose rapporterait sur chaque case : voir [`Weights::placement_gains`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementGains {
    size: usize,
    /// Rangés par case, rangée par rangée.
    gains: Vec<u64>,
}

impl PlacementGains {
    /// Le gain d'une pose sur cette case ; 0 pour une case occupée, hors de toute fenêtre
    /// ou hors du plateau.
    pub fn at(&self, pos: Position) -> u64 {
        self.index(pos)
            .and_then(|index| self.gains.get(index))
            .copied()
            .unwrap_or(0)
    }

    fn cell_mut(&mut self, pos: Position) -> Option<&mut u64> {
        let index = self.index(pos)?;
        self.gains.get_mut(index)
    }

    /// La seule conversion case → rang dans le tableau.
    fn index(&self, pos: Position) -> Option<usize> {
        (pos.row < self.size && pos.col < self.size).then(|| pos.row * self.size + pos.col)
    }
}

/// Les pions de chaque joueur dans cette fenêtre, rangés par [`Player::index`].
fn pions_in(board: &Board, window: &[Position]) -> [u32; 2] {
    let mut pions = [0_u32; 2];
    for &pos in window {
        if let Some(owner) = board.owner(pos) {
            pions[owner.index()] += 1;
        }
    }
    pions
}

/// Les actions dont `player` dispose : sa banque, plus ce qu'il lui reste dans le tour en
/// cours si c'est à lui de jouer.
fn available_actions(game: &Game, player: Player) -> u32 {
    let this_turn = if game.current_player() == player {
        game.actions_left()
    } else {
        0
    };
    game.banked_actions(player).saturating_add(this_turn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target_rules;
    use fight_of_pions::{Action, GameConfig};

    /// Seules les fenêtres comptent.
    const WINDOWS_ONLY: Weights = Weights {
        double_threat: 0,
        action: 0,
        ..Weights::DEFAULT
    };

    fn at(row: usize, col: usize) -> Position {
        Position { row, col }
    }

    /// Un morpion dans les règles cibles sans la règle d'or : une ligne faite en posant
    /// gagne aussitôt.
    fn morpion() -> Game {
        let config = GameConfig {
            board_size: 3,
            align_to_win: 3,
            pions_per_player: 5,
            golden_rule: false,
            ..target_rules()
        };
        Game::new(config, 0).unwrap()
    }

    fn windows_of(game: &Game) -> Windows {
        Windows::new(game.board(), game.config().align_to_win)
    }

    /// Chaque joueur pose à son tour un de ces pions, puis finit son tour.
    fn alternate(game: &mut Game, cells: &[Position]) {
        for &pos in cells {
            let player = game.current_player();
            game.apply(Action::Place(pos)).unwrap();
            if game.end().is_none() && game.current_player() == player {
                game.apply(Action::EndTurn).unwrap();
            }
        }
    }

    #[test]
    fn an_empty_board_is_even() {
        let game = morpion();
        let windows = windows_of(&game);
        assert_eq!(WINDOWS_ONLY.points(&game, &windows), [0, 0]);
        assert_eq!(WINDOWS_ONLY.win_chance(&game, &windows, Player::One), 0.5);
    }

    #[test]
    fn open_windows_count_and_shared_ones_do_not() {
        let mut game = morpion();
        alternate(&mut game, &[at(1, 1), at(0, 1)]); // J1 au centre, J2 en B1
        let windows = windows_of(&game);
        // Le centre ouvre 4 fenêtres, dont la colonne B que J2 ferme. B1 n'ouvre plus
        // que la première rangée.
        assert_eq!(WINDOWS_ONLY.points(&game, &windows), [3 * 5, 5]);
        assert!(WINDOWS_ONLY.win_chance(&game, &windows, Player::One) > 0.5);
        assert!(WINDOWS_ONLY.win_chance(&game, &windows, Player::Two) < 0.5);
    }

    #[test]
    fn a_double_threat_needs_two_different_finishing_cells() {
        let with_bonus = Weights {
            double_threat: 4,
            ..WINDOWS_ONLY
        };
        let bonus_of = |game: &Game| {
            let windows = windows_of(game);
            with_bonus.points(game, &windows)[0] - WINDOWS_ONLY.points(game, &windows)[0]
        };
        let mut game = morpion();
        // J1 a A1 et C1 : seul B1 finit une ligne.
        alternate(&mut game, &[at(0, 0), at(2, 2), at(0, 2), at(1, 2)]);
        assert_eq!(bonus_of(&game), 0);
        // A3 ajoute A2 (colonne A) et B2 (diagonale) : J2 ne peut plus tout bloquer.
        alternate(&mut game, &[at(2, 0)]);
        assert_eq!(bonus_of(&game), 4 * 5 * 5);
    }

    #[test]
    fn available_actions_are_worth_points() {
        let mut game = morpion();
        let windows = windows_of(&game);
        let points = |game: &Game| Weights::DEFAULT.points(game, &windows);
        let action = Weights::DEFAULT.action;
        // J1 a l'action de son tour ; J2 a la Pièce en banque.
        assert_eq!(points(&game), [action, action]);
        game.apply(Action::EndTurn).unwrap();
        // J1 a gardé son action ; J2 a la Pièce et l'action de son tour.
        assert_eq!(points(&game), [action, 2 * action]);
    }

    #[test]
    fn a_finished_game_is_certain() {
        let mut game = morpion();
        alternate(
            &mut game,
            &[at(0, 0), at(1, 0), at(0, 1), at(1, 1), at(0, 2)],
        );
        assert_eq!(game.winner(), Some(Player::One));
        let windows = windows_of(&game);
        assert_eq!(WINDOWS_ONLY.win_chance(&game, &windows, Player::One), 1.0);
        assert_eq!(WINDOWS_ONLY.win_chance(&game, &windows, Player::Two), 0.0);
    }

    #[test]
    fn a_placement_gain_is_the_change_it_makes_to_the_windows() {
        let mut game = morpion();
        alternate(&mut game, &[at(0, 0), at(1, 0), at(2, 2), at(1, 1)]);
        let windows = windows_of(&game);
        let player = game.current_player();
        let [own, opponent] = [player.index(), player.opponent().index()];
        let gains = WINDOWS_ONLY.placement_gains(game.board(), &windows, player);
        let before = WINDOWS_ONLY.points(&game, &windows);
        for pos in game.board().positions() {
            let gain = gains.at(pos);
            if game.board().owner(pos).is_some() {
                assert_eq!(gain, 0, "{pos}");
                continue;
            }
            let mut copy = game.clone();
            copy.apply(Action::Place(pos)).unwrap();
            let after = WINDOWS_ONLY.points(&copy, &windows);
            let change = (after[own] - before[own]) + (before[opponent] - after[opponent]);
            assert_eq!(gain, change, "{pos}");
        }
    }
}
