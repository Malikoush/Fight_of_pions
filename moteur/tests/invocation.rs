//! Invoquer (B5a, règles précisées le 2026-10-30 dans le carnet) : la séduction (Beauté
//! fatale Épique et Légendaire), Meilleurs amis, le pacte et la vengeance.
//!
//! Premières règles : J1 a 1 action à son premier tour, puis 2 par tour ; chacun
//! garde jusqu'à 2 actions.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
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

fn seduction(rarity: Rarity) -> Attack {
    card(AttackKind::BeauteFatale, rarity)
}

fn best_friends() -> Attack {
    card(AttackKind::MeilleursAmis, Rarity::Epic)
}

/// Les pions qui meurent ou arrivent, dans l'ordre, sous une forme courte.
fn arrivals(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::PionKilled { at, .. } => Some(format!("tué {at}")),
            Event::PionPlaced { at, pion } => Some(format!("posé {at} ({:?})", pion.owner())),
            _ => None,
        })
        .collect()
}

fn pacts(game: &Game) -> usize {
    game.active_effects()
        .iter()
        .filter(|effect| matches!(effect.kind(), EffectKind::Pact { .. }))
        .count()
}

/// J1 a ces cartes et pose E5 ; J2 a les siennes et pose ces cases. J1 garde ses actions
/// un tour, J2 aussi : c'est à J1, au tour 5, avec 4 actions.
fn ready(config: GameConfig, player_two: &[Position]) -> Game {
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    let mut second: Vec<Action> = player_two.iter().map(|&pos| Action::Place(pos)).collect();
    if second.len() < 2 {
        second.push(Action::EndTurn);
    }
    play(&mut game, &second);
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    assert_eq!(game.actions_left(), 4);
    game
}

#[test]
fn the_epic_charm_seduces_an_enemy_in_sight() {
    let config = GameConfig {
        summoning_sickness: true,
        ..config_with_attacks(vec![seduction(Rarity::Epic)], Vec::new())
    };
    let mut game = ready(config, &[at(4, 6), at(9, 9)]);
    let reserve = game.reserve_pions(Player::One);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    // Tuer, puis poser un des siens à la place : ce n'est pas un changement de camp.
    assert_eq!(arrivals(&events), ["tué G5", "posé G5 (One)"]);
    assert_eq!(game.reserve_pions(Player::One), reserve - 1);
    // Le nouveau pion compte comme posé ce tour.
    assert!(game.is_summoning_sick(at(4, 6)));
}

#[test]
fn seduction_needs_a_pion_in_reserve() {
    // 2 pions chacun ; J2 garde le Garde royal, toujours jouable, pour que la partie ne se
    // fige pas quand les réserves sont vides.
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let config = GameConfig {
        pions_per_player: 2,
        ..config_with_attacks(vec![seduction(Rarity::Epic)], vec![royal_guard])
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(4, 6), place(9, 9)]); // tour 2, J2 : G5, J10
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 3, J1 : A1, réserve vide
    play(&mut game, &[Action::EndTurn]); // tour 4
    assert_eq!(game.reserve_pions(Player::One), 0);
    assert_eq!(game.actions_left(), 3);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(seduction(Rarity::Epic)))
    );
}

#[test]
fn seduction_cannot_target_a_protected_pion() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let config = config_with_attacks(vec![seduction(Rarity::Epic)], vec![royal_guard]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(
        &mut game,
        &[place(4, 6), Action::UseAttack(0), choose(4, 6)],
    ); // G5 protégé
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(seduction(Rarity::Epic)))
    );
}

#[test]
fn the_legendary_charm_seduces_anywhere_and_protects_the_new_pion() {
    let config = config_with_attacks(vec![seduction(Rarity::Legendary)], Vec::new());
    let mut game = ready(config, &[at(9, 0), at(9, 9)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(9, 0)],
    );
    assert_eq!(arrivals(&events), ["tué A10", "posé A10 (One)"]);
    assert!(game.has_effect(at(9, 0), EffectKind::Protected));
    // Jusqu'au prochain tour de J1 : la protection couvre le tour 6 de J2.
    let guard = game.active_effects()[0];
    assert_eq!(guard.last_turn(), Some(6));
}

