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
fn cannot_place_outside_the_board() {
    let mut game = new_game();
    assert_eq!(
        game.apply(place(10, 10)),
        Err(GameError::OutOfBounds(Position { row: 10, col: 10 }))
    );
}

#[test]
fn absurd_position_is_refused_and_displayable() {
    let mut game = new_game();
    let far = Position {
        row: usize::MAX,
        col: usize::MAX,
    };
    let error = game.apply(Action::Place(far)).unwrap_err();
    assert_eq!(error, GameError::OutOfBounds(far));
    assert!(error.to_string().contains("hors du plateau"));
}

#[test]
fn refused_action_changes_nothing() {
    let mut game = new_game();
    play(&mut game, &[place(4, 4)]);
    let before = game.clone();

    assert!(game.apply(place(4, 4)).is_err());
    assert!(game.apply(place(10, 10)).is_err());
    assert_eq!(game, before);
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

#[test]
fn ending_the_turn_is_free_and_keeps_unused_actions() {
    let mut game = new_game();
    play(&mut game, &[place(0, 0)]);

    // Le joueur 2 finit son tour sans rien faire : ses 2 actions sont gardées.
    play(&mut game, &[Action::EndTurn]);
    play(&mut game, &[place(0, 1), place(0, 2)]);
    assert_eq!(game.current_player(), Player::Two);
    assert_eq!(game.actions_left(), 4);
}

#[test]
fn banked_actions_are_capped() {
    let mut game = new_game();
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[Action::EndTurn]); // joueur 2 garde 2 actions
    play(&mut game, &[place(0, 1), place(0, 2)]);

    // Le joueur 2 a 4 actions et n'en utilise aucune : il n'en garde que 2.
    play(&mut game, &[Action::EndTurn]);
    play(&mut game, &[place(0, 3), place(0, 4)]);
    assert_eq!(game.current_player(), Player::Two);
    assert_eq!(game.actions_left(), 4);
}

#[test]
fn placing_costs_the_configured_number_of_actions() {
    let config = GameConfig {
        first_turn_actions: 3,
        place_cost: 2,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();
    play(&mut game, &[place(0, 0)]);
    assert_eq!(game.actions_left(), 1);
    assert_eq!(
        game.apply(place(0, 1)),
        Err(GameError::NotEnoughActions {
            needed: 2,
            available: 1
        })
    );
}

#[test]
fn invalid_config_refuses_to_start() {
    let config = GameConfig {
        board_size: 27,
        ..GameConfig::default()
    };
    assert!(matches!(
        Game::new(config),
        Err(GameError::InvalidConfig(_))
    ));
}
