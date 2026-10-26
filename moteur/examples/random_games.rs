//! Mesure combien de parties complètes, jouées au hasard, le moteur enchaîne par seconde.
//! C'est la référence de performance pour les bots : on n'optimise rien sans elle.
//!
//! Lancer : `cargo run --release -p fight_of_pions --example random_games`

use fight_of_pions::{Game, GameConfig, Rng};
use std::time::Instant;

/// Assez de parties pour une mesure d'environ une seconde, donc stable.
const GAMES: u64 = 20_000;

fn main() {
    let config = GameConfig::default();
    let start = Instant::now();
    let mut actions = 0_u64;

    for seed in 0..GAMES {
        let mut rng = Rng::new(seed);
        let Ok(mut game) = Game::new(config, seed) else {
            eprintln!("configuration invalide");
            return;
        };
        while game.end().is_none() {
            let legal = game.legal_actions();
            let Some(&action) = rng.pick(&legal) else {
                break;
            };
            // Une action listée par `legal_actions` et refusée serait un bug du moteur.
            if let Err(error) = game.apply(action) {
                eprintln!("partie {seed} : action légale refusée ({action:?}) : {error}");
                return;
            }
            actions += 1;
        }
    }

    let seconds = start.elapsed().as_secs_f64();
    println!("{GAMES} parties, {actions} actions en {seconds:.2} s");
    println!(
        "{:.0} parties/s, {:.0} actions/s",
        GAMES as f64 / seconds,
        actions as f64 / seconds
    );
}
