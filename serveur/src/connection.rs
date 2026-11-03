//! Une connexion WebSocket : elle lit les messages du client, les traduit en demandes au
//! hall ou à sa salle, et écrit les réponses. Aucune règle ici.

use crate::lobby::{Lobby, Outbox, Seating, SharedTable, lock};
use crate::{ClientMessage, ServerError, ServerMessage};
use axum::extract::ws::{Message, WebSocket};
use fight_of_pions::Player;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

/// La place d'un client connecté, une fois assis.
type Place = Option<(SharedTable, Player)>;

/// Sert un client jusqu'à ce qu'il se déconnecte. Sa place lui reste : il peut revenir
/// avec son jeton ([`ClientMessage::Rejoin`]).
pub async fn serve(socket: WebSocket, lobby: Arc<Lobby>) {
    let (mut sink, mut stream) = socket.split();
    let (outbox, mut inbox) = mpsc::unbounded_channel::<ServerMessage>();
    // Un seul écrivain sur la WebSocket : les messages de la salle (coups adverses, coups
    // des bots) et les réponses arrivent tous par `outbox`.
    let writer = tokio::spawn(async move {
        while let Some(message) = inbox.recv().await {
            let Ok(text) = serde_json::to_string(&message) else {
                continue;
            };
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });
    let mut place: Place = None;
    while let Some(Ok(message)) = stream.next().await {
        let Message::Text(text) = message else {
            continue;
        };
        if let Err(error) = handle(&lobby, &mut place, &outbox, text.as_str()).await {
            let _ = outbox.send(ServerMessage::Error {
                message: error.to_string(),
            });
        }
    }
    if let Some((table, player)) = place {
        lock(&table).disconnect(player);
    }
    drop(outbox);
    let _ = writer.await;
}

async fn handle(
    lobby: &Lobby,
    place: &mut Place,
    outbox: &Outbox,
    text: &str,
) -> Result<(), ServerError> {
    let message: ClientMessage =
        serde_json::from_str(text).map_err(|error| ServerError::BadMessage(error.to_string()))?;
    match message {
        ClientMessage::Play { action } => {
            let (table, player) = place.as_ref().ok_or(ServerError::NotSeated)?;
            lock(table).play(*player, action)?;
        }
        ClientMessage::Preview { actions } => {
            let (table, player) = place.as_ref().ok_or(ServerError::NotSeated)?;
            let preview = lock(table).preview(*player, &actions)?;
            let _ = outbox.send(preview);
            return Ok(());
        }
        seat_request => {
            if place.is_some() {
                return Err(ServerError::AlreadySeated);
            }
            let seating = match seat_request {
                ClientMessage::Create { opponent, variant } => {
                    lobby.create(opponent, variant.as_deref())?
                }
                ClientMessage::Join { code } => lobby.join(&code)?,
                ClientMessage::Rejoin {
                    code,
                    player,
                    token,
                } => lobby.rejoin(&code, player, &token)?,
                ClientMessage::Play { .. } | ClientMessage::Preview { .. } => {
                    return Err(ServerError::NotSeated);
                }
            };
            *place = Some(sit(seating, outbox));
        }
    }
    if let Some((table, _)) = place {
        let_bots_play(Arc::clone(table)).await;
    }
    Ok(())
}

/// Annonce sa place au client, puis le branche sur la salle, qui lui envoie sa vue.
fn sit(seating: Seating, outbox: &Outbox) -> (SharedTable, Player) {
    let Seating {
        table,
        code,
        player,
        token,
    } = seating;
    let _ = outbox.send(ServerMessage::Seated {
        code,
        player,
        token,
    });
    lock(&table).connect(player, outbox.clone());
    (table, player)
}

/// Fait jouer les bots tant que c'est à eux de décider, un coup à la fois, chacun envoyé à
/// tous. La recherche d'un bot occupe un cœur : elle tourne à part, sans bloquer les
/// autres connexions.
async fn let_bots_play(table: SharedTable) {
    loop {
        let shared = Arc::clone(&table);
        match tokio::task::spawn_blocking(move || lock(&shared).play_bot()).await {
            Ok(Ok(true)) => {}
            Ok(Ok(false)) => break,
            // Un coup de bot refusé ou une panique : un bug, signalé, la partie s'arrête là.
            Ok(Err(error)) => {
                eprintln!("bot refusé : {error}");
                break;
            }
            Err(error) => {
                eprintln!("bot en panique : {error}");
                break;
            }
        }
    }
}
