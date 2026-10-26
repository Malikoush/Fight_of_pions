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

/// Le plateau et le contenu de ses cases.
pub mod board;
/// Types de base partagés : joueurs, positions, directions.
pub mod common;
/// Les réglages chiffrés d'une partie.
pub mod config;
/// Les raisons pour lesquelles une action est refusée.
pub mod error;
/// L'état d'une partie, les actions et les événements.
pub mod game;
/// Les règles pures, qui lisent le plateau sans le modifier.
pub mod rules;

pub use board::{Board, Cell, Pion, PionId};
pub use common::{Direction, Player, Position};
pub use config::{ConfigError, GameConfig};
pub use error::GameError;
pub use game::{Action, Decision, EndReason, Event, Game, GameEnd, Outcome};
