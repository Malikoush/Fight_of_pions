//! Règles des attaques (carnet de règles, « Attaques ») et de l'Ours Rare et Épique.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
use fight_of_pions::{
    Action, Decision, Event, Game, GameConfig, GameError, Pion, Player, Position, Rarity,
};

const RARE: usize = 0;
const EPIC: usize = 1;

/// Les deux joueurs ont l'Ours Rare (emplacement 0) et l'Ours Épique (emplacement 1).
fn duel() -> Game {
    let arsenal = vec![ours(Rarity::Rare), ours(Rarity::Epic)];
    game_with(config_with_attacks(arsenal.clone(), arsenal))
}

fn pion_at(game: &Game, row: usize, col: usize) -> Pion {
    game.board()
        .get(Position { row, col })
        .and_then(|cell| cell.pion())
        .unwrap()
}

/// Le joueur 1 pose en E5, le joueur 2 pose en F5 et F6 (à côté) : c'est au joueur 1,
/// avec 2 actions.
fn with_two_enemies_next_to_e5() -> Game {
    let mut game = duel();
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), place(5, 5)]);
    game
}

/// Comme ci-dessus, mais le joueur 1 a gardé ses actions un tour : il en a 4.
fn with_four_actions() -> Game {
    let mut game = with_two_enemies_next_to_e5();
    play(&mut game, &[Action::EndTurn]); // le joueur 1 garde 2 actions
    play(&mut game, &[Action::EndTurn]); // le joueur 2 aussi
    assert_eq!(game.actions_left(), 4);
    game
}

#[test]
fn ours_rare_kills_an_adjacent_enemy_who_goes_back_to_reserve() {
    let mut game = with_two_enemies_next_to_e5();
    let launcher = pion_at(&game, 4, 4);
    let victim = pion_at(&game, 4, 5);

    play(&mut game, &[Action::UseAttack(RARE), choose(4, 4)]);
    let events = play(&mut game, &[choose(4, 5)]);

    assert_eq!(
        events,
        vec![
            Event::AttackUsed {
                player: Player::One,
                attack: ours(Rarity::Rare),
                launcher: Position { row: 4, col: 4 },
            },
            Event::ActionsChanged {
                player: Player::One,
                actions_left: 0,
                banked: 0,
            },
            Event::CooldownChanged {
                player: Player::One,
                slot: RARE,
                turns_left: 1,
            },
            Event::PionKilled {
                pion: victim,
                at: Position { row: 4, col: 5 },
                by: Some(launcher.id()),
            },
            Event::ReserveChanged {
                player: Player::Two,
                pions: 14,
            },
            // L'attaque a dépensé les 2 actions : le tour se termine.
            Event::TurnEnded { next: Player::Two },
            Event::ActionsChanged {
                player: Player::Two,
                actions_left: 2,
                banked: 0,
            },
        ]
    );
    assert_eq!(game.board().owner(Position { row: 4, col: 5 }), None);
    assert_eq!(game.reserve_pions(Player::Two), 14);
}

#[test]
fn choices_before_the_last_one_change_nothing_visible() {
    let mut game = with_two_enemies_next_to_e5();
    let board_before = game.board().clone();

    assert_eq!(play(&mut game, &[Action::UseAttack(RARE)]), vec![]);
    assert_eq!(
        game.pending_decision(),
        Some(Decision::ChooseLauncher {
            player: Player::One,
            attack: ours(Rarity::Rare),
        })
    );
    assert_eq!(play(&mut game, &[choose(4, 4)]), vec![]);
    assert_eq!(
        game.pending_decision(),
        Some(Decision::ChooseTarget {
            player: Player::One,
            attack: ours(Rarity::Rare),
            launcher: Position { row: 4, col: 4 },
        })
    );
    assert_eq!(game.board(), &board_before);
    assert_eq!(game.actions_left(), 2);
}

#[test]
fn during_an_attack_only_choices_are_allowed() {
    let mut game = with_two_enemies_next_to_e5();
    play(&mut game, &[Action::UseAttack(RARE)]);
    for action in [place(0, 0), Action::EndTurn, Action::UseAttack(RARE)] {
        assert_eq!(game.apply(action), Err(GameError::UnexpectedAction));
    }
    // Et hors d'une attaque, choisir une case n'a pas de sens.
    let mut fresh = with_two_enemies_next_to_e5();
    assert_eq!(fresh.apply(choose(4, 4)), Err(GameError::UnexpectedAction));
}

#[test]
fn the_launcher_must_be_one_of_your_pions_with_a_target() {
    let mut game = with_two_enemies_next_to_e5();
    play(&mut game, &[place(0, 0)]); // un pion du joueur 1 sans ennemi autour
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    play(&mut game, &[Action::UseAttack(RARE)]);

    for (row, col) in [(4, 5), (9, 9), (0, 0)] {
        assert_eq!(
            game.apply(choose(row, col)),
            Err(GameError::InvalidChoice(Position { row, col })),
            "lanceur ({row}, {col})"
        );
    }
    assert!(game.apply(choose(4, 4)).is_ok());
}

#[test]
fn targets_must_be_enemies_on_the_eight_surrounding_cells() {
    let mut game = with_two_enemies_next_to_e5();
    play(&mut game, &[Action::UseAttack(RARE), choose(4, 4)]);

    // F6 est en diagonale : visable. E5 est le lanceur, J10 est vide et loin.
    for (row, col) in [(4, 4), (9, 9), (3, 3)] {
        assert_eq!(
            game.apply(choose(row, col)),
            Err(GameError::InvalidChoice(Position { row, col }))
        );
    }
    assert!(game.apply(choose(5, 5)).is_ok());
}

