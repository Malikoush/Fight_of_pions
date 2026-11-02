//! B6c (règles précisées le 2026-11-02 dans le carnet) : l'Interchange Commune
//! (téléportation et retour programmé), l'Interchange Légendaire (double échange) et le
//! Lièvre Épique (jetons de pas).
//!
//! Configuration : les premières règles (sans infirmerie ni règle d'or, une pose coûte
//! 1 action), avec 8 actions à chaque tour et aucune action gardée, sauf mention contraire.

mod common;

use common::{choose, config_with_attacks, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, EffectKind, EndReason, Event, Game, GameConfig, GameEnd, GameError,
    Outcome, Player, Position, Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn interchange(rarity: Rarity) -> Attack {
    card(AttackKind::Interchange, rarity)
}

fn hare_epic() -> Attack {
    card(AttackKind::Lievre, Rarity::Epic)
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

fn step(from: Position, to: Position) -> Action {
    Action::HareStep { from, to }
}

/// Les pas du Lièvre permis maintenant.
fn hare_steps(game: &Game) -> Vec<Action> {
    game.legal_actions()
        .into_iter()
        .filter(|action| matches!(action, Action::HareStep { .. }))
        .collect()
}

/// Les jetons de pas qui restent au pion de cette case.
fn step_tokens(game: &Game, pos: Position) -> Option<u32> {
    let pion = game.board().get(pos)?.pion()?;
    game.active_effects()
        .iter()
        .find(|effect| effect.pion() == pion.id() && effect.kind() == EffectKind::StepTokens)
        .and_then(|effect| effect.tokens())
}

/// Au tour 1, J1 pose A1 puis le téléporte en F6 avec son Interchange Commune (premier
/// emplacement). Rien d'autre.
fn teleported(player_one: Vec<Attack>, player_two: Vec<Attack>) -> Game {
    let mut attacks = vec![interchange(Rarity::Common)];
    attacks.extend(player_one);
    let mut game = game_with(config(attacks, player_two));
    play(
        &mut game,
        &[
            place(0, 0),
            Action::UseAttack(0),
            choose(0, 0),
            choose(5, 5),
        ],
    );
    game
}

#[test]
fn the_teleported_pion_comes_back_at_the_start_of_its_next_turn() {
    let mut game = teleported(Vec::new(), Vec::new());
    assert_eq!(game.board().owner(at(0, 0)), None);
    assert!(game.has_effect(at(5, 5), EffectKind::Return { origin: at(0, 0) }));
    play(&mut game, &[Action::EndTurn]); // tour 1
    // Pendant le tour adverse, il est toujours là-bas.
    assert_eq!(game.board().owner(at(5, 5)), Some(Player::One));
    play(&mut game, &[Action::EndTurn]); // tour 2 : J1 commence le tour 3
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::One));
    assert_eq!(game.board().owner(at(5, 5)), None);
    assert!(game.active_effects().is_empty());
}

#[test]
fn a_pion_waiting_for_its_return_cannot_teleport_again() {
    let mut game = teleported(vec![interchange(Rarity::Common)], Vec::new());
    play(&mut game, &[place(9, 9), Action::UseAttack(1)]);
    assert_eq!(
        game.apply(choose(5, 5)),
        Err(GameError::InvalidChoice(at(5, 5)))
    );
}

#[test]
fn the_return_kills_the_pion_on_its_cell_whatever_its_camp() {
    // Un pion adverse.
    let mut game = teleported(Vec::new(), Vec::new());
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 2 : J2 occupe A1
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::One));
    // Sans infirmerie, le pion tué est rentré dans la réserve de J2.
    assert_eq!(game.reserve_pions(Player::Two), 15);

    // Un pion de J1, qui avait un Massacre divin : son propre pion n'use aucun jeton.
    let massacre = card(AttackKind::MassacreDivin, Rarity::Common);
    let mut game = teleported(vec![massacre], Vec::new());
    play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(5, 5),
            place(0, 0),
            Action::EndTurn,
        ],
    ); // tour 1 : J1 occupe A1 lui-même
    let events = play(&mut game, &[Action::EndTurn]); // tour 2
    assert!(events.iter().any(|event| matches!(
        event,
        Event::PionKilled { at: killed, .. } if *killed == at(0, 0)
    )));
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::One));
    assert_eq!(game.board().owner(at(5, 5)), None);
    assert_eq!(game.token_piles().len(), 1);
    assert_eq!(game.token_piles()[0].tokens(), 1);
}

