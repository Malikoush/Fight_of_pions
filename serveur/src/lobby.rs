//! Les salles ouvertes, et la diffusion : après chaque changement d'une partie, chaque
//! humain connecté reçoit sa vue et les événements qu'il peut voir. Un compte n'est assis
//! que dans une salle à la fois : le hall sait laquelle, pour l'y rasseoir à sa connexion.

use crate::collection::Collection;
use crate::limits::{ABSENCE_LIMIT, MAX_ROOMS, ROOM_IDLE_LIMIT};
use crate::random::{random_code, random_key, random_u64};
use crate::room::{BotMove, BotSearch, FinishedGame, Room, Seat};
use crate::{Opponent, ServerError, ServerMessage};
use bots::{BotKind, ONLINE_VARIANT, variants};
use fight_of_pions::{Action, Event, GameConfig, Player};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;
use tokio::sync::mpsc::Sender;

/// La longueur d'un code de salle (ses caractères : [`crate::random::CODE_ALPHABET`]).
const CODE_LENGTH: usize = 5;

/// Les variantes qu'on peut jouer en ligne : celle du jeu. Les autres (`bots::variants`)
/// servent aux essais des bots, pas aux joueurs.
const ONLINE_VARIANTS: [&str; 1] = [ONLINE_VARIANT];

/// Où envoyer les messages d'un client connecté.
pub type Outbox = Sender<ServerMessage>;

/// Une salle, et les clients connectés à chacune de ses places.
pub struct Table {
    room: Room,
    /// Le nom de la variante jouée, pour l'enregistrement de la partie.
    variant: String,
    outboxes: [Option<Outbox>; 2],
    /// Le pseudo de chaque humain assis, à montrer à son adversaire ; vide sans compte (les
    /// tests).
    pseudos: [String; 2],
    /// Depuis quand chaque humain est débranché de sa place : trop longtemps pendant une
    /// partie lancée, il la perd par forfait.
    away_since: [Option<Instant>; 2],
    /// Vrai pendant qu'une tâche fait jouer les bots de la salle : il n'y en a qu'une.
    bots_running: bool,
    /// La place de celui qui a créé la salle : lui seul la lance.
    host: Player,
    /// Vrai une fois la partie lancée (tout de suite contre un bot) : avant, personne ne
    /// joue.
    launched: bool,
    /// Le dernier branchement ou débranchement d'un client : une salle abandonnée depuis
    /// trop longtemps est fermée.
    last_seen: Instant,
}

/// Une salle partagée entre les connexions de ses joueurs et la tâche de ses bots.
pub type SharedTable = Arc<Mutex<Table>>;

/// Ce que devient la salle quand un joueur la quitte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Departure {
    /// Elle reste ouverte : partie perdue par forfait, finie, ou place d'invité libérée.
    Kept,
    /// Elle est à fermer : son créateur l'a quittée avant de la lancer.
    Closed,
}

