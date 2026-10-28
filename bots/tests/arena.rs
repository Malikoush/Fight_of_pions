//! L'arène : des parties complètes, des bilans cohérents et reproductibles.

use bots::{
    Banking, Bot, BotKind, GreedyBot, RandomBot, STANDARD_PAIRS, current_rules, record_game, run,
    run_pairs, target_rules,
};
use fight_of_pions::{Action, Event, Game, GameConfig, GameError, Position, Rng};

/// Des configurations variées, et combien de parties y jouer : la règle actuelle (une
/// seule, ses parties entre gloutons durent jusqu'à la limite de tours), un petit plateau
/// où il faut économiser pour attaquer, un morpion où le plateau se remplit, deux où
/// poser coûte 2 actions (il faut économiser pour poser, dès le premier tour), et
/// les règles cibles sur un petit plateau.
fn configs() -> Vec<(GameConfig, u64)> {
    let current = current_rules();
    let mut tiny = GameConfig {
        board_size: 2,
        align_to_win: 2,
        pions_per_player: 2,
        actions_per_turn: 1,
        max_banked_actions: 1,
        place_cost: 2,
        ..current.clone()
    };
    // 1 action par tour + 1 gardée : aucune attaque ne peut coûter plus de 2.
    for entry in &mut tiny.attack_catalog {
        entry.stats.cost = entry.stats.cost.min(2);
    }
    let small_target_rules = GameConfig {
        board_size: 5,
        align_to_win: 4,
        pions_per_player: 6,
        ..target_rules()
    };
    vec![
        (tiny, 3),
        (small_target_rules, 3),
        (
            GameConfig {
                board_size: 5,
                align_to_win: 4,
                pions_per_player: 6,
                place_cost: 2,
                ..current.clone()
            },
            3,
        ),
        (current.clone(), 1),
        (
            GameConfig {
                board_size: 5,
                align_to_win: 4,
                pions_per_player: 6,
                max_banked_actions: 1,
                ..current.clone()
            },
            3,
        ),
        (
            GameConfig {
                board_size: 3,
                align_to_win: 3,
                pions_per_player: 5,
                ..current
            },
            3,
        ),
    ]
}

#[test]
fn every_bot_plays_only_legal_actions_until_the_end() {
    let random = RandomBot;
    let greedy = GreedyBot::new(Banking::SpendAll);
    let banker = GreedyBot::new(Banking::KeepMax);
    let bots: [&dyn Bot; 3] = [&random, &greedy, &banker];
    for (config, games) in configs() {
        for &first in &bots {
            for &second in &bots {
                let report = run(&config, [first, second], games).unwrap();
                assert_eq!(report.refused, 0, "{:?}", report.first_refusal);
                assert_eq!(report.games, games);
                let draws = report.frozen + report.turn_limit;
                assert_eq!(report.wins[0] + report.wins[1] + draws, report.games);
            }
        }
    }
}

#[test]
fn the_same_run_gives_the_same_report() {
    let greedy = GreedyBot::new(Banking::SpendAll);
    let banker = GreedyBot::new(Banking::KeepMax);
    let config = current_rules();
    let first = run(&config, [&banker, &greedy], 4).unwrap();
    assert_eq!(run(&config, [&banker, &greedy], 4).unwrap(), first);
    assert_eq!(first.bots, ["banquier", "glouton"]);
}

#[test]
fn the_greedy_bot_beats_the_random_bot_from_both_seats() {
    let greedy = GreedyBot::new(Banking::SpendAll);
    let config = current_rules();
    let games = 10;
    assert_eq!(
        run(&config, [&greedy, &RandomBot], games).unwrap().wins[0],
        games
    );
    assert_eq!(
        run(&config, [&RandomBot, &greedy], games).unwrap().wins[1],
        games
    );
}

#[test]
fn an_invalid_config_is_refused_before_any_game() {
    let config = GameConfig {
        board_size: 0,
        ..GameConfig::default()
    };
    assert!(matches!(
        run(&config, [&RandomBot, &RandomBot], 1),
        Err(GameError::InvalidConfig(_))
    ));
}

