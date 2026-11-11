//! Les tâches de fond : faire jouer les bots, enregistrer les parties finies, balayer les
//! salles. Les enregistrements sont suivis : à l'arrêt, le serveur les attend.

use crate::ServerError;
use crate::ServerMessage;
use crate::backup;
use crate::database::Database;
use crate::limits::{
    BACKUP_HOUR_UTC, BACKUPS_KEPT, RECORD_ATTEMPTS, RECORD_RETRY_DELAY, SWEEP_INTERVAL, bot_slots,
};
use crate::lobby::{Lobby, SharedTable, lock};
use crate::room::FinishedGame;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

/// Ce que partagent les tâches de fond : la base, les cœurs réservés aux bots, et le suivi
/// des tâches à attendre à l'arrêt.
#[derive(Clone)]
pub struct Background {
    database: Database,
    bot_slots: Arc<Semaphore>,
    tasks: TaskTracker,
}

impl Background {
    pub fn new(database: Database) -> Background {
        Background {
            database,
            bot_slots: Arc::new(Semaphore::new(bot_slots())),
            tasks: TaskTracker::new(),
        }
    }

    /// La base des comptes.
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Le suivi des tâches à attendre à l'arrêt : les enregistrements, et les connexions.
    pub fn tasks(&self) -> &TaskTracker {
        &self.tasks
    }

    /// Après un changement de cette salle, même un coup refusé : une partie qui vient de
    /// finir est enregistrée, et un bot qui attend est relancé.
    pub fn after_change(&self, table: &SharedTable) {
        self.record_if_finished(table);
        self.wake_bots(table);
    }

    /// Fait jouer les bots de cette salle tant que c'est à eux de décider, dans une tâche à
    /// part : la connexion continue de lire ses messages. Chaque recherche tourne sur une
    /// copie de la partie, hors du verrou de la salle, et attend un des cœurs réservés aux
    /// bots ; la salle n'est verrouillée que pour jouer le coup trouvé et l'envoyer à tous.
    fn wake_bots(&self, table: &SharedTable) {
        let Some(mut search) = lock(table).start_bots() else {
            return;
        };
        let table = Arc::clone(table);
        let background = self.clone();
        tokio::spawn(async move {
            loop {
                let Ok(slot) = Arc::clone(&background.bot_slots).acquire_owned().await else {
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
                background.record_if_finished(&table);
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

    /// Si la partie de cette salle vient de finir, l'enregistre dans une tâche suivie (la
    /// salle reste libre), puis annonce sa carte au gagnant récompensé. Une partie n'est
    /// rendue qu'une fois ([`crate::room::Room::take_finished`]).
    pub fn record_if_finished(&self, table: &SharedTable) {
        let Some((game, variant)) = lock(table).take_finished() else {
            return;
        };
        let table = Arc::clone(table);
        let database = self.database.clone();
        self.tasks
            .spawn(async move { record(&table, &database, &game, &variant).await });
    }

    /// Balaie les salles à intervalles réguliers, jusqu'à l'arrêt : les joueurs absents trop
    /// longtemps perdent par forfait (la partie est enregistrée), les salles abandonnées
    /// ferment.
    pub fn sweep_regularly(&self, lobby: Arc<Lobby>, shutdown: CancellationToken) {
        let background = self.clone();
        tokio::spawn(async move {
            let start = tokio::time::Instant::now() + SWEEP_INTERVAL;
            let mut ticks = tokio::time::interval_at(start, SWEEP_INTERVAL);
            loop {
                tokio::select! {
                    () = shutdown.cancelled() => return,
                    _ = ticks.tick() => {}
                }
                for table in lobby.sweep(Instant::now()) {
                    background.record_if_finished(&table);
                }
            }
        });
    }

    /// Chaque nuit à [`BACKUP_HOUR_UTC`], une sauvegarde de la base dans ce dossier (les
    /// [`BACKUPS_KEPT`] dernières y restent), jusqu'à l'arrêt. Une sauvegarde ratée (dossier
    /// absent, plein, interdit) est écrite dans la sortie du serveur, et la suivante tentée
    /// la nuit d'après.
    pub fn back_up_nightly(&self, folder: PathBuf, shutdown: CancellationToken) {
        let database = self.database.clone();
        self.tasks.spawn(async move {
            loop {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                let wait = backup::delay_until_hour(now, BACKUP_HOUR_UTC);
                tokio::select! {
                    () = shutdown.cancelled() => return,
                    () = tokio::time::sleep(wait) => {}
                }
                match backup::back_up(&database, &folder, BACKUPS_KEPT).await {
                    Ok(file) => println!("Sauvegarde : {}", file.display()),
                    Err(_) => eprintln!("sauvegarde ratée dans {}", folder.display()),
                }
            }
        });
    }

    /// À l'arrêt : attend les tâches suivies, au plus ce temps.
    pub async fn finish(&self, grace: Duration) {
        self.tasks.close();
        if tokio::time::timeout(grace, self.tasks.wait())
            .await
            .is_err()
        {
            eprintln!(
                "arrêt : {} tâche(s) pas finie(s) à temps, abandonnée(s)",
                self.tasks.len()
            );
        }
    }
}

/// Enregistre la partie, en réessayant si la base refuse (occupée un instant) ; annonce sa
/// carte au gagnant récompensé. Un essai raté ne laisse rien en base : il est annulé en
/// entier.
async fn record(table: &SharedTable, database: &Database, game: &FinishedGame, variant: &str) {
    for attempt in 1..=RECORD_ATTEMPTS {
        match database.record_game(game, variant).await {
            Ok(card) => {
                if let (Some(card), Some((winner, _))) = (card, game.rewarded()) {
                    lock(table).send_to(winner, ServerMessage::Reward { card });
                }
                return;
            }
            Err(error) => {
                eprintln!(
                    "partie non enregistrée (essai {attempt} sur {RECORD_ATTEMPTS}) : {error}"
                );
                if attempt < RECORD_ATTEMPTS {
                    tokio::time::sleep(RECORD_RETRY_DELAY).await;
                }
            }
        }
    }
}