/// Verrouille une table ou le hall. Une tâche qui a paniqué en le tenant ne le rend pas
/// inutilisable : la partie reste cohérente (le moteur ne modifie rien sur un coup
/// refusé).
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Table {
    fn new(room: Room, variant: &str, host: Player, launched: bool) -> Table {
        Table {
            room,
            variant: variant.to_string(),
            outboxes: [None, None],
            pseudos: [String::new(), String::new()],
            away_since: [None, None],
            bots_running: false,
            host,
            launched,
            last_seen: Instant::now(),
        }
    }

    /// Branche ce client sur cette place, et lui envoie où il en est. Un client déjà
    /// branché à cette place (une ancienne connexion) apprend qu'elle a été reprise, et ne
    /// reçoit plus rien. Face à un humain, chacun apprend si l'autre est là, et son pseudo.
    pub fn connect(&mut self, player: Player, outbox: Outbox) {
        if let Some(old) = self.outboxes[player.index()].replace(outbox.clone())
            && !old.same_channel(&outbox)
        {
            let _ = old.try_send(ServerMessage::error(&ServerError::SeatTaken));
        }
        self.away_since[player.index()] = None;
        self.last_seen = Instant::now();
        self.send_state(player, &[]);
        let opponent = player.opponent();
        if self.room.is_human(opponent) {
            let present = self.outboxes[opponent.index()].is_some();
            self.send_presence(player, present);
            if present {
                self.send_presence(opponent, true);
            }
        }
    }

    /// Débranche ce client de cette place, s'il y est encore branché (une reconnexion a
    /// pu le remplacer) ; sa place lui reste, avec sa clé.
    pub fn disconnect(&mut self, player: Player, outbox: &Outbox) {
        if self.unplug(player, outbox) {
            self.away_since[player.index()] = Some(Instant::now());
            self.send_presence(player.opponent(), false);
        }
    }

    /// Ce joueur quitte la salle :
    /// - partie lancée et pas finie : il la perd par forfait ;
    /// - avant le lancement, l'invité libère sa place ; le créateur ferme la salle, et son
    ///   invité l'apprend (`room_closed`).
    ///
    /// `None` si ce client n'est plus branché sur cette place (une autre connexion l'a
    /// reprise) : rien ne change.
    pub fn leave(&mut self, player: Player, outbox: &Outbox) -> Option<Departure> {
        if !self.unplug(player, outbox) {
            return None;
        }
        let opponent = player.opponent();
        if !self.launched && player == self.host {
            if let Some(guest) = self.outboxes[opponent.index()].take() {
                let _ = guest.try_send(ServerMessage::error(&ServerError::RoomClosed));
            }
            return Some(Departure::Closed);
        }
        if !self.launched {
            // Libérer la place ne peut échouer : la cache se limite à tout le catalogue.
            let _ = self.room.free(player);
            self.pseudos[player.index()].clear();
        } else if !self.room.is_over()
            && let Ok(events) = self.room.resign(player)
        {
            self.broadcast(&events);
        }
        self.send_presence(opponent, false);
        Some(Departure::Kept)
    }

    /// Les joueurs débranchés depuis trop longtemps d'une partie lancée la perdent par
    /// forfait ; vrai si la partie vient ainsi de finir.
    fn forfeit_absent(&mut self, now: Instant) -> bool {
        if !self.launched || self.room.is_over() {
            return false;
        }
        let absent = [Player::One, Player::Two].into_iter().find(|player| {
            self.away_since[player.index()]
                .is_some_and(|since| now.duration_since(since) >= ABSENCE_LIMIT)
        });
        let Some(absent) = absent else {
            return false;
        };
        match self.room.resign(absent) {
            Ok(events) => {
                self.broadcast(&events);
                true
            }
            Err(_) => false,
        }
    }

    /// Retire ce client de sa place, s'il y est encore branché ; vrai s'il l'était.
    fn unplug(&mut self, player: Player, outbox: &Outbox) -> bool {
        let slot = &mut self.outboxes[player.index()];
        if slot.as_ref().is_some_and(|own| own.same_channel(outbox)) {
            *slot = None;
            self.last_seen = Instant::now();
            true
        } else {
            false
        }
    }

    /// La partie et le nom de sa variante, si elle vient de finir : rendue une seule fois,
    /// pour être enregistrée (E4c).
    pub fn take_finished(&mut self) -> Option<(FinishedGame, String)> {
        let finished = self.room.take_finished()?;
        Some((finished, self.variant.clone()))
    }

    /// Envoie ce message à ce joueur, s'il est branché.
    pub fn send_to(&self, player: Player, message: ServerMessage) {
        if let Some(outbox) = &self.outboxes[player.index()] {
            let _ = outbox.try_send(message);
        }
    }

    /// Vérifie que ce client est bien celui branché sur cette place : une connexion
    /// remplacée par une reconnexion ne joue plus.
    pub fn check_connected(&self, player: Player, outbox: &Outbox) -> Result<(), ServerError> {
        match &self.outboxes[player.index()] {
            Some(own) if own.same_channel(outbox) => Ok(()),
            _ => Err(ServerError::SeatTaken),
        }
    }

    /// La vue actuelle de ce joueur, sans événement : de quoi se remettre à jour.
    pub fn state(&self, player: Player) -> ServerMessage {
        ServerMessage::State {
            view: Box::new(self.room.view(player)),
            events: Vec::new(),
        }
    }

    /// Lance la partie, à la demande de ce joueur : le créateur de la salle, une fois son
    /// adversaire assis et branché (débranché, il perdrait par absence une partie qu'il n'a
    /// pas vue commencer). Les deux joueurs l'apprennent ; une partie déjà lancée le reste.
    pub fn launch(&mut self, player: Player) -> Result<(), ServerError> {
        if self.launched {
            return Ok(());
        }
        if player != self.host {
            return Err(ServerError::NotHost);
        }
        let opponent = player.opponent();
        if self.room.free_seat().is_some()
            || (self.room.is_human(opponent) && self.outboxes[opponent.index()].is_none())
        {
            return Err(ServerError::OpponentMissing);
        }
        self.launched = true;
        self.send_all(&ServerMessage::Launched);
        Ok(())
    }

    /// Joue le coup de cet humain, puis l'envoie à tous.
    pub fn play(&mut self, player: Player, action: Action) -> Result<(), ServerError> {
        self.check_launched()?;
        let events = self.room.play(player, action)?;
        self.broadcast(&events);
        Ok(())
    }

    /// Ce que verrait ce joueur au fil des choix d'une carte, sans rien jouer.
    pub fn preview(
        &self,
        player: Player,
        actions: &[Action],
    ) -> Result<ServerMessage, ServerError> {
        self.check_launched()?;
        let view = self.room.preview(player, actions)?;
        Ok(ServerMessage::Preview {
            view: view.map(Box::new),
        })
    }

    /// La recherche du bot qui doit décider, si c'est à un bot et qu'aucune tâche ne fait
    /// déjà jouer les bots de la salle. La tâche qui la reçoit les fait jouer jusqu'au
    /// bout ([`Table::finish_bot`]).
    pub fn start_bots(&mut self) -> Option<BotSearch> {
        if self.bots_running || !self.launched {
            return None;
        }
        let search = self.room.bot_search()?;
        self.bots_running = true;
        Some(search)
    }

    /// Joue le coup trouvé par le bot (`None` : sa recherche a paniqué, voir
    /// [`Room::play_bot`]) et l'envoie à tous ; rend la recherche suivante, s'il faut
    /// encore un coup de bot.
    pub fn finish_bot(&mut self, found: Option<BotMove>) -> Result<Option<BotSearch>, ServerError> {
        match self.room.play_bot(found) {
            Ok(Some(events)) => self.broadcast(&events),
            Ok(None) => {}
            Err(error) => {
                self.bots_running = false;
                return Err(error);
            }
        }
        let next = self.room.bot_search();
        self.bots_running = next.is_some();
        Ok(next)
    }

    /// Les bots n'ont pas pu jouer, même le premier coup permis : la partie attend, et
    /// chaque humain connecté est prévenu. Le prochain message d'un humain relancera les
    /// bots.
    pub fn stop_bots(&mut self, error: &ServerError) {
        self.bots_running = false;
        self.send_all(&ServerMessage::error(error));
    }

    /// Envoie ce message à chaque humain branché.
    fn send_all(&self, message: &ServerMessage) {
        for outbox in self.outboxes.iter().flatten() {
            let _ = outbox.try_send(message.clone());
        }
    }

    /// Assied ce joueur avec son compte : sa cache limitée par sa collection, son pseudo
    /// montré à l'adversaire. Sans compte (les tests), rien ne change.
    fn welcome(&mut self, player: Player, entrant: Option<Entrant>) -> Result<(), ServerError> {
        let Some(entrant) = entrant else {
            return Ok(());
        };
        let choices = entrant.collection.cache_choices(&self.room.catalog());
        self.room.limit_cache(player, &choices)?;
        self.pseudos[player.index()] = entrant.pseudo.to_string();
        Ok(())
    }

    fn check_launched(&self) -> Result<(), ServerError> {
        if self.launched {
            Ok(())
        } else {
            Err(ServerError::NotLaunched)
        }
    }

    /// Vrai si personne n'est connecté et que la partie est finie, ou abandonnée depuis
    /// trop longtemps.
    fn is_abandoned(&self, now: Instant) -> bool {
        self.outboxes.iter().all(Option::is_none)
            && (self.room.is_over() || now.duration_since(self.last_seen) >= ROOM_IDLE_LIMIT)
    }

    /// Envoie à chaque humain connecté sa vue et les événements qu'il peut voir.
    fn broadcast(&self, events: &[Event]) {
        for player in [Player::One, Player::Two] {
            self.send_state(player, events);
        }
    }

    /// Dit à ce joueur, s'il est branché, si son adversaire l'est, et son pseudo.
    fn send_presence(&self, player: Player, connected: bool) {
        self.send_to(
            player,
            ServerMessage::Opponent {
                connected,
                pseudo: self.pseudos[player.opponent().index()].clone(),
            },
        );
    }

    fn send_state(&self, player: Player, events: &[Event]) {
        let Some(outbox) = &self.outboxes[player.index()] else {
            return;
        };
        let events = events.iter().map(|event| event.seen_by(player)).collect();
        // Un client parti ou trop lent : cette vue est perdue pour lui ; la suivante, ou
        // un `sync`, le remettra à jour (limits.rs).
        let _ = outbox.try_send(ServerMessage::State {
            view: Box::new(self.room.view(player)),
            events,
        });
    }
}

