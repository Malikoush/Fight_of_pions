//! Le draft (B9a, carnet « Draft et cache secrète », 2026-11-03) : la cache choisie dans
//! tout le catalogue, puis des bans et des picks en serpent parmi des cartes tirées avec la
//! graine.

mod common;

use common::{choose, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, ConfigError, Decision, DraftRules, DraftStep, Event, Game,
    GameConfig, GameError, Player, Rarity,
};

/// Les règles actuelles avec le draft standard, sans carte de départ.
fn drafted() -> GameConfig {
    GameConfig {
        draft: Some(DraftRules::standard()),
        ..GameConfig::default()
    }
}

/// La carte de la première action permise : une carte que l'étape en cours accepte.
fn first_choice(game: &Game) -> Attack {
    match game.legal_actions().first() {
        Some(Action::Draft(attack)) => *attack,
        other => panic!("un choix de draft était attendu, reçu : {other:?}"),
    }
}

#[test]
fn the_standard_draft_is_one_cached_card_one_ban_each_then_three_snake_picks() {
    use DraftStep::{Ban, Cache, Pick};
    use Player::{One, Two};
    let rules = DraftRules::standard();
    assert_eq!(rules.pool_size, 10);
    assert_eq!(
        rules.steps,
        [
            Cache(One),
            Cache(Two),
            Ban(One),
            Ban(Two),
            Pick(Two),
            Pick(One),
            Pick(One),
            Pick(Two),
            Pick(Two),
            Pick(One),
        ]
    );
}

#[test]
fn the_cache_is_chosen_in_the_whole_catalog_legendaries_included() {
    let config = drafted();
    let game = game_with(config.clone());
    assert_eq!(
        game.pending_decision(),
        Some(Decision::Draft {
            step: DraftStep::Cache(Player::One)
        })
    );
    let catalog: Vec<Action> = config
        .catalog_attacks()
        .into_iter()
        .map(Action::Draft)
        .collect();
    assert_eq!(game.legal_actions(), catalog);
    assert!(catalog.iter().any(
        |action| matches!(action, Action::Draft(attack) if attack.rarity == Rarity::Legendary)
    ));
}

#[test]
fn ten_different_cards_are_drawn_with_the_seed() {
    let pool_of = |seed| Game::new(drafted(), seed).unwrap().draft_pool().to_vec();
    let pool = pool_of(7);
    assert_eq!(pool.len(), 10);
    let mut distinct = pool.clone();
    distinct.dedup();
    assert_eq!(
        distinct, pool,
        "toutes différentes, dans l'ordre du catalogue"
    );
    assert_eq!(pool_of(7), pool, "même graine, mêmes cartes");
    assert_ne!(pool_of(8), pool);
}

#[test]
fn bans_and_picks_come_from_the_drawn_cards_and_fill_the_arsenals() {
    let mut game = game_with(drafted());
    let ours = Attack {
        kind: AttackKind::Ours,
        rarity: Rarity::Legendary,
    };
    // Les deux joueurs peuvent cacher la même carte, tirée ou non.
    play(&mut game, &[Action::Draft(ours), Action::Draft(ours)]);
    assert_eq!(game.draft_pool().len(), 10);

    let banned = first_choice(&game);
    play(&mut game, &[Action::Draft(banned)]);
    assert!(!game.draft_pool().contains(&banned));
    let second_ban = first_choice(&game);
    play(&mut game, &[Action::Draft(second_ban)]);
    assert_eq!(
        game.pending_decision(),
        Some(Decision::Draft {
            step: DraftStep::Pick(Player::Two)
        })
    );
    // Une carte bannie ne se prend plus ; une carte non tirée non plus.
    assert_eq!(
        game.apply(Action::Draft(banned)),
        Err(GameError::NotInDraft(banned))
    );
    let mut picked = [Vec::new(), Vec::new()];
    while let Some(Decision::Draft { step }) = game.pending_decision() {
        let card = first_choice(&game);
        picked[step.player().index()].push(card);
        play(&mut game, &[Action::Draft(card)]);
    }

    assert_eq!(picked.iter().map(Vec::len).sum::<usize>(), 6);
    assert!(
        game.draft_pool().is_empty(),
        "les 2 cartes ni bannies ni prises partent avec le draft"
    );
    assert_eq!(
        game.pending_decision(),
        Some(Decision::MainAction {
            player: Player::One
        })
    );
    for player in [Player::One, Player::Two] {
        let arsenal: Vec<Attack> = game
            .arsenal(player)
            .iter()
            .map(|slot| slot.attack())
            .collect();
        let mut expected = vec![ours];
        expected.extend(&picked[player.index()]);
        assert_eq!(arsenal, expected, "{player:?} : la cache, puis ses picks");
    }
    // La partie commence normalement.
    play(&mut game, &[place(4, 4)]);
}

