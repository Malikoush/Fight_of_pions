//! Les demandes d'un client : chaque message du protocole devient une demande au hall, à
//! sa salle ou à la base, et sa réponse. Aucune règle ici.

use crate::Shared;
use crate::account::{
    Account, checked_pseudo, default_pseudo, new_recovery_code, new_token, recovery_hash,
    token_hash,
};
use crate::cards;
use crate::collection::{Collection, starter_cards};
use crate::lobby::{Entrant, Outbox, Seating, SharedTable, lock};
use crate::{ClientMessage, Opponent, ServerError, ServerMessage};
use fight_of_pions::{Action, Attack, Player};
use std::time::Instant;

/// La place d'un client assis : sa salle, son code et sa place.
struct Place {
    table: SharedTable,
    code: String,
    player: Player,
}

/// Un client connecté : ce que le serveur sait de lui, et où lui répondre.
pub struct Client {
    shared: Shared,
    outbox: Outbox,
    /// Sa place, une fois assis.
    place: Option<Place>,
    /// Son compte, une fois connecté.
    account: Option<Account>,
}

impl Client {
    pub fn new(shared: Shared, outbox: Outbox) -> Client {
        Client {
            shared,
            outbox,
            place: None,
            account: None,
        }
    }

    /// Traite ce message du client. Un refus ne change rien ; il est à lui renvoyer.
    pub async fn handle(&mut self, text: &str) -> Result<(), ServerError> {
        let message: ClientMessage = serde_json::from_str(text)
            .map_err(|error| ServerError::BadMessage(error.to_string()))?;
        match message {
            ClientMessage::Cards => {
                let cards = cards::catalog()?.to_vec();
                self.reply(ServerMessage::Cards { cards }).await;
            }
            ClientMessage::Register { pseudo } => self.register(pseudo).await?,
            ClientMessage::Login { token } => self.login(&token).await?,
            ClientMessage::Recover { code } => self.recover(&code).await?,
            ClientMessage::NewRecoveryCode => self.new_recovery_code().await?,
            ClientMessage::SetPseudo { pseudo } => self.set_pseudo(&pseudo).await?,
            ClientMessage::Collection => {
                let collection = self.collection().await?.1;
                self.reply(collection_message(collection)).await;
            }
            ClientMessage::SetCacheTrio { cards } => self.set_cache_trio(cards).await?,
            ClientMessage::Create { opponent, variant } => {
                self.create(opponent, variant.as_deref()).await?;
            }
            ClientMessage::Join { code } => self.join(&code).await?,
            ClientMessage::Rejoin { code, player, key } => {
                self.rejoin(&code, player, &key).await?;
            }
            ClientMessage::Leave => self.leave().await,
            ClientMessage::Launch => {
                let (table, player) = self.seated()?;
                lock(table).launch(player)?;
            }
            ClientMessage::Sync => {
                let (table, player) = self.seated()?;
                let state = lock(table).state(player);
                self.reply(state).await;
            }
            ClientMessage::Play { action } => {
                let (table, player) = self.seated()?;
                lock(table).play(player, action)?;
            }
            ClientMessage::Preview { actions } => self.preview(&actions).await?,
        }
        Ok(())
    }

    /// Vrai une fois le client connecté à un compte.
    pub fn is_logged_in(&self) -> bool {
        self.account.is_some()
    }

    /// Après chaque message, même refusé : une partie qui vient de finir est enregistrée,
    /// et un bot qui attend est relancé.
    pub fn after_message(&self) {
        if let Some(place) = &self.place {
            self.shared.background.after_change(&place.table);
        }
    }

    /// La connexion se ferme : le client est débranché de sa place, qui lui reste.
    pub fn disconnect(self) {
        if let Some(place) = self.place {
            lock(&place.table).disconnect(place.player, &self.outbox);
        }
    }

    // --- Le compte ---

    async fn register(&mut self, pseudo: Option<String>) -> Result<(), ServerError> {
        self.refuse_if_logged_in()?;
        let pseudo = match pseudo {
            Some(pseudo) => checked_pseudo(&pseudo)?,
            None => default_pseudo(),
        };
        if !self.shared.registrations.allow(Instant::now()) {
            return Err(ServerError::TooManyRegistrations);
        }
        let token = new_token();
        let account = self
            .shared
            .background
            .database()
            .create_account(&pseudo, &token_hash(&token), starter_cards()?)
            .await?;
        self.log_in(account, Some(token)).await
    }

