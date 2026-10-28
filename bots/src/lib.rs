//! Bots et arène de Fight of Pions : des joueurs automatiques, et de quoi les faire
//! s'affronter sur des milliers de parties pour mesurer l'effet d'un réglage.
//!
//! Comme tout client, un bot ne contient aucune règle : il choisit parmi
//! [`fight_of_pions::Game::legal_actions`], et quand il veut savoir ce que donne une
//! attaque, il la joue sur une copie de la partie. Une nouvelle famille d'attaques est
//! donc jouée par les bots sans rien changer ici.
//!
//! Lancer l'arène : `cargo arene` (voir `examples/arene.rs`).

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::cast_possible_truncation
    )
)]
#![warn(missing_docs)]

/// L'arène : jouer des parties entre deux bots et en faire le bilan.
pub mod arena;
/// Ce qu'est un bot.
pub mod bot;
/// Le bot glouton, qui lit le plateau en fenêtres.
pub mod greedy;
/// Les réglages prêts à comparer : bots disponibles, paires et variantes.
pub mod presets;
/// Le bot aléatoire.
pub mod random;
/// Les fenêtres : les lignes encore possibles sur le plateau.
pub mod windows;

mod reading;

pub use arena::{Report, record_game, run, run_pairs, start_game};
pub use bot::Bot;
pub use greedy::{Banking, GreedyBot};
pub use presets::{BotKind, STANDARD_PAIRS, Variant, current_rules, target_rules, variants};
pub use random::RandomBot;
pub use windows::Windows;
