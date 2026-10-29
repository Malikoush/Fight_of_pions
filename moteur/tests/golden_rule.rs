//! La règle d'or (carnet de règles, « Règles cibles » et questions B2) : une ligne ne gagne
//! qu'au début du tour de son propriétaire, si elle y est encore.

mod common;

use common::{choose, game_with, ours, place, play};
use fight_of_pions::{
    Action, EndReason, Event, Game, GameConfig, GameEnd, Outcome, Player, Rarity,
};

/// Un petit plateau 5 × 5 où `align_to_win` pions alignés gagnent, avec la règle d'or.
fn golden(align_to_win: usize) -> GameConfig {
    GameConfig {
        board_size: 5,
        align_to_win,
        golden_rule: true,
        ..GameConfig::default()
    }
}

fn won_by(player: Player) -> Event {
    Event::GameEnded(GameEnd {
        outcome: Outcome::Won(player),
        reason: EndReason::Alignment,
    })
}

/// J1 aligne A1, A2, A3 à son deuxième tour ; J2 a posé E5 et E4 entre-temps.
fn line_of_three_for_player_one(config: GameConfig) -> Game {
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(4, 4), place(3, 4)]);
    play(&mut game, &[place(1, 0), place(2, 0)]);
    game
}

#[test]
fn a_line_made_by_placing_does_not_win_at_once() {
    let game = line_of_three_for_player_one(golden(3));
    assert_eq!(game.end(), None);
    assert_eq!(game.current_player(), Player::Two);
}

#[test]
fn a_line_still_there_at_the_start_of_your_turn_wins() {
    let mut game = line_of_three_for_player_one(golden(3));
    let events = play(&mut game, &[place(4, 0), place(4, 1)]);

    assert_eq!(game.winner(), Some(Player::One));
    // Le tour de J1 commence, puis la règle d'or le fait gagner aussitôt : il ne reçoit
    // pas ses actions (la règle d'or passe avant le gain des actions).
    let turn_ended = events
        .iter()
        .position(|event| *event == Event::TurnEnded { next: Player::One })
        .unwrap();
    assert_eq!(events.last(), Some(&won_by(Player::One)));
    assert!(!events[turn_ended..].iter().any(|event| matches!(
        event,
        Event::ActionsChanged {
            player: Player::One,
            ..
        }
    )));
    assert_eq!(game.actions_left(), 0);
}

#[test]
fn the_golden_rule_comes_before_the_cooldowns() {
    let mut config = GameConfig {
        starting_attacks: [vec![ours(Rarity::Rare)], Vec::new()],
        ..golden(3)
    };
    config.attack_catalog[0].stats.cooldown = 2;
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]); // J1 : A1
    play(&mut game, &[place(0, 1), place(4, 4)]); // J2 : B1, E5
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 0), choose(0, 1)],
    ); // J1 tue B1 : recharge de 2 tours
    play(&mut game, &[place(3, 4), place(4, 3)]); // J2
    play(&mut game, &[place(1, 0), place(2, 0)]); // J1 : A1-A2-A3, recharge à 1
    assert_eq!(game.arsenal(Player::One)[0].cooldown_left(), 1);

    let events = play(&mut game, &[place(0, 4), place(1, 3)]); // J2, sans aligner
    assert_eq!(game.winner(), Some(Player::One));
    // La règle d'or a fini la partie avant les recharges : celle de J1 n'a pas bougé.
    let turn_ended = events
        .iter()
        .position(|event| *event == Event::TurnEnded { next: Player::One })
        .unwrap();
    assert!(!events[turn_ended..].iter().any(|event| matches!(
        event,
        Event::CooldownChanged {
            player: Player::One,
            ..
        }
    )));
    assert_eq!(game.arsenal(Player::One)[0].cooldown_left(), 1);
}

#[test]
fn breaking_the_line_during_the_opponent_turn_prevents_the_win() {
    let config = GameConfig {
        starting_attacks: [Vec::new(), vec![ours(Rarity::Rare)]],
        ..golden(3)
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(0, 1), place(4, 4)]); // J2 : B1, contre la future ligne
    play(&mut game, &[place(1, 0), place(2, 0)]); // J1 : A1-A2-A3

    // J2 tue A2 avec l'Ours depuis B1 : la ligne est cassée avant le tour de J1.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 1), choose(1, 0)],
    );
    assert_eq!(game.end(), None);
    assert_eq!(game.current_player(), Player::One);
}

#[test]
fn without_the_golden_rule_placing_a_line_wins_at_once() {
    let config = GameConfig {
        golden_rule: false,
        ..golden(3)
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(4, 4), place(3, 4)]);
    let events = play(&mut game, &[place(1, 0), place(2, 0)]);
    assert_eq!(events.last(), Some(&won_by(Player::One)));
}

#[test]
fn a_pending_line_wins_when_the_turn_limit_stops_the_game() {
    let config = GameConfig {
        max_turns: 3,
        ..golden(3)
    };
    let mut game = line_of_three_for_player_one(config);
    // La ligne de J1 est faite au tour 3, le dernier : elle gagne quand même.
    assert_eq!(game.winner(), Some(Player::One));
    assert_eq!(game.end().map(|end| end.reason), Some(EndReason::Alignment));
    assert!(game.apply(Action::EndTurn).is_err());
}

#[test]
fn when_both_have_a_line_at_the_stop_the_next_player_wins_first() {
    // Avec 2 à aligner : J2 aligne E5-E4 au tour 2, J1 aligne A1-A2 au tour 3, le dernier.
    let config = GameConfig {
        max_turns: 3,
        ..golden(2)
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(4, 4), place(3, 4)]);
    play(&mut game, &[place(1, 0), place(2, 2)]);
    // Si les tours continuaient, celui de J2 viendrait d'abord : J2 gagne.
    assert_eq!(game.winner(), Some(Player::Two));
}

#[test]
fn a_pending_line_wins_when_the_game_freezes() {
    let config = GameConfig {
        board_size: 4,
        pions_per_player: 3,
        ..golden(3)
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(3, 3), place(2, 3)]);
    play(&mut game, &[place(1, 0), place(2, 0)]); // J1 : A1-A2-A3, réserve vide
    // J2 pose son dernier pion sans aligner : plus personne ne peut agir.
    let events = play(&mut game, &[place(0, 3)]);
    assert_eq!(events.last(), Some(&won_by(Player::One)));
}
