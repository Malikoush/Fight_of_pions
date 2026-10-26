//! Règles de fin de partie (carnet de règles, « Fin de partie »).

mod common;

use common::{TestRng, new_game, place, play};
use fight_of_pions::{
    Action, Decision, EndReason, Event, Game, GameConfig, GameEnd, Outcome, Player,
};

fn draw(reason: EndReason) -> Event {
    Event::GameEnded(GameEnd {
        outcome: Outcome::Draw,
        reason,
    })
}

/// Vérifie qu'une réponse est exactement « un pion posé, puis la fin » : aucun
/// `TurnEnded` ne doit se glisser entre les deux, même si la pose a vidé les actions.
fn assert_placed_then(events: &[Event], end: Event) {
    assert_eq!(events.len(), 2, "événements inattendus : {events:?}");
    assert!(matches!(events[0], Event::PionPlaced { .. }));
    assert_eq!(events[1], end);
}

/// La définition du carnet, écrite indépendamment du moteur : avec les règles actuelles,
/// une partie est figée quand le plateau est plein ou que les deux réserves sont vides.
fn frozen_by_the_rules(game: &Game) -> bool {
    let board_full = game
        .board()
        .positions()
        .all(|pos| game.board().owner(pos).is_some());
    let reserves_empty =
        game.reserve_pions(Player::One) == 0 && game.reserve_pions(Player::Two) == 0;
    board_full || reserves_empty
}

/// Une configuration valide tirée au hasard, petite pour que les fins arrivent vite.
/// Elle couvre les cas où il faut économiser ses actions pour pouvoir poser.
fn random_config(rng: &mut TestRng) -> GameConfig {
    let board_size = 2 + rng.below(3);
    let actions_per_turn = 1 + rng.below(2) as u32;
    let max_banked_actions = rng.below(3) as u32;
    let config = GameConfig {
        board_size,
        align_to_win: 2 + rng.below(board_size - 1),
        pions_per_player: 1 + rng.below(9) as u32,
        first_turn_actions: 1 + rng.below(5) as u32,
        actions_per_turn,
        max_banked_actions,
        place_cost: 1 + rng.below((actions_per_turn + max_banked_actions) as usize) as u32,
        max_turns: 1 + rng.below(60) as u32,
    };
    assert_eq!(config.validate(), Ok(()), "{config:?}");
    config
}

#[test]
fn both_reserves_empty_freezes_the_game_into_a_draw() {
    let config = GameConfig {
        pions_per_player: 1,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();

    play(&mut game, &[place(0, 0)]);
    // Une seule réserve vide : le joueur 2 peut encore poser, la partie continue.
    assert_eq!(game.end(), None);

    let events = play(&mut game, &[place(9, 9)]);
    assert_eq!(events.last(), Some(&draw(EndReason::Frozen)));
    assert_eq!(game.winner(), None);
    assert_eq!(game.pending_decision(), None);
    assert!(game.legal_actions().is_empty());
}

#[test]
fn full_board_freezes_the_game_into_a_draw() {
    let config = GameConfig {
        board_size: 3,
        align_to_win: 3,
        pions_per_player: 5,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();

    // Une grille de morpion remplie sans ligne de 3 :
    //   X O X
    //   X O O
    //   O X X
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(0, 1), place(1, 1)]);
    play(&mut game, &[place(0, 2), place(1, 0)]);
    play(&mut game, &[place(1, 2), place(2, 0)]);
    play(&mut game, &[place(2, 1)]);
    // Cette dernière pose remplit le plateau et vide aussi les actions du tour.
    let events = play(&mut game, &[place(2, 2)]);

    assert_placed_then(&events, draw(EndReason::Frozen));
}

#[test]
fn turn_limit_reached_by_placing_ends_without_starting_a_new_turn() {
    let config = GameConfig {
        max_turns: 1,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();
    // Le premier tour n'a qu'une action : la pose finit le tour, qui était le dernier.
    let events = play(&mut game, &[place(0, 0)]);

    assert_placed_then(&events, draw(EndReason::TurnLimit));
    assert_eq!(game.turn(), 1);
}

#[test]
fn random_games_freeze_exactly_when_the_rules_say_so() {
    for seed in 0..400 {
        let mut rng = TestRng::new(seed);
        let mut game = Game::new(random_config(&mut rng)).unwrap();

        while game.end().is_none() {
            let legal = game.legal_actions();
            game.apply(legal[rng.below(legal.len())]).unwrap();

            match game.end().map(|end| end.reason) {
                // Une victoire peut arriver sur une partie qui se fige au même coup.
                Some(EndReason::Alignment) => {}
                Some(EndReason::Frozen) => assert!(
                    frozen_by_the_rules(&game),
                    "partie {seed} déclarée figée à tort : {:?}",
                    game.config()
                ),
                Some(EndReason::TurnLimit) | None => assert!(
                    !frozen_by_the_rules(&game),
                    "partie {seed} figée mais pas arrêtée : {:?}",
                    game.config()
                ),
            }
        }
        assert!(game.turn() <= game.config().max_turns);
    }
}

#[test]
fn alignment_wins_even_when_the_game_freezes_at_the_same_time() {
    let config = GameConfig {
        board_size: 3,
        align_to_win: 2,
        pions_per_player: 2,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(2, 2), place(0, 2)]);

    // Ce pion aligne 2 et vide la dernière réserve : la victoire passe avant.
    let events = play(&mut game, &[place(0, 1)]);
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded(GameEnd {
            outcome: Outcome::Won(Player::One),
            reason: EndReason::Alignment,
        }))
    );
}

#[test]
fn turn_limit_ends_the_game_in_a_draw() {
    let config = GameConfig {
        max_turns: 3,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();
    play(&mut game, &[place(0, 0)]); // fin du tour 1
    play(&mut game, &[Action::EndTurn]); // fin du tour 2

    let events = play(&mut game, &[Action::EndTurn]); // fin du tour 3 : limite
    assert_eq!(events, vec![draw(EndReason::TurnLimit)]);
    assert_eq!(game.turn(), 3);
    assert_eq!(game.pending_decision(), None);
}

#[test]
fn players_who_only_pass_reach_the_turn_limit() {
    let mut game = new_game();
    while game.end().is_none() {
        play(&mut game, &[Action::EndTurn]);
    }
    assert_eq!(game.end().map(|end| end.reason), Some(EndReason::TurnLimit));
    assert_eq!(game.turn(), game.config().max_turns);
}

#[test]
fn pending_decision_follows_the_current_player() {
    let mut game = new_game();
    assert_eq!(
        game.pending_decision(),
        Some(Decision::MainAction {
            player: Player::One
        })
    );
    play(&mut game, &[place(0, 0)]);
    assert_eq!(
        game.pending_decision().map(Decision::player),
        Some(Player::Two)
    );
}