#[test]
fn a_protected_pion_on_the_cell_cancels_the_return() {
    let royal_guard = card(AttackKind::Reine, Rarity::Common);
    let mut game = teleported(Vec::new(), vec![royal_guard]);
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(0, 0),
            Action::UseAttack(0),
            choose(0, 0),
            Action::EndTurn,
        ],
    ); // tour 2 : J2 protège son pion en A1
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::Two));
    assert_eq!(game.board().owner(at(5, 5)), Some(Player::One));
    assert!(
        !game
            .active_effects()
            .iter()
            .any(|effect| matches!(effect.kind(), EffectKind::Return { .. }))
    );
}

#[test]
fn a_killed_pion_never_returns_and_attila_cancels_the_return() {
    let bear = card(AttackKind::Ours, Rarity::Rare);
    let mut game = teleported(Vec::new(), vec![bear]);
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(5, 6),
            Action::UseAttack(0),
            choose(5, 6),
            choose(5, 5),
            Action::EndTurn,
        ],
    ); // tour 2 : J2 tue F6
    assert_eq!(game.board().owner(at(0, 0)), None);

    let attila = card(AttackKind::Attila, Rarity::Rare);
    let mut game = teleported(Vec::new(), vec![attila]);
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            choose(5, 5),
            Action::EndTurn,
        ],
    ); // tour 2 : J2 annule le retour
    assert_eq!(game.board().owner(at(5, 5)), Some(Player::One));
    assert_eq!(game.board().owner(at(0, 0)), None);
}

#[test]
fn an_immobilized_or_moved_pion_still_returns() {
    let growl = card(AttackKind::Ours, Rarity::Common);
    let mut game = teleported(Vec::new(), vec![growl]);
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(5, 6),
            Action::UseAttack(0),
            choose(5, 6),
            Action::EndTurn,
        ],
    ); // tour 2 : F6 grogné
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::One));
    assert!(game.has_effect(at(0, 0), EffectKind::Immobilized));

    let swap = interchange(Rarity::Epic);
    let mut game = teleported(Vec::new(), vec![swap]);
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            choose(5, 5),
            Action::EndTurn,
        ],
    ); // tour 2 : J2 échange J10 et F6
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::One));
    assert_eq!(game.board().owner(at(9, 9)), None);
}

#[test]
fn the_return_comes_before_the_golden_rule() {
    let config_of = |attacks| GameConfig {
        golden_rule: true,
        align_to_win: 2,
        ..config(attacks, Vec::new())
    };
    // La ligne faite par la téléportation est défaite avant d'être vérifiée.
    let mut game = game_with(config_of(vec![interchange(Rarity::Common)]));
    play(
        &mut game,
        &[
            place(0, 0),
            place(5, 4),
            Action::UseAttack(0),
            choose(0, 0),
            choose(5, 5),
        ],
    );
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 1 et 2
    assert_eq!(game.end(), None);
}

#[test]
fn a_line_completed_by_the_return_wins() {
    let mut game = game_with(GameConfig {
        golden_rule: true,
        align_to_win: 2,
        ..config(vec![interchange(Rarity::Common)], Vec::new())
    });
    play(
        &mut game,
        &[
            place(0, 0),
            Action::UseAttack(0),
            choose(0, 0),
            choose(5, 5),
            place(0, 1),
        ],
    ); // tour 1 : B1 posé, A1 libre
    play(&mut game, &[Action::EndTurn]); // tour 1
    let events = play(&mut game, &[Action::EndTurn]); // tour 2 : A1 revient, A1-B1
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded(GameEnd {
            outcome: Outcome::Won(Player::One),
            reason: EndReason::Alignment,
        }))
    );
}

#[test]
fn a_friend_killed_by_its_own_camp_is_not_avenged() {
    let friends = card(AttackKind::MeilleursAmis, Rarity::Epic);
    let mut game = teleported(vec![friends], Vec::new());
    play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(5, 5),
            choose(0, 0),
            choose(9, 9),
        ],
    ); // tour 1 : les amis en A1 (la case de retour) et J10
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 1 et 2
    // A1 est tué par le pion qui revient ; J10 ne se venge pas sur son propre camp.
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::One));
    assert_eq!(game.board().owner(at(9, 9)), Some(Player::One));
    assert_eq!(game.pact_friend(at(9, 9)), None);
}