/// Les salles ouvertes, par code, et la salle de chaque compte assis.
#[derive(Default)]
struct Hall {
    tables: BTreeMap<String, SharedTable>,
    /// La salle (son code) et la place de chaque compte assis. Une salle fermée n'y laisse
    /// rien ; une partie finie y reste jusqu'à ce que le compte la quitte, ou que sa salle
    /// ferme.
    seats: BTreeMap<i64, (String, Player)>,
}

impl Hall {
    /// Refuse un compte déjà assis dans une partie pas finie : un compte, une salle.
    fn check_free(&self, account: i64) -> Result<(), ServerError> {
        let playing = self
            .seats
            .get(&account)
            .and_then(|(code, _)| self.tables.get(code))
            .is_some_and(|table| !lock(table).room.is_over());
        if playing {
            Err(ServerError::AlreadySeated)
        } else {
            Ok(())
        }
    }

    fn table(&self, code: &str) -> Result<SharedTable, ServerError> {
        self.tables
            .get(&code.to_uppercase())
            .cloned()
            .ok_or_else(|| ServerError::UnknownRoom(code.to_string()))
    }

    /// Ferme les salles abandonnées à cet instant, et oublie qui y était assis. Le hall est
    /// verrouillé avant chaque table, jamais l'inverse : pas d'interblocage.
    fn close_abandoned(&mut self, now: Instant) {
        self.tables
            .retain(|_, table| !lock(table).is_abandoned(now));
        self.forget_closed();
    }

    /// Oublie les places des salles qui ne sont plus ouvertes.
    fn forget_closed(&mut self) {
        let tables = &self.tables;
        self.seats.retain(|_, (code, _)| tables.contains_key(code));
    }
}

/// Les salles ouvertes, et la place donnée à leur créateur.
#[derive(Default)]
pub struct Lobby {
    hall: Mutex<Hall>,
    /// La place donnée au créateur d'une salle ; `None` : tirée au sort (le jeu).
    creator_seat: Option<Player>,
}

/// Celui qui s'assoit : son compte, son pseudo, et sa collection, qui limite sa cache.
#[derive(Clone, Copy)]
pub struct Entrant<'a> {
    /// Son compte.
    pub account: i64,
    /// Son pseudo, montré à son adversaire.
    pub pseudo: &'a str,
    /// Ce qu'il possède.
    pub collection: &'a Collection,
}

