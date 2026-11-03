//! Les statistiques d'équilibrage : comment finissent les parties MCTS contre MCTS d'une
//! variante avec draft, et ce que vaut chaque carte quand elle est en main.
//!
//! `cargo equilibrage [parties] [itérations]` (par défaut 200 et 300). La variante est
//! « règles actuelles avec draft » : chaque partie donne des mains différentes. Le tableau
//! sort au format CSV (`;`), à coller dans un tableur.
//!
//! Score en main : les victoires de qui l'a en main, un nul comptant pour moitié. À lire
//! avec sa marge : sur 150 mains, environ ±8 points.

use bots::{Bot, MctsBot, Weights, record_games, variants};
use fight_of_pions::{Attack, DraftStep, EndReason, Event, Game, GameRecord, Outcome, Player};
use std::collections::BTreeMap;
use std::time::Instant;

const VARIANT: &str = "règles actuelles avec draft";

/// Ce qu'on sait d'une carte, sur toutes les parties.
#[derive(Default)]
struct CardStats {
    drawn: u32,
    banned: u32,
    picked: u32,
    cached: u32,
    /// Les mains où elle est.
    held: u32,
    /// Les victoires de ces mains, un nul comptant pour moitié.
    score: f64,
    /// Ses usages par ces mains.
    played: u32,
}

/// Ce que donne une partie, relue depuis son enregistrement.
struct Played {
    end: (EndReason, Option<Player>),
    turns: u32,
    pool: Vec<Attack>,
    drafted: Vec<(DraftStep, Attack)>,
    hands: [Vec<Attack>; 2],
    plays: Vec<(Player, Attack)>,
}

fn replay(record: &GameRecord) -> Played {
    let mut game = Game::new(record.config.clone(), record.seed).expect("partie enregistrée");
    let pool = game.draft_pool().to_vec();
    let (mut drafted, mut plays) = (Vec::new(), Vec::new());
    for &action in &record.actions {
        for event in game.apply(action).expect("action enregistrée") {
            match event {
                Event::Drafted { step, attack } => drafted.push((step, attack)),
                Event::AttackUsed { player, attack, .. } => plays.push((player, attack)),
                _ => {}
            }
        }
    }
    let end = game.end().expect("partie finie");
    let winner = match end.outcome {
        Outcome::Won(player) => Some(player),
        Outcome::Draw => None,
    };
    Played {
        end: (end.reason, winner),
        turns: game.turn(),
        pool,
        drafted,
        hands: [Player::One, Player::Two].map(|player| {
            game.arsenal(player)
                .iter()
                .map(|slot| slot.attack())
                .collect()
        }),
        plays,
    }
}

fn card_stats(games: &[Played]) -> BTreeMap<Attack, CardStats> {
    let mut cards: BTreeMap<Attack, CardStats> = BTreeMap::new();
    for game in games {
        for &attack in &game.pool {
            cards.entry(attack).or_default().drawn += 1;
        }
        for &(step, attack) in &game.drafted {
            let card = cards.entry(attack).or_default();
            match step {
                DraftStep::Cache(_) => card.cached += 1,
                DraftStep::Ban(_) => card.banned += 1,
                DraftStep::Pick(_) => card.picked += 1,
            }
        }
        for player in [Player::One, Player::Two] {
            let score = match game.end.1 {
                Some(winner) if winner == player => 1.0,
                Some(_) => 0.0,
                None => 0.5,
            };
            let mut hand = game.hands[player.index()].clone();
            // Deux fois la même carte (cache et pick) : une seule main.
            hand.sort();
            hand.dedup();
            for attack in hand {
                let card = cards.entry(attack).or_default();
                card.held += 1;
                card.score += score;
                card.played += game
                    .plays
                    .iter()
                    .filter(|&&(by, played)| by == player && played == attack)
                    .count() as u32;
            }
        }
    }
    cards
}

fn main() {
    let mut args = std::env::args().skip(1).map(|arg| arg.parse::<u64>());
    let games = args
        .next()
        .transpose()
        .expect("nombre de parties")
        .unwrap_or(200);
    let iterations = args.next().transpose().expect("itérations").unwrap_or(300);
    let config = variants()
        .into_iter()
        .find(|variant| variant.name == VARIANT)
        .expect("la variante avec draft")
        .config;
    let bot = MctsBot::new(
        u32::try_from(iterations).unwrap_or(u32::MAX),
        Weights::DEFAULT,
    );
    let start = Instant::now();
    let records = record_games(&config, [&bot as &dyn Bot, &bot], games).expect("parties");
    let played: Vec<Played> = records.iter().map(replay).collect();

    let count = played.len() as f64;
    println!(
        "{} parties « {VARIANT} », MCTS {iterations} itérations, en {:.0?}",
        played.len(),
        start.elapsed()
    );
    let mut ends: BTreeMap<String, u32> = BTreeMap::new();
    for game in &played {
        *ends.entry(format!("{:?}", game.end.0)).or_default() += 1;
    }
    for (reason, n) in &ends {
        println!("  {reason} : {n} ({:.0} %)", 100.0 * f64::from(*n) / count);
    }
    let wins = |player| {
        played
            .iter()
            .filter(|game| game.end.1 == Some(player))
            .count()
    };
    println!(
        "  J1 gagne {}, J2 gagne {} ; durée moyenne {:.0} tours",
        wins(Player::One),
        wins(Player::Two),
        played.iter().map(|game| f64::from(game.turns)).sum::<f64>() / count
    );

    let rate = |card: &CardStats| card.score / f64::from(card.held.max(1));
    let mut rows: Vec<(Attack, CardStats)> = card_stats(&played).into_iter().collect();
    rows.sort_by(|(_, a), (_, b)| rate(b).total_cmp(&rate(a)));
    println!("\ncarte;en main;score %;jouée par partie;tirée;prise;bannie;cachée");
    for (attack, card) in rows {
        println!(
            "{attack};{};{:.0};{:.1};{};{};{};{}",
            card.held,
            100.0 * rate(&card),
            f64::from(card.played) / f64::from(card.held.max(1)),
            card.drawn,
            card.picked,
            card.banned,
            card.cached
        );
    }
}
