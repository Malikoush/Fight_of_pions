//! Le Médecin et les tombes (B5b, règles précisées le 2026-10-30 dans le carnet).
//!
//! Premières règles, avec l'infirmerie et le mal du débarquement : J1 a 1 action à
//! son premier tour, puis 2 par tour ; chacun garde jusqu'à 2 actions.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, EffectKind, Event, Game, GameConfig, GameError, Player, Position,
    Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn doctor(rarity: Rarity) -> Attack {
    Attack {
        kind: AttackKind::Medecin,
        rarity,
    }
}

fn config(infirmary: u32, player_one: Vec<Attack>, player_two: Vec<Attack>) -> GameConfig {
    GameConfig {
        infirmary_turns: infirmary,
        summoning_sickness: true,
        ..config_with_attacks(player_one, player_two)
    }
}

/// J1 pose E5 puis D5 ; J2 pose F5, et au tour 4, tue E5 avec l'Ours Rare (emplacement 0),
/// puis pose sur la tombe si `occupy`. C'est à J1, au tour 5, avec 3 actions.
fn after_e5_is_killed(config: GameConfig, occupy: bool) -> Game {
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(4, 5), Action::EndTurn]); // tour 2, J2 : F5, garde 1
    play(&mut game, &[place(4, 3), Action::EndTurn]); // tour 3, J1 : D5, garde 1
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(4, 4)],
    );
    play(
        &mut game,
        &[if occupy { place(4, 4) } else { Action::EndTurn }],
    );
    assert_eq!(game.turn(), 5);
    assert_eq!(game.actions_left(), 3);
    game
}

fn pion_at(game: &Game, pos: Position) -> fight_of_pions::Pion {
    game.board().get(pos).and_then(|cell| cell.pion()).unwrap()
}

#[test]
fn a_killed_pion_leaves_a_tomb_until_it_leaves_the_infirmary() {
    let mut game = after_e5_is_killed(config(2, Vec::new(), vec![ours(Rarity::Rare)]), false);
    let dead = game.infirmary()[0].pion();
    assert_eq!(game.board().tomb(at(4, 4)), Some(dead));
    // Une tombe ne bloque pas la case.
    play(&mut game, &[place(4, 4)]);
    assert_eq!(game.board().tomb(at(4, 4)), Some(dead));
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 5 et 6
    // La tombe s'efface à la fin du tour 7, quand le pion sort de l'infirmerie.
    assert_eq!(game.board().tomb(at(4, 4)), Some(dead));
    let events = play(&mut game, &[Action::EndTurn]);
    assert!(events.contains(&Event::TombRemoved {
        pion: dead,
        at: at(4, 4)
    }));
    assert_eq!(game.board().tomb(at(4, 4)), None);
}

#[test]
fn the_common_doctor_brings_a_pion_back_on_its_free_tomb() {
    let config = config(2, vec![doctor(Rarity::Common)], vec![ours(Rarity::Rare)]);
    let mut game = after_e5_is_killed(config, false);
    let dead = game.infirmary()[0];
    let reserve = game.reserve_pions(Player::One);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 3), choose(4, 4)],
    );
    let revived = pion_at(&game, at(4, 4));
    let tail: Vec<Event> = events
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                Event::InfirmaryLeft(_) | Event::TombRemoved { .. } | Event::PionPlaced { .. }
            )
        })
        .collect();
    assert_eq!(
        tail,
        [
            Event::InfirmaryLeft(dead),
            Event::TombRemoved {
                pion: dead.pion(),
                at: at(4, 4)
            },
            Event::PionPlaced {
                pion: revived,
                at: at(4, 4)
            },
        ]
    );
    // Il vient de l'infirmerie, pas de la réserve, et compte comme posé ce tour.
    assert_eq!(game.reserve_pions(Player::One), reserve);
    assert!(game.infirmary().is_empty());
    assert!(game.is_summoning_sick(at(4, 4)));
}

#[test]
fn the_epic_doctor_kills_the_enemy_on_the_tomb_and_protects_the_pion() {
    let doctors = vec![doctor(Rarity::Common), doctor(Rarity::Epic)];
    let mut game = after_e5_is_killed(config(2, doctors, vec![ours(Rarity::Rare)]), true);
    // La Commune ne ressuscite que sur une tombe libre.
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(doctor(Rarity::Common)))
    );
    let occupant = pion_at(&game, at(4, 4));
    play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 3), choose(4, 4)],
    );
    assert_eq!(game.board().owner(at(4, 4)), Some(Player::One));
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
    // 2 tours de J2 : les tours 6 et 8.
    let guard = game
        .active_effects()
        .iter()
        .find(|effect| effect.kind() == EffectKind::Protected)
        .unwrap();
    assert_eq!(guard.last_turn(), Some(8));
    // L'occupant tué laisse sa tombe à son tour, à la place de celle du pion revenu.
    assert_eq!(game.board().tomb(at(4, 4)), Some(occupant));
}