#[test]
fn nothing_but_draft_choices_before_the_game() {
    let mut game = game_with(drafted());
    assert_eq!(game.apply(place(4, 4)), Err(GameError::UnexpectedAction));
    assert_eq!(
        game.apply(Action::EndTurn),
        Err(GameError::UnexpectedAction)
    );
    let absent = Attack {
        kind: AttackKind::MeilleursAmis,
        rarity: Rarity::Common,
    };
    assert_eq!(
        game.apply(Action::Draft(absent)),
        Err(GameError::NotInDraft(absent))
    );
}

#[test]
fn the_draft_format_is_checked_against_the_catalog() {
    let with_draft = |pool_size, bans, picks| GameConfig {
        draft: Some(DraftRules::new(1, pool_size, bans, picks)),
        ..GameConfig::default()
    };
    let catalog = GameConfig::default().attack_catalog.len();
    assert_eq!(with_draft(catalog, 1, 3).validate(), Ok(()));
    assert_eq!(
        with_draft(catalog + 1, 1, 3).validate(),
        Err(GameError::InvalidConfig(ConfigError::DraftPoolTooLarge))
    );
    assert_eq!(
        with_draft(7, 1, 3).validate(),
        Err(GameError::InvalidConfig(ConfigError::DraftPoolTooSmall))
    );
}

/// Seulement la cache, sans mur : J1 et J2 cachent chacun un Lièvre Commune.
fn hares_in_cache() -> Game {
    let config = GameConfig {
        wall_thickness: 0,
        draft: Some(DraftRules::new(1, 0, 0, 0)),
        ..GameConfig::default()
    };
    let mut game = game_with(config);
    play(&mut game, &[Action::Draft(hare()), Action::Draft(hare())]);
    game
}

fn hare() -> Attack {
    Attack {
        kind: AttackKind::Lievre,
        rarity: Rarity::Common,
    }
}

#[test]
fn the_cache_is_seen_only_by_its_owner() {
    let game = hares_in_cache();
    assert!(game.arsenal(Player::One)[0].is_secret());
    assert_eq!(game.arsenal_seen_by(Player::One, Player::Two), [None]);
    assert_eq!(
        game.arsenal_seen_by(Player::One, Player::One),
        [Some(game.arsenal(Player::One)[0])]
    );
}

#[test]
fn a_cached_card_is_revealed_when_first_played() {
    let mut game = hares_in_cache();
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    let events = play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
            Action::FinishAttack,
        ],
    );
    assert!(events.contains(&Event::CardRevealed {
        player: Player::One,
        slot: 0
    }));
    assert!(!game.arsenal(Player::One)[0].is_secret());
    assert_eq!(
        game.arsenal_seen_by(Player::One, Player::Two),
        [Some(game.arsenal(Player::One)[0])]
    );
    // Celle de J2, jamais jouée, reste secrète.
    assert!(game.arsenal(Player::Two)[0].is_secret());
}

#[test]
fn a_guess_replaces_only_the_opponent_secret_cards() {
    let game = hares_in_cache();
    let ours = Attack {
        kind: AttackKind::Ours,
        rarity: Rarity::Rare,
    };
    let guessed = game.with_secrets_guessed(Player::One, || ours);
    assert_eq!(guessed.arsenal(Player::One)[0].attack(), hare());
    assert_eq!(guessed.arsenal(Player::Two)[0].attack(), ours);
    assert_eq!(guessed.board(), game.board());
}