    async fn login(&mut self, token: &str) -> Result<(), ServerError> {
        self.refuse_if_logged_in()?;
        let account = self
            .shared
            .background
            .database()
            .account_by_token(&token_hash(token))
            .await?
            .ok_or(ServerError::UnknownAccount)?;
        self.log_in(account, None).await
    }

    /// Retrouve le compte de ce code de récupération : cet appareil reçoit un jeton à lui.
    async fn recover(&mut self, code: &str) -> Result<(), ServerError> {
        self.refuse_if_logged_in()?;
        let hash = recovery_hash(code).ok_or(ServerError::BadRecoveryCode)?;
        let token = new_token();
        let account = self
            .shared
            .background
            .database()
            .recover(&hash, &token_hash(&token))
            .await?
            .ok_or(ServerError::BadRecoveryCode)?;
        self.log_in(account, Some(token)).await
    }

    /// Un code de récupération neuf pour ce compte, à la place de l'ancien.
    async fn new_recovery_code(&mut self) -> Result<(), ServerError> {
        let account = self.account.as_ref().ok_or(ServerError::NotLoggedIn)?.id;
        let code = new_recovery_code();
        let hash = recovery_hash(&code).ok_or(ServerError::Storage)?;
        self.shared
            .background
            .database()
            .set_recovery(account, &hash)
            .await?;
        self.reply(ServerMessage::RecoveryCode { code }).await;
        Ok(())
    }

    /// Connecte cette connexion à ce compte, et lui envoie son compte (avec son jeton s'il
    /// vient d'être créé), puis sa place s'il était assis dans une salle encore ouverte (il
    /// y est rassis), puis sa collection.
    async fn log_in(&mut self, account: Account, token: Option<String>) -> Result<(), ServerError> {
        let greeting = |resumed| account_message(&account, token.clone(), resumed);
        let resumed = match self.place {
            None => self.shared.lobby.resume(account.id, &self.outbox, greeting),
            Some(_) => None,
        };
        match resumed {
            Some((table, code, player)) => {
                self.place = Some(Place {
                    table,
                    code,
                    player,
                });
            }
            None => self.reply(greeting(false)).await,
        }
        self.account = Some(account);
        let collection = self.collection().await?.1;
        self.reply(collection_message(collection)).await;
        Ok(())
    }

    fn refuse_if_logged_in(&self) -> Result<(), ServerError> {
        match self.account {
            Some(_) => Err(ServerError::AlreadyLoggedIn),
            None => Ok(()),
        }
    }

    async fn set_pseudo(&mut self, pseudo: &str) -> Result<(), ServerError> {
        let pseudo = checked_pseudo(pseudo)?;
        let account = self.account.as_mut().ok_or(ServerError::NotLoggedIn)?;
        self.shared
            .background
            .database()
            .set_pseudo(account.id, &pseudo)
            .await?;
        account.pseudo = pseudo;
        let message = account_message(account, None, false);
        self.reply(message).await;
        Ok(())
    }

    async fn set_cache_trio(&mut self, cards: Vec<Attack>) -> Result<(), ServerError> {
        let (account, collection) = self.collection().await?;
        collection.check_trio(&cards)?;
        self.shared
            .background
            .database()
            .set_cache_trio(account.id, &cards)
            .await?;
        let collection = Collection {
            cache_trio: cards,
            ..collection
        };
        self.reply(collection_message(collection)).await;
        Ok(())
    }

    /// Le compte de ce client, s'il est connecté, et sa collection.
    async fn collection(&self) -> Result<(&Account, Collection), ServerError> {
        let account = self.account.as_ref().ok_or(ServerError::NotLoggedIn)?;
        let collection = self
            .shared
            .background
            .database()
            .collection(account.id)
            .await?;
        Ok((account, collection))
    }

    // --- La salle ---

