//! Une connexion WebSocket : une tâche lit les messages du client et les confie à
//! [`Client`] ; une autre écrit les réponses et les nouvelles de la salle. Aucune règle ici.
//!
//! Une connexion morte ne reste pas ouverte : le serveur envoie un ping régulièrement (le
//! client y répond tout seul), ferme une connexion muette trop longtemps, abandonne un
//! client qui ne lit plus, ferme celui qui envoie trop de trames, et celui qui ne s'est
//! pas connecté à un compte à temps.

use crate::Shared;
use crate::limits::{
    IDLE_LIMIT, LOGIN_LIMIT, MAX_MESSAGES_PER_WINDOW, MESSAGE_WINDOW, OUTBOX_CAPACITY,
    PING_INTERVAL, RateLimit, SEND_LIMIT,
};
use crate::requests::Client;
use crate::{ServerError, ServerMessage};
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::time::Instant;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::mpsc::{self, Receiver};
use tokio::time::timeout;

/// Sert un client jusqu'à ce qu'il se déconnecte, se taise trop longtemps, ou que le
/// serveur s'arrête. Sa place lui reste : se reconnecter à son compte l'y rassoit.
/// `_permit` compte la connexion parmi les connexions ouvertes, jusqu'à sa fin.
pub async fn serve(socket: WebSocket, shared: Shared, _permit: OwnedSemaphorePermit) {
    let (sink, mut stream) = socket.split();
    let (outbox, inbox) = mpsc::channel::<ServerMessage>(OUTBOX_CAPACITY);
    // Un seul écrivain sur la WebSocket : les messages de la salle (coups adverses, coups
    // des bots) et les réponses arrivent tous par `outbox`.
    let writer = tokio::spawn(write(sink, inbox));
    let shutdown = shared.shutdown.clone();
    let rate = RateLimit::new(MAX_MESSAGES_PER_WINDOW, MESSAGE_WINDOW);
    let mut client = Client::new(shared, outbox.clone());
    let login_deadline = tokio::time::Instant::now() + LOGIN_LIMIT;
    loop {
        let message = tokio::select! {
            () = shutdown.cancelled() => break,
            // Une connexion sans compte n'occupe pas une place pour rien.
            () = tokio::time::sleep_until(login_deadline), if !client.is_logged_in() => break,
            // L'écrivain a abandonné : le client ne lit plus.
            () = outbox.closed() => break,
            next = timeout(IDLE_LIMIT, stream.next()) => match next {
                Ok(Some(Ok(message))) => message,
                // Muette trop longtemps, fermée, ou en erreur.
                _ => break,
            },
        };
        // Toute trame compte, même un pong ou une trame binaire.
        if !rate.allow(Instant::now()) {
            let _ = outbox
                .send(ServerMessage::error(&ServerError::TooManyMessages))
                .await;
            break;
        }
        // Un pong, un ping ou une trame binaire prouvent seulement que la connexion vit.
        let Message::Text(text) = message else {
            continue;
        };
        if let Err(error) = client.handle(text.as_str()).await {
            let _ = outbox.send(ServerMessage::error(&error)).await;
        }
        client.after_message();
    }
    client.disconnect();
    // L'écrivain envoie ce qui reste, puis ferme la WebSocket.
    drop(outbox);
    let _ = writer.await;
}

/// Écrit au client les messages qui arrivent, et un ping régulièrement. Abandonne si un
/// envoi attend trop (le client ne lit plus) ; ferme la WebSocket quand il n'y a plus rien
/// à écrire.
async fn write(mut sink: SplitSink<WebSocket, Message>, mut inbox: Receiver<ServerMessage>) {
    let start = tokio::time::Instant::now() + PING_INTERVAL;
    let mut pings = tokio::time::interval_at(start, PING_INTERVAL);
    loop {
        let message = tokio::select! {
            next = inbox.recv() => match next {
                Some(message) => match serde_json::to_string(&message) {
                    Ok(text) => Message::Text(text.into()),
                    Err(_) => continue,
                },
                None => break,
            },
            _ = pings.tick() => Message::Ping(Bytes::new()),
        };
        if !matches!(timeout(SEND_LIMIT, sink.send(message)).await, Ok(Ok(()))) {
            return;
        }
    }
    let _ = timeout(SEND_LIMIT, sink.send(Message::Close(None))).await;
}
