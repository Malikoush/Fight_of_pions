//! Les limites du serveur, réunies ici pour les régler sans chercher. Ce sont des réglages
//! d'hébergement, pas des règles du jeu (celles-ci sont dans `GameConfig`).

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// La taille maximale d'un message du client, en octets. Le plus gros, un aperçu, tient en
/// quelques centaines d'octets.
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;

/// Les messages en attente d'envoi vers un client. Ses propres réponses attendent qu'il y
/// ait de la place ; une vue diffusée à un client trop lent est perdue, ce qui ne le
/// désynchronise pas : chaque vue est complète.
pub const OUTBOX_CAPACITY: usize = 64;

/// Le nombre maximal de salles ouvertes en même temps.
pub const MAX_ROOMS: usize = 1000;

/// Le nombre maximal de connexions ouvertes en même temps : au-delà, le serveur répond
/// « occupé » (503) sans ouvrir la WebSocket.
pub const MAX_CONNECTIONS: usize = 2000;

/// Une salle sans humain connecté depuis ce temps est fermée : on ne peut plus y revenir.
/// Une partie finie est fermée dès que plus personne n'y est connecté.
pub const ROOM_IDLE_LIMIT: Duration = Duration::from_secs(30 * 60);

/// Un joueur débranché depuis ce temps d'une partie lancée et pas finie la perd par forfait
/// (carnet de règles, « Abandon »).
pub const ABSENCE_LIMIT: Duration = Duration::from_secs(3 * 60);

/// Le temps entre deux balayages des salles : fermer les abandonnées, déclarer forfait les
/// joueurs absents trop longtemps.
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(30);

/// Le temps entre deux pings du serveur à un client : le client y répond tout seul (le
/// navigateur, ou le `WebSocketPeer` de Godot), ce qui prouve que la connexion vit.
pub const PING_INTERVAL: Duration = Duration::from_secs(20);

/// Une connexion dont rien n'est arrivé depuis ce temps (ni message, ni réponse à un ping)
/// est fermée : le client est parti sans le dire (coupure, onglet tué).
pub const IDLE_LIMIT: Duration = Duration::from_secs(60);

/// Une connexion qui ne s'est pas connectée à un compte (`register` ou `login`) dans ce
/// temps est fermée : le client du jeu le fait dès l'ouverture.
pub const LOGIN_LIMIT: Duration = Duration::from_secs(30);

/// Le temps qu'un envoi au client peut attendre : au-delà, le client ne lit plus, et sa
/// connexion est fermée.
pub const SEND_LIMIT: Duration = Duration::from_secs(10);

/// Les trames qu'un client peut envoyer par fenêtre de [`MESSAGE_WINDOW`] (messages,
/// pongs, trames binaires) : bien plus qu'un joueur n'en envoie, même en cliquant vite.
/// Au-delà, sa connexion est fermée.
pub const MAX_MESSAGES_PER_WINDOW: u32 = 200;
/// La fenêtre de [`MAX_MESSAGES_PER_WINDOW`].
pub const MESSAGE_WINDOW: Duration = Duration::from_secs(10);

/// Les comptes que le serveur crée par fenêtre de [`REGISTRATION_WINDOW`], tous clients
/// confondus : de quoi accueillir une salle de classe d'un coup, pas de remplir la base.
pub const MAX_REGISTRATIONS_PER_WINDOW: u32 = 60;
/// La fenêtre de [`MAX_REGISTRATIONS_PER_WINDOW`].
pub const REGISTRATION_WINDOW: Duration = Duration::from_secs(60);

/// Les essais d'enregistrement d'une partie finie, et l'attente entre deux : une base
/// occupée un instant ne fait pas perdre la partie.
pub const RECORD_ATTEMPTS: u32 = 3;
/// L'attente entre deux essais d'enregistrement.
pub const RECORD_RETRY_DELAY: Duration = Duration::from_secs(2);

/// À l'arrêt du serveur, le temps laissé aux enregistrements en cours pour finir.
pub const SHUTDOWN_GRACE: Duration = Duration::from_secs(10);

/// L'heure (UTC) de la sauvegarde de chaque nuit, quand le serveur a un dossier où la
/// ranger (`BACKUP_DIR`) : 2 h, soit 3 ou 4 h à Paris, quand personne ne joue.
pub const BACKUP_HOUR_UTC: u64 = 2;

/// Les sauvegardes gardées dans ce dossier : une semaine de nuits.
pub const BACKUPS_KEPT: usize = 7;

/// Les recherches de bot menées en même temps : chacune occupe un cœur, et il en reste au
/// moins un pour les connexions.
pub fn bot_slots() -> usize {
    std::thread::available_parallelism().map_or(1, |cores| cores.get().saturating_sub(1).max(1))
}

/// Au plus tant d'usages par fenêtre de temps : une fenêtre fixe, qui repart de zéro à son
/// terme. Simple, et assez juste pour freiner un abus.
pub struct RateLimit {
    max: u32,
    window: Duration,
    /// Le début de la fenêtre en cours, et ses usages.
    current: Mutex<(Instant, u32)>,
}

impl RateLimit {
    /// Au plus `max` usages par `window`.
    pub fn new(max: u32, window: Duration) -> RateLimit {
        RateLimit {
            max,
            window,
            current: Mutex::new((Instant::now(), 0)),
        }
    }

    /// Compte un usage à cet instant ; faux s'il dépasse la limite (il n'est pas compté).
    pub fn allow(&self, now: Instant) -> bool {
        let mut current = crate::lobby::lock(&self.current);
        let (start, used) = &mut *current;
        if now.duration_since(*start) >= self.window {
            *start = now;
            *used = 0;
        }
        if *used >= self.max {
            return false;
        }
        *used += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rate_limit_refuses_past_its_maximum_until_the_window_ends() {
        let limit = RateLimit::new(2, Duration::from_secs(10));
        let start = Instant::now();
        assert!(limit.allow(start));
        assert!(limit.allow(start));
        assert!(!limit.allow(start + Duration::from_secs(9)));
        assert!(
            limit.allow(start + Duration::from_secs(10)),
            "fenêtre neuve"
        );
    }
}
