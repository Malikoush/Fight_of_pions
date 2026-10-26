//! `legal_actions()` doit dire exactement ce que `apply()` accepte.

mod common;

use common::{TestRng, new_game, place, play};
use fight_of_pions::{Action, Game, GameConfig};

/// Toutes les actions qu'un client pourrait tenter : chaque case, des cases hors du
/// plateau, et finir son tour.
fn candidates(game: &Game) -> Vec<Action> {
    let size = game.board().size();
    game.board()
        .positions()
        .map(Action::Place)
        .chain([place(size, 0), place(0, size), Action::EndTurn])
        .collect()
}

/// Vérifie que chaque candidate est listée si et seulement si `apply` l'accepte.
fn assert_legal_actions_match_apply(game: &Game) {
    let legal = game.legal_actions();
    for action in candidates(game) {
        let accepted = game.clone().apply(action).is_ok();
        assert_eq!(
            accepted,
            legal.contains(&action),
            "{action:?} : apply et legal_actions ne sont pas d'accord"
        );
    }
}

#[test]
fn at_start_every_cell_and_ending_the_turn_are_legal() {
    let game = new_game();
    let legal = game.legal_actions();
    assert_eq!(legal.len(), 10 * 10 + 1);
    assert_eq!(legal.last(), Some(&Action::EndTurn));
}

#[test]
fn without_pions_only_ending_the_turn_is_legal() {
    let config = GameConfig {
        pions_per_player: 1,
        ..GameConfig::default()
    };
    let mut game = Game::new(config).unwrap();
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[Action::EndTurn]);

    // Le joueur 1 a vidé sa réserve ; le joueur 2 a encore un pion : pas figé.
    assert_eq!(game.end(), None);
    assert_eq!(game.legal_actions(), vec![Action::EndTurn]);
}

#[test]
fn without_enough_actions_only_ending_the_turn_is_legal() {
    let config = GameConfig {
        place_cost: 2,
        ..GameConfig::default()
    };
    let game = Game::new(config).unwrap();
    // Premier tour : 1 action, la pose en coûte 2.
    assert_eq!(game.legal_actions(), vec![Action::EndTurn]);
}

#[test]
fn random_games_agree_with_apply_and_always_end() {
    for seed in 0..100 {
        let mut rng = TestRng::new(seed);
        let mut game = new_game();
        // Borne large : chaque tour finit au plus après quelques actions.
        let mut steps_left = 10_000;

        while game.end().is_none() {
            assert_legal_actions_match_apply(&game);
            let legal = game.legal_actions();
            let action = legal[rng.below(legal.len())];
            game.apply(action).unwrap();

            steps_left -= 1;
            assert!(steps_left > 0, "la partie {seed} ne se termine pas");
        }
        assert!(game.legal_actions().is_empty());
    }
}
