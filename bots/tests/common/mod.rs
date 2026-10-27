//! Outils partagés par les tests d'intégration des bots.
#![allow(dead_code)] // chaque fichier de tests n'utilise qu'une partie de ces outils

use fight_of_pions::{Action, Game, GameConfig, Position};

/// L'action « poser un pion » sur la case (rangée, colonne).
pub fn place(row: usize, col: usize) -> Action {
    Action::Place(Position { row, col })
}

/// Une partie avec la configuration par défaut, où chaque joueur a toutes les attaques.
pub fn duel() -> Game {
    Game::new(GameConfig::default().with_full_arsenals(), 0).unwrap()
}

/// Joue une suite d'actions qui doivent toutes réussir.
pub fn play(game: &mut Game, actions: &[Action]) {
    for &action in actions {
        game.apply(action).unwrap();
    }
}