#[test]
fn a_recorded_game_is_the_one_the_arena_played() {
    let config = current_rules();
    let bots = [BotKind::Banker.bot() as &dyn Bot, BotKind::Greedy.bot()];
    let report = run(&config, bots, 1).unwrap();

    // Rejoue l'enregistrement en comptant ce que le bilan compte.
    let record = record_game(&config, 0, bots).unwrap();
    let mut game = Game::new(record.config.clone(), record.seed).unwrap();
    let (mut attacks, mut kills) = (0, 0);
    for &action in &record.actions {
        for event in game.apply(action).unwrap() {
            match event {
                Event::AttackUsed { .. } => attacks += 1,
                Event::PionKilled { .. } => kills += 1,
                _ => {}
            }
        }
    }
    let mut wins = [0, 0];
    if let Some(winner) = game.winner() {
        wins[winner.index()] = 1;
    }
    assert_eq!(report.wins, wins);
    assert_eq!(report.turns, u64::from(game.turn()));
    assert_eq!(report.attacks.values().sum::<u64>(), attacks);
    assert_eq!(report.kills, kills);
}

/// Ce que fait le bot fautif au tour 3 de la partie de graine 1.
#[derive(Clone, Copy)]
enum Fault {
    /// Proposer une case hors du plateau.
    Refused,
    /// Ne rien proposer alors que la partie continue.
    Nothing,
}

/// Joue comme le glouton, sauf au tour 3 de la partie de graine 1.
struct FaultyBot(Fault);

impl Bot for FaultyBot {
    fn name(&self) -> &'static str {
        "fautif"
    }

    fn choose(&self, game: &Game, rng: &mut Rng) -> Option<Action> {
        if game.seed() == 1 && game.turn() == 3 {
            return match self.0 {
                Fault::Refused => Some(Action::Place(Position { row: 99, col: 99 })),
                Fault::Nothing => None,
            };
        }
        BotKind::Greedy.bot().choose(game, rng)
    }
}

#[test]
fn a_refused_game_leaves_only_its_refusal_in_the_report() {
    let config = GameConfig {
        board_size: 5,
        align_to_win: 4,
        pions_per_player: 6,
        ..current_rules()
    };
    let greedy = BotKind::Greedy.bot();
    for (fault, message) in [
        (Fault::Refused, "refusée"),
        (Fault::Nothing, "aucune action"),
    ] {
        let faulty = FaultyBot(fault);
        let report = run(&config, [&faulty, greedy], 3).unwrap();
        assert_eq!(report.refused, 1);
        assert_eq!(report.games, 2);
        let refusal = report.first_refusal.clone().unwrap();
        assert!(refusal.starts_with("graine 1, tour 3"), "{refusal}");
        assert!(refusal.contains(message), "{refusal}");
        assert!(record_game(&config, 1, [&faulty, greedy]).is_err());

        // Les autres parties sont celles du glouton, et seules elles sont comptées.
        let turns: u64 = [0, 2]
            .iter()
            .map(|&seed| {
                let record = record_game(&config, seed, [greedy, greedy]).unwrap();
                u64::from(record.replay().unwrap().turn())
            })
            .sum();
        assert_eq!(report.turns, turns);
    }
}

#[test]
fn pairs_run_in_parallel_give_the_reports_of_a_sequential_run() {
    let config = GameConfig {
        board_size: 5,
        align_to_win: 4,
        pions_per_player: 6,
        ..current_rules()
    };
    let in_parallel = run_pairs(&config, &STANDARD_PAIRS, 2).unwrap();
    let one_by_one: Vec<_> = STANDARD_PAIRS
        .iter()
        .map(|[first, second]| run(&config, [first.bot(), second.bot()], 2).unwrap())
        .collect();
    assert_eq!(in_parallel, one_by_one);
}