/// Une place prise : la salle, son code, la place et sa clé, et si la partie est lancée.
pub struct Seating {
    pub table: SharedTable,
    pub code: String,
    pub player: Player,
    pub key: String,
    pub launched: bool,
}

impl Lobby {
    /// Ouvre une salle contre cet adversaire, et y assied le demandeur à une place tirée au
    /// sort (J1 commence), sa cache limitée par sa collection (`None`, pour les tests : sans
    /// compte, la cache dans tout le catalogue). Contre un bot, la partie est lancée tout de
    /// suite. Ferme d'abord les salles abandonnées. Refusé à un compte déjà assis dans une
    /// partie pas finie.
    pub fn create(
        &self,
        opponent: Opponent,
        variant: Option<&str>,
        entrant: Option<Entrant>,
    ) -> Result<Seating, ServerError> {
        let variant = variant.unwrap_or(ONLINE_VARIANT);
        let config = online_config(variant)?;
        let other = match opponent {
            Opponent::Human => Seat::Free,
            Opponent::Mcts => Seat::Bot(BotKind::Mcts),
            Opponent::Random => Seat::Bot(BotKind::Random),
        };
        let launched = !matches!(other, Seat::Free);
        let key = random_key();
        let host = Seat::Human {
            key: key.clone(),
            account: entrant.map(|entrant| entrant.account),
        };
        let player = self.creator_seat();
        let seats = match player {
            Player::One => [host, other],
            Player::Two => [other, host],
        };
        let room = Room::new(config, random_u64(), seats)?;
        let mut table = Table::new(room, variant, player, launched);
        table.welcome(player, entrant)?;
        let table = Arc::new(Mutex::new(table));
        let mut hall = lock(&self.hall);
        hall.close_abandoned(Instant::now());
        if let Some(entrant) = entrant {
            hall.check_free(entrant.account)?;
        }
        if hall.tables.len() >= MAX_ROOMS {
            return Err(ServerError::ServerFull);
        }
        let code = loop {
            let code = random_room_code();
            if !hall.tables.contains_key(&code) {
                break code;
            }
        };
        hall.tables.insert(code.clone(), Arc::clone(&table));
        if let Some(entrant) = entrant {
            hall.seats.insert(entrant.account, (code.clone(), player));
        }
        Ok(Seating {
            table,
            code,
            player,
            key,
            launched,
        })
    }

    /// Assied le demandeur à la place libre de la salle de ce code, sa cache limitée par sa
    /// collection (`None`, pour les tests : tout le catalogue). Refusé à un compte déjà
    /// assis dans une partie pas finie.
    pub fn join(&self, code: &str, entrant: Option<Entrant>) -> Result<Seating, ServerError> {
        let mut hall = lock(&self.hall);
        if let Some(entrant) = entrant {
            hall.check_free(entrant.account)?;
        }
        let table = hall.table(code)?;
        let code = code.to_uppercase();
        let key = random_key();
        let (player, launched) = {
            let mut guard = lock(&table);
            let player = guard.room.free_seat().ok_or(ServerError::RoomFull)?;
            guard.welcome(player, entrant)?;
            let account = entrant.map(|entrant| entrant.account);
            guard.room.sit(player, key.clone(), account)?;
            (player, guard.launched)
        };
        if let Some(entrant) = entrant {
            hall.seats.insert(entrant.account, (code.clone(), player));
        }
        Ok(Seating {
            table,
            code,
            player,
            key,
            launched,
        })
    }

    /// Rend sa place à un joueur qui revient avec sa clé, depuis le compte assis là.
    pub fn rejoin(
        &self,
        code: &str,
        player: Player,
        key: &str,
        account: Option<i64>,
    ) -> Result<Seating, ServerError> {
        let table = lock(&self.hall).table(code)?;
        let launched = {
            let guard = lock(&table);
            guard.room.check_key(player, key, account)?;
            guard.launched
        };
        Ok(Seating {
            table,
            code: code.to_uppercase(),
            player,
            key: key.to_string(),
            launched,
        })
    }

    /// Rassoit ce compte dans sa salle, s'il est assis dans une salle encore ouverte, à
    /// sa connexion : ce client reçoit `greeting(true)` (son compte), sa place (`seated`),
    /// puis la salle le branche et lui envoie sa vue. Tout se fait sous le verrou du hall :
    /// ni un départ, ni un balayage ne passent entre les deux. Sinon, rien n'est envoyé, et
    /// rend `None`.
    pub fn resume(
        &self,
        account: i64,
        outbox: &Outbox,
        greeting: impl FnOnce(bool) -> ServerMessage,
    ) -> Option<(SharedTable, String, Player)> {
        let hall = lock(&self.hall);
        let (code, player) = hall.seats.get(&account)?.clone();
        let table = Arc::clone(hall.tables.get(&code)?);
        let mut guard = lock(&table);
        if guard.room.account_of(player) != Some(account) {
            return None;
        }
        let key = guard.room.key_of(player)?.to_string();
        let _ = outbox.try_send(greeting(true));
        let _ = outbox.try_send(ServerMessage::Seated {
            code: code.clone(),
            player,
            key,
            launched: guard.launched,
        });
        guard.connect(player, outbox.clone());
        drop(guard);
        Some((table, code, player))
    }