#[test]
fn a_tomb_under_your_own_pion_cannot_be_used() {
    let doctors = vec![doctor(Rarity::Common), doctor(Rarity::Epic)];
    let mut game = after_e5_is_killed(config(2, doctors, vec![ours(Rarity::Rare)]), false);
    play(&mut game, &[place(4, 4), Action::EndTurn]); // tour 5 : un pion de J1 sur sa tombe
    play(&mut game, &[Action::EndTurn]); // tour 6
    assert_eq!(game.actions_left(), 4);
    for slot in 0..2 {
        assert!(matches!(
            game.apply(Action::UseAttack(slot)),
            Err(GameError::NoTarget(_))
        ));
    }
}

#[test]
fn a_tomb_under_a_protected_enemy_cannot_be_used() {
    let royal_guard = Attack {
        kind: AttackKind::Reine,
        rarity: Rarity::Common,
    };
    let player_two = vec![ours(Rarity::Rare), royal_guard];
    let mut game = after_e5_is_killed(config(2, vec![doctor(Rarity::Epic)], player_two), true);
    play(&mut game, &[Action::EndTurn]); // tour 5
    play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 4), Action::EndTurn],
    ); // tour 6 : l'occupant se protège
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(doctor(Rarity::Epic)))
    );
}

/// J1 pose E5, puis E6 et D5 ; au tour 4, J2 tue E5 et E6 avec l'Ours Épique. C'est à J1,
/// au tour 5, avec 2 actions.
fn after_e5_and_e6_are_killed(infirmary: u32, player_one: Vec<Attack>) -> Game {
    let mut game = game_with(config(infirmary, player_one, vec![ours(Rarity::Epic)]));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(&mut game, &[place(5, 4), place(4, 3)]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 5),
            choose(4, 4),
            choose(5, 4),
        ],
    );
    assert_eq!(game.infirmary().len(), 2);
    game
}

#[test]
fn the_rare_doctor_brings_back_up_to_two_pions() {
    let mut game = after_e5_and_e6_are_killed(2, vec![doctor(Rarity::Rare)]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 3), choose(4, 4)],
    );
    // Un seul suffit ; on peut aussi en prendre un second.
    assert!(game.legal_actions().contains(&Action::FinishAttack));
    play(&mut game, &[choose(5, 4)]);
    assert!(game.infirmary().is_empty());
    assert_eq!(game.board().owner(at(5, 4)), Some(Player::One));
}

#[test]
fn the_legendary_doctor_brings_back_every_pion_that_can_come_back() {
    // 4 tours d'infirmerie : le pion qui ne peut pas revenir y est encore après le tour 9.
    let mut game = after_e5_and_e6_are_killed(4, vec![doctor(Rarity::Legendary)]);
    play(&mut game, &[place(5, 4), Action::EndTurn]); // tour 5 : un pion de J1 sur la tombe E6
    play(
        &mut game,
        &[Action::EndTurn, Action::EndTurn, Action::EndTurn],
    ); // tours 6 à 8
    assert_eq!(game.actions_left(), 4);
    // Pas de cible à choisir : tous ceux qui peuvent revenir reviennent. E6 est occupée
    // par un pion de J1 : son pion reste à l'infirmerie.
    play(&mut game, &[Action::UseAttack(0), choose(4, 3)]);
    assert_eq!(game.board().owner(at(4, 4)), Some(Player::One));
    assert_eq!(game.infirmary().len(), 1);
}

#[test]
fn a_new_tomb_replaces_the_old_one_on_the_same_cell() {
    let mut game = after_e5_is_killed(config(2, Vec::new(), vec![ours(Rarity::Rare)]), false);
    let first = game.infirmary()[0].pion();
    play(&mut game, &[place(4, 4), Action::EndTurn]); // tour 5 : un autre pion en E5
    let second = pion_at(&game, at(4, 4));
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(4, 4)],
    ); // tour 6
    let tombs: Vec<&Event> = events
        .iter()
        .filter(|event| matches!(event, Event::TombAdded { .. } | Event::TombRemoved { .. }))
        .collect();
    assert_eq!(
        tombs,
        [
            &Event::TombRemoved {
                pion: first,
                at: at(4, 4)
            },
            &Event::TombAdded {
                pion: second,
                at: at(4, 4)
            },
        ]
    );
    // Le premier pion reste à l'infirmerie, sans tombe.
    assert_eq!(game.infirmary().len(), 2);
    assert_eq!(game.board().tomb_of(first.id()), None);
}

