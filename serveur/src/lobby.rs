//! Les salles ouvertes, et la diffusion : après chaque changement d'une partie, chaque
//! humain connecté reçoit sa vue et les événements qu'il peut voir.

use crate::collection::Collection;
use crate::limits::{MAX_ROOMS, ROOM_IDLE_LIMIT, bot_slots};
use crate::random::{random_key, random_u64};
use crate::room::{BotMove, BotSearch, FinishedGame, Room, Seat};
use crate::{Opponent, ServerError, ServerMessage};
use bots::{BotKind, ONLINE_VARIANT, variants};
use fight_of_pions::{Action, Event, GameConfig, Player};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::Sender;

/// Les caractères d'un code de salle : sans 0, O, 1 ni I, faciles à confondre.
const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LENGTH: usize = 5;

/// Où envoyer les messages d'un client connecté.
pub type Outbox = Sender<ServerMessage>;

/// Une salle, et les clients connectés à chacune de ses places.
pub struct Table {
    room: Room,
    /// Le nom de la variante jouée, pour l'enregistrement de la partie.
    variant: String,
    outboxes: [Option<Outbox>; 2],
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
            bots_running: false,
            host,
            launched,
            last_seen: Instant::now(),
        }
    }

    /// Branche ce client sur cette place, et lui envoie où il en est. Un client déjà
    /// branché à cette place (une ancienne connexion) ne reçoit plus rien. Les deux
    /// joueurs apprennent qu'ils sont là l'un et l'autre (la salle d'attente du client).
    pub fn connect(&mut self, player: Player, outbox: Outbox) {
        self.outboxes[player.index()] = Some(outbox);
        self.last_seen = Instant::now();
        self.send_state(player, &[]);
        if self.outboxes[player.opponent().index()].is_some() {
            self.send_presence(player, true);
            self.send_presence(player.opponent(), true);
        }
    }

    /// Débranche ce client de cette place, s'il y est encore branché (une reconnexion a
    /// pu le remplacer) ; sa place lui reste, avec sa clé.
    pub fn disconnect(&mut self, player: Player, outbox: &Outbox) {
        let slot = &mut self.outboxes[player.index()];
        if slot.as_ref().is_some_and(|own| own.same_channel(outbox)) {
            *slot = None;
            self.last_seen = Instant::now();
            self.send_presence(player.opponent(), false);
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
    /// adversaire assis. Les deux joueurs l'apprennent ; une partie déjà lancée le reste.
    pub fn launch(&mut self, player: Player) -> Result<(), ServerError> {
        if self.launched {
            return Ok(());
        }
        if player != self.host {
            return Err(ServerError::NotHost);
        }
        if self.room.free_seat().is_some() {
            return Err(ServerError::OpponentMissing);
        }
        self.launched = true;
        for outbox in self.outboxes.iter().flatten() {
            let _ = outbox.try_send(ServerMessage::Launched);
        }
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
        for outbox in self.outboxes.iter().flatten() {
            let _ = outbox.try_send(ServerMessage::error(error));
        }
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

    /// Dit à ce joueur, s'il est branché, si son adversaire l'est.
    fn send_presence(&self, player: Player, connected: bool) {
        if let Some(outbox) = &self.outboxes[player.index()] {
            let _ = outbox.try_send(ServerMessage::Opponent { connected });
        }
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

/// Les salles ouvertes, par code, et les cœurs réservés aux bots.
pub struct Lobby {
    tables: Mutex<BTreeMap<String, SharedTable>>,
    bot_slots: Arc<Semaphore>,
    /// La place donnée au créateur d'une salle ; `None` : tirée au sort (le jeu).
    creator_seat: Option<Player>,
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            tables: Mutex::default(),
            bot_slots: Arc::new(Semaphore::new(bot_slots())),
            creator_seat: None,
        }
    }
}

/// Celui qui s'assoit : son compte, et sa collection, qui limite sa cache.
#[derive(Clone, Copy)]
pub struct Entrant<'a> {
    /// Son compte.
    pub account: i64,
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
    /// suite. Ferme d'abord les salles abandonnées.
    pub fn create(
        &self,
        opponent: Opponent,
        variant: Option<&str>,
        entrant: Option<Entrant>,
    ) -> Result<Seating, ServerError> {
        let variant = variant.unwrap_or(ONLINE_VARIANT);
        let config = config_of(variant)?;
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
        let mut room = Room::new(config, random_u64(), seats)?;
        if let Some(entrant) = entrant {
            let choices = entrant.collection.cache_choices(&room.catalog());
            room.limit_cache(player, &choices)?;
        }
        let table = Arc::new(Mutex::new(Table::new(room, variant, player, launched)));
        let mut tables = lock(&self.tables);
        close_abandoned(&mut tables, Instant::now());
        if tables.len() >= MAX_ROOMS {
            return Err(ServerError::ServerFull);
        }
        let code = loop {
            let code = random_code();
            if !tables.contains_key(&code) {
                break code;
            }
        };
        tables.insert(code.clone(), Arc::clone(&table));
        Ok(Seating {
            table,
            code,
            player,
            key,
            launched,
        })
    }

    /// Assied le demandeur à la place libre de la salle de ce code, sa cache limitée par sa
    /// collection (`None`, pour les tests : tout le catalogue).
    pub fn join(&self, code: &str, entrant: Option<Entrant>) -> Result<Seating, ServerError> {
        let table = self.table(code)?;
        let key = random_key();
        let (player, launched) = {
            let mut guard = lock(&table);
            let player = guard.room.free_seat().ok_or(ServerError::RoomFull)?;
            if let Some(entrant) = entrant {
                let choices = entrant.collection.cache_choices(&guard.room.catalog());
                guard.room.limit_cache(player, &choices)?;
            }
            let account = entrant.map(|entrant| entrant.account);
            guard.room.sit(player, key.clone(), account)?;
            (player, guard.launched)
        };
        Ok(Seating {
            table,
            code: code.to_uppercase(),
            player,
            key,
            launched,
        })
    }

    /// Rend sa place à un joueur qui revient avec sa clé.
    pub fn rejoin(&self, code: &str, player: Player, key: &str) -> Result<Seating, ServerError> {
        let table = self.table(code)?;
        let launched = {
            let guard = lock(&table);
            guard.room.check_key(player, key)?;
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

    /// Les cœurs réservés aux recherches des bots.
    pub fn bot_slots(&self) -> &Arc<Semaphore> {
        &self.bot_slots
    }

    fn table(&self, code: &str) -> Result<SharedTable, ServerError> {
        lock(&self.tables)
            .get(&code.to_uppercase())
            .cloned()
            .ok_or_else(|| ServerError::UnknownRoom(code.to_string()))
    }
}

/// Ferme les salles abandonnées à cet instant. Le hall est verrouillé avant chaque table,
/// jamais l'inverse : pas d'interblocage.
fn close_abandoned(tables: &mut BTreeMap<String, SharedTable>, now: Instant) {
    tables.retain(|_, table| !lock(table).is_abandoned(now));
}

/// La configuration de la variante de ce nom.
fn config_of(name: &str) -> Result<GameConfig, ServerError> {
    variants()
        .into_iter()
        .find(|variant| variant.name == name)
        .map(|variant| variant.config)
        .ok_or_else(|| ServerError::UnknownVariant(name.to_string()))
}

fn random_code() -> String {
    let mut value = random_u64();
    (0..CODE_LENGTH)
        .map(|_| {
            let len = CODE_ALPHABET.len() as u64;
            let letter = CODE_ALPHABET[(value % len) as usize];
            value /= len;
            char::from(letter)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    /// Un hall où le créateur d'une salle est toujours J1.
    fn lobby_seating_the_creator_first() -> Lobby {
        Lobby {
            creator_seat: Some(Player::One),
            ..Lobby::default()
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
    fn a_friends_room_waits_for_its_creator_to_launch_it() {
        let lobby = lobby_seating_the_creator_first();
        let host = lobby.create(Opponent::Human, None, None).unwrap();
        assert!(!host.launched, "entre amis, la partie attend");
        let (host_outbox, mut host_inbox) = mpsc::channel(8);
        lock(&host.table).connect(host.player, host_outbox);
        assert_eq!(
            lock(&host.table).launch(host.player),
            Err(ServerError::OpponentMissing)
        );
        let guest = lobby.join(&host.code, None).unwrap();
        assert_eq!(guest.player, Player::Two, "la place laissée libre");
        assert!(!guest.launched);
        let (guest_outbox, mut guest_inbox) = mpsc::channel(8);
        let mut table = lock(&host.table);
        table.connect(guest.player, guest_outbox);
        assert_eq!(
            table.play(Player::One, Action::EndTurn),
            Err(ServerError::NotLaunched)
        );
        assert_eq!(table.launch(guest.player), Err(ServerError::NotHost));
        while host_inbox.try_recv().is_ok() {}
        while guest_inbox.try_recv().is_ok() {}
        assert_eq!(table.launch(host.player), Ok(()));
        assert!(table.launched);
        assert_eq!(host_inbox.try_recv(), Ok(ServerMessage::Launched));
        assert_eq!(guest_inbox.try_recv(), Ok(ServerMessage::Launched));
        assert_eq!(table.launch(host.player), Ok(()), "déjà lancée");
        drop(table);
        let back = lobby.rejoin(&host.code, guest.player, &guest.key).unwrap();
        assert!(back.launched, "qui revient apprend qu'elle est lancée");
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
        let seating = lobby.create(Opponent::Human, None, None).unwrap();
        let (outbox, _inbox) = mpsc::channel(1);
        lock(&seating.table).connect(seating.player, outbox.clone());
        let later = Instant::now() + ROOM_IDLE_LIMIT;
        let mut tables = lock(&lobby.tables);
        close_abandoned(&mut tables, later);
        assert_eq!(tables.len(), 1, "un joueur est connecté");

        lock(&seating.table).disconnect(seating.player, &outbox);
        close_abandoned(&mut tables, Instant::now());
        assert_eq!(tables.len(), 1, "on peut encore revenir");
        close_abandoned(&mut tables, later + ROOM_IDLE_LIMIT);
        assert!(tables.is_empty());
    }

    #[test]
    fn a_finished_game_is_closed_as_soon_as_nobody_is_connected() {
        // Un plateau de 3 cases sur 3, 2 alignés pour gagner : J1 aligne à son 2e tour, puis
        // chacun finit ses tours jusqu'à la fin de la partie.
        let config = GameConfig {
            board_size: 3,
            align_to_win: 2,
            wall_thickness: 0,
            ..GameConfig::default()
        };
        let room = Room::new(config, 0, [Seat::Free, Seat::Free]).unwrap();
        let mut table = Table::new(room, "essai", Player::One, true);
        let at = |row, col| fight_of_pions::Position { row, col };
        for (player, action) in [
            (Player::One, Action::Place(at(0, 0))),
            (Player::One, Action::EndTurn),
            (Player::Two, Action::Place(at(2, 2))),
            (Player::Two, Action::EndTurn),
            (Player::One, Action::Place(at(0, 1))),
        ] {
            table.play(player, action).unwrap();
        }
        for _ in 0..4 {
            for player in [Player::One, Player::Two] {
                let _ = table.play(player, Action::EndTurn);
            }
        }
        assert!(table.room.is_over());
        assert!(table.is_abandoned(Instant::now()));
        let (outbox, _inbox) = mpsc::channel(4);
        table.connect(Player::One, outbox);
        assert!(!table.is_abandoned(Instant::now()));
    }

    #[test]
    fn the_server_opens_at_most_its_room_limit() {
        let lobby = Lobby::default();
        let mut outboxes = Vec::new();
        for _ in 0..MAX_ROOMS {
            let seating = lobby.create(Opponent::Human, None, None).unwrap();
            let (outbox, inbox) = mpsc::channel(4);
            lock(&seating.table).connect(seating.player, outbox);
            outboxes.push(inbox);
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
