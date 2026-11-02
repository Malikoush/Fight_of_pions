//! Terre brûlée, Attila Légendaire (B6d, règles précisées le 2026-11-02 dans le carnet) :
//! annuler comme l'Épique, puis interdire à l'adversaire, pendant son prochain tour, tout
//! coup qui créerait un effet continu.
//!
//! Configuration : les premières règles (sans infirmerie, une pose coûte 1 action), avec
//! 8 actions à chaque tour et aucune action gardée.

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

fn scorched_earth() -> Attack {
    card(AttackKind::Attila, Rarity::Legendary)
}

/// Chaque joueur commence avec ces attaques, et a 8 actions à chaque tour.
fn config(player_one: Vec<Attack>, player_two: Vec<Attack>) -> GameConfig {
    GameConfig {
        first_turn_actions: 8,
        actions_per_turn: 8,
        max_banked_actions: 0,
        ..config_with_attacks(player_one, player_two)
    }
}

/// Au tour 1, J1 pose F6 et G5, puis joue Terre brûlée depuis F6 : J2 est sous Terre brûlée
/// pendant le tour 2.
fn scorched(player_two: Vec<Attack>) -> Game {
    let mut game = game_with(config(vec![scorched_earth()], player_two));
    play(
        &mut game,
        &[place(5, 5), place(4, 6), Action::UseAttack(0), choose(5, 5)],
    );
    play(&mut game, &[Action::EndTurn]); // tour 1
    game
}

#[test]
fn with_nothing_to_cancel_it_is_played_on_its_launcher_alone() {
    let mut game = game_with(config(vec![scorched_earth()], Vec::new()));
    let events = play(
        &mut game,
        &[place(0, 0), Action::UseAttack(0), choose(0, 0)],
    );
    assert!(!game.attack_in_progress());
    assert!(events.contains(&Event::ScorchedEarthStarted {
        player: Player::Two,
        last_turn: 2,
    }));
    assert_eq!(game.scorched_until(Player::Two), Some(2));
    assert_eq!(game.scorched_until(Player::One), None);
}

#[test]
fn it_cancels_like_the_epic_then_scorches() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let mut game = game_with(config(vec![scorched_earth()], vec![royal_guard]));
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            Action::EndTurn,
        ],
    ); // tour 2 : J10 protégé
    play(&mut game, &[Action::UseAttack(0), choose(0, 0)]);
    // Comme l'Épique : il y a quelque chose à annuler, donc au moins une cible.
    assert!(!game.legal_actions().contains(&Action::FinishAttack));
    play(&mut game, &[choose(9, 9)]); // tour 3
    assert!(!game.has_effect(at(9, 9), EffectKind::Protected));
    assert_eq!(game.scorched_until(Player::Two), Some(4));
}

#[test]
fn the_enemy_cannot_play_a_card_that_would_create_a_lasting_effect() {
    let roar = card(AttackKind::Ours, Rarity::Legendary);
    let mut game = scorched(vec![
        card(AttackKind::Reine, Rarity::Common),
        card(AttackKind::MassacreDivin, Rarity::Common),
        card(AttackKind::Interchange, Rarity::Common),
        roar,
    ]);
    play(&mut game, &[place(5, 6)]); // tour 2 : G6, collé à F6 et G5
    for (slot, attack) in [
        (0, card(AttackKind::Reine, Rarity::Common)),
        (1, card(AttackKind::MassacreDivin, Rarity::Common)),
        (2, card(AttackKind::Interchange, Rarity::Common)),
    ] {
        assert_eq!(
            game.apply(Action::UseAttack(slot)),
            Err(GameError::ScorchedEarth(attack))
        );
    }
    // Le Rugissement ne se joue que s'il tue tous les pions collés : sinon il les
    // immobiliserait.
    play(
        &mut game,
        &[Action::UseAttack(3), choose(5, 6), choose(5, 5)],
    );
    assert_eq!(
        game.apply(Action::FinishAttack),
        Err(GameError::TargetsMissing(roar))
    );
    play(&mut game, &[choose(4, 6)]);
    assert_eq!(game.board().owner(at(5, 5)), None);
    assert_eq!(game.board().owner(at(4, 6)), None);
    assert!(game.active_effects().is_empty());
}

