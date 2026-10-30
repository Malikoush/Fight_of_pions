//! L'infirmerie (règles cibles, fiches du 2026-10-29 ; horloge du carnet, 2026-10-30) :
//! un pion tué passe `infirmary_turns` tours à l'infirmerie, comptés en tours de
//! l'adversaire du joueur qui l'a tué (ici son propriétaire), puis revient dans sa réserve.
//!
//! Configuration par défaut : J1 a 1 action à son premier tour, puis 2 par tour.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
use fight_of_pions::{Action, Event, Game, GameConfig, Patient, Player, Rarity};

/// L'infirmerie de 2 tours ; J1 a l'Ours Rare. J1 en E5, J2 en F5 (collé). C'est à J1,
/// au tour 3, avec 2 actions : tuer les dépense toutes et finit son tour.
fn wounded_soon(pions_per_player: u32) -> Game {
    let config = GameConfig {
        infirmary_turns: 2,
        pions_per_player,
        ..config_with_attacks(vec![ours(Rarity::Rare)], Vec::new())
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5)]);
    if game.current_player() == Player::Two {
        play(&mut game, &[Action::EndTurn]);
    }
    game
}

fn entered(events: &[Event]) -> Vec<Patient> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::InfirmaryEntered(patient) => Some(*patient),
            _ => None,
        })
        .collect()
}

#[test]
fn a_killed_pion_spends_two_turns_of_its_owner_in_the_infirmary() {
    let mut game = wounded_soon(15);
    assert_eq!(game.reserve_pions(Player::Two), 14);

    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );
    let patients = entered(&events);
    assert_eq!(patients.len(), 1);
    assert_eq!(patients[0].pion().owner(), Player::Two);
    // Les tours 4 et 6 sont ceux de J2 : il sort à la fin du tour 6.
    assert_eq!(patients[0].last_turn(), 6);
    assert_eq!(game.infirmary(), patients.as_slice());
    // Il n'est pas revenu dans la réserve.
    assert_eq!(game.reserve_pions(Player::Two), 14);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ReserveChanged { .. }))
    );

    for turn in 4..6 {
        assert_eq!(game.turn(), turn);
        play(&mut game, &[Action::EndTurn]);
        assert_eq!(game.infirmary().len(), 1, "sorti trop tôt, au tour {turn}");
    }
    let events = play(&mut game, &[Action::EndTurn]); // fin du tour 6
    assert!(events.contains(&Event::InfirmaryLeft(patients[0])));
    assert!(events.contains(&Event::ReserveChanged {
        player: Player::Two,
        pions: 15,
    }));
    assert!(game.infirmary().is_empty());
}

#[test]
fn a_game_is_not_frozen_while_a_pion_waits_in_the_infirmary() {
    // Un pion chacun : une fois celui de J2 tué, plus personne ne peut rien faire... sauf
    // attendre qu'il sorte de l'infirmerie.
    let mut game = wounded_soon(1);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );
    assert_eq!(game.end(), None);

    while game.infirmary().len() == 1 {
        play(&mut game, &[Action::EndTurn]);
        assert_eq!(game.end(), None);
    }
    // Revenu dans la réserve, il peut être reposé au tour suivant de J2.
    assert_eq!(game.reserve_pions(Player::Two), 1);
    if game.current_player() == Player::One {
        play(&mut game, &[Action::EndTurn]);
    }
    assert!(game.legal_actions().contains(&place(4, 5)));
}
