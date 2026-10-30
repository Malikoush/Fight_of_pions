//! Le mouvement (B4, règles précisées le 2026-10-30 dans le carnet) : Saut du loup,
//! Lièvre, Interchange Rare et Épique.
//!
//! Configuration par défaut : J1 a 1 action à son premier tour, puis 2 par tour ; chacun
//! garde jusqu'à 2 actions.

mod common;

use common::{choose, config_with_attacks, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, EffectKind, Event, Game, GameConfig, GameError, Player, Position,
    Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn wolf(rarity: Rarity) -> Attack {
    card(AttackKind::SautDuLoup, rarity)
}

fn hare(rarity: Rarity) -> Attack {
    card(AttackKind::Lievre, rarity)
}

fn interchange(rarity: Rarity) -> Attack {
    card(AttackKind::Interchange, rarity)
}

/// Les événements de pions qui bougent ou meurent, dans l'ordre, sous une forme courte.
fn moves(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::PionKilled { at, .. } => Some(format!("tué {at}")),
            Event::PionMoved { from, to, .. } => Some(format!("{from}→{to}")),
            Event::PionsSwapped {
                first_at,
                second_at,
                ..
            } => Some(format!("{first_at}↔{second_at}")),
            _ => None,
        })
        .collect()
}

/// Le joueur 1 a cette carte et pose E5 ; le joueur 2 pose ces deux cases. C'est à J1,
/// au tour 3, avec 2 actions.
fn duel(attack: Attack, player_two: [Position; 2]) -> Game {
    duel_with(config_with_attacks(vec![attack], Vec::new()), player_two)
}

fn duel_with(config: GameConfig, player_two: [Position; 2]) -> Game {
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &player_two.map(Action::Place));
    game
}

/// J1 garde ses actions un tour : au tour 5, il en a 4.
fn save_a_turn(game: &mut Game) {
    play(game, &[Action::EndTurn, Action::EndTurn]);
    assert_eq!(game.actions_left(), 4);
}

