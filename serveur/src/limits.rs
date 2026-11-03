//! Les limites du serveur, réunies ici pour les régler sans chercher. Ce sont des réglages
//! d'hébergement, pas des règles du jeu (celles-ci sont dans `GameConfig`).

use std::time::Duration;

/// La taille maximale d'un message du client, en octets. Le plus gros, un aperçu, tient en
/// quelques centaines d'octets.
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;

/// Les messages en attente d'envoi vers un client. Ses propres réponses attendent qu'il y
/// ait de la place ; une vue diffusée à un client trop lent est perdue, ce qui ne le
/// désynchronise pas : chaque vue est complète.
pub const OUTBOX_CAPACITY: usize = 64;

/// Le nombre maximal de salles ouvertes en même temps.
pub const MAX_ROOMS: usize = 1000;

/// Une salle sans humain connecté depuis ce temps est fermée : on ne peut plus y revenir.
/// Une partie finie est fermée dès que plus personne n'y est connecté.
pub const ROOM_IDLE_LIMIT: Duration = Duration::from_secs(30 * 60);

/// Les recherches de bot menées en même temps : chacune occupe un cœur, et il en reste au
/// moins un pour les connexions.
pub fn bot_slots() -> usize {
    std::thread::available_parallelism().map_or(1, |cores| cores.get().saturating_sub(1).max(1))
}
