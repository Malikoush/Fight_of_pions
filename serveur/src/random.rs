//! Le hasard du serveur : codes de salle, graines, clés de place et jetons de compte.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

/// Un nombre tiré par le système (le hachage aléatoire de la bibliothèque standard, une
/// fonction pseudo-aléatoire à clé secrète tirée au démarrage) : imprévisible pour un code
/// de salle, une graine, une clé de place ou un jeton, sans dépendance.
pub fn random_u64() -> u64 {
    RandomState::new().build_hasher().finish()
}

/// Un secret de 128 bits, en hexadécimal : une clé de place, un jeton de compte.
pub fn random_key() -> String {
    format!("{:016x}{:016x}", random_u64(), random_u64())
}