#[test]
fn ours_epic_kills_two_adjacent_enemies() {
    let mut game = with_four_actions();
    play(
        &mut game,
        &[Action::UseAttack(EPIC), choose(4, 4), choose(4, 5)],
    );
    // On ne peut pas viser deux fois le même pion.
    assert_eq!(
        game.apply(choose(4, 5)),
        Err(GameError::InvalidChoice(Position { row: 4, col: 5 }))
    );
    let events = play(&mut game, &[choose(5, 5)]);

    let kills = events
        .iter()
        .filter(|event| matches!(event, Event::PionKilled { .. }))
        .count();
    assert_eq!(kills, 2);
    assert_eq!(game.reserve_pions(Player::Two), 15);
    assert_eq!(game.actions_left(), 1); // 4 - 3
}

#[test]
fn ours_epic_is_not_offered_with_a_single_adjacent_enemy() {
    let mut game = duel();
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), place(9, 9)]); // un seul ennemi à côté de E5
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // le joueur 1 a 4 actions
    assert_eq!(game.actions_left(), 4);
    // L'Épique demande deux cibles : elle n'a aucune attaque complète possible.
    assert_eq!(
        game.apply(Action::UseAttack(EPIC)),
        Err(GameError::NoTarget(ours(Rarity::Epic)))
    );
    assert!(!game.legal_actions().contains(&Action::UseAttack(EPIC)));
}

#[test]
fn an_attack_needs_enough_actions() {
    let mut game = duel();
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), place(5, 5)]);
    // Le joueur 1 a 2 actions ; l'Épique en coûte 3.
    assert_eq!(
        game.apply(Action::UseAttack(EPIC)),
        Err(GameError::NotEnoughActions {
            needed: 3,
            available: 2
        })
    );
}

#[test]
fn a_used_attack_recharges_until_your_next_turn() {
    let mut game = with_four_actions();
    play(
        &mut game,
        &[Action::UseAttack(RARE), choose(4, 4), choose(4, 5)],
    );
    assert_eq!(game.arsenal(Player::One)[RARE].cooldown_left(), 1);
    assert_eq!(
        game.apply(Action::UseAttack(RARE)),
        Err(GameError::AttackRecharging(ours(Rarity::Rare)))
    );

    play(&mut game, &[Action::EndTurn]);
    // Pendant le tour adverse, la recharge du joueur 1 ne bouge pas.
    assert_eq!(game.current_player(), Player::Two);
    assert_eq!(game.arsenal(Player::One)[RARE].cooldown_left(), 1);

    let events = play(&mut game, &[Action::EndTurn]); // retour au joueur 1
    assert!(events.contains(&Event::CooldownChanged {
        player: Player::One,
        slot: RARE,
        turns_left: 0,
    }));
    assert!(game.legal_actions().contains(&Action::UseAttack(RARE)));
}

#[test]
fn a_two_turn_cooldown_skips_one_of_your_turns() {
    let mut config = config_with_attacks(vec![ours(Rarity::Rare)], Vec::new());
    config.attack_catalog[0].stats.cooldown = 2;
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), place(5, 5)]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );

    // Le tour suivant du joueur 1 : encore 1 tour de recharge.
    play(&mut game, &[Action::EndTurn]);
    assert_eq!(game.current_player(), Player::One);
    assert_eq!(game.arsenal(Player::One)[0].cooldown_left(), 1);
    assert!(!game.legal_actions().contains(&Action::UseAttack(0)));

    // Le surlendemain : prête.
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    assert_eq!(game.arsenal(Player::One)[0].cooldown_left(), 0);
    assert!(game.legal_actions().contains(&Action::UseAttack(0)));
}

#[test]
fn an_unknown_attack_slot_is_refused_and_displayable() {
    let mut game = with_two_enemies_next_to_e5();
    assert_eq!(
        game.apply(Action::UseAttack(2)),
        Err(GameError::NoSuchAttack(2))
    );
    // Un emplacement absurde, venu d'un JSON modifié : refusé, et affichable sans planter.
    let error = game.apply(Action::UseAttack(usize::MAX)).unwrap_err();
    assert_eq!(error, GameError::NoSuchAttack(usize::MAX));
    assert!(error.to_string().contains("aucune attaque"));
}

#[test]
fn a_killed_pion_can_be_placed_again() {
    let mut game = with_two_enemies_next_to_e5();
    play(
        &mut game,
        &[Action::UseAttack(RARE), choose(4, 4), choose(4, 5)],
    );
    // C'est au joueur 2, dont le pion tué est revenu en réserve.
    assert_eq!(game.reserve_pions(Player::Two), 14);
    assert!(game.apply(place(4, 5)).is_ok());
}

#[test]
fn empty_reserves_do_not_freeze_a_game_where_an_attack_is_possible() {
    let config = GameConfig {
        pions_per_player: 1,
        ..config_with_attacks(vec![ours(Rarity::Rare)], vec![])
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(0, 1)]);
    // Les deux réserves sont vides, mais le joueur 1 peut encore tuer : la partie continue.
    assert_eq!(game.end(), None);
}
