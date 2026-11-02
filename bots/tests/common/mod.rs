//! Outils partagés par les tests d'intégration des bots.
#![allow(dead_code)] // chaque fichier de tests n'utilise qu'une partie de ces outils

use bots::Bot;
use fight_of_pions::{Action, Game, Position, Rng};

/// La case (rangée, colonne).
pub fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

/// L'action « poser un pion » sur la case (rangée, colonne).
pub fn place(row: usize, col: usize) -> Action {
    Action::Place(at(row, col))
}

/// Une partie avec les premières règles, où chaque joueur a toutes les attaques : les
/// scénarios comptent leurs actions (2 par tour, une pose coûte 1).
pub fn duel() -> Game {
    Game::new(bots::first_rules(), 0).unwrap()
}

/// Chaque joueur pose à son tour un de ces pions, puis finit son tour en gardant ses
/// actions, si la pose ne l'a pas déjà fini.
pub fn alternate(game: &mut Game, cells: &[Position]) {
    for &pos in cells {
        let player = game.current_player();
        game.apply(Action::Place(pos)).unwrap();
        if game.end().is_none() && game.current_player() == player {
            game.apply(Action::EndTurn).unwrap();
        }
    }
}

/// Laisse le bot jouer pour le joueur courant jusqu'à la fin de son tour (ou de la
/// partie), et rend les actions qu'il a jouées.
pub fn play_the_turn(bot: &dyn Bot, game: &mut Game) -> Vec<Action> {
    let player = game.current_player();
    let mut rng = Rng::new(0);
    let mut played = Vec::new();
    while game.end().is_none() && game.current_player() == player {
        let action = bot.choose(game, &mut rng).unwrap();
        game.apply(action).unwrap();
        played.push(action);
    }
    played
}

/// Joue une suite d'actions qui doivent toutes réussir.
pub fn play(game: &mut Game, actions: &[Action]) {
    for &action in actions {
        game.apply(action).unwrap();
    }
}
