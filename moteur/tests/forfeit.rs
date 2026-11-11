//! L'abandon (carnet de règles, « Fin de partie », 2026-11-10) : un joueur peut abandonner
//! à tout moment, même hors de son tour ; il perd par forfait et l'autre gagne.

mod common;

use common::{choose, config_with_attacks, game_with, new_game, ours, place, play};
use fight_of_pions::{
    Action, Decision, DraftRules, EndReason, Event, GameConfig, GameEnd, GameError, GameRecord,
    Outcome, Player, Rarity,
};

/// La fin par forfait de ce joueur : l'autre gagne.
fn forfeit_of(player: Player) -> GameEnd {
    GameEnd {
        outcome: Outcome::Won(player.opponent()),
        reason: EndReason::Forfeit,
    }
}

#[test]
fn resigning_makes_the_opponent_win_by_forfeit() {
    let mut game = new_game();
    play(&mut game, &[place(4, 4)]);

    let events = game.apply(Action::Resign(Player::One)).unwrap();

    // Comme toute fin : les poses restantes tombent à zéro, puis la fin, en dernier.
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded(forfeit_of(Player::One)))
    );
    assert_eq!(game.end(), Some(forfeit_of(Player::One)));
    assert_eq!(game.pending_decision(), None);
    assert!(game.legal_actions().is_empty());
}

#[test]
fn a_player_can_resign_outside_of_their_turn() {
    let mut game = new_game();
    assert_eq!(game.current_player(), Player::One);

    game.apply(Action::Resign(Player::Two)).unwrap();

    assert_eq!(game.end(), Some(forfeit_of(Player::Two)));
}

#[test]
fn a_player_can_resign_during_the_draft() {
    let mut game = game_with(GameConfig {
        draft: Some(DraftRules::standard()),
        ..GameConfig::default()
    });
    assert!(matches!(
        game.pending_decision(),
        Some(Decision::Draft { .. })
    ));

    game.apply(Action::Resign(Player::Two)).unwrap();

    assert_eq!(game.end(), Some(forfeit_of(Player::Two)));
}

#[test]
fn a_player_can_resign_in_the_middle_of_an_attack() {
    let bear = ours(Rarity::Rare);
    let mut game = game_with(config_with_attacks(vec![bear], vec![]));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), place(5, 5)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert!(matches!(
        game.pending_decision(),
        Some(Decision::ChooseTarget { .. })
    ));

    game.apply(Action::Resign(Player::One)).unwrap();

    assert_eq!(game.end(), Some(forfeit_of(Player::One)));
    assert_eq!(game.pending_decision(), None);
}

#[test]
fn resigning_is_never_an_expected_decision() {
    let game = new_game();

    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Resign(_)))
    );
}

#[test]
fn nobody_can_resign_a_finished_game() {
    let mut game = new_game();
    game.apply(Action::Resign(Player::One)).unwrap();

    assert_eq!(
        game.apply(Action::Resign(Player::Two)),
        Err(GameError::GameOver)
    );
    assert_eq!(game.end(), Some(forfeit_of(Player::One)));
}

#[test]
fn a_forfeit_is_replayed_from_the_record() {
    let mut record = GameRecord::new(GameConfig::first_rules(), 7);
    record.actions = vec![place(4, 4), Action::Resign(Player::Two)];

    let game = record.replay().unwrap();

    assert_eq!(game.end(), Some(forfeit_of(Player::Two)));
}
