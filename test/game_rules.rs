//! Tests d'intégration : on utilise le moteur comme le ferait l'interface.

use fight_of_pions::action::Action;
use fight_of_pions::config::GameConfig;
use fight_of_pions::error::GameError;
use fight_of_payon::game::{Game, Outcome};
use fight_of_pions::player::Player;
use fight_of_pions::position::Position;

/// Raccourci pour une action « poser en (ligne, colonne) ».
fn place(row: usize, col: usize) -> Action {
    Action::Place(Position { row, col })
}

fn new_game() -> Game {
    Game::new(GameConfig::default()).unwrap()
}

/// Joue une suite d'actions qui doivent toutes réussir ; renvoie le dernier résultat.
fn play(game: &mut Game, actions: &[Action]) -> Outcome {
    let mut last = Outcome::Continue;
    for &action in actions {
        last = game.apply(action).unwrap();
    }
    last
}

#[test]
fn first_turn_has_one_action_then_two() {
    let mut game = new_game();
    assert_eq!(game.actions_left(), 1);

    let outcome = play(&mut game, &[place(0, 0)]);
    assert_eq!(outcome, Outcome::TurnEnded { next: Player::Two });
    assert_eq!(game.actions_left(), 2);
}

#[test]
fn six_in_a_row_wins() {
    let mut game = new_game();
    play(&mut game, &[place(0, 0)]);               // J1
    play(&mut game, &[place(9, 0), place(9, 2)]);  // J2
    play(&mut game, &[place(0, 1), place(0, 2)]);  // J1
    play(&mut game, &[place(8, 0), place(8, 2)]);  // J2
    play(&mut game, &[place(0, 3), place(0, 4)]);  // J1
    play(&mut game, &[place(7, 0), place(7, 2)]);  // J2
    let outcome = play(&mut game, &[place(0, 5)]); // J1 : 6e pion

    assert_eq!(outcome, Outcome::Victory(Player::One));
    assert_eq!(game.apply(Action::EndTurn), Err(GameError::GameOver));
}

#[test]
fn cannot_place_on_occupied_cell() {
    let mut game = new_game();
    play(&mut game, &[place(4, 4)]);
    assert_eq!(
        game.apply(place(4, 4)),
        Err(GameError::CellOccupied(Position { row: 4, col: 4 }))
    );
}

#[test]
fn refused_action_changes_nothing() {
    let mut game = new_game();
    assert!(game.apply(place(10, 10)).is_err());
    assert_eq!(game.actions_left(), 1);
    assert_eq!(game.reserve_pions(Player::One), 15);
}

#[test]
fn empty_reserve_forbids_placing() {
    let config = GameConfig { pawns_per_player: 1, ..GameConfig::default() };
    let mut game = Game::new(config).unwrap();
    play(&mut game, &[place(0, 0)]);                      // J1 pose son seul pion
    play(&mut game, &[place(9, 9), Action::EndTurn]);     // J2
    assert_eq!(game.apply(place(1, 1)), Err(GameError::EmptyReserve));
}