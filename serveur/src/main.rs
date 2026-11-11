//! Lance le serveur : `cargo serveur`. Il écoute sur le port de la variable `PORT` (celle
//! que donnent les hébergeurs), 3000 par défaut (8080 est souvent réservé par Windows).
//! Sa base est celle de `DATABASE_URL`, par défaut le fichier `fight_of_pions.db` du
//! dossier où il est lancé ; elle est créée au premier lancement. En production, ce fichier
//! doit être sur un disque qui survit aux redéploiements (un volume).
//!
//! Il sert aussi l'export web du client sur `/`, depuis le dossier de la variable `WEB_DIR`
//! (par défaut `client-godot/export/web`, celui de l'export lancé depuis le dépôt) ; sans ce
//! dossier, il ne sert que la WebSocket.
//!
//! Il s'arrête proprement sur Ctrl+C ou SIGTERM (`docker stop`) : il ferme les connexions
//! et attend les parties en cours d'enregistrement.
//!
//! `serveur --health` vérifie qu'un serveur répond sur ce port (`/health`), sans rien
//! lancer : c'est le contrôle de santé du conteneur, sans outil de plus dans l'image.
//!
//! Avec un dossier dans la variable `BACKUP_DIR`, il y sauvegarde la base chaque nuit (E4i).
//! `serveur admin <commande>` administre la même base, à côté du serveur qui tourne :
//! comptes, cartes, parties, sauvegarde (`serveur admin aide`).

use serveur::{Database, Server, administer};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_DATABASE: &str = "sqlite://fight_of_pions.db";
const DEFAULT_WEB_DIR: &str = "client-godot/export/web";
/// L'option du contrôle de santé.
const HEALTH_FLAG: &str = "--health";
/// La commande d'administration.
const ADMIN_COMMAND: &str = "admin";
/// Le temps laissé au contrôle de santé pour se connecter, écrire et lire.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(3);

fn main() -> ExitCode {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some(HEALTH_FLAG) {
        return check_health(SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
    }
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("impossible de démarrer : {error}");
            return ExitCode::FAILURE;
        }
    };
    match args.first().map(String::as_str) {
        None => {}
        Some(ADMIN_COMMAND) => return runtime.block_on(admin(&args[1..])),
        // Un mot inconnu ne lance pas un second serveur : il le dit.
        Some(other) => {
            eprintln!(
                "commande inconnue : {other} (serveur, serveur {HEALTH_FLAG}, serveur {ADMIN_COMMAND} aide)"
            );
            return ExitCode::FAILURE;
        }
    }
    match runtime.block_on(serve(address)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// L'adresse de la base : `DATABASE_URL`, ou le fichier du dossier où l'on est.
fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE.to_string())
}

/// Le dossier des sauvegardes, s'il y en a un (`BACKUP_DIR`).
fn backup_dir() -> Option<PathBuf> {
    std::env::var("BACKUP_DIR")
        .ok()
        .filter(|folder| !folder.is_empty())
        .map(PathBuf::from)
}

/// `serveur admin …` : la commande sur la base, son texte sur la sortie, une erreur sur la
/// sortie d'erreur (et un code d'échec).
async fn admin(args: &[String]) -> ExitCode {
    let database_url = database_url();
    // Une base qui n'existe pas n'est pas créée : l'adresse est sans doute mal réglée.
    let database = match Database::open_existing(&database_url).await {
        Ok(database) => database,
        Err(error) => {
            eprintln!("base {database_url} introuvable ou illisible : {error}");
            return ExitCode::FAILURE;
        }
    };
    match administer(&database, args, backup_dir().as_deref()).await {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn serve(address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let database_url = database_url();
    let database = Database::open(&database_url)
        .await
        .map_err(|error| format!("base {database_url} : {error}"))?;
    let web =
        PathBuf::from(std::env::var("WEB_DIR").unwrap_or_else(|_| DEFAULT_WEB_DIR.to_string()));
    let web = web.join("index.html").is_file().then_some(web);
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Fight of Pions : serveur sur ws://{address}/ws, base {database_url}");
    match &web {
        Some(folder) => println!(
            "Client web : http://{address}/ (dossier {})",
            folder.display()
        ),
        None => println!("Pas de client web (aucun index.html dans WEB_DIR) : WebSocket seule"),
    }
    let server = Server::new(database, web.as_deref());
    match backup_dir() {
        Some(folder) => {
            println!("Sauvegardes : chaque nuit dans {}", folder.display());
            server.back_up_nightly(folder);
        }
        None => println!("Pas de sauvegarde automatique (aucun dossier dans BACKUP_DIR)"),
    }
    // Au signal, le serveur n'accepte plus personne et ferme les connexions ; puis il attend
    // les enregistrements en cours.
    let close_connections = server.connection_closer();
    let stopping = async move {
        stop_signal().await;
        println!("Arrêt demandé : fermeture des connexions");
        close_connections();
    };
    axum::serve(listener, server.router())
        .with_graceful_shutdown(stopping)
        .await?;
    server.shutdown().await;
    println!("Serveur arrêté");
    Ok(())
}

/// Attend Ctrl+C, ou SIGTERM sous Unix (ce qu'envoie `docker stop`).
async fn stop_signal() {
    let interrupt = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => {
                terminate.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }
}

/// Le contrôle de santé : un `GET /health` sur ce port, qui doit répondre 200.
fn check_health(address: SocketAddr) -> ExitCode {
    let answer = (|| -> std::io::Result<String> {
        let mut stream = TcpStream::connect_timeout(&address, HEALTH_TIMEOUT)?;
        stream.set_read_timeout(Some(HEALTH_TIMEOUT))?;
        stream.set_write_timeout(Some(HEALTH_TIMEOUT))?;
        stream
            .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
        let mut answer = String::new();
        stream.read_to_string(&mut answer)?;
        Ok(answer)
    })();
    match answer {
        Ok(answer) if answer.starts_with("HTTP/1.1 200") => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}
