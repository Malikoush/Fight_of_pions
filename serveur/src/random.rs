//! Le hasard du serveur : codes de salle, graines, clés de place et jetons de compte.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

/// Un nombre tiré par le générateur cryptographique du système (`getrandom`) : imprévisible
/// pour un code de salle, une graine, une clé de place ou un jeton. Si le système ne répond
/// pas (cela n'arrive pas en pratique), le hachage aléatoire de la bibliothèque standard
/// prend le relais.
pub fn random_u64() -> u64 {
    let mut bytes = [0_u8; 8];
    match getrandom::fill(&mut bytes) {
        Ok(()) => u64::from_le_bytes(bytes),
        Err(_) => RandomState::new().build_hasher().finish(),
    }
}

/// Un secret de 128 bits, en hexadécimal : une clé de place, un jeton de compte.
pub fn random_key() -> String {
    format!("{:016x}{:016x}", random_u64(), random_u64())
}
