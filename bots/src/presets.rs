//! Les réglages prêts à comparer, partagés par l'exemple `arene` et le labo : les bots
//! disponibles, les paires de l'arène et les variantes de configuration.

use crate::{Banking, Bot, GreedyBot, RandomBot};
use fight_of_pions::{Attack, AttackKind, GameConfig, Rarity};

/// Un des bots disponibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BotKind {
    /// Le bot aléatoire, l'étalon.
    Random,
    /// Le bot glouton qui dépense toutes ses actions.
    Greedy,
    /// Le bot glouton qui garde le maximum d'actions permis.
    Banker,
}

static RANDOM: RandomBot = RandomBot;
static GREEDY: GreedyBot = GreedyBot::new(Banking::SpendAll);
static BANKER: GreedyBot = GreedyBot::new(Banking::KeepMax);

impl BotKind {
    /// Tous les bots disponibles, dans l'ordre où les proposer.
    pub const ALL: [BotKind; 3] = [BotKind::Random, BotKind::Greedy, BotKind::Banker];

    /// Le bot lui-même.
    pub fn bot(self) -> &'static (dyn Bot + Sync) {
        match self {
            BotKind::Random => &RANDOM,
            BotKind::Greedy => &GREEDY,
            BotKind::Banker => &BANKER,
        }
    }

    /// Son nom, celui des bilans.
    pub fn name(self) -> &'static str {
        self.bot().name()
    }
}

/// Les paires jouées par défaut dans l'arène (joueur 1, joueur 2) : deux aléatoires, puis
/// le glouton face à l'étalon aléatoire, puis glouton et banquier, toujours dans les deux
/// places.
pub const STANDARD_PAIRS: [[BotKind; 2]; 7] = [
    [BotKind::Random, BotKind::Random],
    // Face à l'étalon : un glouton qui ne bat pas largement l'aléatoire ne sait pas jouer.
    [BotKind::Greedy, BotKind::Random],
    [BotKind::Random, BotKind::Greedy],
    [BotKind::Greedy, BotKind::Greedy],
    [BotKind::Banker, BotKind::Greedy],
    [BotKind::Greedy, BotKind::Banker],
    [BotKind::Banker, BotKind::Banker],
];

/// Une configuration à comparer, avec son nom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// Le nom affiché.
    pub name: &'static str,
    /// La configuration.
    pub config: GameConfig,
}

/// La règle actuelle, où chaque joueur a toutes les attaques du catalogue en attendant le
/// draft : la première variante, et la partie avec laquelle le labo démarre.
pub fn current_rules() -> GameConfig {
    GameConfig::default().with_full_arsenals()
}

/// Les règles cibles (carnet de règles, « Règles cibles ») : 1 pose gratuite par tour, 1
/// action par tour gardée jusqu'à 3, la Pièce, le mal du débarquement et la règle d'or. Le
/// reste suit la règle actuelle.
pub fn target_rules() -> GameConfig {
    GameConfig {
        first_turn_actions: 1,
        actions_per_turn: 1,
        max_banked_actions: 3,
        starting_banked_actions: [0, 1],
        place_cost: 0,
        max_placements_per_turn: Some(1),
        summoning_sickness: true,
        golden_rule: true,
        ..current_rules()
    }
}

/// Les variantes comparées par défaut, la règle actuelle d'abord. Une ligne de plus ici
/// suffit pour mesurer un nouveau réglage, tant qu'il tient dans la configuration :
/// l'arène et le labo la proposent aussitôt.
pub fn variants() -> Vec<Variant> {
    let current = current_rules();
    let bank_of_one = GameConfig {
        max_banked_actions: 1,
        ..current.clone()
    };
    let ours_rare = Attack {
        kind: AttackKind::Ours,
        rarity: Rarity::Rare,
    };
    vec![
        Variant {
            name: "règles actuelles",
            config: current.clone(),
        },
        Variant {
            name: "banque max 1",
            config: bank_of_one.clone(),
        },
        Variant {
            name: "Ours Rare à 1 action",
            config: with_cost(current, ours_rare, 1),
        },
        Variant {
            name: "banque max 1 et Ours Rare à 1 action",
            config: with_cost(bank_of_one, ours_rare, 1),
        },
        Variant {
            name: "règles cibles",
            config: target_rules(),
        },
    ]
}

/// Cette configuration, où l'attaque coûte `cost` actions.
fn with_cost(mut config: GameConfig, attack: Attack, cost: u32) -> GameConfig {
    for entry in &mut config.attack_catalog {
        if entry.attack == attack {
            entry.stats.cost = cost;
        }
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_is_valid_and_different_from_the_others() {
        let variants = variants();
        assert_eq!(variants[0].config, current_rules());
        for (index, variant) in variants.iter().enumerate() {
            assert_eq!(variant.config.validate(), Ok(()), "{}", variant.name);
            // Un réglage sans effet (une attaque absente du catalogue, par exemple)
            // donnerait une variante identique à une autre.
            for other in &variants[..index] {
                assert_ne!(other.name, variant.name);
                assert_ne!(other.config, variant.config, "{}", variant.name);
            }
        }
    }

    #[test]
    fn every_bot_has_its_own_name() {
        let names: Vec<&str> = BotKind::ALL.iter().map(|kind| kind.name()).collect();
        assert_eq!(names, ["aléatoire", "glouton", "banquier"]);
    }
}
