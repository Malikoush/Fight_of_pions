//! Statuts et durées (carnet de règles, « Règles communes des attaques » du 2026-10-30) :
//! immobilisé, protégé, la fin d'une durée, et les cartes de B3 qui les donnent ou les
//! retirent (Ours Commune et Légendaire, Beauté fatale, Garde royal, Attila Commune).
//!
//! Configuration par défaut : J1 a 1 action à son premier tour, puis 2 par tour ; chacun
//! garde jusqu'à 2 actions.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
use fight_of_pions::{
    Action, ActiveEffect, Attack, AttackKind, EffectKind, Event, Game, GameError, Player, Position,
    Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn charm() -> Attack {
    card(AttackKind::BeauteFatale, Rarity::Common)
}

fn royal_guard() -> Attack {
    card(AttackKind::Reine, Rarity::Common)
}

fn attila() -> Attack {
    card(AttackKind::Attila, Rarity::Common)
}

fn started(events: &[Event]) -> Vec<ActiveEffect> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::EffectStarted(effect) => Some(*effect),
            _ => None,
        })
        .collect()
}

fn ended(events: &[Event]) -> Vec<ActiveEffect> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::EffectEnded(effect) => Some(*effect),
            _ => None,
        })
        .collect()
}

fn count_kills(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, Event::PionKilled { .. }))
        .count()
}

fn effects_on(game: &Game, pos: Position, kind: EffectKind) -> usize {
    let pion = game.board().get(pos).and_then(|cell| cell.pion()).unwrap();
    game.active_effects()
        .iter()
        .filter(|effect| effect.pion() == pion.id() && effect.kind() == kind)
        .count()
}

/// J1 en E5 ; J2 en G5, à 2 cases dans le regard de E5 (F5 libre), et en J10. C'est à J1.
fn charm_duel(player_two: Vec<Attack>) -> Game {
    let mut game = game_with(config_with_attacks(vec![charm()], player_two));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 6), place(9, 9)]);
    game
}

#[test]
fn the_growl_immobilizes_adjacent_enemies_for_one_turn_of_the_victim() {
    let config = config_with_attacks(
        vec![ours(Rarity::Common), ours(Rarity::Rare)],
        vec![ours(Rarity::Rare)],
    );
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(4, 5), place(9, 9)]); // tour 2, J2 : F5 collé, J10 loin

    // Tour 3 : le Grognement n'a pas de cible à choisir, il est joué avec son lanceur.
    let events = play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    let growl = started(&events);
    assert_eq!(growl.len(), 1, "seul F5 est collé à E5 : {events:?}");
    assert_eq!(growl[0].kind(), EffectKind::Immobilized);
    assert_eq!(growl[0].creator(), Player::One);
    assert_eq!(growl[0].source(), ours(Rarity::Common));
    // 1 tour de la victime, le tour en cours ne compte pas : le tour 4.
    assert_eq!(growl[0].last_turn(), Some(4));
    play(&mut game, &[Action::EndTurn]);

    // Tour 4 : F5, immobilisé, ne peut pas lancer ; J10 n'a pas de cible.
    assert!(game.has_effect(at(4, 5), EffectKind::Immobilized));
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoReadyLauncher(ours(Rarity::Rare)))
    );
    // L'effet s'éteint à la fin de ce tour.
    let events = play(&mut game, &[Action::EndTurn]);
    assert_eq!(ended(&events), growl);
    assert!(game.active_effects().is_empty());

    // Tour 6 : F5 peut de nouveau attaquer.
    play(&mut game, &[Action::EndTurn]);
    assert!(game.legal_actions().contains(&Action::UseAttack(0)));
}

#[test]
fn the_growl_is_not_offered_without_an_adjacent_enemy() {
    let mut game = game_with(config_with_attacks(vec![ours(Rarity::Common)], Vec::new()));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(9, 9), place(0, 9)]);
    // Il n'immobiliserait personne : aucun coup sans effet n'est proposé.
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(ours(Rarity::Common)))
    );
}

#[test]
fn the_charm_lasts_until_the_end_of_the_second_turn_of_the_victim() {
    let mut game = charm_duel(Vec::new());
    // Tour 3 : la Beauté fatale vise G5, à 2 cases.
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    let charmed = started(&events);
    assert_eq!(charmed.len(), 1);
    // Les tours 4 et 6 sont ceux de J2 : l'effet s'éteint à la fin du tour 6.
    assert_eq!(charmed[0].last_turn(), Some(6));

    for turn in 3..6 {
        assert_eq!(game.turn(), turn);
        let events = play(&mut game, &[Action::EndTurn]);
        assert!(ended(&events).is_empty(), "fini trop tôt, au tour {turn}");
        assert!(game.has_effect(at(4, 6), EffectKind::Immobilized));
    }
    let events = play(&mut game, &[Action::EndTurn]); // fin du tour 6
    assert_eq!(ended(&events), charmed);
    assert!(!game.has_effect(at(4, 6), EffectKind::Immobilized));
}

