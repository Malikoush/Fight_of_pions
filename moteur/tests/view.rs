//! La vue joueur (`Game::view`, `Event::is_visible_to`) : tout ce qu'un client affiche, et
//! jamais la cache adverse pas encore jouée.

mod common;

use common::{game_with, random_config};
use fight_of_pions::{
    Action, Attack, AttackKind, Decision, DraftRules, DraftStep, Event, Game, GameConfig, Player,
    PlayerView, Position, Rarity, Rng,
};

fn hare() -> Attack {
    Attack {
        kind: AttackKind::Lievre,
        rarity: Rarity::Common,
    }
}

/// Seulement la cache, sans mur : chacun cache un Lièvre Commune.
fn hares_in_cache() -> Game {
    let config = GameConfig {
        wall_thickness: 0,
        draft: Some(DraftRules::new(1, 0, 0, 0)),
        ..GameConfig::default()
    };
    let mut game = game_with(config);
    for _ in 0..2 {
        game.apply(Action::Draft(hare())).unwrap();
    }
    game
}

/// Vérifie qu'une vue dit la vérité sur la partie, sauf les cartes secrètes adverses.
fn assert_faithful(game: &Game, view: &PlayerView) {
    let viewer = view.viewer;
    assert_eq!(view.decision, game.pending_decision());
    let decides = game
        .pending_decision()
        .is_some_and(|decision| decision.player() == viewer);
    let expected_legal = if decides {
        game.legal_actions()
    } else {
        Vec::new()
    };
    assert_eq!(view.legal_actions, expected_legal);
    let board = game.board();
    for row in 0..board.size() {
        for col in 0..board.size() {
            let pos = Position { row, col };
            let expected = (!board.is_wall(pos)).then(|| board.get(pos)).flatten();
            assert_eq!(view.board.cell(pos), expected, "{pos}");
        }
    }
    for player in [Player::One, Player::Two] {
        let seen = &view.sides[player.index()].arsenal;
        let real = game.arsenal(player);
        assert_eq!(seen.len(), real.len());
        for (seen, real) in seen.iter().zip(real) {
            let hidden = player != viewer && real.is_secret();
            assert_eq!(*seen, (!hidden).then_some(*real));
        }
        assert_eq!(
            view.sides[player.index()].reserve,
            game.reserve_pions(player)
        );
    }
}

#[test]
fn each_player_sees_its_own_cache_but_not_the_opponent_one() {
    let game = hares_in_cache();
    let one = game.view(Player::One);
    assert_eq!(one.sides[0].arsenal, [Some(game.arsenal(Player::One)[0])]);
    assert_eq!(one.sides[1].arsenal, [None]);
    let two = game.view(Player::Two);
    assert_eq!(two.sides[0].arsenal, [None]);
    assert_faithful(&game, &one);
    assert_faithful(&game, &two);
}

#[test]
fn only_the_player_who_decides_gets_the_legal_actions() {
    let game = hares_in_cache();
    assert_eq!(
        game.pending_decision(),
        Some(Decision::MainAction {
            player: Player::One
        })
    );
    assert_eq!(game.view(Player::One).legal_actions, game.legal_actions());
    assert!(game.view(Player::Two).legal_actions.is_empty());
}

#[test]
fn the_opponent_cache_choice_is_not_shown_in_the_events() {
    let config = GameConfig {
        draft: Some(DraftRules::new(1, 0, 0, 0)),
        ..GameConfig::default()
    };
    let mut game = game_with(config);
    let events = game.apply(Action::Draft(hare())).unwrap();
    let cached = Event::Drafted {
        step: DraftStep::Cache(Player::One),
        attack: hare(),
    };
    assert_eq!(events, [cached]);
    assert!(cached.is_visible_to(Player::One));
    assert!(!cached.is_visible_to(Player::Two));
    let picked = Event::Drafted {
        step: DraftStep::Pick(Player::One),
        attack: hare(),
    };
    assert!(picked.is_visible_to(Player::Two));
}

#[test]
fn the_views_tell_the_truth_and_nothing_secret_in_random_games() {
    let mut views = 0;
    for seed in 0..200 {
        let mut rng = Rng::new(seed);
        let config = GameConfig {
            draft: Some(DraftRules::standard()),
            ..random_config(&mut rng)
        };
        // Un catalogue au hasard peut être trop petit pour le draft standard.
        let Ok(mut game) = Game::new(config, seed) else {
            continue;
        };
        while game.end().is_none() {
            for viewer in [Player::One, Player::Two] {
                assert_faithful(&game, &game.view(viewer));
                views += 1;
            }
            let action = *rng.pick(&game.legal_actions()).unwrap();
            game.apply(action).unwrap();
        }
    }
    assert!(views > 5000, "seulement {views} vues vérifiées");
}

/// La vue passe en JSON et en revient à l'identique : c'est ce que le serveur enverra.
#[cfg(feature = "serde")]
#[test]
fn a_view_goes_through_json_unchanged() {
    let game = hares_in_cache();
    let view = game.view(Player::Two);
    let json = serde_json::to_string(&view).unwrap();
    let back: PlayerView = serde_json::from_str(&json).unwrap();
    assert_eq!(back, view);
    // La cache de J1 arrive chez J2 comme une carte inconnue.
    assert_eq!(back.sides[0].arsenal, [None]);
}