    /// Ce client quitte la salle de ce code ([`Table::leave`]) ; son compte n'y est plus
    /// assis, et la salle est fermée si son créateur l'a quittée avant de la lancer. Tout se
    /// fait sous le verrou du hall : une reprise ou un `join` ne voient jamais une place
    /// libérée encore attribuée. `None` si ce client n'y était plus branché : rien ne change.
    pub fn leave(
        &self,
        account: Option<i64>,
        code: &str,
        player: Player,
        outbox: &Outbox,
    ) -> Option<Departure> {
        let mut hall = lock(&self.hall);
        let table = Arc::clone(hall.tables.get(code)?);
        let departure = lock(&table).leave(player, outbox)?;
        if let Some(account) = account
            && hall
                .seats
                .get(&account)
                .is_some_and(|(seated, _)| seated == code)
        {
            hall.seats.remove(&account);
        }
        if departure == Departure::Closed {
            hall.tables.remove(code);
            hall.forget_closed();
        }
        Some(departure)
    }

    /// Le balayage périodique : les joueurs absents trop longtemps perdent leur partie par
    /// forfait, puis les salles abandonnées sont fermées. Rend les salles dont la partie
    /// vient ainsi de finir, à enregistrer.
    pub fn sweep(&self, now: Instant) -> Vec<SharedTable> {
        let mut hall = lock(&self.hall);
        let ended = hall
            .tables
            .values()
            .filter(|table| lock(table).forfeit_absent(now))
            .cloned()
            .collect();
        hall.close_abandoned(now);
        ended
    }

    /// La place du créateur d'une salle : tirée au sort, ou fixée (les tests).
    fn creator_seat(&self) -> Player {
        self.creator_seat.unwrap_or_else(|| {
            if random_u64().is_multiple_of(2) {
                Player::One
            } else {
                Player::Two
            }
        })
    }
}

/// La configuration de cette variante, si elle se joue en ligne.
fn online_config(name: &str) -> Result<GameConfig, ServerError> {
    variants()
        .into_iter()
        .filter(|variant| ONLINE_VARIANTS.contains(&variant.name))
        .find(|variant| variant.name == name)
        .map(|variant| variant.config)
        .ok_or_else(|| ServerError::UnknownVariant(name.to_string()))
}

