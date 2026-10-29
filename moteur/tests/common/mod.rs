//! Outils partagés par les tests d'intégration du moteur.
#![allow(dead_code)] // chaque fichier de tests n'utilise qu'une partie de ces outils

use fight_of_pions::{
    Action, Attack, AttackKind, AttackStats, CatalogEntry, Event, Game, GameConfig, Pion, Position,
    Rarity, Rng,
};

/// L'Ours dans cette rareté.
pub fn ours(rarity: Rarity) -> Attack {
    Attack {
        kind: AttackKind::Ours,
        rarity,
    }
}

/// La configuration par défaut, où chaque joueur commence avec ces attaques.
pub fn config_with_attacks(player_one: Vec<Attack>, player_two: Vec<Attack>) -> GameConfig {
    GameConfig {
        starting_attacks: [player_one, player_two],
        ..GameConfig::default()
    }
}

/// L'action « choisir la case » (rangée, colonne) : lanceur ou cible d'une attaque.
pub fn choose(row: usize, col: usize) -> Action {
    Action::Choose(Position { row, col })
}

/// L'action « poser un pion » sur la case (rangée, colonne).
pub fn place(row: usize, col: usize) -> Action {
    Action::Place(Position { row, col })
}

/// La graine utilisée par les tests qui ne s'intéressent pas au hasard.
pub const SEED: u64 = 0;

/// Une partie avec la configuration par défaut.
pub fn new_game() -> Game {
    game_with(GameConfig::default())
}

/// Une partie avec cette configuration, qui doit être valide.
pub fn game_with(config: GameConfig) -> Game {
    Game::new(config, SEED).unwrap()
}

/// Joue une suite d'actions qui doivent toutes réussir ;
/// renvoie les événements de la dernière action.
pub fn play(game: &mut Game, actions: &[Action]) -> Vec<Event> {
    let mut last = Vec::new();
    for &action in actions {
        last = game.apply(action).unwrap();
    }
    last
}

/// Le pion annoncé par le premier événement d'une pose.
pub fn placed_pion(events: &[Event]) -> Pion {
    match events.first() {
        Some(Event::PionPlaced { pion, .. }) => *pion,
        other => panic!("une pose devait être annoncée, reçu : {other:?}"),
    }
}

/// Un indice tiré au hasard dans `0..len` (`len` > 0), avec le générateur du moteur.
pub fn random_below(rng: &mut Rng, len: usize) -> usize {
    let value = rng.below(len as u64).expect("len doit être positif");
    value as usize
}

/// Une configuration valide tirée au hasard, petite pour que les fins arrivent vite.
/// Elle couvre les cas où il faut économiser ses actions pour poser ou attaquer, les poses
/// limitées par tour (parfois gratuites), la banque de départ, le mal du débarquement et la règle d'or,
/// et donne souvent des attaques aux joueurs (parfois deux fois la même carte).
pub fn random_config(rng: &mut Rng) -> GameConfig {
    let board_size = 2 + random_below(rng, 4);
    let actions_per_turn = 1 + random_below(rng, 2) as u32;
    let max_banked_actions = random_below(rng, 3) as u32;
    let max_actions = (actions_per_turn + max_banked_actions) as usize;
    let max_placements_per_turn =
        (random_below(rng, 2) == 1).then(|| 1 + random_below(rng, 2) as u32);
    // La pose ne peut être gratuite que si elle est limitée par tour.
    let cheapest_place = usize::from(max_placements_per_turn.is_none());
    let mut random_bank = || random_below(rng, max_banked_actions as usize + 1) as u32;
    let starting_banked_actions = [random_bank(), random_bank()];
    let attacks = [ours(Rarity::Rare), ours(Rarity::Epic)];
    let attack_catalog = attacks
        .iter()
        .map(|&attack| CatalogEntry {
            attack,
            stats: AttackStats {
                cost: 1 + random_below(rng, max_actions) as u32,
                cooldown: 1 + random_below(rng, 2) as u32,
            },
        })
        .collect();
    let mut random_arsenal = || -> Vec<Attack> {
        (0..random_below(rng, 4))
            .map(|_| attacks[random_below(rng, attacks.len())])
            .collect()
    };
    let starting_attacks = [random_arsenal(), random_arsenal()];

    let config = GameConfig {
        board_size,
        align_to_win: 2 + random_below(rng, board_size - 1),
        pions_per_player: 1 + random_below(rng, 9) as u32,
        first_turn_actions: 1 + random_below(rng, 5) as u32,
        actions_per_turn,
        max_banked_actions,
        starting_banked_actions,
        place_cost: (cheapest_place + random_below(rng, max_actions + 1 - cheapest_place)) as u32,
        max_placements_per_turn,
        summoning_sickness: random_below(rng, 2) == 1,
        golden_rule: random_below(rng, 2) == 1,
        max_turns: 1 + random_below(rng, 60) as u32,
        attack_catalog,
        starting_attacks,
    };
    assert_eq!(config.validate(), Ok(()), "{config:?}");
    config
}
