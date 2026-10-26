pub mod board;
pub mod common;
pub mod config;
pub mod error;
pub mod game;
pub mod rules;

pub use board::{Board, Cell};
pub use common::{Direction, Player, Position};
pub use config::GameConfig;
pub use error::GameError;
pub use game::{Action, Event, Game};