#[test]
fn the_wolf_jumps_two_cells_and_kills_the_enemy_it_jumps_over() {
    let mut game = duel(wolf(Rarity::Rare), [at(4, 5), at(9, 9)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    assert_eq!(moves(&events), ["tué F5", "E5→G5"]);
    assert_eq!(game.board().owner(at(4, 6)), Some(Player::One));
}

#[test]
fn a_jump_always_spans_two_cells_even_over_nothing() {
    let mut game = duel(wolf(Rarity::Rare), [at(9, 9), at(0, 9)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert_eq!(
        game.apply(choose(4, 5)),
        Err(GameError::InvalidChoice(at(4, 5)))
    );
    let events = play(&mut game, &[choose(4, 6)]);
    assert_eq!(moves(&events), ["E5→G5"]);
}

#[test]
fn the_wolf_cannot_land_on_an_occupied_cell() {
    let mut game = duel(wolf(Rarity::Rare), [at(4, 5), at(4, 6)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert_eq!(
        game.apply(choose(4, 6)),
        Err(GameError::InvalidChoice(at(4, 6)))
    );
}

#[test]
fn chained_jumps_kill_at_once_and_free_the_cell() {
    let mut game = duel(wolf(Rarity::Epic), [at(4, 5), at(4, 7)]);
    save_a_turn(&mut game);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    // F5 est déjà mort : le loup peut repasser par sa case pour revenir en E5.
    assert!(game.legal_actions().contains(&choose(4, 4)));
    let events = play(&mut game, &[choose(4, 8)]);
    assert_eq!(moves(&events), ["tué F5", "E5→G5", "tué H5", "G5→I5"]);
    assert_eq!(game.board().owner(at(4, 8)), Some(Player::One));
}

#[test]
fn a_pion_cannot_be_jumped_over_twice() {
    let mut game = duel(wolf(Rarity::Epic), [at(9, 9), at(0, 9)]);
    play(&mut game, &[place(4, 5), Action::EndTurn]); // tour 3 : J1 pose F5 et garde 1
    play(&mut game, &[Action::EndTurn]); // tour 4
    assert_eq!(game.actions_left(), 3);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    // Revenir en E5 survolerait encore F5 : interdit. S'arrêter là est permis.
    assert_eq!(
        game.apply(choose(4, 4)),
        Err(GameError::InvalidChoice(at(4, 4)))
    );
    let events = play(&mut game, &[Action::FinishAttack]);
    assert_eq!(moves(&events), ["E5→G5"]);
}

#[test]
fn a_protected_enemy_can_be_jumped_over_but_survives() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let config = config_with_attacks(vec![wolf(Rarity::Rare)], vec![royal_guard]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(
        &mut game,
        &[place(4, 5), Action::UseAttack(0), choose(4, 5)],
    ); // F5 protégé
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    assert_eq!(moves(&events), ["E5→G5"]);
    assert_eq!(game.board().owner(at(4, 5)), Some(Player::Two));
}

#[test]
fn the_common_wolf_immobilizes_the_enemy_it_jumps_over() {
    let mut game = duel(wolf(Rarity::Common), [at(4, 5), at(9, 9)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    assert_eq!(moves(&events), ["E5→G5"]);
    assert!(game.has_effect(at(4, 5), EffectKind::Immobilized));
}

#[test]
fn the_hare_steps_up_to_two_cells_and_can_turn() {
    let mut game = duel(hare(Rarity::Common), [at(9, 9), at(0, 9)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    // Pas de saut : un pas va sur une case collée.
    assert_eq!(
        game.apply(choose(4, 6)),
        Err(GameError::InvalidChoice(at(4, 6)))
    );
    play(&mut game, &[choose(4, 5)]);
    // Un pas suffit (« jusqu'à 2 ») ; on peut aussi tourner pour le second.
    assert!(game.legal_actions().contains(&Action::FinishAttack));
    let events = play(&mut game, &[choose(3, 6)]);
    assert_eq!(moves(&events), ["E5→G4"]);
}

#[test]
fn the_hare_cannot_end_on_its_starting_cell() {
    let mut game = duel(hare(Rarity::Common), [at(9, 9), at(0, 9)]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );
    assert_eq!(
        game.apply(choose(4, 4)),
        Err(GameError::InvalidChoice(at(4, 4)))
    );
}

#[test]
fn the_rare_hare_moves_two_pions_one_after_the_other() {
    let config = GameConfig {
        summoning_sickness: true,
        ..config_with_attacks(vec![hare(Rarity::Rare)], Vec::new())
    };
    let mut game = duel_with(config, [at(9, 9), at(0, 9)]);
    play(&mut game, &[place(4, 3), Action::EndTurn]); // tour 3 : J1 pose D5, garde 1
    play(&mut game, &[Action::EndTurn]); // tour 4
    play(&mut game, &[place(0, 0)]); // tour 5 : A1, posé ce tour
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    // Le lanceur doit bouger avant qu'on passe au pion suivant.
    assert_eq!(
        game.apply(choose(4, 3)),
        Err(GameError::InvalidChoice(at(4, 3)))
    );
    play(&mut game, &[choose(4, 5)]);
    // A1 vient d'arriver : il ne peut pas agir. Le lanceur, déjà bougé en F5, ne peut
    // pas être repris. D5 le peut.
    for (row, col) in [(0, 0), (4, 5)] {
        assert_eq!(
            game.apply(choose(row, col)),
            Err(GameError::InvalidChoice(at(row, col)))
        );
    }
    play(&mut game, &[choose(4, 3)]);
    // Le second pion voit la nouvelle place du premier : E5 est libre.
    let events = play(&mut game, &[choose(4, 4), Action::FinishAttack]);
    assert_eq!(moves(&events), ["E5→F5", "D5→E5"]);
}

#[test]
fn the_legendary_hare_moves_three_pions_three_steps_each() {
    let mut game = duel(hare(Rarity::Legendary), [at(9, 9), at(0, 9)]);
    play(&mut game, &[place(1, 1), place(7, 7)]); // tour 3 : J1 pose B2 et H8
    play(&mut game, &[Action::EndTurn]); // tour 4
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tour 7 : 4 actions
    assert_eq!(game.actions_left(), 4);

    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    play(&mut game, &[choose(4, 5), choose(4, 6), choose(4, 7)]);
    // Trois pas au plus : un quatrième est refusé.
    assert_eq!(
        game.apply(choose(4, 8)),
        Err(GameError::InvalidChoice(at(4, 8)))
    );
    play(
        &mut game,
        &[choose(1, 1), choose(1, 2), choose(1, 3), choose(1, 4)],
    );
    // Le troisième pion fait ses trois pas : il n'y a plus rien à choisir.
    let events = play(
        &mut game,
        &[choose(7, 7), choose(7, 8), choose(7, 9), choose(8, 9)],
    );
    assert_eq!(moves(&events), ["E5→H5", "B2→E2", "H8→J9"]);
    assert!(!game.attack_in_progress());
}

#[test]
fn the_legendary_wolf_jumps_three_times() {
    let mut game = duel(wolf(Rarity::Legendary), [at(4, 5), at(4, 7)]);
    play(&mut game, &[Action::EndTurn]); // tour 3 : J1 garde 2 actions
    play(&mut game, &[place(5, 8), Action::EndTurn]); // tour 4 : I6
    assert_eq!(game.actions_left(), 4);
    let events = play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 6),
            choose(4, 8),
            choose(6, 8),
        ],
    );
    assert_eq!(
        moves(&events),
        ["tué F5", "E5→G5", "tué H5", "G5→I5", "tué I6", "I5→I7"]
    );
    assert!(!game.attack_in_progress());
}

#[test]
fn the_interchange_never_targets_one_of_your_pions() {
    let mut game = duel(interchange(Rarity::Rare), [at(4, 6), at(9, 9)]);
    play(&mut game, &[place(4, 5), Action::EndTurn]); // tour 3 : J1 pose F5, garde 1
    play(&mut game, &[Action::EndTurn]); // tour 4
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert_eq!(
        game.apply(choose(4, 5)),
        Err(GameError::InvalidChoice(at(4, 5)))
    );
    assert!(game.apply(choose(4, 6)).is_ok());
}

#[test]
fn a_line_given_to_the_opponent_by_a_swap_wins_at_the_start_of_its_turn() {
    let config = GameConfig {
        align_to_win: 3,
        ..config_with_attacks(vec![interchange(Rarity::Epic)], Vec::new())
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 2)]); // tour 1, J1 : C1
    play(&mut game, &[place(0, 0), place(0, 1)]); // tour 2, J2 : A1, B1
    play(&mut game, &[Action::EndTurn]); // tour 3 : J1 garde 2 actions
    play(&mut game, &[place(9, 9), Action::EndTurn]); // tour 4, J2 : J10
    // Tour 5 : C1 échange sa case avec J10 : J2 a maintenant A1-B1-C1.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 2), choose(9, 9)],
    );
    assert_eq!(game.end(), None);
    play(&mut game, &[Action::EndTurn]); // début du tour 6, celui de J2
    assert_eq!(game.winner(), Some(Player::Two));
}

#[test]
fn the_rare_interchange_swaps_with_an_enemy_two_cells_away_even_behind_a_pion() {
    let mut game = duel(interchange(Rarity::Rare), [at(4, 5), at(4, 6)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    let events = play(&mut game, &[choose(4, 6)]);
    assert_eq!(moves(&events), ["E5↔G5"]);
    assert_eq!(game.board().owner(at(4, 4)), Some(Player::Two));
    assert_eq!(game.board().owner(at(4, 6)), Some(Player::One));
}

#[test]
fn the_rare_interchange_does_not_reach_three_cells() {
    let mut game = duel(interchange(Rarity::Rare), [at(4, 7), at(9, 9)]);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(interchange(Rarity::Rare)))
    );
}

#[test]
fn the_epic_interchange_reaches_any_enemy_and_its_effects_follow_it() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let config = config_with_attacks(vec![interchange(Rarity::Epic)], vec![royal_guard]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(
        &mut game,
        &[place(9, 0), Action::UseAttack(0), choose(9, 0)],
    ); // A10 protégé
    save_a_turn(&mut game);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(9, 0)],
    );
    assert_eq!(moves(&events), ["E5↔A10"]);
    // La protection suit le pion, qui est maintenant en E5.
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
    assert_eq!(game.board().owner(at(4, 4)), Some(Player::Two));
}

#[test]
fn an_immobilized_enemy_can_still_be_swapped() {
    let charm = card(AttackKind::BeauteFatale, Rarity::Common);
    let config = config_with_attacks(vec![charm, interchange(Rarity::Rare)], Vec::new());
    let mut game = duel_with(config, [at(4, 6), at(9, 9)]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    assert!(game.has_effect(at(4, 6), EffectKind::Immobilized));
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tour 5 : 3 actions
    play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 4), choose(4, 6)],
    );
    assert!(game.has_effect(at(4, 4), EffectKind::Immobilized));
    assert_eq!(game.board().owner(at(4, 4)), Some(Player::Two));
}

#[test]
fn a_line_made_by_moving_wins_at_the_start_of_your_next_turn() {
    let config = GameConfig {
        align_to_win: 3,
        ..config_with_attacks(vec![hare(Rarity::Common)], Vec::new())
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]); // tour 1, J1 : A1
    play(&mut game, &[place(9, 9), place(9, 7)]); // tour 2
    play(&mut game, &[place(0, 1), place(0, 3)]); // tour 3 : B1, D1
    play(&mut game, &[Action::EndTurn]); // tour 4
    // Tour 5 : D1 fait un pas en C1. A1-B1-C1 : pas de victoire tout de suite.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 3),
            choose(0, 2),
            Action::FinishAttack,
        ],
    );
    assert_eq!(game.end(), None);
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // fin des tours 5 et 6
    assert_eq!(game.winner(), Some(Player::One));
}
