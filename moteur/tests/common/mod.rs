//! Outils partagés par les tests d'intégration du moteur.
#![allow(dead_code)] // chaque fichier de tests n'utilise qu'une partie de ces outils

use fight_of_pions::{Action, Event, Game, GameConfig, Pion, Position, Rng};

/// L'action « poser un pion » sur la case (rangée, colonne).
pub fn place(row: usize, col: usize) -> Action {
    Action::Place(Position { row, col })
}

/// La graine utilisée par les tests qui ne s'intéressent pas au hasard.
pub const SEED: u64 = 0;

/// Une partie avec la configuration par défaut.
pub fn new_game() -> Game {
    game_with(GameConfig::default())
}

/// Une partie avec cette configuration, qui doit être valide.
pub fn game_with(config: GameConfig) -> Game {
    Game::new(config, SEED).unwrap()
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

/// Un indice tiré au hasard dans `0..len` (`len` > 0), avec le générateur du moteur.
pub fn random_below(rng: &mut Rng, len: usize) -> usize {
    let value = rng.below(len as u64).expect("len doit être positif");
    value as usize
}

/// Une configuration valide tirée au hasard, petite pour que les fins arrivent vite.
/// Elle couvre les cas où il faut économiser ses actions pour pouvoir poser.
pub fn random_config(rng: &mut Rng) -> GameConfig {
    let board_size = 2 + random_below(rng, 3);
    let actions_per_turn = 1 + random_below(rng, 2) as u32;
    let max_banked_actions = random_below(rng, 3) as u32;
    let config = GameConfig {
        board_size,
        align_to_win: 2 + random_below(rng, board_size - 1),
        pions_per_player: 1 + random_below(rng, 9) as u32,
        first_turn_actions: 1 + random_below(rng, 5) as u32,
        actions_per_turn,
        max_banked_actions,
        place_cost: 1 + random_below(rng, (actions_per_turn + max_banked_actions) as usize) as u32,
        max_turns: 1 + random_below(rng, 60) as u32,
    };
    assert_eq!(config.validate(), Ok(()), "{config:?}");
    config
}
