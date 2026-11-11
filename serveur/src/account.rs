//! Les comptes invités (E4a) : un compte est créé au premier lancement du client, qui en
//! garde le jeton secret ; il revient avec ce jeton ([`crate::ClientMessage::Login`]). Ni
//! e-mail ni mot de passe. La base ne garde que l'empreinte du jeton : qui la lirait ne
//! pourrait pas se connecter.

use crate::ServerError;
use crate::random::{CODE_ALPHABET, random_code, random_key, random_u64};
use sha2::{Digest, Sha256};
use std::ops::RangeInclusive;

/// Les longueurs permises d'un pseudo, en caractères.
const PSEUDO_LENGTH: RangeInclusive<usize> = 3..=16;

/// Les signes permis dans un pseudo, en plus des lettres (accents compris) et des chiffres.
const PSEUDO_SIGNS: &str = " -_.";

/// La longueur d'un code de récupération (80 bits : même si la base fuyait, retrouver un
/// code à partir de son empreinte resterait hors de portée), et ses groupes à l'affichage
/// (`K7QX-M2PA-9ZTR-4HWD`).
const RECOVERY_CODE_LENGTH: usize = 16;
const RECOVERY_GROUP: usize = 4;
const RECOVERY_SEPARATOR: char = '-';

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

/// Un code de récupération neuf, à montrer une seule fois (`K7QX-M2PA-9ZTR-4HWD`) : il permet de
/// retrouver son compte sur un autre appareil. La base n'en garde que l'empreinte
/// ([`recovery_hash`]).
pub fn new_recovery_code() -> String {
    let code: Vec<char> = random_code(RECOVERY_CODE_LENGTH).chars().collect();
    code.chunks(RECOVERY_GROUP)
        .map(|group| group.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(&RECOVERY_SEPARATOR.to_string())
}

/// L'empreinte d'un code de récupération tapé : sans espaces ni tirets, en capitales ; un
/// signe hors de l'alphabet des codes le rend faux (aucune empreinte : `None`).
pub fn recovery_hash(typed: &str) -> Option<String> {
    let code: String = typed
        .chars()
        .filter(|sign| !sign.is_whitespace() && *sign != RECOVERY_SEPARATOR)
        .flat_map(char::to_uppercase)
        .collect();
    let valid = code.chars().count() == RECOVERY_CODE_LENGTH
        && code.bytes().all(|byte| CODE_ALPHABET.contains(&byte));
    valid.then(|| token_hash(&code))
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
    fn a_recovery_code_is_read_whatever_its_dashes_and_case() {
        let code = new_recovery_code();
        assert_eq!(code.len(), 19, "{code}");
        assert_eq!(code.matches('-').count(), 3, "{code}");
        let hash = recovery_hash(&code).unwrap();
        let typed = format!(" {} ", code.replace('-', " ").to_lowercase());
        assert_eq!(recovery_hash(&typed), Some(hash.clone()));
        assert_ne!(recovery_hash(&new_recovery_code()), Some(hash));
        // Trop court, ou un signe ambigu (O, 0) : aucune empreinte.
        assert_eq!(recovery_hash("K7QX-M2PA-9ZTR"), None);
        assert_eq!(recovery_hash("K7QX-M2PA-9ZTR-4HW0"), None);
    }

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
