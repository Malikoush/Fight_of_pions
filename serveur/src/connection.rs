//! Une connexion WebSocket : elle lit les messages du client, les traduit en demandes au
//! hall, à sa salle ou à la base, et écrit les réponses. Aucune règle ici.

use crate::account::{Account, checked_pseudo, default_pseudo, new_token, token_hash};
use crate::cards;
use crate::collection::{Collection, starter_cards};
use crate::database::Database;
use crate::limits::OUTBOX_CAPACITY;
use crate::lobby::{Entrant, Lobby, Outbox, Seating, SharedTable, lock};
use crate::room::FinishedGame;
use crate::{ClientMessage, ServerError, ServerMessage};
use axum::extract::ws::{Message, WebSocket};
use fight_of_pions::Player;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};

/// La place d'un client connecté, une fois assis.
type Place = Option<(SharedTable, Player)>;

/// Ce que le serveur sait du client de cette connexion.
#[derive(Default)]
struct Client {
    /// Sa place, une fois assis.
    place: Place,
    /// Son compte, une fois connecté.
    account: Option<Account>,
}

/// Sert un client jusqu'à ce qu'il se déconnecte. Sa place lui reste : il peut revenir
/// avec sa clé ([`ClientMessage::Rejoin`]).
pub async fn serve(socket: WebSocket, lobby: Arc<Lobby>, database: Database) {
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
    let mut client = Client::default();
    while let Some(Ok(message)) = stream.next().await {
        let Message::Text(text) = message else {
            continue;
        };
        if let Err(error) = handle(&lobby, &database, &mut client, &outbox, text.as_str()).await
            && outbox.send(ServerMessage::error(&error)).await.is_err()
        {
            break;
        }
        // Après chaque message, même refusé : une partie qui vient de finir est enregistrée,
        // et un bot qui attend est relancé.
        if let Some((table, _)) = &client.place {
            record_if_finished(table, &database);
            wake_bots(table, lobby.bot_slots(), &database);
        }
    }
    if let Some((table, player)) = client.place {
        lock(&table).disconnect(player, &outbox);
    }
    drop(outbox);
    let _ = writer.await;
}

async fn handle(
    lobby: &Lobby,
    database: &Database,
    client: &mut Client,
    outbox: &Outbox,
    text: &str,
) -> Result<(), ServerError> {
    let message: ClientMessage =
        serde_json::from_str(text).map_err(|error| ServerError::BadMessage(error.to_string()))?;
    match message {
        ClientMessage::Play { action } => {
            let (table, player) = seated(&client.place, outbox)?;
            lock(table).play(player, action)?;
        }
        ClientMessage::Preview { actions } => {
            let (table, player) = seated(&client.place, outbox)?;
            let preview = lock(table).preview(player, &actions)?;
            let _ = outbox.send(preview).await;
        }
        ClientMessage::Sync => {
            let (table, player) = seated(&client.place, outbox)?;
            let state = lock(table).state(player);
            let _ = outbox.send(state).await;
        }
        ClientMessage::Cards => {
            let cards = cards::catalog()?.to_vec();
            let _ = outbox.send(ServerMessage::Cards { cards }).await;
        }
        ClientMessage::Register { pseudo } => {
            if client.account.is_some() {
                return Err(ServerError::AlreadyLoggedIn);
            }
            let pseudo = match pseudo {
                Some(pseudo) => checked_pseudo(&pseudo)?,
                None => default_pseudo(),
            };
            let token = new_token();
            let account = database
                .create_account(&pseudo, &token_hash(&token), starter_cards()?)
                .await?;
            log_in(database, client, account, Some(token), outbox).await?;
        }
        ClientMessage::Login { token } => {
            if client.account.is_some() {
                return Err(ServerError::AlreadyLoggedIn);
            }
            let account = database
                .account_by_token(&token_hash(&token))
                .await?
                .ok_or(ServerError::UnknownAccount)?;
            log_in(database, client, account, None, outbox).await?;
        }
        ClientMessage::SetPseudo { pseudo } => {
            let account = client.account.as_mut().ok_or(ServerError::NotLoggedIn)?;
            let pseudo = checked_pseudo(&pseudo)?;
            database.set_pseudo(account.id, &pseudo).await?;
            account.pseudo = pseudo;
            let _ = outbox.send(account_message(account, None)).await;
        }
        ClientMessage::Collection => {
            let collection = collection_of(database, client).await?;
            let _ = outbox.send(collection_message(collection)).await;
        }
        ClientMessage::SetCacheTrio { cards } => {
            let (account, collection) = account_and_collection(database, client).await?;
            collection.check_trio(&cards)?;
            database.set_cache_trio(account, &cards).await?;
            let collection = Collection {
                cache_trio: cards,
                ..collection
            };
            let _ = outbox.send(collection_message(collection)).await;
        }
        ClientMessage::Create { opponent, variant } => {
            refuse_if_seated(client)?;
            let (account, collection) = account_and_collection(database, client).await?;
            let entrant = Entrant {
                account,
                collection: &collection,
            };
            let seating = lobby.create(opponent, variant.as_deref(), Some(entrant))?;
            client.place = Some(sit(seating, outbox).await);
        }
        ClientMessage::Join { code } => {
            refuse_if_seated(client)?;
            let (account, collection) = account_and_collection(database, client).await?;
            let entrant = Entrant {
                account,
                collection: &collection,
            };
            let seating = lobby.join(&code, Some(entrant))?;
            client.place = Some(sit(seating, outbox).await);
        }
        ClientMessage::Launch => {
            let (table, player) = seated(&client.place, outbox)?;
            lock(table).launch(player)?;
        }
        // Reprendre sa place ne demande que sa clé : la cache a été limitée en s'asseyant.
        ClientMessage::Rejoin { code, player, key } => {
            refuse_if_seated(client)?;
            let seating = lobby.rejoin(&code, player, &key)?;
            client.place = Some(sit(seating, outbox).await);
        }
    }
    Ok(())
}