fn random_room_code() -> String {
    random_code(CODE_LENGTH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::starter_cards;
    use tokio::sync::mpsc::{self, Receiver};

    /// Un hall où le créateur d'une salle est toujours J1.
    fn lobby_seating_the_creator_first() -> Lobby {
        Lobby {
            creator_seat: Some(Player::One),
            ..Lobby::default()
        }
    }

    /// Le paquet de départ, sans trio : la collection d'un compte neuf.
    fn starter() -> Collection {
        Collection {
            cards: starter_cards().unwrap().to_vec(),
            cache_trio: Vec::new(),
        }
    }

    fn entrant<'a>(account: i64, pseudo: &'a str, collection: &'a Collection) -> Entrant<'a> {
        Entrant {
            account,
            pseudo,
            collection,
        }
    }

    /// Branche un client sur cette place ; rend ce qu'il reçoit.
    fn plug(seating: &Seating) -> (Outbox, Receiver<ServerMessage>) {
        let (outbox, inbox) = mpsc::channel(16);
        lock(&seating.table).connect(seating.player, outbox.clone());
        (outbox, inbox)
    }

    /// Les messages reçus jusqu'ici, sauf les vues.
    fn received(inbox: &mut Receiver<ServerMessage>) -> Vec<ServerMessage> {
        std::iter::from_fn(|| inbox.try_recv().ok())
            .filter(|message| !matches!(message, ServerMessage::State { .. }))
            .collect()
    }

    /// Une salle entre amis lancée : Ana (compte 1, J1) et Bob (compte 2, J2), branchés.
    struct Launched {
        lobby: Lobby,
        host: Seating,
        guest: Seating,
        host_line: (Outbox, Receiver<ServerMessage>),
        guest_line: (Outbox, Receiver<ServerMessage>),
    }

    fn launched_friends_room() -> Launched {
        let lobby = lobby_seating_the_creator_first();
        let collection = starter();
        let host = lobby
            .create(Opponent::Human, None, Some(entrant(1, "Ana", &collection)))
            .unwrap();
        let mut host_line = plug(&host);
        let guest = lobby
            .join(&host.code, Some(entrant(2, "Bob", &collection)))
            .unwrap();
        let mut guest_line = plug(&guest);
        lock(&host.table).launch(host.player).unwrap();
        received(&mut host_line.1);
        received(&mut guest_line.1);
        Launched {
            lobby,
            host,
            guest,
            host_line,
            guest_line,
        }
    }

    #[test]
    fn the_creator_seat_is_drawn_at_random() {
        let lobby = Lobby::default();
        let seats: Vec<Player> = (0..64)
            .map(|_| lobby.create(Opponent::Random, None, None).unwrap().player)
            .collect();
        assert!(seats.contains(&Player::One) && seats.contains(&Player::Two));
    }

    #[test]
    fn only_the_online_variant_can_be_played() {
        let lobby = Lobby::default();
        assert!(
            lobby
                .create(Opponent::Random, Some(ONLINE_VARIANT), None)
                .is_ok()
        );
        let trial = variants()
            .into_iter()
            .map(|variant| variant.name)
            .find(|name| *name != ONLINE_VARIANT)
            .unwrap();
        assert!(matches!(
            lobby.create(Opponent::Random, Some(trial), None),
            Err(ServerError::UnknownVariant(_))
        ));
    }

    #[test]
    fn a_friends_room_waits_for_its_creator_to_launch_it() {
        let lobby = lobby_seating_the_creator_first();
        let host = lobby.create(Opponent::Human, None, None).unwrap();
        assert!(!host.launched, "entre amis, la partie attend");
        let (_host_outbox, mut host_inbox) = plug(&host);
        assert_eq!(
            lock(&host.table).launch(host.player),
            Err(ServerError::OpponentMissing)
        );
        let guest = lobby.join(&host.code, None).unwrap();
        assert_eq!(guest.player, Player::Two, "la place laissée libre");
        assert!(!guest.launched);
        let (_guest_outbox, mut guest_inbox) = plug(&guest);
        let mut table = lock(&host.table);
        assert_eq!(
            table.play(Player::One, Action::EndTurn),
            Err(ServerError::NotLaunched)
        );
        assert_eq!(table.launch(guest.player), Err(ServerError::NotHost));
        received(&mut host_inbox);
        received(&mut guest_inbox);
        assert_eq!(table.launch(host.player), Ok(()));
        assert!(table.launched);
        assert_eq!(host_inbox.try_recv(), Ok(ServerMessage::Launched));
        assert_eq!(guest_inbox.try_recv(), Ok(ServerMessage::Launched));
        assert_eq!(table.launch(host.player), Ok(()), "déjà lancée");
        drop(table);
        let back = lobby
            .rejoin(&host.code, guest.player, &guest.key, None)
            .unwrap();
        assert!(back.launched, "qui revient apprend qu'elle est lancée");
        assert!(
            lobby
                .rejoin(&host.code, guest.player, &guest.key, Some(9))
                .is_err(),
            "pas depuis un autre compte"
        );
    }

    #[test]
    fn a_friends_room_is_not_launched_while_the_guest_is_away() {
        let lobby = lobby_seating_the_creator_first();
        let host = lobby.create(Opponent::Human, None, None).unwrap();
        let _host_line = plug(&host);
        let guest = lobby.join(&host.code, None).unwrap();
        let (guest_outbox, _guest_inbox) = plug(&guest);
        lock(&guest.table).disconnect(guest.player, &guest_outbox);
        assert_eq!(
            lock(&host.table).launch(host.player),
            Err(ServerError::OpponentMissing)
        );
        assert!(lobby.sweep(Instant::now() + ABSENCE_LIMIT).is_empty());
        assert!(!lock(&host.table).room.is_over(), "personne ne perd");
    }

    #[test]
    fn each_player_learns_the_presence_and_pseudo_of_the_other() {
        let lobby = lobby_seating_the_creator_first();
        let collection = starter();
        let host = lobby
            .create(Opponent::Human, None, Some(entrant(1, "Ana", &collection)))
            .unwrap();
        let (host_outbox, mut host_inbox) = plug(&host);
        assert_eq!(received(&mut host_inbox), [], "personne en face");
        let guest = lobby
            .join(&host.code, Some(entrant(2, "Bob", &collection)))
            .unwrap();
        let (_guest_outbox, mut guest_inbox) = plug(&guest);
        let present = |pseudo: &str| ServerMessage::Opponent {
            connected: true,
            pseudo: pseudo.to_string(),
        };
        assert_eq!(received(&mut guest_inbox), [present("Ana")]);
        assert_eq!(received(&mut host_inbox), [present("Bob")]);

        lock(&host.table).disconnect(host.player, &host_outbox);
        let away = ServerMessage::Opponent {
            connected: false,
            pseudo: "Ana".to_string(),
        };
        assert_eq!(received(&mut guest_inbox), std::slice::from_ref(&away));
        // Qui s'assoit face à un humain absent l'apprend aussi.
        let (_again, mut again_inbox) = plug(&guest);
        assert_eq!(received(&mut again_inbox), [away]);
    }

    #[test]
    fn a_new_connection_tells_the_old_one_that_the_seat_is_taken() {
        let lobby = Lobby::default();
        let seating = lobby.create(Opponent::Random, None, None).unwrap();
        let (_old, mut old_inbox) = plug(&seating);
        let _new = plug(&seating);
        assert!(matches!(
            received(&mut old_inbox).as_slice(),
            [ServerMessage::Error { kind, .. }] if kind == "seat_taken"
        ));
    }

    #[test]
    fn an_account_sits_in_one_unfinished_game_at_a_time() {
        let lobby = lobby_seating_the_creator_first();
        let collection = starter();
        let ana = entrant(1, "Ana", &collection);
        let first = lobby.create(Opponent::Human, None, Some(ana)).unwrap();
        assert!(matches!(
            lobby.create(Opponent::Random, None, Some(ana)),
            Err(ServerError::AlreadySeated)
        ));
        assert!(matches!(
            lobby.join(&first.code, Some(ana)),
            Err(ServerError::AlreadySeated)
        ));
        let other = lobby.create(Opponent::Human, None, None).unwrap();
        assert!(matches!(
            lobby.join(&other.code, Some(ana)),
            Err(ServerError::AlreadySeated)
        ));

        // Sa partie finie, il peut en commencer une autre.
        lock(&first.table).room.resign(Player::Two).unwrap();
        assert!(lobby.create(Opponent::Random, None, Some(ana)).is_ok());
    }

    /// Rassoit ce compte avec un client neuf ; rend sa place, et ce que le client reçoit.
    fn resume(lobby: &Lobby, account: i64) -> Option<(SharedTable, String, Player)> {
        let (outbox, _inbox) = mpsc::channel(16);
        lobby.resume(account, &outbox, |_| ServerMessage::Left)
    }

    #[test]
    fn an_account_is_seated_again_in_its_room() {
        let lobby = lobby_seating_the_creator_first();
        let collection = starter();
        let (outbox, mut inbox) = mpsc::channel(16);
        let greeting = |resumed| ServerMessage::Opponent {
            connected: resumed,
            pseudo: "compte".to_string(),
        };
        assert!(
            lobby.resume(1, &outbox, greeting).is_none(),
            "assis nulle part"
        );
        assert!(inbox.try_recv().is_err(), "rien d'envoyé");
        let seating = lobby
            .create(Opponent::Random, None, Some(entrant(1, "Ana", &collection)))
            .unwrap();
        let (table, code, player) = lobby.resume(1, &outbox, greeting).unwrap();
        assert_eq!(
            (code.as_str(), player),
            (seating.code.as_str(), seating.player)
        );
        assert!(Arc::ptr_eq(&table, &seating.table));
        assert_eq!(inbox.try_recv(), Ok(greeting(true)), "le compte d'abord");
        assert_eq!(
            inbox.try_recv(),
            Ok(ServerMessage::Seated {
                code: seating.code.clone(),
                player: seating.player,
                key: seating.key.clone(),
                launched: true,
            })
        );
        assert!(matches!(inbox.try_recv(), Ok(ServerMessage::State { .. })));
        assert_eq!(lock(&table).check_connected(player, &outbox), Ok(()));
    }

    #[test]
    fn leaving_a_launched_game_forfeits_it() {
        let Launched {
            lobby,
            host,
            mut guest_line,
            host_line,
            ..
        } = launched_friends_room();
        let departure = lobby.leave(Some(1), &host.code, host.player, &host_line.0);
        assert_eq!(departure, Some(Departure::Kept));
        let ended = std::iter::from_fn(|| guest_line.1.try_recv().ok()).any(|message| {
            matches!(message, ServerMessage::State { view, .. } if view.decision.is_none())
        });
        assert!(ended, "l'invité voit la fin");
        let finished = lock(&host.table).take_finished().unwrap().0;
        assert_eq!(finished.end.reason, fight_of_pions::EndReason::Forfeit);
        assert_eq!(finished.end.winner(), Some(Player::Two));
        assert!(resume(&lobby, 1).is_none(), "elle n'est plus sa salle");
        assert!(resume(&lobby, 2).is_some(), "l'invité y est encore");
    }

    #[test]
    fn a_guest_who_leaves_before_the_launch_frees_the_seat() {
        let lobby = lobby_seating_the_creator_first();
        let collection = starter();
        let host = lobby
            .create(Opponent::Human, None, Some(entrant(1, "Ana", &collection)))
            .unwrap();
        let (_host_outbox, mut host_inbox) = plug(&host);
        let guest = lobby
            .join(&host.code, Some(entrant(2, "Bob", &collection)))
            .unwrap();
        let (guest_outbox, _guest_inbox) = plug(&guest);
        received(&mut host_inbox);

        let departure = lobby.leave(Some(2), &guest.code, guest.player, &guest_outbox);
        assert_eq!(departure, Some(Departure::Kept));
        assert!(matches!(
            received(&mut host_inbox).as_slice(),
            [ServerMessage::Opponent {
                connected: false,
                ..
            }]
        ));
        assert!(resume(&lobby, 2).is_none());
        let next = lobby
            .join(&host.code, Some(entrant(3, "Cléo", &collection)))
            .unwrap();
        assert_eq!(next.player, guest.player, "la place est libre");
    }

    #[test]
    fn a_host_who_leaves_before_the_launch_closes_the_room() {
        let lobby = lobby_seating_the_creator_first();
        let collection = starter();
        let host = lobby
            .create(Opponent::Human, None, Some(entrant(1, "Ana", &collection)))
            .unwrap();
        let (host_outbox, _host_inbox) = plug(&host);
        let guest = lobby
            .join(&host.code, Some(entrant(2, "Bob", &collection)))
            .unwrap();
        let (_guest_outbox, mut guest_inbox) = plug(&guest);
        received(&mut guest_inbox);

        let departure = lobby.leave(Some(1), &host.code, host.player, &host_outbox);
        assert_eq!(departure, Some(Departure::Closed));
        assert!(matches!(
            received(&mut guest_inbox).as_slice(),
            [ServerMessage::Error { kind, .. }] if kind == "room_closed"
        ));
        assert!(matches!(
            lobby.join(&host.code, None),
            Err(ServerError::UnknownRoom(_))
        ));
        assert!(resume(&lobby, 2).is_none(), "l'invité n'y est plus assis");
    }

    #[test]
    fn a_replaced_connection_leaves_nothing() {
        let Launched {
            lobby,
            host,
            host_line,
            ..
        } = launched_friends_room();
        let _new = plug(&host);
        assert_eq!(
            lobby.leave(Some(1), &host.code, host.player, &host_line.0),
            None
        );
        assert!(!lock(&host.table).room.is_over());
        assert!(resume(&lobby, 1).is_some(), "le compte reste assis");
    }

    #[test]
    fn a_player_away_too_long_loses_by_forfeit() {
        let Launched {
            lobby,
            host,
            guest,
            guest_line,
            ..
        } = launched_friends_room();
        lock(&guest.table).disconnect(guest.player, &guest_line.0);
        let away_since = lock(&host.table).away_since[guest.player.index()].unwrap();

        assert!(lobby.sweep(away_since + ABSENCE_LIMIT / 2).is_empty());
        let ended = lobby.sweep(away_since + ABSENCE_LIMIT);
        assert_eq!(ended.len(), 1);
        let finished = lock(&host.table).take_finished().unwrap().0;
        assert_eq!(finished.end.reason, fight_of_pions::EndReason::Forfeit);
        assert_eq!(finished.end.winner(), Some(host.player));
        assert!(resume(&lobby, 1).is_some(), "Ana, branchée, voit la fin");
    }

    #[test]
    fn a_room_waiting_for_its_launch_is_not_forfeited() {
        let lobby = lobby_seating_the_creator_first();
        let host = lobby.create(Opponent::Human, None, None).unwrap();
        let (outbox, _inbox) = plug(&host);
        lock(&host.table).disconnect(host.player, &outbox);
        assert!(lobby.sweep(Instant::now() + ABSENCE_LIMIT).is_empty());
        assert!(!lock(&host.table).room.is_over());
    }

    #[test]
    fn a_room_against_a_bot_is_launched_at_once() {
        let seating = Lobby::default()
            .create(Opponent::Random, None, None)
            .unwrap();
        assert!(seating.launched);
        assert!(lock(&seating.table).launched);
    }

    #[test]
    fn an_abandoned_room_is_closed_after_the_idle_limit() {
        let lobby = Lobby::default();
        let collection = starter();
        let seating = lobby
            .create(Opponent::Human, None, Some(entrant(1, "Ana", &collection)))
            .unwrap();
        let (outbox, _inbox) = plug(&seating);
        let later = Instant::now() + ROOM_IDLE_LIMIT;
        lobby.sweep(later);
        assert_eq!(lock(&lobby.hall).tables.len(), 1, "un joueur est connecté");

        lock(&seating.table).disconnect(seating.player, &outbox);
        lobby.sweep(Instant::now());
        assert_eq!(lock(&lobby.hall).tables.len(), 1, "on peut encore revenir");
        lobby.sweep(later + ROOM_IDLE_LIMIT);
        assert!(lock(&lobby.hall).tables.is_empty());
        assert!(resume(&lobby, 1).is_none(), "sa place est oubliée");
    }

    #[test]
    fn a_finished_game_is_closed_as_soon_as_nobody_is_connected() {
        let room = Room::won_by_the_first_player([Seat::Free, Seat::Free]);
        let mut table = Table::new(room, "essai", Player::One, true);
        assert!(table.room.is_over());
        assert!(table.is_abandoned(Instant::now()));
        let (outbox, _inbox) = mpsc::channel(4);
        table.connect(Player::One, outbox);
        assert!(!table.is_abandoned(Instant::now()));
    }

    #[test]
    fn the_server_opens_at_most_its_room_limit() {
        let lobby = Lobby::default();
        let mut lines = Vec::new();
        for _ in 0..MAX_ROOMS {
            let seating = lobby.create(Opponent::Human, None, None).unwrap();
            lines.push(plug(&seating));
        }
        assert!(matches!(
            lobby.create(Opponent::Human, None, None),
            Err(ServerError::ServerFull)
        ));
    }

    #[test]
    fn stopped_bots_warn_the_players_and_can_start_again() {
        let lobby = lobby_seating_the_creator_first();
        let seating = lobby.create(Opponent::Random, None, None).unwrap();
        let (outbox, mut inbox) = mpsc::channel(4);
        let mut table = lock(&seating.table);
        table.connect(seating.player, outbox);
        let _view = inbox.try_recv();
        // Le draft commence par un choix de J1, le joueur : ce n'est pas au bot.
        assert!(table.start_bots().is_none());
        table.bots_running = true;
        table.stop_bots(&ServerError::BotFailed("essai".to_string()));
        assert!(!table.bots_running);
        assert!(matches!(
            inbox.try_recv(),
            Ok(ServerMessage::Error { kind, .. }) if kind == "bot_failed"
        ));
    }

    #[test]
    fn an_old_connection_does_not_unplug_the_new_one() {
        let lobby = Lobby::default();
        let seating = lobby.create(Opponent::Human, None, None).unwrap();
        let (old, _old_inbox) = mpsc::channel(4);
        let (new, mut new_inbox) = mpsc::channel(4);
        let mut table = lock(&seating.table);
        table.connect(Player::One, old.clone());
        table.connect(Player::One, new.clone());
        assert!(new_inbox.try_recv().is_ok(), "la vue actuelle");
        assert_eq!(
            table.check_connected(Player::One, &old),
            Err(ServerError::SeatTaken)
        );
        assert_eq!(table.check_connected(Player::One, &new), Ok(()));
        table.disconnect(Player::One, &old);
        assert!(table.outboxes[0].is_some());
        table.disconnect(Player::One, &new);
        assert!(table.outboxes[0].is_none());
    }
}
