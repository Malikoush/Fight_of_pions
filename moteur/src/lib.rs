//! Moteur de Fight of Pions : toutes les règles du jeu, sans aucune dépendance.
//!
//! Un client (terminal, labo, bot, Godot) crée une [`Game`], lui envoie des [`Action`]
//! avec [`Game::apply`] et réagit aux [`Event`] renvoyés. Il ne contient jamais de règle.
//!
//! Pour savoir quoi jouer, un client (ou un bot) demande [`Game::pending_decision`]
//! (qui doit choisir quoi) et [`Game::legal_actions`] (la liste exacte des actions permises).

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

/// Les attaques : familles, raretés, caractéristiques, et leurs règles.
pub mod attacks;
/// Le plateau et le contenu de ses cases.
pub mod board;
/// Types de base partagés : joueurs, positions, directions.
pub mod common;
/// Les réglages d'une partie.
pub mod config;
/// Les raisons pour lesquelles une action est refusée.
pub mod error;
/// L'état d'une partie, les actions et les événements.
pub mod game;
/// Le générateur pseudo-aléatoire déterministe : tout le hasard du jeu passe par lui.
pub mod rng;
/// Les règles pures, qui lisent le plateau sans le modifier.
pub mod rules;

pub use attacks::{Attack, AttackKind, AttackStats, CatalogEntry, Rarity};
pub use board::{Board, Cell, Pion, PionId};
pub use common::{Direction, Player, Position};
pub use config::{ConfigError, GameConfig};
pub use error::GameError;
pub use game::{
    Action, ArsenalSlot, Decision, EndReason, Event, Game, GameEnd, GameRecord, Outcome,
    ReplayError,
};
pub use rng::Rng;