fn refuse_if_seated(client: &Client) -> Result<(), ServerError> {
    if client.place.is_some() {
        Err(ServerError::AlreadySeated)
    } else {
        Ok(())
    }
}

/// Connecte cette connexion à ce compte, et lui envoie son compte (avec son jeton s'il vient
/// d'être créé) et sa collection.
async fn log_in(
    database: &Database,
    client: &mut Client,
    account: Account,
    token: Option<String>,
    outbox: &Outbox,
) -> Result<(), ServerError> {
    let _ = outbox.send(account_message(&account, token)).await;
    client.account = Some(account);
    let collection = collection_of(database, client).await?;
    let _ = outbox.send(collection_message(collection)).await;
    Ok(())
}

/// La collection du compte de ce client, s'il est connecté.
async fn collection_of(database: &Database, client: &Client) -> Result<Collection, ServerError> {
    Ok(account_and_collection(database, client).await?.1)
}

/// Le numéro du compte de ce client, s'il est connecté, et sa collection.
async fn account_and_collection(
    database: &Database,
    client: &Client,
) -> Result<(i64, Collection), ServerError> {
    let account = client.account.as_ref().ok_or(ServerError::NotLoggedIn)?;
    Ok((account.id, database.collection(account.id).await?))
}

fn account_message(account: &Account, token: Option<String>) -> ServerMessage {
    ServerMessage::Account {
        id: account.id,
        pseudo: account.pseudo.clone(),
        token,
    }
}

fn collection_message(collection: Collection) -> ServerMessage {
    ServerMessage::Collection {
        cards: collection.cards,
        cache_trio: collection.cache_trio,
    }
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
        launched,
    } = seating;
    let _ = outbox
        .send(ServerMessage::Seated {
            code,
            player,
            key,
            launched,
        })
        .await;
    lock(&table).connect(player, outbox.clone());
    (table, player)
}

/// Fait jouer les bots de cette salle tant que c'est à eux de décider, dans une tâche à
/// part : la connexion continue de lire ses messages. Chaque recherche tourne sur une
/// copie de la partie, hors du verrou de la salle, et attend un des cœurs réservés aux
/// bots ; la salle n'est verrouillée que pour jouer le coup trouvé et l'envoyer à tous.
fn wake_bots(table: &SharedTable, slots: &Arc<Semaphore>, database: &Database) {
    let Some(mut search) = lock(table).start_bots() else {
        return;
    };
    let table = Arc::clone(table);
    let slots = Arc::clone(slots);
    let database = database.clone();
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
            record_if_finished(&table, &database);
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

/// Si la partie de cette salle vient de finir, l'enregistre dans une tâche à part (la salle
/// reste libre), puis annonce sa carte au gagnant récompensé. Une partie n'est rendue qu'une
/// fois ([`crate::room::Room::take_finished`]).
fn record_if_finished(table: &SharedTable, database: &Database) {
    let Some((game, variant)) = lock(table).take_finished() else {
        return;
    };
    let table = Arc::clone(table);
    let database = database.clone();
    tokio::spawn(async move { record(&table, &database, &game, &variant).await });
}

async fn record(table: &SharedTable, database: &Database, game: &FinishedGame, variant: &str) {
    match database.record_game(game, variant).await {
        Ok(Some(card)) => {
            if let Some((winner, _)) = game.rewarded() {
                lock(table).send_to(winner, ServerMessage::Reward { card });
            }
        }
        Ok(None) => {}
        Err(error) => eprintln!("partie non enregistrée : {error}"),
    }
}
