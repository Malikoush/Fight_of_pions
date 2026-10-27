mod common;

use common::{SEED, game_with, new_game, place, placed_pion, play};
use fight_of_pions::{
    Action, EndReason, Event, Game, GameConfig, GameEnd, GameError, Outcome, Player, Position,
};

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

    assert!(events.contains(&Event::GameEnded(GameEnd {
        outcome: Outcome::Won(Player::One),
        reason: EndReason::Alignment,
    })));
    assert_eq!(game.winner(), Some(Player::One));
    assert_eq!(game.apply(Action::EndTurn), Err(GameError::GameOver));
}

#[test]
fn placing_announces_the_pion() {
    let mut game = new_game();
    let events = play(&mut game, &[place(4, 4)]);
    let pion = placed_pion(&events);
    assert_eq!(pion.owner(), Player::One);
    assert_eq!(
        events[0],
        Event::PionPlaced {
            pion,
            at: Position { row: 4, col: 4 }
        }
    );
}

#[test]
fn board_holds_the_announced_pion() {
    let mut game = new_game();
    let pion = placed_pion(&play(&mut game, &[place(4, 4)]));
    let cell = game.board().get(Position { row: 4, col: 4 }).unwrap();
    assert_eq!(cell.pion(), Some(pion));
}

#[test]
fn each_pion_gets_its_own_id_in_placement_order() {
    let mut game = new_game();
    let first = placed_pion(&play(&mut game, &[place(0, 0)]));
    let second = placed_pion(&play(&mut game, &[place(5, 5)]));
    let third = placed_pion(&play(&mut game, &[place(9, 9)]));

    assert_eq!(first.id().to_string(), "#0");
    assert_eq!(second.owner(), Player::Two);
    assert!(first.id() < second.id());
    assert!(second.id() < third.id());
}

#[test]
fn same_actions_give_same_ids() {
    let actions = [place(0, 0), place(5, 5), place(9, 9)];
    let mut a = new_game();
    let mut b = new_game();
    play(&mut a, &actions);
    play(&mut b, &actions);
    assert_eq!(a, b);
}

#[test]
fn refused_action_does_not_consume_an_id() {
    let mut with_refusal = new_game();
    assert!(with_refusal.apply(place(10, 10)).is_err());
    let after_refusal = placed_pion(&play(&mut with_refusal, &[place(0, 0)]));

    let mut direct = new_game();
    let direct_pion = placed_pion(&play(&mut direct, &[place(0, 0)]));
    assert_eq!(after_refusal, direct_pion);
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
        pions_per_player: 1,
        ..GameConfig::default()
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    // Le joueur 2 garde son pion : sinon les deux réserves seraient vides et la partie
    // serait figée, donc finie.
    play(&mut game, &[Action::EndTurn]);
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
fn placing_announces_the_remaining_actions() {
    let mut game = new_game();
    play(&mut game, &[place(0, 0)]);
    let events = play(&mut game, &[place(5, 5)]);
    assert_eq!(
        events[2],
        Event::ActionsChanged {
            player: Player::Two,
            actions_left: 1,
            banked: 0,
        }
    );
}

#[test]
fn placing_the_last_action_announces_each_counter_once() {
    let mut game = new_game();
    let events = play(&mut game, &[place(0, 0)]);
    let pion = placed_pion(&events);
    assert_eq!(
        events,
        vec![
            Event::PionPlaced {
                pion,
                at: Position { row: 0, col: 0 },
            },
            Event::ReserveChanged {
                player: Player::One,
                pions: 14,
            },
            Event::ActionsChanged {
                player: Player::One,
                actions_left: 0,
                banked: 0,
            },
            Event::TurnEnded { next: Player::Two },
            Event::ActionsChanged {
                player: Player::Two,
                actions_left: 2,
                banked: 0,
            },
        ]
    );
}

#[test]
fn ending_the_turn_announces_both_players_actions() {
    let mut game = new_game();
    play(&mut game, &[place(0, 0)]);
    // Le joueur 2 finit son tour avec ses 2 actions : il les garde.
    let events = play(&mut game, &[Action::EndTurn]);
    assert_eq!(
        events,
        vec![
            Event::ActionsChanged {
                player: Player::Two,
                actions_left: 0,
                banked: 2,
            },
            Event::TurnEnded { next: Player::One },
            Event::ActionsChanged {
                player: Player::One,
                actions_left: 2,
                banked: 0,
            },
        ]
    );
    assert_eq!(game.banked_actions(Player::Two), 2);
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
    let mut game = game_with(config);
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
        Game::new(config, SEED),
        Err(GameError::InvalidConfig(_))
    ));
}
