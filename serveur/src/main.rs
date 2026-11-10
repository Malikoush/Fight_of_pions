//! Lance le serveur : `cargo serveur`. Il écoute sur le port de la variable `PORT` (celle
//! que donnent les hébergeurs), 3000 par défaut (8080 est souvent réservé par Windows).
//! Sa base est celle de `DATABASE_URL`, par défaut le fichier `fight_of_pions.db` du
//! dossier où il est lancé ; elle est créée au premier lancement. En production, ce fichier
//! doit être sur un disque qui survit aux redéploiements (un volume).
//!
//! Il sert aussi l'export web du client sur `/`, depuis le dossier de la variable `WEB_DIR`
//! (par défaut `client-godot/export/web`, celui de l'export lancé depuis le dépôt) ; sans ce
//! dossier, il ne sert que la WebSocket.

use serveur::Database;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_DATABASE: &str = "sqlite://fight_of_pions.db";
const DEFAULT_WEB_DIR: &str = "client-godot/export/web";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE.to_string());
    let database = Database::open(&database_url)
        .await
        .map_err(|error| format!("base {database_url} : {error}"))?;
    let web =
        PathBuf::from(std::env::var("WEB_DIR").unwrap_or_else(|_| DEFAULT_WEB_DIR.to_string()));
    let web = web.join("index.html").is_file().then_some(web);
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Fight of Pions : serveur sur ws://{address}/ws, base {database_url}");
    match &web {
        Some(folder) => println!(
            "Client web : http://{address}/ (dossier {})",
            folder.display()
        ),
        None => println!("Pas de client web (aucun index.html dans WEB_DIR) : WebSocket seule"),
    }
    axum::serve(listener, serveur::app(database, web.as_deref())).await?;
    Ok(())
}
