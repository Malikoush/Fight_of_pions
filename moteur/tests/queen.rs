//! La reine (B5c, règles précisées le 2026-10-31 dans le carnet) : la pose, le pas gratuit,
//! le bouclier et le tir des gardes, une seule reine à la fois.
//!
//! Configuration par défaut : J1 a 1 action à son premier tour, puis 2 par tour ; chacun
//! garde jusqu'à 2 actions ; un tir coûte 1 action.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, EffectKind, Event, Game, GameError, Player, Position, Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn queen(rarity: Rarity) -> Attack {
    Attack {
        kind: AttackKind::Reine,
        rarity,
    }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn guards(game: &Game, pos: Position) -> usize {
    let pion = game.board().get(pos).and_then(|cell| cell.pion()).unwrap();
    game.guards_of(pion.id())
}

#[test]
fn a_queen_is_placed_anywhere_and_steps_once_per_turn_from_the_next_turn() {
    let mut game = game_with(config_with_attacks(vec![queen(Rarity::Rare)], Vec::new()));
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(9, 9), place(9, 7)]); // tour 2
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(0, 0)],
    ); // tour 3 : la reine en A1
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EffectStarted(effect) if effect.kind() == EffectKind::Crown
    )));
    assert_eq!(game.queen_of(Player::One), Some(at(0, 0)));
    // Posée ce tour, elle ne bouge pas : le tour de J1 s'est fini tout seul.
    assert_eq!(game.current_player(), Player::Two);
    play(&mut game, &[Action::EndTurn]); // tour 4

    // Tour 5 : un pas, gratuit, puis plus de second pas.
    play(&mut game, &[Action::StepQueen(at(0, 1))]);
    assert_eq!(game.queen_of(Player::One), Some(at(0, 1)));
    assert_eq!(game.actions_left(), 2);
    assert_eq!(
        game.apply(Action::StepQueen(at(0, 2))),
        Err(GameError::QueenCannotStep(at(0, 2)))
    );
    // Une seule reine à la fois.
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(queen(Rarity::Rare)))
    );
}

#[test]
fn the_turn_waits_for_the_free_step_of_the_queen() {
    let mut game = game_with(config_with_attacks(vec![queen(Rarity::Rare)], Vec::new()));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(9, 9), place(9, 7)]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(0, 0)],
    );
    play(&mut game, &[Action::EndTurn]); // tour 4
    // Tour 5 : deux poses dépensent les actions, mais le pas de la reine reste à faire.
    play(&mut game, &[place(2, 2), place(2, 3)]);
    assert_eq!(game.current_player(), Player::One);
    play(&mut game, &[Action::StepQueen(at(1, 1))]);
    assert_eq!(game.current_player(), Player::Two);
}

#[test]
fn an_immobilized_queen_does_not_step() {
    let charm = card(AttackKind::BeauteFatale, Rarity::Common);
    let config = config_with_attacks(vec![queen(Rarity::Rare)], vec![charm]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(2, 0), place(9, 9)]); // tour 2, J2 : A3
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(0, 0)],
    ); // reine en A1
    // Tour 4 : A3 immobilise la reine, à 2 cases dans son regard.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(2, 0), choose(0, 0)],
    );
    play(&mut game, &[Action::EndTurn]);
    assert_eq!(
        game.apply(Action::StepQueen(at(0, 1))),
        Err(GameError::QueenCannotStep(at(0, 1)))
    );
}

/// J1 pose E5 ; J2 pose F5 (son lanceur) et J10. Au tour 5, J1 pose sa reine en G5,
/// collée à F5, avec la Reine de cette rareté ; c'est à J2, au tour 6, avec 4 actions.
fn guarded_queen_next_to_f5(rarity: Rarity, player_two: Vec<Attack>) -> Game {
    let mut game = game_with(config_with_attacks(vec![queen(rarity)], player_two));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), place(9, 9)]);
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    if game.current_player() == Player::One {
        play(&mut game, &[Action::EndTurn]);
    }
    assert_eq!(game.actions_left(), 4);
    game
}

#[test]
fn a_guard_saves_the_queen_from_a_kill_once() {
    let mut game = guarded_queen_next_to_f5(Rarity::Epic, vec![ours(Rarity::Rare)]);
    assert_eq!(guards(&game, at(4, 6)), 1);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(4, 6)],
    );
    assert!(!events.iter().any(|e| matches!(e, Event::PionKilled { .. })));
    assert_eq!(game.queen_of(Player::One), Some(at(4, 6)));
    assert_eq!(guards(&game, at(4, 6)), 0);
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 6 et 7
    // Plus de garde : la reine meurt, et sa couronne avec elle.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(4, 6)],
    );
    assert_eq!(game.queen_of(Player::One), None);
    assert!(game.active_effects().is_empty());
}

#[test]
fn a_guard_does_not_stop_what_would_not_kill_the_queen() {
    let mut game = guarded_queen_next_to_f5(Rarity::Epic, vec![ours(Rarity::Common)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 5)]); // le Grognement
    assert!(game.has_effect(at(4, 6), EffectKind::Immobilized));
    assert_eq!(guards(&game, at(4, 6)), 1);
}

#[test]
fn a_guard_saves_the_queen_from_a_seduction_which_then_does_nothing() {
    let seduction = card(AttackKind::BeauteFatale, Rarity::Epic);
    let mut game = guarded_queen_next_to_f5(Rarity::Epic, vec![seduction]);
    let reserve = game.reserve_pions(Player::Two);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(4, 6)],
    );
    assert_eq!(game.queen_of(Player::One), Some(at(4, 6)));
    assert_eq!(game.reserve_pions(Player::Two), reserve);
    assert_eq!(guards(&game, at(4, 6)), 0);
}

