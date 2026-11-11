//! Le client web servi sur `/`, à côté de la WebSocket : une seule adresse pour la page et
//! le jeu. Requêtes HTTP écrites à la main, sur une vraie connexion.

use serveur::Database;
use std::net::SocketAddr;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn start_server(web: &Path) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let database = Database::in_memory().await.unwrap();
    let app = serveur::app(database, Some(web));
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
}

/// La réponse entière à `GET path`, en texte (le corps compressé reste illisible).
async fn get(address: SocketAddr, path: &str, gzip: bool) -> String {
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let encoding = if gzip {
        "Accept-Encoding: gzip\r\n"
    } else {
        ""
    };
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: pions\r\n{encoding}Connection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8_lossy(&response).to_lowercase()
}

#[tokio::test]
async fn the_web_client_is_served_next_to_the_websocket() {
    let folder = std::env::temp_dir().join(format!("pions-web-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let page = format!("<p>Fight of Pions</p>{}", " ".repeat(2000));
    std::fs::write(folder.join("index.html"), &page).unwrap();
    std::fs::write(folder.join("index.wasm"), b"\0asm").unwrap();
    // Un fichier déjà compressé à côté de l'original : envoyé tel quel.
    std::fs::write(folder.join("index.pck"), b"original").unwrap();
    std::fs::write(folder.join("index.pck.gz"), b"precompresse").unwrap();
    let address = start_server(&folder).await;

    let index = get(address, "/", false).await;
    assert!(index.starts_with("http/1.1 200"), "{index}");
    assert!(
        index.contains("fight of pions"),
        "la page d'accueil est index.html"
    );
    let wasm = get(address, "/index.wasm", false).await;
    assert!(wasm.contains("content-type: application/wasm"), "{wasm}");
    assert!(
        index.contains("cache-control: no-cache"),
        "revalidée à chaque visite"
    );
    let compressed = get(address, "/index.html", true).await;
    assert!(
        compressed.contains("content-encoding: gzip"),
        "{compressed}"
    );
    let precompressed = get(address, "/index.pck", true).await;
    assert!(
        precompressed.contains("content-encoding: gzip") && precompressed.ends_with("precompresse"),
        "{precompressed}"
    );
    assert!(
        get(address, "/index.pck", false)
            .await
            .ends_with("original")
    );
    assert!(
        get(address, "/absent.js", false)
            .await
            .starts_with("http/1.1 404")
    );
    assert!(get(address, "/health", false).await.ends_with("ok"));

    let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .expect("la WebSocket répond toujours");
    drop(socket);
    std::fs::remove_dir_all(&folder).unwrap();
}
