//! L'économie réglable (carnet de règles, « Économie cible B ») : poses limitées par tour
//! et gratuites, banque de départ (la Pièce), mal du débarquement.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
use fight_of_pions::{Action, Event, Game, GameConfig, GameError, Player, Position, Rarity};

const RARE: usize = 0;

/// Les règles cibles : 1 pose gratuite par tour, 1 action par tour gardée jusqu'à 3, la Pièce
/// et le mal du débarquement ; chaque joueur a l'Ours Rare (2 actions) et l'Épique (3).
fn target_rules() -> GameConfig {
    let arsenal = vec![ours(Rarity::Rare), ours(Rarity::Epic)];
    GameConfig {
        first_turn_actions: 1,
        actions_per_turn: 1,
        max_banked_actions: 3,
        starting_banked_actions: [0, 1],
        place_cost: 0,
        max_placements_per_turn: Some(1),
        summoning_sickness: true,
        ..config_with_attacks(arsenal.clone(), arsenal)
    }
}

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

#[test]
fn a_free_placement_leaves_the_actions_and_the_turn_alone() {
    let mut game = game_with(target_rules());
    assert_eq!(game.placements_left(), 1);

    let events = play(&mut game, &[place(4, 4)]);
    assert!(
        !events.iter().any(|event| matches!(
            event,
            Event::ActionsChanged { .. } | Event::TurnEnded { .. }
        )),
        "{events:?}"
    );
    assert_eq!(game.actions_left(), 1);
    assert_eq!(game.placements_left(), 0);
    assert_eq!(game.current_player(), Player::One);
}

#[test]
fn only_the_allowed_placements_can_be_made_in_a_turn() {
    let mut game = game_with(target_rules());
    play(&mut game, &[place(4, 4)]);

    assert_eq!(game.apply(place(0, 0)), Err(GameError::NoPlacementLeft));
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Place(_)))
    );
    assert_eq!(
        GameError::NoPlacementLeft.to_string(),
        "plus aucune pose permise ce tour"
    );
}

#[test]
fn placements_not_made_are_not_kept() {
    let mut game = game_with(target_rules());
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    assert_eq!(game.current_player(), Player::One);
    assert_eq!(game.placements_left(), 1);
}

#[test]
fn the_second_player_starts_with_the_coin() {
    let mut game = game_with(target_rules());
    assert_eq!(game.banked_actions(Player::Two), 1);
    assert_eq!(game.actions_left(), 1);

    play(&mut game, &[Action::EndTurn]);
    assert_eq!(game.actions_left(), 2); // 1 par tour + la Pièce
}

#[test]
fn the_turn_ends_by_itself_once_actions_and_placements_are_spent() {
    let mut game = game_with(target_rules());
    play(&mut game, &[place(4, 4), Action::EndTurn]); // J1 garde 1 action
    play(&mut game, &[place(4, 5), Action::EndTurn]); // J2, au contact de E5
    assert_eq!(game.actions_left(), 2);

    // J1 tue F5 avec l'Ours Rare (2 actions) : il lui reste sa pose, le tour continue.
    play(
        &mut game,
        &[Action::UseAttack(RARE), choose(4, 4), choose(4, 5)],
    );
    assert_eq!(game.actions_left(), 0);
    assert_eq!(game.current_player(), Player::One);

    let events = play(&mut game, &[place(0, 0)]);
    assert!(events.contains(&Event::TurnEnded { next: Player::Two }));
}

/// J1 pose E5 ; J2 pose F5 à côté, avec 2 actions (1 + la Pièce) : de quoi payer l'Ours
/// Rare, mais F5 vient d'arriver.
fn with_a_pion_just_placed_next_to_an_enemy(summoning_sickness: bool) -> Game {
    let mut game = game_with(GameConfig {
        summoning_sickness,
        ..target_rules()
    });
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5)]);
    game
}

#[test]
fn a_pion_placed_this_turn_cannot_launch_an_attack() {
    let mut game = with_a_pion_just_placed_next_to_an_enemy(true);
    assert!(game.is_summoning_sick(at(4, 5)));
    assert!(!game.is_summoning_sick(at(4, 4))); // le pion adverse n'est jamais nouveau
    let refusal = game.apply(Action::UseAttack(RARE));
    assert_eq!(refusal, Err(GameError::NoReadyLauncher(ours(Rarity::Rare))));
    assert_eq!(
        refusal.unwrap_err().to_string(),
        "l'attaque Ours (Rare) n'a aucun lanceur prêt : un pion posé ce tour ne peut pas attaquer"
    );

    // Au tour suivant de J2, F5 n'est plus nouveau : il peut lancer.
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    assert!(!game.is_summoning_sick(at(4, 5)));
    play(
        &mut game,
        &[Action::UseAttack(RARE), choose(4, 5), choose(4, 4)],
    );
    assert!(game.board().get(at(4, 4)).unwrap().pion().is_none());
}

#[test]
fn without_summoning_sickness_a_new_pion_attacks_at_once() {
    let mut game = with_a_pion_just_placed_next_to_an_enemy(false);
    assert!(!game.is_summoning_sick(at(4, 5)));
    play(
        &mut game,
        &[Action::UseAttack(RARE), choose(4, 5), choose(4, 4)],
    );
    assert!(game.board().get(at(4, 4)).unwrap().pion().is_none());
}

#[test]
fn a_new_pion_that_will_be_able_to_attack_does_not_freeze_the_game() {
    // Un pion chacun, et seul J2 a une attaque : quand J2 pose B1 contre A1, plus personne
    // ne peut poser, et la seule attaque possible part d'un pion qui vient d'arriver.
    let mut game = game_with(GameConfig {
        board_size: 3,
        align_to_win: 3,
        pions_per_player: 1,
        starting_attacks: [Vec::new(), vec![ours(Rarity::Rare)]],
        ..target_rules()
    });
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    play(&mut game, &[place(0, 1)]);
    assert_eq!(game.end(), None);

    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 1), choose(0, 0)],
    );
    assert_eq!(game.reserve_pions(Player::One), 1);
}

#[test]
fn no_placement_is_left_on_a_full_board() {
    // Morpion sans ligne, 2 poses gratuites par tour : à la dernière case, J1 pourrait
    // encore poser (limite, réserve), mais le plateau est plein.
    let mut game = game_with(GameConfig {
        board_size: 3,
        align_to_win: 3,
        max_placements_per_turn: Some(2),
        summoning_sickness: false,
        ..target_rules()
    });
    play(&mut game, &[place(0, 0), place(0, 2), Action::EndTurn]);
    play(&mut game, &[place(0, 1), place(1, 1), Action::EndTurn]);
    play(&mut game, &[place(1, 0), place(2, 1), Action::EndTurn]);
    play(&mut game, &[place(1, 2), place(2, 0), Action::EndTurn]);
    play(&mut game, &[place(2, 2)]);

    assert_eq!(game.end(), None); // les Ours ont encore des cibles
    assert_eq!(game.current_player(), Player::One);
    assert_eq!(game.placements_left(), 0);
}

#[test]
fn placements_left_follow_the_actions_when_placing_costs_actions() {
    let game = game_with(GameConfig::default());
    // Règles actuelles : 1 action au premier tour, et une pose coûte 1 action.
    assert_eq!(game.placements_left(), 1);
}
