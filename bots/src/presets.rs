//! Les réglages prêts à comparer, partagés par l'exemple `arene` et le labo : les bots
//! disponibles, les paires de l'arène et les variantes de configuration.

use crate::{Banking, Bot, GreedyBot, MctsBot, RandomBot, Weights};
use fight_of_pions::{Attack, AttackKind, DraftRules, GameConfig, Rarity};

/// Un des bots disponibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BotKind {
    /// Le bot aléatoire, l'étalon.
    Random,
    /// Le bot glouton qui dépense toutes ses actions.
    Greedy,
    /// Le bot glouton qui garde le maximum d'actions permis.
    Banker,
    /// Le bot MCTS, avec les poids de départ de l'évaluation.
    Mcts,
}

/// Les simulations du bot MCTS par décision.
const MCTS_ITERATIONS: u32 = 3000;

static RANDOM: RandomBot = RandomBot;
static GREEDY: GreedyBot = GreedyBot::new(Banking::SpendAll);
static BANKER: GreedyBot = GreedyBot::new(Banking::KeepMax);
static MCTS: MctsBot = MctsBot::new(MCTS_ITERATIONS, Weights::DEFAULT);

impl BotKind {
    /// Tous les bots disponibles, dans l'ordre où les proposer.
    pub const ALL: [BotKind; 4] = [
        BotKind::Random,
        BotKind::Greedy,
        BotKind::Banker,
        BotKind::Mcts,
    ];

    /// Le bot lui-même.
    pub fn bot(self) -> &'static (dyn Bot + Sync) {
        match self {
            BotKind::Random => &RANDOM,
            BotKind::Greedy => &GREEDY,
            BotKind::Banker => &BANKER,
            BotKind::Mcts => &MCTS,
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

/// Le nom de la variante jouée en ligne et mesurée par l'outil d'équilibrage : les règles
/// actuelles avec draft.
pub const ONLINE_VARIANT: &str = "règles actuelles avec draft";

/// Une configuration à comparer, avec son nom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// Le nom affiché.
    pub name: &'static str,
    /// La configuration.
    pub config: GameConfig,
}

/// Les règles actuelles (la configuration par défaut du moteur), où chaque joueur a toutes
/// les attaques du catalogue en attendant le draft : la première variante, et la partie
/// avec laquelle le labo démarre.
pub fn current_rules() -> GameConfig {
    GameConfig::default().with_full_arsenals()
}

/// Les premières règles ([`GameConfig::first_rules`]), gardées au cas où, avec toutes les
/// attaques du catalogue.
pub fn first_rules() -> GameConfig {
    GameConfig::first_rules().with_full_arsenals()
}

/// Les variantes comparées par défaut, les règles actuelles d'abord. Une ligne de plus ici
/// suffit pour mesurer un nouveau réglage, tant qu'il tient dans la configuration :
/// l'arène et le labo la proposent aussitôt.
pub fn variants() -> Vec<Variant> {
    let first = first_rules();
    // Avec 3 actions au plus par tour, les Légendaires (4) ne sont plus payables.
    let bank_of_one = GameConfig {
        max_banked_actions: 1,
        ..first.clone()
    }
    .without_unaffordable_attacks();
    let ours_rare = Attack {
        kind: AttackKind::Ours,
        rarity: Rarity::Rare,
    };
    vec![
        Variant {
            name: "règles actuelles",
            config: current_rules(),
        },
        Variant {
            name: "premières règles",
            config: first.clone(),
        },
        Variant {
            name: "premières règles, banque max 1",
            config: bank_of_one.clone(),
        },
        Variant {
            name: "premières règles, Ours Rare à 1 action",
            config: with_cost(first, ours_rare, 1),
        },
        Variant {
            name: "premières règles, banque max 1 et Ours Rare à 1 action",
            config: with_cost(bank_of_one, ours_rare, 1),
        },
        Variant {
            name: ONLINE_VARIANT,
            config: GameConfig {
                draft: Some(DraftRules::standard()),
                ..GameConfig::default()
            },
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
        assert_eq!(names, ["aléatoire", "glouton", "banquier", "MCTS"]);
    }
}
