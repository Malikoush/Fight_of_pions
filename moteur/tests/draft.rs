//! Le draft (B9a, carnet « Draft et cache secrète », 2026-11-03 ; la cache en dernier,
//! 2026-11-05) : des bans et des picks en serpent parmi des cartes tirées avec la graine,
//! puis la cache choisie dans tout le catalogue. Le format est une suite de phases.

mod common;

use common::{choose, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, ConfigError, Decision, DraftPhase, DraftRules, DraftStep, Event,
    Game, GameConfig, GameError, Player, Rarity,
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

/// Joue la première carte permise à chaque étape, jusqu'à la première cache ; rend les
/// cartes prises par chaque joueur.
fn draft_until_the_caches(game: &mut Game) -> [Vec<Attack>; 2] {
    let mut picked = [Vec::new(), Vec::new()];
    while let Some(Decision::Draft { step }) = game.pending_decision() {
        if matches!(step, DraftStep::Cache(_)) {
            break;
        }
        let card = first_choice(game);
        if matches!(step, DraftStep::Pick(_)) {
            picked[step.player().index()].push(card);
        }
        play(game, &[Action::Draft(card)]);
    }
    picked
}

#[test]
fn the_standard_draft_is_one_ban_each_three_snake_picks_then_one_cached_card() {
    use DraftStep::{Ban, Cache, Pick};
    use Player::{One, Two};
    let rules = DraftRules::standard();
    assert_eq!(rules.pool_size, 10);
    assert_eq!(
        rules.steps,
        [
            Ban(One),
            Ban(Two),
            Pick(Two),
            Pick(One),
            Pick(One),
            Pick(Two),
            Pick(Two),
            Pick(One),
            Cache(One),
            Cache(Two),
        ]
    );
}

#[test]
fn a_format_is_its_phases_in_the_chosen_order() {
    use DraftStep::{Ban, Cache, Pick};
    use Player::{One, Two};
    let rules = DraftRules::new(
        4,
        &[
            DraftPhase::Caches(2),
            DraftPhase::Picks(1),
            DraftPhase::Bans(1),
        ],
    );
    assert_eq!(
        rules.steps,
        [
            Cache(One),
            Cache(One),
            Cache(Two),
            Cache(Two),
            Pick(Two),
            Pick(One),
            Ban(One),
            Ban(Two),
        ]
    );
    assert!(DraftRules::new(0, &[]).steps.is_empty());
}

#[test]
fn the_cache_is_chosen_last_in_the_whole_catalog_legendaries_included() {
    let config = drafted();
    let mut game = game_with(config.clone());
    draft_until_the_caches(&mut game);
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
fn the_drawn_cards_show_from_the_first_ban() {
    let game = game_with(drafted());
    assert_eq!(game.draft_pool().len(), 10);
    assert_eq!(game.visible_draft_pool(), game.draft_pool());
}

#[test]
fn a_cache_chosen_first_is_chosen_without_seeing_the_drawn_cards() {
    let mut game = game_with(GameConfig {
        draft: Some(DraftRules::new(
            10,
            &[DraftPhase::Caches(1), DraftPhase::Bans(1)],
        )),
        ..GameConfig::default()
    });
    assert_eq!(game.draft_pool().len(), 10);
    assert!(game.visible_draft_pool().is_empty());
    assert!(game.view(Player::One).draft_pool.is_empty());
    let first = first_choice(&game);
    play(&mut game, &[Action::Draft(first), Action::Draft(first)]);
    // Premier ban : les cartes tirées se montrent.
    assert_eq!(game.visible_draft_pool(), game.draft_pool());
    assert_eq!(game.view(Player::Two).draft_pool.len(), 10);
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
    let picked = draft_until_the_caches(&mut game);
    assert_eq!(picked.iter().map(Vec::len).sum::<usize>(), 6);
    // Les deux joueurs peuvent cacher la même carte, tirée ou non, prise ou non.
    play(&mut game, &[Action::Draft(ours), Action::Draft(ours)]);
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
        let mut expected = picked[player.index()].clone();
        expected.push(ours);
        assert_eq!(arsenal, expected, "{player:?} : ses picks, puis la cache");
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
        draft: Some(DraftRules::new(
            pool_size,
            &[
                DraftPhase::Bans(bans),
                DraftPhase::Picks(picks),
                DraftPhase::Caches(1),
            ],
        )),
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
        draft: Some(DraftRules::new(0, &[DraftPhase::Caches(1)])),
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
    // Révélée dès qu'elle est lancée : une attaque commencée va au bout.
    let events = play(&mut game, &[Action::UseAttack(0)]);
    assert_eq!(
        events,
        [Event::CardRevealed {
            player: Player::One,
            slot: 0,
            attack: hare(),
        }]
    );
    assert!(!game.arsenal(Player::One)[0].is_secret());
    assert_eq!(
        game.arsenal_seen_by(Player::One, Player::Two),
        [Some(game.arsenal(Player::One)[0])]
    );
    let events = play(
        &mut game,
        &[choose(4, 4), choose(4, 5), Action::FinishAttack],
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardRevealed { .. }))
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