#[test]
fn the_legendary_swaps_two_pions_with_two_enemies_side_by_side() {
    let mut game = game_with(config(vec![interchange(Rarity::Legendary)], Vec::new()));
    play(&mut game, &[place(0, 0), place(0, 9), Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[place(5, 5), place(5, 6), place(9, 9), Action::EndTurn],
    ); // tour 2
    play(&mut game, &[Action::UseAttack(0), choose(0, 0)]);
    // Le premier pion adverse doit en avoir un autre collé : pas J10.
    assert_eq!(
        game.apply(choose(9, 9)),
        Err(GameError::InvalidChoice(at(9, 9)))
    );
    play(&mut game, &[choose(5, 5), choose(0, 9)]);
    // Le second pion adverse est collé au premier.
    assert_eq!(
        game.apply(choose(9, 9)),
        Err(GameError::InvalidChoice(at(9, 9)))
    );
    play(&mut game, &[choose(5, 6)]); // tour 3
    for (pos, owner) in [
        (at(5, 5), Player::One),
        (at(5, 6), Player::One),
        (at(0, 0), Player::Two),
        (at(0, 9), Player::Two),
    ] {
        assert_eq!(game.board().owner(pos), Some(owner), "{pos}");
    }
}

#[test]
fn the_legendary_needs_a_second_ready_pion() {
    let mut game = game_with(config(vec![interchange(Rarity::Legendary)], Vec::new()));
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 1
    play(&mut game, &[place(5, 5), place(5, 6), Action::EndTurn]); // tour 2
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(interchange(Rarity::Legendary)))
    );
}

#[test]
fn the_hare_epic_gives_three_tokens_spent_one_per_turn() {
    let mut game = game_with(config(vec![hare_epic()], Vec::new()));
    play(
        &mut game,
        &[place(5, 5), Action::UseAttack(0), choose(5, 5)],
    );
    assert_eq!(step_tokens(&game, at(5, 5)), Some(3));
    // Dès ce tour : un premier pas dépense un jeton.
    let events = play(&mut game, &[step(at(5, 5), at(5, 6))]);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::EffectChanged(_)))
    );
    assert_eq!(step_tokens(&game, at(5, 6)), Some(2));
    // Le second pas ne revient pas sur la case de départ.
    assert_eq!(
        game.apply(step(at(5, 6), at(5, 5))),
        Err(GameError::HareCannotStep {
            from: at(5, 6),
            to: at(5, 5)
        })
    );
    play(&mut game, &[step(at(5, 6), at(5, 7))]);
    // Deux pas : plus rien ce tour, même avec des jetons.
    assert!(hare_steps(&game).is_empty());
    assert_eq!(step_tokens(&game, at(5, 7)), Some(2));
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 1 et 2
    // Un seul pas suffit ; le tour suivant, encore un jeton.
    play(
        &mut game,
        &[step(at(5, 7), at(5, 8)), Action::EndTurn, Action::EndTurn],
    );
    assert_eq!(step_tokens(&game, at(5, 8)), Some(1));
    let events = play(&mut game, &[step(at(5, 8), at(5, 9))]); // tour 5 : le dernier
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::EffectEnded(_)))
    );
    assert_eq!(step_tokens(&game, at(5, 9)), None);
    // Le second pas du dernier jeton reste permis.
    assert!(!hare_steps(&game).is_empty());
}

#[test]
fn a_pion_with_step_tokens_cannot_get_more_and_waits_when_immobilized() {
    let growl = card(AttackKind::Ours, Rarity::Common);
    let mut game = game_with(config(vec![hare_epic()], vec![growl]));
    play(
        &mut game,
        &[
            place(5, 5),
            place(0, 0),
            Action::UseAttack(0),
            choose(5, 5),
            Action::EndTurn,
        ],
    ); // tour 1
    play(
        &mut game,
        &[
            place(5, 6),
            Action::UseAttack(0),
            choose(5, 6),
            Action::EndTurn,
        ],
    ); // tour 2 : F6 grogné
    assert!(hare_steps(&game).is_empty());
    assert_eq!(step_tokens(&game, at(5, 5)), Some(3));
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 0),
            Action::EndTurn,
            Action::EndTurn,
        ],
    ); // tours 3 et 4 : A1 reçoit des jetons à son tour
    // Libéré : ses jetons servent de nouveau.
    assert!(hare_steps(&game).contains(&step(at(5, 5), at(4, 4))));
    // Un pion qui a encore des jetons n'en reçoit pas d'autres : ici, aucun lanceur.
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(hare_epic()))
    );
}

