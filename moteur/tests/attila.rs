//! Attila Rare et Épique (B6a, règles précisées le 2026-11-02 dans le carnet) : annuler des
//! effets adverses entiers, c'est-à-dire tout ce qu'un jeu de carte a laissé en jeu.
//!
//! Configuration : les premières règles, avec 4 actions à chaque tour et aucune action
//! gardée ; une pose coûte 1 action.

mod common;

use common::{choose, config_with_attacks, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, EffectKind, Game, GameConfig, GameError, Player, Position, Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn attila(rarity: Rarity) -> Attack {
    card(AttackKind::Attila, rarity)
}

/// Chaque joueur commence avec ces attaques, et a 4 actions à chaque tour.
fn config(player_one: Vec<Attack>, player_two: Vec<Attack>) -> GameConfig {
    GameConfig {
        first_turn_actions: 4,
        actions_per_turn: 4,
        max_banked_actions: 0,
        ..config_with_attacks(player_one, player_two)
    }
}

/// Les cases proposées pour le prochain choix de l'attaque en cours.
fn choices(game: &Game) -> Vec<Position> {
    game.legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::Choose(pos) => Some(pos),
            _ => None,
        })
        .collect()
}

fn immobilized(game: &Game, pos: Position) -> bool {
    game.has_effect(pos, EffectKind::Immobilized)
}

/// Au tour 5, J1 (Attila Rare et Épique) a trois pions immobilisés par deux jeux de carte
/// de J2 : C5 et G5 par une Beauté fatale Rare (lancée d'E5), I9 par un Grognement (lancé
/// de J10). A1, libre, sert de lanceur.
fn two_enemy_plays() -> Game {
    let mut game = game_with(config(
        vec![attila(Rarity::Rare), attila(Rarity::Epic)],
        vec![
            card(AttackKind::BeauteFatale, Rarity::Rare),
            card(AttackKind::Ours, Rarity::Common),
        ],
    ));
    play(
        &mut game,
        &[place(4, 2), place(4, 6), place(8, 8), place(0, 0)],
    ); // tour 1
    play(&mut game, &[place(4, 4), place(9, 9), Action::EndTurn]); // tour 2
    play(&mut game, &[Action::EndTurn]); // tour 3
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 2),
            choose(4, 6),
            Action::UseAttack(1),
            choose(9, 9),
            Action::EndTurn,
        ],
    ); // tour 4
    for pos in [at(4, 2), at(4, 6), at(8, 8)] {
        assert!(immobilized(&game, pos), "{pos}");
    }
    game
}

#[test]
fn the_rare_cancels_a_whole_effect_on_every_pion_it_touches() {
    let mut game = two_enemy_plays();
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 0), choose(4, 2)],
    ); // tour 5
    // La Beauté fatale libère C5 et G5 ; le Grognement reste.
    assert!(!immobilized(&game, at(4, 2)));
    assert!(!immobilized(&game, at(4, 6)));
    assert!(immobilized(&game, at(8, 8)));
}

#[test]
fn the_epic_cancels_two_whole_effects_and_never_the_same_twice() {
    let mut game = two_enemy_plays();
    play(
        &mut game,
        &[Action::UseAttack(1), choose(0, 0), choose(4, 2)],
    ); // tour 5
    // G5 ne porte que la Beauté fatale, déjà annulée : seul I9 reste à viser.
    assert_eq!(choices(&game), [at(8, 8)]);
    play(&mut game, &[choose(8, 8)]);
    for pos in [at(4, 2), at(4, 6), at(8, 8)] {
        assert!(!immobilized(&game, pos), "{pos}");
    }
}

#[test]
fn the_epic_can_stop_after_one_pion() {
    let mut game = two_enemy_plays();
    play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(0, 0),
            choose(4, 2),
            Action::FinishAttack,
        ],
    ); // tour 5
    assert!(!immobilized(&game, at(4, 2)));
    assert!(!immobilized(&game, at(4, 6)));
    assert!(immobilized(&game, at(8, 8)));
}

#[test]
fn the_epic_cancels_both_halves_of_a_pact_even_with_a_single_enemy_play() {
    let friends = card(AttackKind::MeilleursAmis, Rarity::Epic);
    let mut game = game_with(config(
        vec![attila(Rarity::Rare), attila(Rarity::Epic)],
        vec![friends],
    ));
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            choose(9, 0),
            choose(0, 9),
        ],
    ); // tour 2 : A10 et J1 liés
    // Un seul jeu de carte adverse : l'Épique se joue quand même, dès son premier pion.
    play(
        &mut game,
        &[Action::UseAttack(1), choose(0, 0), choose(9, 0)],
    ); // tour 3
    assert!(!game.legal_actions().contains(&Action::FinishAttack));
    assert_eq!(game.pact_friend(at(9, 0)), None);
    assert_eq!(game.pact_friend(at(0, 9)), None);
}

