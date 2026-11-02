//! Le bot MCTS : des parties complètes sans refus, reproductibles, des situations dont la
//! bonne réponse est connue, et la preuve que sa recherche compte.

mod common;

use bots::{
    Banking, Bot, GreedyBot, MctsBot, RandomBot, Weights, Windows, current_rules, first_rules, run,
};
use common::{alternate, at, play_the_turn};
use fight_of_pions::{Action, Game, GameConfig, Player, Rng};

/// Assez pour trouver les bons coups des situations, et rester rapide.
const ITERATIONS: u32 = 300;

fn mcts() -> MctsBot {
    MctsBot::new(ITERATIONS, Weights::DEFAULT)
}

/// Les règles actuelles sans la règle d'or : une ligne faite en posant gagne aussitôt, et la
/// bonne réponse se lit en un tour.
fn immediate_win_rules() -> Game {
    let config = GameConfig {
        golden_rule: false,
        ..current_rules()
    };
    Game::new(config, 0).unwrap()
}

/// Vrai si `player` a une ligne entière sur le plateau.
fn has_a_full_window(game: &Game, player: Player) -> bool {
    let windows = Windows::new(game.board(), game.config().align_to_win);
    windows.iter().any(|window| {
        window
            .iter()
            .all(|&pos| game.board().owner(pos) == Some(player))
    })
}

#[test]
fn it_completes_its_line_when_it_can() {
    let mut game = immediate_win_rules();
    // J1 aligne A1 à E1 ; J2 pose loin, sans bloquer.
    alternate(
        &mut game,
        &[
            at(0, 0),
            at(9, 9),
            at(0, 1),
            at(9, 7),
            at(0, 2),
            at(7, 9),
            at(0, 3),
            at(7, 7),
            at(0, 4),
            at(5, 9),
        ],
    );
    assert_eq!(
        mcts().choose(&game, &mut Rng::new(0)),
        Some(Action::Place(at(0, 5)))
    );
}

#[test]
fn it_blocks_a_line_the_opponent_would_finish_next_turn() {
    let mut game = immediate_win_rules();
    // J2 aligne A10 à E10, ouverte en F10 ; J1 est trop loin pour l'attaquer.
    alternate(
        &mut game,
        &[
            at(0, 0),
            at(9, 0),
            at(0, 3),
            at(9, 1),
            at(0, 6),
            at(9, 2),
            at(0, 9),
            at(9, 3),
            at(3, 0),
            at(9, 4),
        ],
    );
    play_the_turn(&mcts(), &mut game);

    let windows = Windows::new(game.board(), game.config().align_to_win);
    assert!(windows.finishable(game.board(), Player::Two, 1).is_empty());
}

#[test]
fn it_plays_a_whole_attack_to_break_a_blocked_line_then_completes_it() {
    let mut game = immediate_win_rules();
    // J1 aligne B5 à F5, bloquée par J2 en A5 et G5 ; J2 pose ses autres pions loin.
    alternate(
        &mut game,
        &[
            at(4, 1),
            at(4, 0),
            at(4, 2),
            at(4, 6),
            at(4, 3),
            at(9, 9),
            at(4, 4),
            at(7, 9),
            at(4, 5),
            at(5, 9),
        ],
    );
    assert!(game.actions_left() >= 2, "l'Ours Rare coûte 2 actions");

    let played = play_the_turn(&mcts(), &mut game);
    assert!(
        matches!(played.first(), Some(Action::UseAttack(_))),
        "{played:?}"
    );
    assert_eq!(game.winner(), Some(Player::One));
}

#[test]
fn with_the_golden_rule_it_breaks_a_finished_line_before_it_wins() {
    let mut game = Game::new(current_rules(), 0).unwrap();
    // J2 aligne A10 à F10 : sa ligne gagnera au début de son prochain tour. J1 a un pion
    // en A9, à côté d'elle, posé depuis longtemps.
    alternate(
        &mut game,
        &[
            at(8, 0),
            at(9, 0),
            at(0, 0),
            at(9, 1),
            at(0, 3),
            at(9, 2),
            at(0, 6),
            at(9, 3),
            at(0, 9),
            at(9, 4),
            at(3, 0),
            at(9, 5),
        ],
    );
    assert!(has_a_full_window(&game, Player::Two));
    assert_eq!(game.end(), None);

    let played = play_the_turn(&mcts(), &mut game);
    assert!(
        played
            .iter()
            .any(|action| matches!(action, Action::UseAttack(_))),
        "{played:?}"
    );
    assert!(!has_a_full_window(&game, Player::Two));
    assert_eq!(game.winner(), None);
}

/// Des configurations variées, et combien de parties y jouer : les règles actuelles (une
/// seule partie, sur le grand plateau), et en petit pour en jouer plus, avec et sans la
/// règle d'or ; un plateau minuscule où l'on n'attaque qu'en gardant ses actions ; les
/// premières règles sur un petit plateau.
fn configs() -> Vec<(GameConfig, u64)> {
    let small = |config: GameConfig| GameConfig {
        board_size: 5,
        align_to_win: 4,
        pions_per_player: 6,
        ..config
    };
    let tiny = GameConfig {
        board_size: 3,
        align_to_win: 3,
        pions_per_player: 4,
        ..current_rules()
    };
    vec![
        (current_rules(), 1),
        (small(current_rules()), 3),
        (
            small(GameConfig {
                golden_rule: false,
                ..current_rules()
            }),
            3,
        ),
        (tiny, 3),
        (small(first_rules()), 2),
    ]
}

#[test]
fn it_plays_only_legal_actions_until_the_end() {
    let mcts = MctsBot::new(50, Weights::DEFAULT);
    let greedy = GreedyBot::new(Banking::SpendAll);
    let opponents: [&dyn Bot; 3] = [&RandomBot, &greedy, &mcts];
    for (config, games) in configs() {
        for &opponent in &opponents {
            for bots in [[&mcts as &dyn Bot, opponent], [opponent, &mcts]] {
                let report = run(&config, bots, games).unwrap();
                assert_eq!(report.refused, 0, "{:?}", report.first_refusal);
                assert_eq!(report.games, games);
            }
        }
    }
}

#[test]
fn the_same_run_gives_the_same_report() {
    let greedy = GreedyBot::new(Banking::SpendAll);
    let config = current_rules();
    let first = run(&config, [&mcts(), &greedy], 2).unwrap();
    assert_eq!(run(&config, [&mcts(), &greedy], 2).unwrap(), first);
    assert_eq!(first.bots, ["MCTS", "glouton"]);
}

/// `bots[seat]` gagne plus de la moitié des parties, dans cette place.
fn wins_most_games(bots: [&dyn Bot; 2], seat: usize) -> bool {
    let games = 6;
    let report = run(&current_rules(), bots, games).unwrap();
    report.wins[seat] * 2 > games
}

#[test]
fn it_beats_the_greedy_bot_from_both_seats() {
    let greedy = GreedyBot::new(Banking::SpendAll);
    assert!(wins_most_games([&mcts(), &greedy], 0));
    assert!(wins_most_games([&greedy, &mcts()], 1));
}

#[test]
fn searching_beats_trusting_the_evaluation_alone() {
    // Une seule itération : le bot joue le coup le mieux jugé, sans chercher plus loin.
    let no_search = MctsBot::new(1, Weights::DEFAULT);
    assert!(wins_most_games([&mcts(), &no_search], 0));
    assert!(wins_most_games([&no_search, &mcts()], 1));
}
