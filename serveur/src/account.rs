//! Les comptes invités (E4a) : un compte est créé au premier lancement du client, qui en
//! garde le jeton secret ; il revient avec ce jeton ([`crate::ClientMessage::Login`]). Ni
//! e-mail ni mot de passe. La base ne garde que l'empreinte du jeton : qui la lirait ne
//! pourrait pas se connecter.

use crate::ServerError;
use crate::random::{random_key, random_u64};
use sha2::{Digest, Sha256};
use std::ops::RangeInclusive;

/// Les longueurs permises d'un pseudo, en caractères.
const PSEUDO_LENGTH: RangeInclusive<usize> = 3..=16;

/// Les signes permis dans un pseudo, en plus des lettres (accents compris) et des chiffres.
const PSEUDO_SIGNS: &str = " -_.";

/// Un compte connecté.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// Son numéro, stable.
    pub id: i64,
    /// Son pseudo, affiché aux autres joueurs.
    pub pseudo: String,
}

/// Un jeton de compte neuf : secret, à donner une seule fois au client.
pub fn new_token() -> String {
    random_key()
}

/// L'empreinte d'un jeton, telle que la base la garde (SHA-256, en hexadécimal). Un jeton
/// est long et tiré au hasard : une empreinte simple suffit, sans sel ni lenteur voulue.
pub fn token_hash(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Le pseudo donné à un compte créé sans pseudo : « Pion » et quatre chiffres.
pub fn default_pseudo() -> String {
    format!("Pion{:04}", random_u64() % 10_000)
}

/// Le pseudo tel qu'il sera gardé (sans espace autour), s'il est permis : de 3 à 16
/// caractères, des lettres, des chiffres, des espaces, `-`, `_` ou `.`. Deux comptes peuvent
/// avoir le même pseudo : c'est leur numéro qui les distingue.
pub fn checked_pseudo(pseudo: &str) -> Result<String, ServerError> {
    let pseudo = pseudo.trim();
    let length = pseudo.chars().count();
    if !PSEUDO_LENGTH.contains(&length) {
        return Err(ServerError::BadPseudo(format!(
            "il doit faire de {} à {} caractères",
            PSEUDO_LENGTH.start(),
            PSEUDO_LENGTH.end()
        )));
    }
    if let Some(sign) = pseudo
        .chars()
        .find(|&sign| !sign.is_alphanumeric() && !PSEUDO_SIGNS.contains(sign))
    {
        return Err(ServerError::BadPseudo(format!(
            "le signe « {} » n'est pas permis",
            sign.escape_debug()
        )));
    }
    Ok(pseudo.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pseudo_is_trimmed_and_checked() {
        assert_eq!(checked_pseudo("  Élodie_3 ").unwrap(), "Élodie_3");
        assert_eq!(checked_pseudo("Le Pion.fou-2").unwrap(), "Le Pion.fou-2");
        assert!(checked_pseudo("ab").is_err());
        assert!(checked_pseudo("   ab   ").is_err());
        assert!(checked_pseudo("un pseudo bien trop long").is_err());
        assert!(checked_pseudo("<script>").is_err());
        assert!(checked_pseudo("pion\nfou").is_err());
    }

    #[test]
    fn the_default_pseudo_is_allowed() {
        let pseudo = default_pseudo();
        assert_eq!(checked_pseudo(&pseudo).unwrap(), pseudo);
    }

    #[test]
    fn a_token_hash_is_stable_and_hides_the_token() {
        let token = new_token();
        assert_eq!(token.len(), 32);
        assert_ne!(token, new_token());
        assert_eq!(token_hash(&token), token_hash(&token));
        assert!(!token_hash(&token).contains(&token));
        assert_eq!(
            token_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
