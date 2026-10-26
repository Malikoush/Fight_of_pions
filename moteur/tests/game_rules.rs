use fight_of_pions::{Action, Event, Game, GameConfig, GameError, Player, Position};

fn place(row: usize, col: usize) -> Action {
    Action::Place(Position { row, col })
}

fn new_game() -> Game {
    Game::new(GameConfig::default()).unwrap()
}

/// Joue une suite d'actions qui doivent toutes réussir ;
/// renvoie les événements de la dernière action.
fn play(game: &mut Game, actions: &[Action]) -> Vec<Event> {
    let mut last = Vec::new();
    for &action in actions {
        last = game.apply(action).unwrap();
    }
    last
}

#[test]
fn first_turn_has_one_action_then_two() {
    let mut game = new_game();
    assert_eq!(game.actions_left(), 1);

    let events = play(&mut game, &[place(0, 0)]);
    assert!(events.contains(&Event::TurnEnded { next: Player::Two }));
    assert_eq!(game.actions_left(), 2);
}

#[test]
fn six_in_a_row_wins() {
    let mut game = new_game();
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(9, 0), place(9, 2)]);
    play(&mut game, &[place(0, 1), place(0, 2)]);
    play(&mut game, &[place(8, 0), place(8, 2)]);
    play(&mut game, &[place(0, 3), place(0, 4)]);
    play(&mut game, &[place(7, 0), place(7, 2)]);
    let events = play(&mut game, &[place(0, 5)]);

    assert!(events.contains(&Event::GameWon {
        winner: Player::One
    }));
    assert_eq!(game.apply(Action::EndTurn), Err(GameError::GameOver));
}

#[test]
fn placing_announces_the_pawn() {
    let mut game = new_game();
    let events = play(&mut game, &[place(4, 4)]);
    assert_eq!(
        events[0],
        Event::PawnPlaced {
            player: Player::One,
            at: Position { row: 4, col: 4 }
        }
    );
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
    let config = GameConfig {
        pawns_per_player: 1,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(9, 9), Action::EndTurn]);
    assert_eq!(game.apply(place(1, 1)), Err(GameError::EmptyReserve));
}
