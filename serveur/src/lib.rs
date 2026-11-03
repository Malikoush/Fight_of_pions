//! Le serveur de Fight of Pions : il fait autorité sur les parties en ligne. Les clients
//! (Godot sur le web, le PC ou le mobile) se connectent en WebSocket sur `/ws`, et
//! échangent des messages JSON ([`ClientMessage`], [`ServerMessage`]).
//!
//! Le serveur garde chaque partie entière ; chaque joueur ne reçoit que sa vue
//! ([`fight_of_pions::PlayerView`]) : la cache adverse ne quitte jamais le serveur. Les
//! règles restent dans le moteur, les bots dans `bots`.
//!
//! - `protocol` : les messages ;
//! - `room` : une partie et ses places, sans réseau ;
//! - `lobby` : les salles ouvertes, et la diffusion des vues ;
//! - `connection` : une WebSocket.

mod connection;
mod error;
mod lobby;
mod protocol;
mod room;

pub use error::ServerError;
pub use protocol::{ClientMessage, Opponent, ServerMessage};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use lobby::Lobby;
use std::sync::Arc;

/// Le routeur du serveur : la WebSocket du jeu sur `/ws`, et `/health` pour l'hébergeur.
pub fn app() -> Router {
    Router::new()
        .route("/ws", get(websocket))
        .route("/health", get(|| async { "ok" }))
        .with_state(Arc::new(Lobby::default()))
}

async fn websocket(
    upgrade: WebSocketUpgrade,
    State(lobby): State<Arc<Lobby>>,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| connection::serve(socket, lobby))
}
