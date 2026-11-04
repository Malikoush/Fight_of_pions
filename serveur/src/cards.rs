//! Le catalogue des cartes, tel que le joueur le lit : pour chaque famille, son nom, ce
//! qu'elle fait, ses règles et l'effet de chacune de ses raretés.
//!
//! Les textes vivent dans un fichier de données (`data/cartes.json`), pas dans le code :
//! on les relit et on les corrige sans toucher au serveur. Ils sont tirés des fiches
//! attaques, corrigées par le carnet de règles. Les noms viennent du moteur (`Display` de
//! `AttackKind` et de `Rarity`, les mêmes que dans le labo) ; le coût et la recharge, de
//! la configuration de chaque partie. Les tests vérifient que chaque carte jouable a son
//! texte, et qu'aucun texte ne décrit une carte qui n'existe pas.

use crate::ServerError;
use fight_of_pions::{AttackKind, Rarity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Les textes des cartes, compris dans le serveur à la compilation.
const CARD_TEXTS: &str = include_str!("../data/cartes.json");

/// Une famille d'attaques, telle que le client l'affiche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardFamily {
    /// La famille, telle que la vue la nomme (`CoupDuTaureau`).
    pub kind: AttackKind,
    /// Son nom affiché (« Coup du taureau »).
    pub name: String,
    /// Vrai pour une réaction, jouée pendant le tour adverse.
    pub reaction: bool,
    /// Ce qu'elle fait, en une phrase.
    pub summary: String,
    /// Ses règles communes à toutes les raretés.
    pub rules: Vec<String>,
    /// Ses raretés jouables, de la Commune à la Légendaire.
    pub rarities: Vec<CardRarity>,
}

/// Une rareté d'une famille : une carte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardRarity {
    /// La rareté, telle que la vue la nomme (`Epic`).
    pub rarity: Rarity,
    /// Son nom affiché (« Épique »).
    pub name: String,
    /// L'effet de cette carte.
    pub effect: String,
}

/// Les textes d'une famille, tels que le fichier de données les écrit.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilyText {
    summary: String,
    rules: Vec<String>,
    effects: BTreeMap<Rarity, String>,
}

static CATALOG: LazyLock<Result<Vec<CardFamily>, String>> = LazyLock::new(|| read(CARD_TEXTS));

/// Toutes les familles, dans l'ordre des fiches attaques. Une erreur ne peut venir que
/// d'un fichier de textes mal écrit, ce que les tests empêchent.
pub fn catalog() -> Result<&'static [CardFamily], ServerError> {
    CATALOG
        .as_deref()
        .map_err(|why| ServerError::CardsUnavailable(why.clone()))
}

/// Les familles décrites par ce texte JSON.
fn read(json: &str) -> Result<Vec<CardFamily>, String> {
    let texts: BTreeMap<AttackKind, FamilyText> =
        serde_json::from_str(json).map_err(|error| error.to_string())?;
    Ok(texts
        .into_iter()
        .map(|(kind, text)| CardFamily {
            kind,
            name: kind.to_string(),
            reaction: kind.is_reaction(),
            summary: text.summary,
            rules: text.rules,
            rarities: text
                .effects
                .into_iter()
                .map(|(rarity, effect)| CardRarity {
                    rarity,
                    name: rarity.to_string(),
                    effect,
                })
                .collect(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fight_of_pions::Attack;
    use std::collections::BTreeSet;

    /// Les cartes que le serveur peut proposer : celles des catalogues de ses variantes.
    fn playable_cards() -> BTreeSet<Attack> {
        bots::variants()
            .iter()
            .flat_map(|variant| variant.config.attack_catalog.iter())
            .map(|entry| entry.attack)
            .collect()
    }

    fn described_cards(families: &[CardFamily]) -> BTreeSet<Attack> {
        families
            .iter()
            .flat_map(|family| {
                family.rarities.iter().map(|card| Attack {
                    kind: family.kind,
                    rarity: card.rarity,
                })
            })
            .collect()
    }

    #[test]
    fn the_card_texts_are_well_formed() {
        assert!(catalog().is_ok(), "{:?}", catalog().err());
    }

    #[test]
    fn every_playable_card_has_its_text() {
        let described = described_cards(catalog().unwrap());
        let missing: Vec<_> = playable_cards().difference(&described).copied().collect();
        assert!(missing.is_empty(), "cartes sans texte : {missing:?}");
    }

    #[test]
    fn no_text_describes_a_card_that_cannot_be_played() {
        let described = described_cards(catalog().unwrap());
        let stray: Vec<_> = described.difference(&playable_cards()).copied().collect();
        assert!(stray.is_empty(), "textes de cartes injouables : {stray:?}");
    }

    #[test]
    fn no_text_is_empty() {
        for family in catalog().unwrap() {
            assert!(
                !family.summary.trim().is_empty(),
                "{} : résumé vide",
                family.name
            );
            assert!(
                family.rules.iter().all(|rule| !rule.trim().is_empty()),
                "{} : règle vide",
                family.name
            );
            for card in &family.rarities {
                assert!(
                    !card.effect.trim().is_empty(),
                    "{} ({}) : effet vide",
                    family.name,
                    card.name
                );
            }
        }
    }

    #[test]
    fn names_come_from_the_engine_and_an_unknown_field_is_refused() {
        let json = r#"{"Ours": {"summary": "s", "rules": [], "effects": {"Rare": "e"}}}"#;
        let families = read(json).unwrap();
        assert_eq!(families[0].name, "Ours");
        assert_eq!(families[0].rarities[0].name, "Rare");
        assert!(!families[0].reaction);
        let unknown = r#"{"Ours": {"summary": "s", "rules": [], "effects": {}, "cost": 2}}"#;
        assert!(read(unknown).is_err());
    }
}
