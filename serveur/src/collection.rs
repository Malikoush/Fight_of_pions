//! La collection (E4b) : les cartes qu'un compte possède, et son trio de cache. La
//! collection ne limite que la cache (carnet, 2026-11-05) : au draft, la cache se choisit
//! dans le trio. Sans trio rangé, le serveur en tire un au hasard dans la collection, à
//! chaque partie (Martin, 2026-11-10 : la cache ne se choisit jamais dans toute la
//! collection, sinon elle ne sert à rien). Le tirage des bans et des picks reste commun aux
//! deux joueurs.
//!
//! Le paquet de départ, donné à chaque compte créé, vit dans un fichier de données
//! (`data/depart.json`) : on le change sans toucher au code. Un compte déjà créé garde ce
//! qu'il a reçu.

use crate::ServerError;
use crate::random::random_u64;
use fight_of_pions::Attack;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Le paquet de départ, compris dans le serveur à la compilation.
const STARTER_CARDS: &str = include_str!("../data/depart.json");

/// Le nombre maximal de cartes d'un trio de cache.
pub const TRIO_SIZE: usize = 3;

static STARTER: LazyLock<Result<Vec<Attack>, String>> =
    LazyLock::new(|| serde_json::from_str(STARTER_CARDS).map_err(|error| error.to_string()));

/// Les cartes données à chaque compte créé. Une erreur ne peut venir que d'un fichier mal
/// écrit, ce que les tests empêchent.
pub fn starter_cards() -> Result<&'static [Attack], ServerError> {
    STARTER
        .as_deref()
        .map_err(|why| ServerError::StarterUnavailable(why.clone()))
}

/// Ce qu'un compte possède.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    /// Ses cartes, dans l'ordre des familles puis des raretés.
    pub cards: Vec<Attack>,
    /// Son trio de cache, dans l'ordre où il l'a rangé ; de 0 à 3 cartes (vide : tiré au
    /// hasard à chaque partie).
    pub cache_trio: Vec<Attack>,
}

impl Collection {
    /// Les cartes parmi lesquelles ce compte choisit sa cache, dans une partie de ce
    /// catalogue : son trio, ou, s'il n'en a pas rangé, 3 de ses cartes du catalogue tirées
    /// au hasard (toutes s'il en a moins). Le tirage est refait à chaque partie.
    pub fn cache_choices(&self, catalog: &[Attack]) -> Vec<Attack> {
        if !self.cache_trio.is_empty() {
            return self.cache_trio.clone();
        }
        let mut owned: Vec<Attack> = self
            .cards
            .iter()
            .filter(|card| catalog.contains(card))
            .copied()
            .collect();
        // Les 3 premières cartes d'un mélange (Fisher-Yates partiel).
        let drawn = TRIO_SIZE.min(owned.len());
        for index in 0..drawn {
            let left = (owned.len() - index) as u64;
            let offset = usize::try_from(random_u64() % left).unwrap_or(0);
            owned.swap(index, index + offset);
        }
        owned.truncate(drawn);
        owned
    }

    /// Vérifie qu'un trio est permis : au plus 3 cartes, différentes, toutes possédées.
    pub fn check_trio(&self, trio: &[Attack]) -> Result<(), ServerError> {
        if trio.len() > TRIO_SIZE {
            return Err(ServerError::BadCacheTrio(format!(
                "un trio a au plus {TRIO_SIZE} cartes"
            )));
        }
        for (index, card) in trio.iter().enumerate() {
            if trio[..index].contains(card) {
                return Err(ServerError::BadCacheTrio(format!("{card} y est deux fois")));
            }
            if !self.cards.contains(card) {
                return Err(ServerError::NotOwned(*card));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fight_of_pions::{AttackKind, Rarity};
    use std::collections::BTreeSet;

    fn card(kind: AttackKind, rarity: Rarity) -> Attack {
        Attack { kind, rarity }
    }

    #[test]
    fn the_starter_cards_are_distinct_cards_of_the_online_catalog() {
        let starter = starter_cards().unwrap();
        assert!(!starter.is_empty());
        let distinct: BTreeSet<_> = starter.iter().collect();
        assert_eq!(distinct.len(), starter.len(), "une carte en double");
        let online = bots::variants()
            .into_iter()
            .find(|variant| variant.name == bots::ONLINE_VARIANT)
            .unwrap()
            .config
            .catalog_attacks();
        let stray: Vec<_> = starter
            .iter()
            .filter(|card| !online.contains(card))
            .collect();
        assert!(stray.is_empty(), "hors du catalogue en ligne : {stray:?}");
    }

    #[test]
    fn the_cache_is_chosen_in_the_trio() {
        let ours = card(AttackKind::Ours, Rarity::Common);
        let hare = card(AttackKind::Lievre, Rarity::Common);
        let collection = Collection {
            cards: starter_cards().unwrap().to_vec(),
            cache_trio: vec![hare, ours],
        };
        assert_eq!(collection.cache_choices(&[]), [hare, ours]);
    }

    #[test]
    fn without_a_trio_three_owned_catalog_cards_are_drawn_at_random() {
        let owned = starter_cards().unwrap().to_vec();
        let legendary = card(AttackKind::Ours, Rarity::Legendary);
        let mut collection = Collection {
            cards: owned.clone(),
            cache_trio: Vec::new(),
        };
        collection.cards.push(legendary);
        // Le catalogue : toutes les cartes possédées sauf la Légendaire et la première.
        let catalog = &owned[1..];
        let mut seen = BTreeSet::new();
        for _ in 0..200 {
            let drawn = collection.cache_choices(catalog);
            assert_eq!(drawn.len(), 3);
            let distinct: BTreeSet<_> = drawn.iter().collect();
            assert_eq!(distinct.len(), 3, "trois cartes différentes");
            assert!(drawn.iter().all(|card| catalog.contains(card)));
            seen.extend(drawn);
        }
        assert_eq!(seen.len(), catalog.len(), "chaque carte peut sortir");

        let few = Collection {
            cards: owned[..2].to_vec(),
            cache_trio: Vec::new(),
        };
        let mut drawn = few.cache_choices(&owned);
        drawn.sort_unstable();
        let mut expected = owned[..2].to_vec();
        expected.sort_unstable();
        assert_eq!(drawn, expected, "moins de 3 cartes : toutes");
        assert!(few.cache_choices(&[legendary]).is_empty());
    }

    #[test]
    fn a_trio_has_at_most_three_distinct_owned_cards() {
        let owned = starter_cards().unwrap().to_vec();
        let collection = Collection {
            cards: owned.clone(),
            cache_trio: Vec::new(),
        };
        assert_eq!(collection.check_trio(&[]), Ok(()));
        assert_eq!(collection.check_trio(&owned[..3]), Ok(()));
        assert!(matches!(
            collection.check_trio(&owned[..4]),
            Err(ServerError::BadCacheTrio(_))
        ));
        assert!(matches!(
            collection.check_trio(&[owned[0], owned[0]]),
            Err(ServerError::BadCacheTrio(_))
        ));
        let legendary = card(AttackKind::Ours, Rarity::Legendary);
        assert_eq!(
            collection.check_trio(&[owned[0], legendary]),
            Err(ServerError::NotOwned(legendary))
        );
    }
}
