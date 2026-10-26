//! Outils partagés par les tests d'intégration du moteur.
#![allow(dead_code)] // chaque fichier de tests n'utilise qu'une partie de ces outils

use fight_of_pions::{Action, Event, Game, GameConfig, Pion, Position};

/// L'action « poser un pion » sur la case (rangée, colonne).
pub fn place(row: usize, col: usize) -> Action {
    Action::Place(Position { row, col })
}

/// Une partie avec la configuration par défaut.
pub fn new_game() -> Game {
    Game::new(GameConfig::default()).unwrap()
}

/// Joue une suite d'actions qui doivent toutes réussir ;
/// renvoie les événements de la dernière action.
pub fn play(game: &mut Game, actions: &[Action]) -> Vec<Event> {
    let mut last = Vec::new();
    for &action in actions {
        last = game.apply(action).unwrap();
    }
    last
}

/// Le pion annoncé par le premier événement d'une pose.
pub fn placed_pion(events: &[Event]) -> Pion {
    match events.first() {
        Some(Event::PionPlaced { pion, .. }) => *pion,
        other => panic!("une pose devait être annoncée, reçu : {other:?}"),
    }
}

/// Petit générateur pseudo-aléatoire déterministe (xorshift), pour jouer des parties
/// au hasard dans les tests. Sera remplacé par le `Rng` du moteur en T5.
pub struct TestRng(u64);

impl TestRng {
    pub fn new(seed: u64) -> TestRng {
        TestRng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// Un indice au hasard dans `0..len` (`len` > 0).
    pub fn below(&mut self, len: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % len as u64) as usize
    }
}
