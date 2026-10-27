//! L'arène en ligne de commande : chaque variante de `bots::variants` est jouée par les
//! paires de `bots::STANDARD_PAIRS`, pour voir l'effet d'un réglage avant de le changer
//! dans le moteur ou dans le carnet. Le labo propose la même chose à l'écran, pour
//! explorer ; ici, c'est la référence pour les grandes séries.
//!
//! Lancer : `cargo arene` (100 parties par paire, environ 2 minutes), ou `cargo arene 2000`.

use bots::{Report, STANDARD_PAIRS, run_pairs, variants};
use std::time::Instant;

const DEFAULT_GAMES: u64 = 100;

fn main() {
    let games = match std::env::args().nth(1) {
        None => DEFAULT_GAMES,
        Some(arg) => match arg.parse::<u64>() {
            Ok(games) if games > 0 => games,
            _ => {
                eprintln!(
                    "« {arg} » : il faut un nombre de parties positif, par exemple `cargo arene 500`."
                );
                std::process::exit(2);
            }
        },
    };

    let pluriel = if games > 1 { "s" } else { "" };
    println!(
        "{games} partie{pluriel} par paire de bots, graines 0 à {}.",
        games - 1
    );
    for variant in variants() {
        let start = Instant::now();
        println!("\n=== {} ===\n{}", variant.name, Report::HEADER);
        match run_pairs(&variant.config, &STANDARD_PAIRS, games) {
            Ok(reports) => reports.iter().for_each(|report| println!("{report}")),
            Err(error) => println!("configuration invalide : {error}"),
        }
        println!("({:.1} s)", start.elapsed().as_secs_f64());
    }
}
