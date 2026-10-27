//! Mesure combien de parties complètes, jouées au hasard, le moteur enchaîne par seconde,
//! et comment elles se terminent. C'est la référence de performance pour les bots (on
//! n'optimise rien sans elle), et un premier indicateur pour l'équilibrage.
//!
//! Lancer : `cargo run --release -p fight_of_pions --example random_games`

use fight_of_pions::{EndReason, Game, GameConfig, Rng};
use std::time::Instant;

fn main() {
    measure("sans attaque", GameConfig::default(), 20_000);

    // Chaque joueur a toutes les attaques jouables du catalogue. Les parties sont bien
    // plus longues : moins de parties, même durée.
    measure(
        "avec attaques",
        GameConfig::default().with_full_arsenals(),
        2_000,
    );
}

fn measure(label: &str, config: GameConfig, games: u64) {
    let start = Instant::now();
    let mut actions = 0_u64;
    // Fins par alignement, partie figée, limite de tours.
    let mut ends = [0_u64; 3];

    for seed in 0..games {
        let mut rng = Rng::new(seed);
        let Ok(mut game) = Game::new(config.clone(), seed) else {
            eprintln!("{label} : configuration invalide");
            return;
        };
        while game.end().is_none() {
            let legal = game.legal_actions();
            let Some(&action) = rng.pick(&legal) else {
                break;
            };
            // Une action listée par `legal_actions` et refusée serait un bug du moteur.
            if let Err(error) = game.apply(action) {
                eprintln!("{label}, partie {seed} : action légale refusée ({action:?}) : {error}");
                return;
            }
            actions += 1;
        }
        match game.end().map(|end| end.reason) {
            Some(EndReason::Alignment) => ends[0] += 1,
            Some(EndReason::Frozen) => ends[1] += 1,
            Some(EndReason::TurnLimit) => ends[2] += 1,
            None => {}
        }
    }

    let seconds = start.elapsed().as_secs_f64();
    let percent = |count: u64| 100.0 * count as f64 / games as f64;
    println!(
        "{label} : {games} parties en {seconds:.2} s — {:.0} parties/s, {:.0} actions/s, \
         {:.0} actions par partie",
        games as f64 / seconds,
        actions as f64 / seconds,
        actions as f64 / games as f64
    );
    println!(
        "    fins : alignement {:.0} %, figée {:.0} %, limite de tours {:.0} %",
        percent(ends[0]),
        percent(ends[1]),
        percent(ends[2])
    );
}