#[test]
fn best_friends_place_two_pions_anywhere_bound_by_a_pact() {
    let config = config_with_attacks(vec![best_friends()], Vec::new());
    let mut game = ready(config, &[at(9, 9), at(9, 8)]);
    let events = play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(0, 0),
            choose(0, 9),
        ],
    );
    assert_eq!(arrivals(&events), ["posé A1 (One)", "posé J1 (One)"]);
    let first = game
        .board()
        .get(at(0, 0))
        .and_then(|cell| cell.pion())
        .unwrap();
    let second = game
        .board()
        .get(at(0, 9))
        .and_then(|cell| cell.pion())
        .unwrap();
    let kinds: Vec<EffectKind> = game.active_effects().iter().map(|e| e.kind()).collect();
    assert_eq!(
        kinds,
        [
            EffectKind::Pact {
                friend: second.id()
            },
            EffectKind::Pact { friend: first.id() },
        ]
    );
    // Un pacte n'a pas de durée.
    assert!(
        game.active_effects()
            .iter()
            .all(|e| e.last_turn().is_none())
    );
}

#[test]
fn best_friends_need_two_pions_in_reserve() {
    let config = GameConfig {
        pions_per_player: 2,
        ..config_with_attacks(vec![best_friends()], Vec::new())
    };
    let mut game = ready(config, &[at(9, 9)]);
    assert_eq!(game.reserve_pions(Player::One), 1);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(best_friends()))
    );
}

/// J2 a ces cartes et un pion en H5 (et J10). Au tour 5, J1 pose ses amis en G5 (collé à
/// H5) et `second_friend` ; c'est à J2, au tour 6, avec 4 actions.
fn friends_next_to_h5(player_two: Vec<Attack>, second_friend: Position, infirmary: u32) -> Game {
    let config = GameConfig {
        infirmary_turns: infirmary,
        ..config_with_attacks(vec![best_friends()], player_two)
    };
    let mut game = ready(config, &[at(4, 7), at(9, 9)]);
    let friend = Action::Choose(second_friend);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6), friend],
    );
    play(&mut game, &[Action::EndTurn]);
    assert_eq!(game.actions_left(), 4);
    game
}

#[test]
fn a_killed_friend_is_avenged_on_its_assassin() {
    let mut game = friends_next_to_h5(vec![ours(Rarity::Rare)], at(0, 0), 0);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 7), choose(4, 6)],
    );
    // L'ami en A1 tue l'assassin, à la fin de l'attaque.
    assert_eq!(arrivals(&events), ["tué G5", "tué H5"]);
    let avenger = game
        .board()
        .get(at(0, 0))
        .and_then(|cell| cell.pion())
        .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::PionKilled { at: killed, by: Some(by), .. } if *killed == at(4, 7) && *by == avenger.id()
    )));
    // Le survivant redevient un pion normal.
    assert_eq!(pacts(&game), 0);
}

#[test]
fn friends_killed_by_the_same_attack_are_not_avenged() {
    let mut game = friends_next_to_h5(vec![ours(Rarity::Epic)], at(5, 6), 0);
    let events = play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 7),
            choose(4, 6),
            choose(5, 6),
        ],
    );
    assert_eq!(arrivals(&events), ["tué G5", "tué G6"]);
    assert_eq!(game.board().owner(at(4, 7)), Some(Player::Two));
}

#[test]
fn vengeance_fails_on_a_protected_assassin_but_the_pact_ends() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let mut game = friends_next_to_h5(vec![royal_guard, ours(Rarity::Rare)], at(0, 0), 0);
    play(&mut game, &[Action::UseAttack(0), choose(4, 7)]); // H5 se protège
    let events = play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 7), choose(4, 6)],
    );
    assert_eq!(arrivals(&events), ["tué G5"]);
    assert_eq!(game.board().owner(at(4, 7)), Some(Player::Two));
    assert_eq!(pacts(&game), 0);
}

#[test]
fn a_pion_killed_during_its_own_turn_counts_its_owners_next_turns_at_the_infirmary() {
    let mut game = friends_next_to_h5(vec![ours(Rarity::Rare)], at(0, 0), 2);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 7), choose(4, 6)],
    );
    let patients: Vec<(Player, u32)> = events
        .iter()
        .filter_map(|event| match event {
            Event::InfirmaryEntered(patient) => Some((patient.pion().owner(), patient.last_turn())),
            _ => None,
        })
        .collect();
    // Tour 6, celui de J2. L'ami de J1 compte les tours 7 et 9 ; l'assassin de J2, tué par
    // la vengeance pendant son propre tour, compte les tours 8 et 10.
    assert_eq!(patients, [(Player::One, 9), (Player::Two, 10)]);
}

#[test]
fn attila_common_does_not_remove_a_pact() {
    let attila = card(AttackKind::Attila, Rarity::Common);
    let mut game = friends_next_to_h5(vec![attila], at(0, 0), 0);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(attila))
    );
}

