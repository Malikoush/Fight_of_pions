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

/// Les caractères d'un code tapé par un joueur (code de salle, code de récupération) : sans
/// 0, O, 1 ni I, faciles à confondre. 32 caractères : 5 bits chacun.
pub const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Un code de cette longueur, tiré dans [`CODE_ALPHABET`] ; au-delà de 12 caractères (60
/// bits d'un tirage), un tirage neuf prend le relais (un code de récupération en a 16).
pub fn random_code(length: usize) -> String {
    let len = CODE_ALPHABET.len() as u64;
    let mut value = random_u64();
    let mut left = u64::MAX;
    (0..length)
        .map(|_| {
            if left < len {
                value = random_u64();
                left = u64::MAX;
            }
            let letter = CODE_ALPHABET[(value % len) as usize];
            value /= len;
            left /= len;
            char::from(letter)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_code_keeps_drawing_past_one_draw() {
        // Au-delà de 12 caractères, un tirage épuisé ne donnerait plus que des « A ».
        let codes: Vec<String> = (0..20).map(|_| random_code(16)).collect();
        assert!(
            codes
                .iter()
                .all(|code| code.len() == 16
                    && code.bytes().all(|byte| CODE_ALPHABET.contains(&byte)))
        );
        assert!(codes.iter().any(|code| &code[12..] != "AAAA"));
    }
}
