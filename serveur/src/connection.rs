//! Une connexion WebSocket : elle lit les messages du client, les traduit en demandes au
//! hall ou à sa salle, et écrit les réponses. Aucune règle ici.

use crate::cards;
use crate::limits::OUTBOX_CAPACITY;
use crate::lobby::{Lobby, Outbox, Seating, SharedTable, lock};
use crate::{ClientMessage, ServerError, ServerMessage};
use axum::extract::ws::{Message, WebSocket};
use fight_of_pions::Player;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};

/// La place d'un client connecté, une fois assis.
type Place = Option<(SharedTable, Player)>;

/// Sert un client jusqu'à ce qu'il se déconnecte. Sa place lui reste : il peut revenir
/// avec sa clé ([`ClientMessage::Rejoin`]).
pub async fn serve(socket: WebSocket, lobby: Arc<Lobby>) {
    let (mut sink, mut stream) = socket.split();
    let (outbox, mut inbox) = mpsc::channel::<ServerMessage>(OUTBOX_CAPACITY);
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
        if let Err(error) = handle(&lobby, &mut place, &outbox, text.as_str()).await
            && outbox.send(ServerMessage::error(&error)).await.is_err()
        {
            break;
        }
        // Après chaque message, même refusé : un bot qui attend est relancé.
        if let Some((table, _)) = &place {
            wake_bots(table, lobby.bot_slots());
        }
    }
    if let Some((table, player)) = place {
        lock(&table).disconnect(player, &outbox);
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
            let (table, player) = seated(place, outbox)?;
            lock(table).play(player, action)?;
        }
        ClientMessage::Preview { actions } => {
            let (table, player) = seated(place, outbox)?;
            let preview = lock(table).preview(player, &actions)?;
            let _ = outbox.send(preview).await;
        }
        ClientMessage::Sync => {
            let (table, player) = seated(place, outbox)?;
            let state = lock(table).state(player);
            let _ = outbox.send(state).await;
        }
        ClientMessage::Cards => {
            let cards = cards::catalog()?.to_vec();
            let _ = outbox.send(ServerMessage::Cards { cards }).await;
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
                ClientMessage::Rejoin { code, player, key } => lobby.rejoin(&code, player, &key)?,
                ClientMessage::Play { .. }
                | ClientMessage::Preview { .. }
                | ClientMessage::Sync
                | ClientMessage::Cards => return Err(ServerError::NotSeated),
            };
            *place = Some(sit(seating, outbox).await);
        }
    }
    Ok(())
}

/// La salle et la place de ce client, s'il est assis et toujours branché dessus.
fn seated<'a>(place: &'a Place, outbox: &Outbox) -> Result<(&'a SharedTable, Player), ServerError> {
    let (table, player) = place.as_ref().ok_or(ServerError::NotSeated)?;
    lock(table).check_connected(*player, outbox)?;
    Ok((table, *player))
}

/// Annonce sa place au client, puis le branche sur la salle, qui lui envoie sa vue.
async fn sit(seating: Seating, outbox: &Outbox) -> (SharedTable, Player) {
    let Seating {
        table,
        code,
        player,
        key,
    } = seating;
    let _ = outbox
        .send(ServerMessage::Seated { code, player, key })
        .await;
    lock(&table).connect(player, outbox.clone());
    (table, player)
}

/// Fait jouer les bots de cette salle tant que c'est à eux de décider, dans une tâche à
/// part : la connexion continue de lire ses messages. Chaque recherche tourne sur une
/// copie de la partie, hors du verrou de la salle, et attend un des cœurs réservés aux
/// bots ; la salle n'est verrouillée que pour jouer le coup trouvé et l'envoyer à tous.
fn wake_bots(table: &SharedTable, slots: &Arc<Semaphore>) {
    let Some(mut search) = lock(table).start_bots() else {
        return;
    };
    let table = Arc::clone(table);
    let slots = Arc::clone(slots);
    tokio::spawn(async move {
        loop {
            let Ok(slot) = Arc::clone(&slots).acquire_owned().await else {
                lock(&table).stop_bots(&ServerError::BotFailed("serveur arrêté".to_string()));
                return;
            };
            let found = tokio::task::spawn_blocking(move || search.think()).await;
            drop(slot);
            let found = match found {
                Ok(found) => Some(found),
                // Une panique du bot : un bug, rattrapé par le premier coup permis.
                Err(error) => {
                    eprintln!("bot en panique : {error}");
                    None
                }
            };
            let next = lock(&table).finish_bot(found);
            match next {
                Ok(Some(next)) => search = next,
                Ok(None) => return,
                Err(error) => {
                    eprintln!("{error}");
                    lock(&table).stop_bots(&error);
                    return;
                }
            }
        }
    });
}