#[test]
fn the_rare_cancels_the_guards_of_a_queen_but_not_her_crown() {
    let queen = card(AttackKind::Reine, Rarity::Legendary);
    let mut game = game_with(config(vec![attila(Rarity::Rare)], vec![queen]));
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 1
    play(&mut game, &[place(9, 9), Action::EndTurn]); // tour 2
    play(&mut game, &[Action::EndTurn]); // tour 3
    play(
        &mut game,
        &[Action::UseAttack(0), choose(9, 9), choose(0, 9)],
    ); // tour 4 : la reine en J1, 2 gardes
    let queen_id = game.board().get(at(0, 9)).unwrap().pion().unwrap().id();
    assert_eq!(game.guards_of(queen_id), 2);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 0), choose(0, 9)],
    ); // tour 5
    assert_eq!(game.guards_of(queen_id), 0);
    assert_eq!(game.queen_of(Player::Two), Some(at(0, 9)));
}

#[test]
fn the_rare_cancels_both_children_of_a_legendary_familia() {
    let familia = card(AttackKind::Familia, Rarity::Legendary);
    let mut game = game_with(config(vec![attila(Rarity::Rare)], vec![familia]));
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 1
    play(&mut game, &[place(9, 4), place(9, 5), Action::EndTurn]); // tour 2
    play(&mut game, &[Action::EndTurn]); // tour 3
    play(
        &mut game,
        &[Action::UseAttack(0), choose(9, 4), choose(9, 5)],
    ); // tour 4 : E10 et F10 enfants
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 0), choose(9, 4)],
    ); // tour 5
    assert!(!game.has_effect(at(9, 4), EffectKind::Protected));
    assert!(!game.has_effect(at(9, 5), EffectKind::Protected));
}

#[test]
fn attila_never_cancels_your_own_effects() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let mut game = game_with(config(vec![royal_guard, attila(Rarity::Rare)], Vec::new()));
    // Sans aucun effet en jeu, la Rare n'a pas de cible.
    play(&mut game, &[place(0, 0)]);
    assert_eq!(
        game.apply(Action::UseAttack(1)),
        Err(GameError::NoTarget(attila(Rarity::Rare)))
    );
    // A1 protégé par son propre Garde royal : toujours rien à annuler.
    play(&mut game, &[Action::UseAttack(0), choose(0, 0)]);
    assert!(game.has_effect(at(0, 0), EffectKind::Protected));
    assert_eq!(
        game.apply(Action::UseAttack(1)),
        Err(GameError::NoTarget(attila(Rarity::Rare)))
    );
}

#[test]
fn every_effect_of_one_card_play_shares_its_play_and_two_plays_differ() {
    let game = two_enemy_plays();
    let play_of = |pos: Position| {
        let pion = game.board().get(pos).unwrap().pion().unwrap().id();
        game.active_effects()
            .iter()
            .find(|effect| effect.pion() == pion)
            .unwrap()
            .play()
    };
    assert_eq!(play_of(at(4, 2)), play_of(at(4, 6)));
    assert_ne!(play_of(at(4, 2)), play_of(at(8, 8)));
}

#[test]
fn every_enemy_play_on_the_targeted_pion_is_cancelled_but_not_your_own() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let mut game = game_with(config(
        vec![attila(Rarity::Rare), royal_guard],
        vec![
            card(AttackKind::BeauteFatale, Rarity::Rare),
            card(AttackKind::Ours, Rarity::Common),
        ],
    ));
    play(
        &mut game,
        &[place(4, 2), place(4, 6), place(0, 0), Action::EndTurn],
    ); // tour 1 : C5, G5, A1
    play(&mut game, &[place(4, 4), place(5, 7), Action::EndTurn]); // tour 2 : E5, H6
    play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 6), Action::EndTurn],
    ); // tour 3 : G5 protégé par son propre Garde royal
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 2),
            choose(4, 6),
            Action::UseAttack(1),
            choose(5, 7),
            Action::EndTurn,
        ],
    ); // tour 4 : C5 et G5 charmés, puis G5 grogné
    let pion = game.board().get(at(4, 6)).unwrap().pion().unwrap().id();
    let immobilizations = || {
        game.active_effects()
            .iter()
            .filter(|effect| effect.pion() == pion && effect.kind() == EffectKind::Immobilized)
            .count()
    };
    assert_eq!(immobilizations(), 2);

    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 0), choose(4, 6)],
    ); // tour 5
    // Les deux jeux adverses partent en entier (C5 compris) ; le Garde royal reste.
    assert!(!immobilized(&game, at(4, 6)));
    assert!(!immobilized(&game, at(4, 2)));
    assert!(game.has_effect(at(4, 6), EffectKind::Protected));
}
