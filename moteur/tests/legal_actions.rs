//! `legal_actions()` doit dire exactement ce que `apply()` accepte.

mod common;

use common::{game_with, new_game, place, play, random_config};
use fight_of_pions::{Action, Game, GameConfig, Position, Rng};

/// Toutes les actions qu'un client pourrait tenter : poser ou choisir chaque case, des
/// cases hors du plateau, chaque emplacement d'attaque et un de trop, finir son tour.
fn candidates(game: &Game) -> Vec<Action> {
    let size = game.board().size();
    let outside = [
        Position { row: size, col: 0 },
        Position { row: 0, col: size },
    ];
    let cells: Vec<Position> = game.board().positions().chain(outside).collect();
    let slots = 0..=game.arsenal(game.current_player()).len();
    cells
        .iter()
        .map(|&pos| Action::Place(pos))
        .chain(cells.iter().map(|&pos| Action::Choose(pos)))
        .chain(slots.map(Action::UseAttack))
        .chain([Action::EndTurn])
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
    let mut game = game_with(config);
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
    let game = game_with(config);
    // Premier tour : 1 action, la pose en coûte 2.
    assert_eq!(game.legal_actions(), vec![Action::EndTurn]);
}

#[test]
fn random_games_agree_with_apply_never_dead_end_and_always_end() {
    for seed in 0..200 {
        let mut rng = Rng::new(seed);
        let mut game = if seed % 2 == 0 {
            new_game()
        } else {
            game_with(random_config(&mut rng))
        };
        // Borne large : chaque tour finit au plus après quelques actions.
        let mut steps_left = 10_000;

        while game.end().is_none() {
            assert_legal_actions_match_apply(&game);
            // Aucune impasse : tant que la partie dure, même au milieu d'une attaque, il
            // reste au moins une action permise.
            let legal = game.legal_actions();
            assert!(
                !legal.is_empty(),
                "partie {seed} bloquée : {:?}",
                game.pending_decision()
            );
            game.apply(*rng.pick(&legal).unwrap()).unwrap();

            steps_left -= 1;
            assert!(steps_left > 0, "la partie {seed} ne se termine pas");
        }
        assert!(game.legal_actions().is_empty());
    }
}
