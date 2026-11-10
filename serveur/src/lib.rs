//! Le serveur de Fight of Pions : il fait autorité sur les parties en ligne. Les clients
//! (Godot sur le web, le PC ou le mobile) se connectent en WebSocket sur `/ws`, et
//! échangent des messages JSON ([`ClientMessage`], [`ServerMessage`]). Il peut aussi servir
//! l'export web du client sur `/` : la page et le jeu ont alors la même adresse, et le
//! client en déduit celle de la WebSocket.
//!
//! Le serveur garde chaque partie entière ; chaque joueur ne reçoit que sa vue
//! ([`fight_of_pions::PlayerView`]) : la cache adverse ne quitte jamais le serveur, et un
//! aperçu ne la sonde pas. Les règles restent dans le moteur, les bots dans `bots`.
//!
//! - `protocol` : les messages ;
//! - `account` : les comptes invités (pseudo, jeton) ;
//! - `collection` : les cartes d'un compte, son trio de cache, le paquet de départ ;
//! - `database` : la base SQLite des comptes ;
//! - `cards` : le catalogue des cartes à afficher (noms et textes, tirés de `data/cartes.json`) ;
//! - `limits` : les limites du serveur (taille des messages, salles, bots) ;
//! - `room` : une partie et ses places, sans réseau ;
//! - `lobby` : les salles ouvertes, et la diffusion des vues ;
//! - `connection` : une WebSocket ;
//! - `random` : le hasard des codes, des clés et des jetons.

mod account;
mod cards;
mod collection;
mod connection;
mod database;
mod error;
mod limits;
mod lobby;
mod protocol;
mod random;
mod room;

pub use cards::{CardFamily, CardRarity};
pub use database::Database;
pub use error::ServerError;
pub use protocol::{ClientMessage, Opponent, ServerMessage};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use lobby::Lobby;
use std::path::Path;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

/// Ce que partagent toutes les connexions : les salles, et la base des comptes.
#[derive(Clone)]
struct Shared {
    lobby: Arc<Lobby>,
    database: Database,
}

/// Le routeur du serveur, avec cette base : la WebSocket du jeu sur `/ws`, `/health` pour
/// l'hébergeur et, si on lui donne le dossier de l'export web du client (`index.html` et
/// ses fichiers), ce dossier sur `/`, compressé.
pub fn app(database: Database, web: Option<&Path>) -> Router {
    let router = Router::new()
        .route("/ws", get(websocket))
        .route("/health", get(|| async { "ok" }))
        .with_state(Shared {
            lobby: Arc::new(Lobby::default()),
            database,
        });
    match web {
        // Seuls les fichiers sont compressés : la WebSocket n'est pas touchée.
        Some(folder) => router.fallback_service(
            Router::new()
                .fallback_service(ServeDir::new(folder))
                .layer(CompressionLayer::new()),
        ),
        None => router,
    }
}

async fn websocket(upgrade: WebSocketUpgrade, State(shared): State<Shared>) -> impl IntoResponse {
    upgrade
        .max_message_size(limits::MAX_MESSAGE_BYTES)
        .max_frame_size(limits::MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| connection::serve(socket, shared.lobby, shared.database))
}