#[test]
fn seducing_a_friend_is_a_kill_that_gets_avenged() {
    let mut game = friends_next_to_h5(vec![seduction(Rarity::Epic)], at(0, 0), 0);
    // J2 séduit G5 depuis H5, collé : H5 est l'assassin.
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 7), choose(4, 6)],
    );
    assert_eq!(arrivals(&events), ["tué G5", "posé G5 (Two)", "tué H5"]);
}

#[test]
fn vengeance_chains_through_another_pact() {
    let config = config_with_attacks(
        vec![best_friends()],
        vec![best_friends(), ours(Rarity::Rare)],
    );
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(9, 9), Action::EndTurn]); // tour 2, J2 : J10, garde 1
    play(&mut game, &[Action::EndTurn]); // tour 3, J1 garde 2
    // Tour 4 : les amis de J2, en H5 et A10.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(9, 9),
            choose(4, 7),
            choose(9, 0),
        ],
    );
    // Tour 5 : les amis de J1, en G5 (collé à H5) et A1.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 6),
            choose(0, 0),
        ],
    );
    play(&mut game, &[Action::EndTurn]);
    // Tour 6 : H5 tue G5 ; A1 venge G5 en tuant H5 ; A10 venge H5 en tuant A1.
    let events = play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 7), choose(4, 6)],
    );
    assert_eq!(arrivals(&events), ["tué G5", "tué H5", "tué A1"]);
    assert_eq!(game.board().owner(at(9, 0)), Some(Player::Two));
    assert_eq!(pacts(&game), 0);
}

#[test]
fn an_immobilized_friend_still_avenges() {
    let player_two = vec![ours(Rarity::Common), ours(Rarity::Rare)];
    let mut game = friends_next_to_h5(player_two, at(3, 6), 0);
    // Le Grognement de H5 fige les deux amis, G5 et G4 ; puis l'Ours tue G5.
    play(&mut game, &[Action::UseAttack(0), choose(4, 7)]);
    assert!(game.has_effect(at(3, 6), EffectKind::Immobilized));
    let events = play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 7), choose(4, 6)],
    );
    assert_eq!(arrivals(&events), ["tué G5", "tué H5"]);
}

#[test]
fn a_card_that_places_does_not_use_the_placement_of_the_turn() {
    let config = GameConfig {
        max_placements_per_turn: Some(1),
        ..config_with_attacks(vec![best_friends()], Vec::new())
    };
    let mut game = ready(config, &[at(9, 9)]);
    assert_eq!(game.placements_left(), 1);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(0, 0),
            choose(0, 9),
        ],
    );
    // La carte n'a coûté que ses 3 actions ; la pose du tour reste permise.
    assert_eq!(game.actions_left(), 1);
    assert_eq!(game.placements_left(), 1);
    assert!(game.legal_actions().contains(&place(5, 5)));
}

#[test]
fn a_wolf_killing_friends_of_two_pacts_is_avenged_once_and_both_pacts_end() {
    let wolf = card(AttackKind::SautDuLoup, Rarity::Epic);
    let mut game = game_with(config_with_attacks(vec![best_friends()], vec![wolf]));
    play(&mut game, &[place(0, 0)]); // tour 1, J1 : A1
    play(&mut game, &[place(4, 4), place(9, 9)]); // tour 2, J2 : E5 (le loup), J10
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 3 et 4
    // Tour 5 : un premier pacte, F5 et A10.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 0),
            choose(4, 5),
            choose(9, 0),
        ],
    );
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 5 et 6
    // Tour 7 : un second pacte, indépendant, H5 et J1.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 0),
            choose(4, 7),
            choose(0, 9),
        ],
    );
    assert_eq!(pacts(&game), 4);
    // Tour 8 : le loup survole F5 puis H5 et finit en I5. A10 le venge là où il a
    // atterri ; la vengeance de J1 échoue (l'assassin est déjà mort), son pacte prend fin.
    let events = play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 6),
            choose(4, 8),
        ],
    );
    assert_eq!(arrivals(&events), ["tué F5", "tué H5", "tué I5"]);
    assert_eq!(pacts(&game), 0);
    assert_eq!(game.board().owner(at(0, 9)), Some(Player::One));
}

#[test]
fn a_line_made_by_placing_friends_wins_at_the_start_of_the_next_turn() {
    let config = GameConfig {
        align_to_win: 3,
        ..config_with_attacks(vec![best_friends()], Vec::new())
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]); // tour 1, J1 : A1
    play(&mut game, &[place(9, 9), place(9, 7)]); // tour 2
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 3 et 4
    // Tour 5 : les amis en B1 et C1 font A1-B1-C1, sans gagner tout de suite.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 0),
            choose(0, 1),
            choose(0, 2),
        ],
    );
    assert_eq!(game.end(), None);
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // fin des tours 5 et 6
    assert_eq!(game.winner(), Some(Player::One));
}