    async fn create(
        &mut self,
        opponent: Opponent,
        variant: Option<&str>,
    ) -> Result<(), ServerError> {
        self.refuse_if_seated()?;
        let (account, collection) = self.collection().await?;
        let entrant = entrant(account, &collection);
        let seating = self.shared.lobby.create(opponent, variant, Some(entrant))?;
        self.sit(seating).await;
        Ok(())
    }

    async fn join(&mut self, code: &str) -> Result<(), ServerError> {
        self.refuse_if_seated()?;
        let (account, collection) = self.collection().await?;
        let entrant = entrant(account, &collection);
        let seating = self.shared.lobby.join(code, Some(entrant))?;
        self.sit(seating).await;
        Ok(())
    }

    /// Reprendre sa place ne demande que sa clé, depuis son compte : la cache a été limitée
    /// en s'asseyant.
    async fn rejoin(&mut self, code: &str, player: Player, key: &str) -> Result<(), ServerError> {
        self.refuse_if_seated()?;
        let account = self.account.as_ref().map(|account| account.id);
        let seating = self.shared.lobby.rejoin(code, player, key, account)?;
        self.sit(seating).await;
        Ok(())
    }

    /// Quitte sa salle, s'il en a une (voir [`ClientMessage::Leave`]) ; répond toujours
    /// `left`.
    async fn leave(&mut self) {
        if let Some(place) = self.place.take() {
            let account = self.account.as_ref().map(|account| account.id);
            let departure =
                self.shared
                    .lobby
                    .leave(account, &place.code, place.player, &self.outbox);
            // Une place reprise par une autre connexion n'est pas libérée.
            if departure.is_some() {
                self.shared.background.after_change(&place.table);
            }
        }
        self.reply(ServerMessage::Left).await;
    }

    async fn preview(&self, actions: &[Action]) -> Result<(), ServerError> {
        let (table, player) = self.seated()?;
        let preview = lock(table).preview(player, actions)?;
        self.reply(preview).await;
        Ok(())
    }

    /// Refuse de l'asseoir s'il est déjà assis. Une place perdue (salle fermée par son
    /// créateur, place reprise par une autre connexion) ne compte plus : elle est oubliée.
    fn refuse_if_seated(&mut self) -> Result<(), ServerError> {
        if self.seated().is_err() {
            self.place = None;
        }
        match self.place {
            Some(_) => Err(ServerError::AlreadySeated),
            None => Ok(()),
        }
    }

    /// La salle et la place de ce client, s'il est assis et toujours branché dessus.
    fn seated(&self) -> Result<(&SharedTable, Player), ServerError> {
        let place = self.place.as_ref().ok_or(ServerError::NotSeated)?;
        lock(&place.table).check_connected(place.player, &self.outbox)?;
        Ok((&place.table, place.player))
    }

    /// Annonce sa place au client, puis le branche sur la salle, qui lui envoie sa vue.
    async fn sit(&mut self, seating: Seating) {
        let Seating {
            table,
            code,
            player,
            key,
            launched,
        } = seating;
        self.reply(ServerMessage::Seated {
            code: code.clone(),
            player,
            key,
            launched,
        })
        .await;
        lock(&table).connect(player, self.outbox.clone());
        self.place = Some(Place {
            table,
            code,
            player,
        });
    }

    /// Répond au client. S'il ne lit plus, l'écrivain de sa connexion abandonne et la
    /// réponse est perdue avec elle.
    async fn reply(&self, message: ServerMessage) {
        let _ = self.outbox.send(message).await;
    }
}

/// Celui qui s'assoit : ce compte, avec sa collection.
fn entrant<'a>(account: &'a Account, collection: &'a Collection) -> Entrant<'a> {
    Entrant {
        account: account.id,
        pseudo: &account.pseudo,
        collection,
    }
}

fn account_message(account: &Account, token: Option<String>, resumed: bool) -> ServerMessage {
    ServerMessage::Account {
        id: account.id,
        pseudo: account.pseudo.clone(),
        token,
        resumed,
    }
}

fn collection_message(collection: Collection) -> ServerMessage {
    ServerMessage::Collection {
        cards: collection.cards,
        cache_trio: collection.cache_trio,
    }
}
