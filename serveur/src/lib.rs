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
//! - `limits` : les limites du serveur (messages, connexions, salles, délais, bots) ;
//! - `room` : une partie et ses places, sans réseau ;
//! - `lobby` : les salles ouvertes, la salle de chaque compte, et la diffusion des vues ;
//! - `connection` : une WebSocket (pings, délais, débit) ;
//! - `requests` : les demandes d'un client, une par message ;
//! - `background` : les bots, l'enregistrement des parties, le balayage des salles ;
//! - `random` : le hasard des codes, des clés et des jetons.

mod account;
mod background;
mod cards;
mod collection;
mod connection;
mod database;
mod error;
mod limits;
mod lobby;
mod protocol;
mod random;
mod requests;
mod room;

pub use cards::{CardFamily, CardRarity};
pub use database::Database;
pub use error::ServerError;
pub use protocol::{ClientMessage, Opponent, ServerMessage};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use background::Background;
use limits::{
    MAX_CONNECTIONS, MAX_REGISTRATIONS_PER_WINDOW, REGISTRATION_WINDOW, RateLimit, SHUTDOWN_GRACE,
};
use lobby::Lobby;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

/// Ce que partagent toutes les connexions.
#[derive(Clone)]
struct Shared {
    /// Les salles.
    lobby: Arc<Lobby>,
    /// La base, les bots, les enregistrements.
    background: Background,
    /// Les comptes créés récemment, tous clients confondus.
    registrations: Arc<RateLimit>,
    /// Les connexions qu'on peut encore ouvrir.
    connections: Arc<Semaphore>,
    /// Annulé à l'arrêt : les connexions se ferment.
    shutdown: CancellationToken,
}

/// Le serveur : son routeur, à servir, et de quoi l'arrêter proprement.
pub struct Server {
    router: Router,
    shared: Shared,
}

impl Server {
    /// Le serveur, avec cette base : la WebSocket du jeu sur `/ws`, `/health` pour
    /// l'hébergeur et, si on lui donne le dossier de l'export web du client (`index.html` et
    /// ses fichiers), ce dossier sur `/`. Lance le balayage régulier des salles.
    pub fn new(database: Database, web: Option<&Path>) -> Server {
        let shared = Shared {
            lobby: Arc::new(Lobby::default()),
            background: Background::new(database),
            registrations: Arc::new(RateLimit::new(
                MAX_REGISTRATIONS_PER_WINDOW,
                REGISTRATION_WINDOW,
            )),
            connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
            shutdown: CancellationToken::new(),
        };
        shared
            .background
            .sweep_regularly(Arc::clone(&shared.lobby), shared.shutdown.clone());
        let router = Router::new()
            .route("/ws", get(websocket))
            .route("/health", get(|| async { "ok" }))
            .with_state(shared.clone());
        let router = match web {
            Some(folder) => router.fallback_service(web_files(folder)),
            None => router,
        };
        Server { router, shared }
    }

    /// Le routeur, à donner à `axum::serve`.
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    /// De quoi fermer les connexions depuis une autre tâche (celle qui attend le signal
    /// d'arrêt) : chaque client l'apprend, et se reconnectera.
    pub fn connection_closer(&self) -> impl FnOnce() + Send + 'static {
        let shutdown = self.shared.shutdown.clone();
        move || shutdown.cancel()
    }

    /// Ferme les connexions, puis attend les parties en cours d'enregistrement, au plus
    /// [`limits::SHUTDOWN_GRACE`]. Les parties pas finies sont perdues : elles ne vivent
    /// qu'en mémoire.
    pub async fn shutdown(&self) {
        self.shared.shutdown.cancel();
        self.shared.background.finish(SHUTDOWN_GRACE).await;
    }
}

/// Le routeur d'un serveur avec cette base (voir [`Server::new`]).
pub fn app(database: Database, web: Option<&Path>) -> Router {
    Server::new(database, web).router()
}

/// Les fichiers de l'export web. Un fichier déjà compressé à côté (`index.wasm.gz`, voir
/// `outils/precompresser_web.py`) est envoyé tel quel ; les autres sont compressés à
/// l'envoi. Leurs noms ne changent pas d'une version à l'autre : le navigateur les
/// redemande à chaque visite (`no-cache`), et le serveur répond « pas changé » quand c'est
/// le cas.
fn web_files(folder: &Path) -> Router {
    Router::new()
        .fallback_service(ServeDir::new(folder).precompressed_gzip())
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        ))
}

/// Ouvre la WebSocket d'un client, s'il reste de la place ; sinon, « occupé » (503).
async fn websocket(upgrade: WebSocketUpgrade, State(shared): State<Shared>) -> Response {
    let Ok(permit) = Arc::clone(&shared.connections).try_acquire_owned() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "serveur complet").into_response();
    };
    let tasks = shared.background.tasks().clone();
    upgrade
        .max_message_size(limits::MAX_MESSAGE_BYTES)
        .max_frame_size(limits::MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| tasks.track_future(connection::serve(socket, shared, permit)))
        .into_response()
}