#[test]
fn a_second_charm_on_an_immobilized_pion_kills_it() {
    let mut game = charm_duel(Vec::new());
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tour 5, la carte est rechargée
    let victim = game
        .board()
        .get(at(4, 6))
        .and_then(|cell| cell.pion())
        .unwrap();

    // Le charme fatal : G5 est déjà immobilisé, il est tué.
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    assert_eq!(game.board().owner(at(4, 6)), None);
    // Le pion tué, puis la fin de ses effets, puis son retour dans la réserve.
    let killed = events
        .iter()
        .position(|event| matches!(event, Event::PionKilled { pion, .. } if *pion == victim));
    let effect_ended = events.iter().position(
        |event| matches!(event, Event::EffectEnded(effect) if effect.pion() == victim.id()),
    );
    let back = events.iter().position(|event| {
        matches!(
            event,
            Event::ReserveChanged {
                player: Player::Two,
                ..
            }
        )
    });
    assert!(killed < effect_ended && effect_ended < back, "{events:?}");
    assert!(killed.is_some());
    assert!(game.active_effects().is_empty());
}

#[test]
fn the_charm_sees_two_cells_in_a_straight_line_without_a_pion_between() {
    let mut game = game_with(config_with_attacks(vec![charm()], Vec::new()));
    play(&mut game, &[place(4, 4)]); // E5
    play(&mut game, &[place(4, 5), place(4, 6)]); // F5 cache G5
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert_eq!(
        game.apply(choose(4, 6)),
        Err(GameError::InvalidChoice(at(4, 6)))
    );
    assert!(game.apply(choose(4, 5)).is_ok());
}

#[test]
fn the_rare_charm_immobilizes_two_different_pions() {
    let rare = card(AttackKind::BeauteFatale, Rarity::Rare);
    let mut game = game_with(config_with_attacks(vec![rare], Vec::new()));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 6), place(2, 2)]); // G5 et C3, tous deux à 2 cases
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    // Il en faut deux : on ne peut ni s'arrêter, ni reprendre le même.
    assert!(game.attack_in_progress());
    assert_eq!(
        game.apply(Action::FinishAttack),
        Err(GameError::TargetsMissing(rare))
    );
    assert_eq!(
        game.apply(choose(4, 6)),
        Err(GameError::InvalidChoice(at(4, 6)))
    );
    let events = play(&mut game, &[choose(2, 2)]);
    assert_eq!(started(&events).len(), 2);
}

#[test]
fn a_protected_pion_cannot_be_chosen_to_be_killed_but_can_be_immobilized() {
    let config = config_with_attacks(
        vec![ours(Rarity::Common), ours(Rarity::Rare)],
        vec![royal_guard()],
    );
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    // Tour 2, J2 : F5, puis le Garde royal depuis F5, joué dès le choix du lanceur.
    play(&mut game, &[place(4, 5), Action::UseAttack(0)]);
    let events = play(&mut game, &[choose(4, 5)]);
    let guard = started(&events);
    assert_eq!(guard.len(), 1);
    assert_eq!(guard[0].kind(), EffectKind::Protected);
    assert_eq!(guard[0].creator(), Player::Two);
    assert_eq!(guard[0].source(), royal_guard());
    // 2 tours de l'adversaire (J1) : les tours 3 et 5.
    assert_eq!(guard[0].last_turn(), Some(5));

    // Tour 3 : l'Ours Rare n'a personne à tuer ; le Grognement immobilise quand même F5.
    assert_eq!(
        game.apply(Action::UseAttack(1)),
        Err(GameError::NoTarget(ours(Rarity::Rare)))
    );
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert!(game.has_effect(at(4, 5), EffectKind::Immobilized));
    assert!(game.has_effect(at(4, 5), EffectKind::Protected));
}

#[test]
fn a_charm_on_a_protected_immobilized_pion_only_immobilizes_it_again() {
    let mut game = game_with(config_with_attacks(vec![charm()], vec![royal_guard()]));
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(
        &mut game,
        &[place(4, 6), Action::UseAttack(0), choose(4, 6)],
    ); // G5 protégé
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    ); // tour 3
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tour 5 : G5 toujours protégé
    assert!(game.has_effect(at(4, 6), EffectKind::Protected));

    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 6)],
    );
    assert_eq!(count_kills(&events), 0);
    assert_eq!(game.board().owner(at(4, 6)), Some(Player::Two));
    assert_eq!(effects_on(&game, at(4, 6), EffectKind::Immobilized), 2);
}