#[test]
fn the_ban_ends_at_the_start_of_your_next_turn() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let mut game = scorched(vec![royal_guard]);
    play(&mut game, &[place(9, 9)]);
    let events = play(&mut game, &[Action::EndTurn]); // tour 2
    assert!(events.contains(&Event::ScorchedEarthEnded {
        player: Player::Two
    }));
    assert_eq!(game.scorched_until(Player::Two), None);
    play(&mut game, &[Action::EndTurn]); // tour 3
    play(&mut game, &[Action::UseAttack(0), choose(9, 9)]); // tour 4 : permis
    assert!(game.has_effect(at(9, 9), EffectKind::Protected));
}

#[test]
fn attila_cannot_cancel_the_ban_but_may_scorch_back() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let mut game = game_with(config(
        vec![royal_guard, scorched_earth()],
        vec![card(AttackKind::Attila, Rarity::Rare), scorched_earth()],
    ));
    play(
        &mut game,
        &[place(0, 0), Action::UseAttack(0), choose(0, 0)],
    );
    play(
        &mut game,
        &[Action::UseAttack(1), choose(0, 0), Action::EndTurn],
    ); // tour 1
    // Tour 2 : Attila Rare ne crée rien, il est permis ; il annule le Garde royal, pas
    // Terre brûlée.
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            choose(0, 0),
        ],
    );
    assert!(!game.has_effect(at(0, 0), EffectKind::Protected));
    assert_eq!(game.scorched_until(Player::Two), Some(2));
    // Terre brûlée n'est pas un effet continu : elle reste permise, et répond.
    play(&mut game, &[Action::UseAttack(1), choose(9, 9)]);
    assert_eq!(game.scorched_until(Player::One), Some(3));
    assert_eq!(game.scorched_until(Player::Two), Some(2));
}

#[test]
fn it_can_cancel_the_enemy_tokens() {
    let massacre = card(AttackKind::MassacreDivin, Rarity::Common);
    let mut game = game_with(config(vec![scorched_earth()], vec![massacre]));
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            Action::EndTurn,
        ],
    ); // tour 2
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 0),
            Action::ChooseEnemyTokens,
        ],
    ); // tour 3
    assert!(game.token_piles().is_empty());
    assert_eq!(game.scorched_until(Player::Two), Some(4));
}

#[test]
fn gestures_shots_returns_and_effects_in_play_stay_allowed() {
    let config = GameConfig {
        first_turn_actions: 12,
        actions_per_turn: 12,
        ..config(
            vec![scorched_earth()],
            vec![
                card(AttackKind::Ours, Rarity::Common),
                card(AttackKind::Reine, Rarity::Common),
                card(AttackKind::Reine, Rarity::Epic),
                card(AttackKind::Interchange, Rarity::Common),
                card(AttackKind::Lievre, Rarity::Epic),
            ],
        )
    };
    let mut game = game_with(config);
    play(&mut game, &[place(5, 5), place(0, 0), Action::EndTurn]); // tour 1
    // Tour 2 : deux leurres (F6 grogné, J10 protégé), une reine gardée en C3, un pion
    // téléporté de I9 en A10, des jetons de pas sur G6.
    play(&mut game, &[place(5, 6), place(9, 9), place(8, 8)]);
    play(&mut game, &[Action::UseAttack(0), choose(5, 6)]);
    play(&mut game, &[Action::UseAttack(1), choose(9, 9)]);
    play(
        &mut game,
        &[Action::UseAttack(2), choose(9, 9), choose(2, 2)],
    );
    play(
        &mut game,
        &[Action::UseAttack(3), choose(8, 8), choose(9, 0)],
    );
    play(
        &mut game,
        &[Action::UseAttack(4), choose(5, 6), Action::EndTurn],
    );
    // Tour 3 : Terre brûlée annule les deux leurres.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 0),
            choose(9, 9),
            choose(5, 5),
            Action::EndTurn,
        ],
    );
    assert_eq!(game.scorched_until(Player::Two), Some(4));
    // Tour 4, sous Terre brûlée : le retour s'est joué, et rien de ce qui reste en jeu n'est
    // parti.
    assert_eq!(game.board().owner(at(8, 8)), Some(Player::Two));
    assert_eq!(game.board().owner(at(9, 0)), None);
    assert!(game.has_effect(at(5, 6), EffectKind::StepTokens));
    // Le tir d'un garde, le pas de la reine et un pas du Lièvre restent permis.
    play(&mut game, &[Action::Shoot(at(0, 0))]);
    assert_eq!(game.board().owner(at(0, 0)), None);
    play(&mut game, &[Action::StepQueen(at(3, 3))]);
    play(
        &mut game,
        &[Action::HareStep {
            from: at(5, 6),
            to: at(6, 7),
        }],
    );
    assert_eq!(game.board().owner(at(6, 7)), Some(Player::Two));
}
