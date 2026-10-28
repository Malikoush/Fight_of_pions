//! Le bot glouton dans des situations dont la bonne réponse est connue.

mod common;

use bots::{Banking, Bot, GreedyBot, Windows, target_rules};
use common::{duel, place, play};
use fight_of_pions::{Action, Game, Player, Rng};

fn greedy() -> GreedyBot {
    GreedyBot::new(Banking::SpendAll)
}

/// Laisse le bot jouer pour le joueur courant jusqu'à la fin de son tour (ou de la partie).
fn play_the_turn(bot: &GreedyBot, game: &mut Game) {
    let player = game.current_player();
    let mut rng = Rng::new(0);
    while game.end().is_none() && game.current_player() == player {
        let action = bot.choose(game, &mut rng).unwrap();
        game.apply(action).unwrap();
    }
}

#[test]
fn it_completes_its_line_when_it_can() {
    let mut game = duel();
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(9, 9), place(9, 0)]);
    play(&mut game, &[place(0, 1), place(0, 2)]);
    play(&mut game, &[place(5, 9), place(9, 5)]);
    play(&mut game, &[place(0, 3), place(0, 4)]); // le joueur 1 a 5 pions de A1 à E1
    play(&mut game, &[place(5, 0), place(7, 7)]); // le joueur 2 ne bloque pas

    let action = greedy().choose(&game, &mut Rng::new(0));
    assert_eq!(action, Some(place(0, 5)));
    play(&mut game, &[place(0, 5)]);
    assert_eq!(game.winner(), Some(Player::One));
}

#[test]
fn it_blocks_every_line_the_opponent_could_finish_next_turn() {
    let mut game = duel();
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(4, 2), place(4, 3)]);
    play(&mut game, &[place(9, 0), place(0, 9)]);
    play(&mut game, &[place(4, 4), place(4, 5)]); // 4 pions ouverts : C5 à F5

    play_the_turn(&greedy(), &mut game);

    let windows = Windows::new(game.board(), game.config().align_to_win);
    assert!(windows.finishable(game.board(), Player::Two, 2).is_empty());
}

#[test]
fn it_breaks_a_blocked_line_with_an_attack_then_completes_it() {
    let mut game = duel();
    play(&mut game, &[place(4, 2)]);
    play(&mut game, &[place(4, 1), place(0, 0)]);
    play(&mut game, &[place(4, 3), place(4, 4)]);
    play(&mut game, &[place(4, 6), place(0, 2)]);
    play(&mut game, &[place(4, 5), Action::EndTurn]); // C5 à F5, bloqués en B5 et G5
    play(&mut game, &[place(9, 0), place(9, 2)]);
    play(&mut game, &[Action::EndTurn]); // le joueur 1 garde 2 actions
    play(&mut game, &[place(9, 4), place(9, 6)]);
    assert_eq!(game.actions_left(), 4);

    let bot = greedy();
    let first = bot.choose(&game, &mut Rng::new(0));
    assert!(matches!(first, Some(Action::UseAttack(_))), "{first:?}");
    play_the_turn(&bot, &mut game);
    assert_eq!(game.winner(), Some(Player::One));
}

#[test]
fn the_banker_keeps_its_actions_and_the_spender_does_not() {
    let game = duel(); // le joueur 1 n'a qu'une action au premier tour
    let banker = GreedyBot::new(Banking::KeepMax);

    assert_eq!(
        banker.choose(&game, &mut Rng::new(0)),
        Some(Action::EndTurn)
    );
    assert!(matches!(
        greedy().choose(&game, &mut Rng::new(0)),
        Some(Action::Place(_))
    ));
}

#[test]
fn the_banker_still_makes_a_free_placement() {
    // Règles cibles : la pose ne coûte aucune action, garder ses actions n'empêche pas de poser.
    let mut game = Game::new(target_rules(), 0).unwrap();
    let banker = GreedyBot::new(Banking::KeepMax);
    let action = banker.choose(&game, &mut Rng::new(0)).unwrap();
    assert!(matches!(action, Action::Place(_)), "{action:?}");

    game.apply(action).unwrap();
    assert_eq!(
        banker.choose(&game, &mut Rng::new(0)),
        Some(Action::EndTurn)
    );
}

#[test]
fn names_tell_the_bots_apart() {
    assert_eq!(greedy().name(), "glouton");
    assert_eq!(GreedyBot::new(Banking::KeepMax).name(), "banquier");
}