#[test]
fn without_infirmary_there_is_no_tomb_and_no_doctor() {
    let config = config(0, vec![doctor(Rarity::Common)], vec![ours(Rarity::Rare)]);
    let mut game = after_e5_is_killed(config, false);
    assert_eq!(game.board().tomb(at(4, 4)), None);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(doctor(Rarity::Common)))
    );
}

#[test]
fn the_doctor_never_uses_an_enemy_tomb() {
    let config = config(
        2,
        vec![ours(Rarity::Rare), doctor(Rarity::Common)],
        Vec::new(),
    );
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(4, 5), place(9, 9)]); // tour 2, J2 : F5, J10
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    ); // tour 3 : J1 tue F5, qui laisse sa tombe
    assert!(game.board().tomb(at(4, 5)).is_some());
    play(&mut game, &[Action::EndTurn]); // tour 4
    assert_eq!(
        game.apply(Action::UseAttack(1)),
        Err(GameError::NoTarget(doctor(Rarity::Common)))
    );
}

/// Comme `after_e5_and_e6_are_killed`, avec 4 tours d'infirmerie et la carte de J2 en plus.
fn e5_and_e6_with(player_one: Vec<Attack>, extra_for_two: Attack) -> Game {
    let player_two = vec![ours(Rarity::Epic), extra_for_two];
    let mut game = game_with(config(4, player_one, player_two));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(&mut game, &[place(5, 4), place(4, 3)]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 5),
            choose(4, 4),
            choose(5, 4),
        ],
    );
    game
}

#[test]
fn the_legendary_doctor_kills_enemies_on_tombs_but_skips_protected_ones() {
    let royal_guard = Attack {
        kind: AttackKind::Reine,
        rarity: Rarity::Common,
    };
    let mut game = e5_and_e6_with(vec![doctor(Rarity::Legendary)], royal_guard);
    play(&mut game, &[Action::EndTurn]); // tour 5 : J1 garde 2
    play(&mut game, &[place(4, 4), place(5, 4)]); // tour 6 : J2 occupe les deux tombes
    play(&mut game, &[Action::EndTurn]); // tour 7
    play(
        &mut game,
        &[Action::UseAttack(1), choose(5, 4), Action::EndTurn],
    ); // tour 8 : E6 se protège
    assert_eq!(game.actions_left(), 4);
    play(&mut game, &[Action::UseAttack(0), choose(4, 3)]); // tour 9
    // L'occupant de E5 est tué et le pion de J1 revient ; celui de E6, protégé, reste.
    assert_eq!(game.board().owner(at(4, 4)), Some(Player::One));
    assert_eq!(game.board().owner(at(5, 4)), Some(Player::Two));
    let left_behind = game
        .infirmary()
        .iter()
        .filter(|patient| patient.pion().owner() == Player::One)
        .count();
    assert_eq!(left_behind, 1);
}

#[test]
fn the_legendary_doctor_is_not_offered_when_nobody_can_come_back() {
    let attila = Attack {
        kind: AttackKind::Attila,
        rarity: Rarity::Common,
    };
    let mut game = e5_and_e6_with(vec![doctor(Rarity::Legendary)], attila);
    play(&mut game, &[place(4, 4), place(5, 4)]); // tour 5 : J1 sur ses deux tombes
    play(
        &mut game,
        &[Action::EndTurn, Action::EndTurn, Action::EndTurn],
    ); // tours 6 à 8
    assert_eq!(game.actions_left(), 4);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(doctor(Rarity::Legendary)))
    );
}

#[test]
fn an_occupant_killed_by_the_epic_doctor_is_avenged_by_its_friend() {
    let best_friends = Attack {
        kind: AttackKind::MeilleursAmis,
        rarity: Rarity::Epic,
    };
    let player_two = vec![ours(Rarity::Rare), best_friends];
    let mut game = after_e5_is_killed(config(2, vec![doctor(Rarity::Epic)], player_two), false);
    play(&mut game, &[Action::EndTurn]); // tour 5 : J1 garde 2
    // Tour 6 : les amis de J2, l'un sur la tombe E5, l'autre en A10.
    play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(4, 5),
            choose(4, 4),
            choose(9, 0),
        ],
    );
    assert_eq!(game.turn(), 7); // les amis ont dépensé les dernières actions de J2
    // Tour 7 : le Médecin Épique tue l'ami sur la tombe ; A10 le venge sur le lanceur D5.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 3), choose(4, 4)],
    );
    assert_eq!(game.board().owner(at(4, 3)), None);
    assert_eq!(game.board().owner(at(4, 4)), Some(Player::One));
}