#[test]
fn the_roar_kills_up_to_two_and_immobilizes_the_other_adjacent_enemies() {
    let mut game = game_with(config_with_attacks(
        vec![ours(Rarity::Legendary)],
        Vec::new(),
    ));
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(3, 3), place(4, 5)]); // tour 2, J2 : D4, F5
    play(&mut game, &[Action::EndTurn]); // tour 3 : J1 garde ses 2 actions
    play(&mut game, &[place(5, 5), place(4, 3)]); // tour 4, J2 : F6, D5
    assert_eq!(game.actions_left(), 4);

    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    // Tuer « jusqu'à 2 » : on pourrait déjà jouer l'attaque, sans tuer personne.
    assert!(game.legal_actions().contains(&Action::FinishAttack));
    play(&mut game, &[choose(3, 3)]);
    let events = play(&mut game, &[choose(4, 5)]); // le 2e kill joue l'attaque

    assert_eq!(count_kills(&events), 2);
    let roared = started(&events);
    let mut immobilized: Vec<PionAt> = roared.iter().map(|e| PionAt::of(&game, e)).collect();
    immobilized.sort();
    assert_eq!(immobilized, vec![PionAt(4, 3), PionAt(5, 5)]);
    assert!(roared.iter().all(|effect| effect.last_turn() == Some(8)));
}

/// La case (rangée, colonne) du pion visé par un effet.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PionAt(usize, usize);

impl PionAt {
    fn of(game: &Game, effect: &ActiveEffect) -> PionAt {
        let pos = game.board().position_of(effect.pion()).unwrap();
        PionAt(pos.row, pos.col)
    }
}

#[test]
fn attila_frees_your_pion_from_an_enemy_immobilization() {
    let config = config_with_attacks(vec![attila()], vec![charm()]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    // Tour 2, J2 : G5, puis la Beauté fatale depuis G5 sur E5.
    play(
        &mut game,
        &[
            place(4, 6),
            Action::UseAttack(0),
            choose(4, 6),
            choose(4, 4),
        ],
    );
    assert!(game.has_effect(at(4, 4), EffectKind::Immobilized));

    // Tour 3 : E5, immobilisé, ne peut pas lancer ; un nouveau pion le peut.
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoReadyLauncher(attila()))
    );
    play(
        &mut game,
        &[place(0, 0), Action::UseAttack(0), choose(0, 0)],
    );
    // Seul E5 porte un effet de l'adversaire : G5 n'en a aucun.
    assert_eq!(
        game.apply(choose(4, 6)),
        Err(GameError::InvalidChoice(at(4, 6)))
    );
    let events = play(&mut game, &[choose(4, 4)]);
    assert_eq!(ended(&events).len(), 1);
    assert!(!game.has_effect(at(4, 4), EffectKind::Immobilized));
}

#[test]
fn attila_removes_an_enemy_protection_so_the_bear_can_kill() {
    let config = config_with_attacks(vec![attila(), ours(Rarity::Rare)], vec![royal_guard()]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(
        &mut game,
        &[place(4, 5), Action::UseAttack(0), choose(4, 5)],
    ); // F5 protégé

    // Tour 3 : Attila retire la protection de F5.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );
    assert!(!game.has_effect(at(4, 5), EffectKind::Protected));
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tour 5 : 3 actions

    play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 4), choose(4, 5)],
    );
    assert_eq!(game.board().owner(at(4, 5)), None);
}

#[test]
fn attila_keeps_your_own_effects_on_the_pion_it_frees() {
    let config = config_with_attacks(vec![royal_guard(), attila()], vec![charm()]);
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]); // tour 1, J1 : E5
    play(&mut game, &[place(4, 6), place(9, 9)]); // tour 2, J2 : G5, J10
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]); // tour 3 : E5 se protège
    play(&mut game, &[Action::EndTurn]);
    // Tour 4 : J2 immobilise E5 depuis G5.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 6), choose(4, 4)],
    );
    play(&mut game, &[Action::EndTurn]);

    // Tour 5 : E5 est protégé (par J1) et immobilisé (par J2). Attila, depuis A1, ne
    // retire que l'effet de l'adversaire.
    play(
        &mut game,
        &[
            place(0, 0),
            Action::UseAttack(1),
            choose(0, 0),
            choose(4, 4),
        ],
    );
    assert!(!game.has_effect(at(4, 4), EffectKind::Immobilized));
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
}

#[test]
fn attila_cannot_target_a_pion_that_only_has_your_own_effects() {
    let mut game = game_with(config_with_attacks(
        vec![royal_guard(), attila()],
        Vec::new(),
    ));
    play(&mut game, &[place(4, 4)]);
    play(&mut game, &[place(4, 5), place(9, 9)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]); // E5 se protège
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
    assert_eq!(
        game.apply(Action::UseAttack(1)),
        Err(GameError::NoTarget(attila()))
    );
}
