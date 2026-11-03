//! Les salles ouvertes, et la diffusion : après chaque changement d'une partie, chaque
//! humain connecté reçoit sa vue et les événements qu'il peut voir.

use crate::room::{Room, Seat};
use crate::{Opponent, ServerError, ServerMessage};
use bots::{BotKind, variants};
use fight_of_pions::{Event, GameConfig, Player};
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use tokio::sync::mpsc::UnboundedSender;

/// La variante jouée quand le client n'en demande pas.
const DEFAULT_VARIANT: &str = "règles actuelles avec draft";
/// Les caractères d'un code de salle : sans 0, O, 1 ni I, faciles à confondre.
const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LENGTH: usize = 5;

/// Où envoyer les messages d'un client connecté.
pub type Outbox = UnboundedSender<ServerMessage>;

/// Une salle, et les clients connectés à chacune de ses places.
pub struct Table {
    room: Room,
    outboxes: [Option<Outbox>; 2],
}

/// Une salle partagée entre les connexions de ses joueurs.
pub type SharedTable = Arc<Mutex<Table>>;

/// Verrouille une table. Une connexion qui a paniqué en la tenant ne la rend pas
/// inutilisable : la partie elle-même reste cohérente (le moteur ne modifie rien sur un
/// coup refusé).
pub fn lock(table: &SharedTable) -> MutexGuard<'_, Table> {
    table.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Table {
    /// Branche ce client sur cette place, et lui envoie où il en est.
    pub fn connect(&mut self, player: Player, outbox: Outbox) {
        self.outboxes[player.index()] = Some(outbox);
        self.send_state(player, &[]);
    }

    /// Débranche le client de cette place ; sa place lui reste, avec son jeton.
    pub fn disconnect(&mut self, player: Player) {
        self.outboxes[player.index()] = None;
    }

    /// Joue le coup de cet humain, puis l'envoie à tous.
    pub fn play(
        &mut self,
        player: Player,
        action: fight_of_pions::Action,
    ) -> Result<(), ServerError> {
        let events = self.room.play(player, action)?;
        self.broadcast(&events);
        Ok(())
    }

    /// Ce que verrait ce joueur après ces coups, sans rien jouer.
    pub fn preview(
        &self,
        player: Player,
        actions: &[fight_of_pions::Action],
    ) -> Result<ServerMessage, ServerError> {
        let view = self.room.preview(player, actions)?;
        Ok(ServerMessage::Preview {
            view: Box::new(view),
        })
    }

    /// Fait jouer une action au bot qui doit décider, et l'envoie à tous. Faux si aucun
    /// bot n'a joué.
    pub fn play_bot(&mut self) -> Result<bool, ServerError> {
        let Some(events) = self.room.play_bot()? else {
            return Ok(false);
        };
        self.broadcast(&events);
        Ok(true)
    }

    /// Envoie à chaque humain connecté sa vue et les événements qu'il peut voir.
    fn broadcast(&self, events: &[Event]) {
        for player in [Player::One, Player::Two] {
            self.send_state(player, events);
        }
    }

    fn send_state(&self, player: Player, events: &[Event]) {
        let Some(outbox) = &self.outboxes[player.index()] else {
            return;
        };
        let events = events.iter().map(|event| event.seen_by(player)).collect();
        // Un client parti entre-temps : rien à faire, sa connexion se ferme.
        let _ = outbox.send(ServerMessage::State {
            view: Box::new(self.room.view(player)),
            events,
        });
    }
}

/// Les salles ouvertes, par code.
#[derive(Default)]
pub struct Lobby {
    tables: Mutex<HashMap<String, SharedTable>>,
}

/// Une place prise : la salle, son code, la place et son jeton.
pub struct Seating {
    pub table: SharedTable,
    pub code: String,
    pub player: Player,
    pub token: String,
}

impl Lobby {
    /// Ouvre une salle contre cet adversaire, et y assied le demandeur comme joueur 1.
    pub fn create(
        &self,
        opponent: Opponent,
        variant: Option<&str>,
    ) -> Result<Seating, ServerError> {
        let config = config_of(variant.unwrap_or(DEFAULT_VARIANT))?;
        let second = match opponent {
            Opponent::Human => Seat::Free,
            Opponent::Mcts => Seat::Bot(BotKind::Mcts),
            Opponent::Random => Seat::Bot(BotKind::Random),
        };
        let token = random_token();
        let seats = [
            Seat::Human {
                token: token.clone(),
            },
            second,
        ];
        let room = Room::new(config, random_u64(), seats)?;
        let table = Arc::new(Mutex::new(Table {
            room,
            outboxes: [None, None],
        }));
        let mut tables = self.tables.lock().unwrap_or_else(PoisonError::into_inner);
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
            player: Player::One,
            token,
        })
    }

    /// Assied le demandeur à la place libre de la salle de ce code.
    pub fn join(&self, code: &str) -> Result<Seating, ServerError> {
        let table = self.table(code)?;
        let token = random_token();
        let player = {
            let mut guard = lock(&table);
            let player = guard.room.free_seat().ok_or(ServerError::RoomFull)?;
            guard.room.sit(player, token.clone())?;
            player
        };
        Ok(Seating {
            table,
            code: code.to_string(),
            player,
            token,
        })
    }

    /// Rend sa place à un joueur qui revient avec son jeton.
    pub fn rejoin(&self, code: &str, player: Player, token: &str) -> Result<Seating, ServerError> {
        let table = self.table(code)?;
        lock(&table).room.check_token(player, token)?;
        Ok(Seating {
            table,
            code: code.to_string(),
            player,
            token: token.to_string(),
        })
    }

    fn table(&self, code: &str) -> Result<SharedTable, ServerError> {
        let tables = self.tables.lock().unwrap_or_else(PoisonError::into_inner);
        tables
            .get(&code.to_uppercase())
            .cloned()
            .ok_or_else(|| ServerError::UnknownRoom(code.to_string()))
    }
}

/// La configuration de la variante de ce nom.
fn config_of(name: &str) -> Result<GameConfig, ServerError> {
    variants()
        .into_iter()
        .find(|variant| variant.name == name)
        .map(|variant| variant.config)
        .ok_or_else(|| ServerError::UnknownVariant(name.to_string()))
}

/// Un nombre tiré par le système (le hachage aléatoire de la bibliothèque standard) :
/// assez imprévisible pour un code de salle, une graine ou un jeton, sans dépendance.
/// Pas fait pour la cryptographie.
fn random_u64() -> u64 {
    RandomState::new().build_hasher().finish()
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

fn random_token() -> String {
    format!("{:016x}{:016x}", random_u64(), random_u64())
}
