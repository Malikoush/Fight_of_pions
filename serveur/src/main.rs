//! Lance le serveur : `cargo serveur`. Il écoute sur le port de la variable `PORT` (celle
//! que donnent les hébergeurs), 3000 par défaut (8080 est souvent réservé par Windows).

use std::net::{Ipv4Addr, SocketAddr};

const DEFAULT_PORT: u16 = 3000;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Fight of Pions : serveur sur ws://{address}/ws");
    axum::serve(listener, serveur::app()).await
}