#[test]
fn attila_cancels_all_the_step_tokens_at_once() {
    let attila = card(AttackKind::Attila, Rarity::Rare);
    let mut game = game_with(config(vec![hare_epic()], vec![attila]));
    play(
        &mut game,
        &[
            place(5, 5),
            Action::UseAttack(0),
            choose(5, 5),
            Action::EndTurn,
        ],
    ); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            choose(5, 5),
            Action::EndTurn,
        ],
    ); // tour 2
    assert_eq!(step_tokens(&game, at(5, 5)), None);
    assert!(hare_steps(&game).is_empty());
}

#[test]
fn the_turn_waits_while_a_hare_step_is_possible() {
    let mut game = game_with(GameConfig {
        first_turn_actions: 4,
        ..config(vec![hare_epic()], Vec::new())
    });
    // Pose (1) et Lièvre Épique (3) : plus d'action, mais un pas possible.
    play(
        &mut game,
        &[place(5, 5), Action::UseAttack(0), choose(5, 5)],
    );
    assert_eq!(game.actions_left(), 0);
    assert_eq!(game.current_player(), Player::One);
    play(
        &mut game,
        &[step(at(5, 5), at(5, 6)), step(at(5, 6), at(5, 7))],
    );
    // Plus rien à faire : le tour se termine seul.
    assert_eq!(game.current_player(), Player::Two);
}

#[test]
fn a_queen_saved_by_her_guard_cancels_the_return() {
    let queen = card(AttackKind::Reine, Rarity::Epic);
    let mut game = teleported(Vec::new(), vec![queen]);
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            choose(0, 0),
            Action::EndTurn,
        ],
    ); // tour 2 : une reine gardée en A1
    assert_eq!(game.queen_of(Player::Two), Some(at(0, 0)));
    assert_eq!(game.board().owner(at(5, 5)), Some(Player::One));
}

#[test]
fn an_enemy_killed_by_the_return_uses_the_tokens_of_its_killer() {
    let massacre = card(AttackKind::MassacreDivin, Rarity::Epic);
    let mut game = teleported(vec![massacre], Vec::new());
    play(
        &mut game,
        &[Action::UseAttack(1), choose(5, 5), Action::EndTurn],
    ); // tour 1
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 2 : J2 occupe A1
    assert_eq!(game.board().owner(at(0, 0)), Some(Player::One));
    assert_eq!(game.graveyard().len(), 1);
    assert_eq!(game.graveyard()[0].owner(), Player::Two);
}

#[test]
fn a_hare_step_is_refused_on_an_enemy_pion_or_during_an_attack() {
    let mut game = game_with(config(
        vec![hare_epic(), card(AttackKind::Ours, Rarity::Rare)],
        Vec::new(),
    ));
    play(
        &mut game,
        &[
            place(5, 5),
            Action::UseAttack(0),
            choose(5, 5),
            Action::EndTurn,
        ],
    ); // tour 1
    // J2 ne fait pas avancer le pion de J1, même s'il porte des jetons.
    play(&mut game, &[place(4, 4)]);
    assert_eq!(
        game.apply(step(at(5, 5), at(6, 6))),
        Err(GameError::HareCannotStep {
            from: at(5, 5),
            to: at(6, 6)
        })
    );
    play(&mut game, &[Action::EndTurn]); // tour 2
    // Pendant une attaque, aucun geste.
    play(&mut game, &[Action::UseAttack(1)]);
    assert_eq!(
        game.apply(step(at(5, 5), at(6, 6))),
        Err(GameError::UnexpectedAction)
    );
    play(&mut game, &[choose(5, 5), choose(4, 4)]); // tour 3
    assert!(hare_steps(&game).contains(&step(at(5, 5), at(6, 6))));
}