#[test]
fn a_guard_shoots_the_first_enemy_in_a_line_once_per_turn() {
    let mut game = game_with(config_with_attacks(
        vec![queen(Rarity::Legendary)],
        Vec::new(),
    ));
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(4, 0), place(9, 0)]); // tour 2, J2 : A5, A10
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 3 et 4
    // Tour 5 : la reine en A1, avec 2 gardes ; la carte dépense les 4 actions du tour.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(0, 0)],
    );
    play(&mut game, &[Action::EndTurn]); // tour 6
    assert_eq!(game.current_player(), Player::One);

    // Tour 7 : E5, sur sa diagonale, est à J1 : pas de tir. A5, au bout de la colonne, oui.
    assert_eq!(
        game.apply(Action::Shoot(at(4, 4))),
        Err(GameError::QueenCannotShoot(at(4, 4)))
    );
    let events = play(&mut game, &[Action::Shoot(at(4, 0))]);
    assert!(events.contains(&Event::QueenShot {
        queen: at(0, 0),
        target: at(4, 0)
    }));
    assert_eq!(game.board().owner(at(4, 0)), None);
    assert_eq!(guards(&game, at(0, 0)), 1);
    assert_eq!(game.actions_left(), 1);
    // A10 est maintenant au bout de la colonne, mais un seul tir par tour.
    assert_eq!(
        game.apply(Action::Shoot(at(9, 0))),
        Err(GameError::QueenCannotShoot(at(9, 0)))
    );
}

#[test]
fn a_queen_without_guards_cannot_shoot() {
    let mut game = guarded_queen_next_to_f5(Rarity::Rare, Vec::new());
    play(&mut game, &[Action::EndTurn]); // tour 6
    assert_eq!(
        game.apply(Action::Shoot(at(4, 5))),
        Err(GameError::QueenCannotShoot(at(4, 5)))
    );
}

#[test]
fn an_attack_stopped_by_a_guard_does_nothing_else_to_the_queen() {
    // La séduction Légendaire tue, pose, puis protège : arrêtée par le garde, elle ne
    // protège pas la reine adverse.
    let seduction = card(AttackKind::BeauteFatale, Rarity::Legendary);
    let mut game = guarded_queen_next_to_f5(Rarity::Epic, vec![seduction]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(4, 6)],
    );
    assert_eq!(game.queen_of(Player::One), Some(at(4, 6)));
    assert!(!game.has_effect(at(4, 6), EffectKind::Protected));
}

#[test]
fn the_step_and_the_shot_come_back_every_turn() {
    let mut game = game_with(config_with_attacks(
        vec![queen(Rarity::Legendary)],
        Vec::new(),
    ));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 0), place(9, 0)]); // A5, A10
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(0, 0)],
    ); // tour 5
    play(&mut game, &[Action::EndTurn]); // tour 6
    // Tour 7 : depuis A1, un tir sur A5, puis un pas en B1.
    play(
        &mut game,
        &[Action::Shoot(at(4, 0)), Action::StepQueen(at(0, 1))],
    );
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 7 et 8
    // Tour 9 : de nouveau un pas (retour en A1) et un tir avec le second garde, sur A10,
    // désormais le premier pion de la colonne.
    play(
        &mut game,
        &[Action::StepQueen(at(0, 0)), Action::Shoot(at(9, 0))],
    );
    assert_eq!(game.board().owner(at(9, 0)), None);
    assert_eq!(guards(&game, at(0, 0)), 0);
}

#[test]
fn the_queen_steps_only_to_a_free_cell_next_to_her() {
    let mut game = game_with(config_with_attacks(vec![queen(Rarity::Rare)], Vec::new()));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(0, 1), place(9, 9)]); // tour 2, J2 : B1, collé à A1
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(0, 0)],
    ); // reine en A1
    play(&mut game, &[Action::EndTurn]);
    for (row, col) in [(0, 1), (0, 2)] {
        assert_eq!(
            game.apply(Action::StepQueen(at(row, col))),
            Err(GameError::QueenCannotStep(at(row, col)))
        );
    }
    assert!(game.apply(Action::StepQueen(at(1, 0))).is_ok());
}

#[test]
fn a_guard_does_not_shoot_a_protected_enemy() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let config = config_with_attacks(vec![queen(Rarity::Epic)], vec![royal_guard]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 0), Action::EndTurn]); // tour 2, J2 : A5
    play(&mut game, &[Action::EndTurn]); // tour 3 : J1 garde 2
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 0), Action::EndTurn],
    ); // tour 4 : A5 protégé
    // Tour 5 : la reine en A1, puis J1 garde son action.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(0, 0),
            Action::EndTurn,
        ],
    );
    play(&mut game, &[Action::EndTurn]); // tour 6
    // Tour 7 : A5, au bout de la colonne, est protégé jusqu'à la fin de ce tour.
    assert!(game.has_effect(at(4, 0), EffectKind::Protected));
    assert_eq!(
        game.apply(Action::Shoot(at(4, 0))),
        Err(GameError::QueenCannotShoot(at(4, 0)))
    );
}

#[test]
fn a_queen_needs_a_pion_in_reserve() {
    let config = fight_of_pions::GameConfig {
        pions_per_player: 2,
        ..config_with_attacks(
            vec![queen(Rarity::Rare), card(AttackKind::Reine, Rarity::Common)],
            vec![card(AttackKind::Reine, Rarity::Common)],
        )
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(9, 9), Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 3 : J1, réserve vide
    play(&mut game, &[Action::EndTurn]); // tour 4
    assert_eq!(game.reserve_pions(Player::One), 0);
    assert_eq!(game.actions_left(), 3);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(queen(Rarity::Rare)))
    );
}
