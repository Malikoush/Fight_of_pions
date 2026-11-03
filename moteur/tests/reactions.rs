//! Les réactions (B8, carnet « Réactions », 2026-11-03) : la fenêtre de réaction et les
//! cinq réactions.
//!
//! Sauf mention contraire, les règles actuelles sans mur : 1 pose gratuite par tour, 1
//! point d'action par tour, J2 commence avec 1 point en banque (la Pièce).

mod common;

use common::{choose, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, Decision, EffectKind, Event, Game, GameConfig, GameError, Player,
    Position, Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn sentinel(rarity: Rarity) -> Attack {
    card(AttackKind::Sentinelle, rarity)
}

fn owl(rarity: Rarity) -> Attack {
    card(AttackKind::Hibou, rarity)
}

/// Les règles actuelles sans mur, J1 et J2 avec ces cartes.
fn rules(player_one: Vec<Attack>, player_two: Vec<Attack>) -> GameConfig {
    GameConfig {
        wall_thickness: 0,
        starting_attacks: [player_one, player_two],
        ..GameConfig::default()
    }
}

/// J2 peut réagir maintenant.
fn two_may_react(game: &Game) -> bool {
    game.pending_decision()
        == Some(Decision::React {
            player: Player::Two,
        })
}

/// C'est à J1 de jouer son tour.
fn one_plays(game: &Game) -> bool {
    game.pending_decision()
        == Some(Decision::MainAction {
            player: Player::One,
        })
}

#[test]
fn the_sentinel_places_next_to_the_pion_the_opponent_just_placed() {
    let mut game = game_with(rules(Vec::new(), vec![sentinel(Rarity::Common)]));
    play(&mut game, &[place(4, 4)]);
    assert!(two_may_react(&game));
    assert_eq!(game.legal_actions(), [Action::UseAttack(0), Action::Pass]);

    play(&mut game, &[Action::UseAttack(0)]);
    // Le lanceur est le pion posé, seul choix possible.
    assert_eq!(game.legal_actions(), [choose(4, 4)]);
    play(&mut game, &[choose(4, 4), choose(4, 5)]);

    assert_eq!(game.board().owner(at(4, 5)), Some(Player::Two));
    assert_eq!(game.reserve_pions(Player::Two), 14);
    assert_eq!(game.banked_actions(Player::Two), 0, "payée avec la Pièce");
    assert_eq!(game.arsenal(Player::Two)[0].cooldown_left(), 1);
    // Le tour de J1 reprend : il a encore son point d'action.
    assert!(one_plays(&game));
    assert_eq!(game.current_player(), Player::One);
    assert_eq!(game.actions_left(), 1);
}

#[test]
fn the_sentinel_only_places_next_to_the_pion_just_placed() {
    let mut game = game_with(rules(Vec::new(), vec![sentinel(Rarity::Common)]));
    play(
        &mut game,
        &[place(0, 0), Action::UseAttack(0), choose(0, 0)],
    );
    assert_eq!(launchers(&game), [choose(0, 1), choose(1, 0), choose(1, 1)]);
    assert_eq!(
        game.apply(choose(5, 5)),
        Err(GameError::InvalidChoice(at(5, 5)))
    );
}

#[test]
fn the_rare_sentinel_also_immobilizes_the_pion_for_two_of_its_owner_turns() {
    let config = GameConfig {
        starting_banked_actions: [0, 2],
        ..rules(Vec::new(), vec![sentinel(Rarity::Rare)])
    };
    let mut game = game_with(config);
    play(
        &mut game,
        &[
            place(4, 4),
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
        ],
    );
    assert_eq!(game.banked_actions(Player::Two), 0);
    assert!(game.has_effect(at(4, 4), EffectKind::Immobilized));

    // Le tour 1, où la Sentinelle est jouée, ne compte pas : les tours 3 et 5 de J1.
    for turn in 2..=5 {
        play(&mut game, &[Action::EndTurn]);
        assert_eq!(game.turn(), turn);
        assert!(
            game.has_effect(at(4, 4), EffectKind::Immobilized),
            "tour {turn}"
        );
    }
    play(&mut game, &[Action::EndTurn]);
    assert!(!game.has_effect(at(4, 4), EffectKind::Immobilized));
}

#[test]
fn no_window_opens_without_a_reaction_the_player_can_pay() {
    let config = GameConfig {
        starting_banked_actions: [0, 0],
        ..rules(Vec::new(), vec![sentinel(Rarity::Common)])
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4)]);
    assert!(one_plays(&game));
}

#[test]
fn no_window_opens_without_a_pion_in_reserve() {
    // Premières règles : 2 actions par tour, une pose coûte 1, plusieurs poses par tour.
    let config = GameConfig {
        pions_per_player: 3,
        starting_banked_actions: [0, 2],
        starting_attacks: [Vec::new(), vec![sentinel(Rarity::Common)]],
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);
    play(
        &mut game,
        &[
            place(0, 0),
            Action::UseAttack(0),
            choose(0, 0),
            choose(1, 0),
        ],
    );
    play(&mut game, &[place(9, 9), place(8, 9), Action::EndTurn]);
    assert_eq!(game.reserve_pions(Player::Two), 0);
    assert_eq!(game.banked_actions(Player::Two), 1);
    assert_eq!(game.arsenal(Player::Two)[0].cooldown_left(), 0);
    // Sa Sentinelle est prête et payable, mais il n'a plus de pion à poser.
    play(&mut game, &[place(5, 5)]);
    assert!(one_plays(&game));
}

#[test]
fn a_reaction_only_answers_its_own_trigger() {
    // Le Hibou répond à un déplacement, pas à une pose.
    let mut game = game_with(rules(Vec::new(), vec![owl(Rarity::Common)]));
    play(&mut game, &[place(4, 4)]);
    assert!(one_plays(&game));
}

#[test]
fn a_reaction_is_never_played_in_its_own_turn_and_an_attack_never_as_a_reaction() {
    let ours = card(AttackKind::Ours, Rarity::Common);
    let mut game = game_with(rules(
        vec![sentinel(Rarity::Common)],
        vec![ours, sentinel(Rarity::Common)],
    ));
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NotTriggered(sentinel(Rarity::Common)))
    );
    assert_eq!(game.apply(Action::Pass), Err(GameError::UnexpectedAction));

    play(&mut game, &[place(4, 4)]);
    assert!(two_may_react(&game));
    assert_eq!(game.legal_actions(), [Action::UseAttack(1), Action::Pass]);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NotAReaction(ours))
    );
    assert_eq!(game.apply(place(0, 0)), Err(GameError::UnexpectedAction));
    assert_eq!(
        game.apply(Action::EndTurn),
        Err(GameError::UnexpectedAction)
    );
}

#[test]
fn passing_lets_the_turn_go_on_and_a_later_trigger_opens_a_new_window() {
    // Premières règles : 2 actions par tour, une pose coûte 1, plusieurs poses par tour.
    let config = GameConfig {
        starting_attacks: [Vec::new(), vec![sentinel(Rarity::Common)]],
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]); // J1 n'avait qu'une action : son tour finit
    play(&mut game, &[Action::EndTurn]); // J2 garde ses 2 actions
    assert_eq!(game.banked_actions(Player::Two), 2);

    play(&mut game, &[place(4, 4)]);
    assert!(two_may_react(&game));
    play(&mut game, &[Action::Pass]);
    assert!(one_plays(&game));
    assert_eq!(game.banked_actions(Player::Two), 2);

    play(&mut game, &[place(6, 6)]);
    assert!(
        two_may_react(&game),
        "passer ne compte pas comme une réaction"
    );
}

#[test]
fn a_player_reacts_at_most_once_per_opponent_turn() {
    let config = GameConfig {
        starting_attacks: [
            Vec::new(),
            vec![sentinel(Rarity::Common), sentinel(Rarity::Common)],
        ],
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    play(
        &mut game,
        &[
            place(4, 4),
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
        ],
    );
    assert_eq!(game.banked_actions(Player::Two), 1);
    // Sa seconde Sentinelle est prête et payable, mais il a déjà réagi ce tour.
    let events = play(&mut game, &[place(6, 6)]);
    assert!(!two_may_react(&game));
    assert_eq!(game.current_player(), Player::Two, "{events:?}");

    // Au tour adverse suivant, il peut de nouveau réagir.
    play(&mut game, &[Action::EndTurn, place(8, 8)]);
    assert!(two_may_react(&game));
}

#[test]
fn the_reaction_recharges_and_is_ready_again_at_its_owner_next_turn() {
    let config = GameConfig {
        starting_banked_actions: [0, 3],
        ..rules(Vec::new(), vec![sentinel(Rarity::Common)])
    };
    let mut game = game_with(config);
    play(
        &mut game,
        &[
            place(4, 4),
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
        ],
    );
    play(&mut game, &[Action::EndTurn]);
    assert_eq!(game.arsenal(Player::Two)[0].cooldown_left(), 0);
    play(&mut game, &[Action::EndTurn, place(7, 7)]);
    assert!(two_may_react(&game));
}

#[test]
fn the_owl_steps_one_of_its_pions_when_the_opponent_moves_a_pion() {
    let hare = card(AttackKind::Lievre, Rarity::Common);
    let mut game = game_with(rules(vec![hare], vec![owl(Rarity::Common)]));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    assert_eq!(game.banked_actions(Player::Two), 2);

    // Le Lièvre de J1 avance d'un pas : J2 peut répondre.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
            Action::FinishAttack,
        ],
    );
    assert!(two_may_react(&game));
    play(&mut game, &[Action::UseAttack(0)]);
    assert_eq!(game.legal_actions(), [choose(0, 0)]);
    play(&mut game, &[choose(0, 0), choose(1, 1)]);

    assert_eq!(game.board().owner(at(1, 1)), Some(Player::Two));
    assert_eq!(game.board().owner(at(0, 0)), None);
    assert_eq!(game.banked_actions(Player::Two), 1);
    assert_eq!(game.current_player(), Player::One);
}

#[test]
fn the_owl_moves_only_pions_ready_to_act() {
    let hare = card(AttackKind::Lievre, Rarity::Common);
    let config = GameConfig {
        starting_banked_actions: [0, 3],
        ..rules(
            vec![hare],
            vec![owl(Rarity::Common), sentinel(Rarity::Common)],
        )
    };
    let mut game = game_with(config);
    // J2 répond à la pose de J1 par une Sentinelle : son pion, posé ce tour, ne pourra
    // pas avancer avec le Hibou.
    play(
        &mut game,
        &[
            place(4, 4),
            Action::UseAttack(1),
            choose(4, 4),
            choose(5, 5),
            Action::EndTurn,
        ],
    );
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(3, 3),
            Action::FinishAttack,
        ],
    );
    play(&mut game, &[Action::UseAttack(0)]);
    // F6 a été posé au tour 1, A1 au tour 2 : tous deux avant ce tour.
    assert_eq!(game.legal_actions(), [choose(0, 0), choose(5, 5)]);
}

#[test]
fn the_rare_owl_steps_up_to_two_different_pions() {
    let hare = card(AttackKind::Lievre, Rarity::Common);
    let config = GameConfig {
        starting_banked_actions: [0, 3],
        ..rules(vec![hare], vec![owl(Rarity::Rare)])
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    play(&mut game, &[place(4, 6), Action::EndTurn]);
    play(&mut game, &[place(9, 9), Action::EndTurn]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
            Action::FinishAttack,
        ],
    );
    play(
        &mut game,
        &[Action::UseAttack(0), choose(0, 0), choose(0, 1)],
    );
    // Un seul pion suffit : on peut jouer la carte après le premier.
    assert!(game.legal_actions().contains(&Action::FinishAttack));
    play(&mut game, &[choose(9, 9), choose(8, 8)]);
    assert_eq!(game.board().owner(at(0, 1)), Some(Player::Two));
    assert_eq!(game.board().owner(at(8, 8)), Some(Player::Two));
    assert_eq!(game.banked_actions(Player::Two), 1);
}

#[test]
fn a_line_made_by_a_reaction_must_survive_a_whole_opponent_turn() {
    let config = GameConfig {
        align_to_win: 3,
        ..rules(Vec::new(), vec![sentinel(Rarity::Common)])
    };
    let mut game = game_with(config);
    play(&mut game, &[place(9, 9), Action::Pass, Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    play(&mut game, &[place(7, 9), Action::Pass, Action::EndTurn]);
    play(&mut game, &[place(0, 1), Action::EndTurn]);
    // Tour 5 : J1 pose en B2 ; la Sentinelle de J2 pose en C1 et fait A1-B1-C1.
    play(
        &mut game,
        &[
            place(1, 1),
            Action::UseAttack(0),
            choose(1, 1),
            choose(0, 2),
            Action::EndTurn,
        ],
    );
    // Tour 6 : la ligne a moins d'un tour adverse complet, elle ne gagne pas encore.
    assert_eq!(game.turn(), 6);
    assert_eq!(game.end(), None);
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    // Tour 8 : J1 ne l'a pas cassée pendant tout son tour 7.
    assert_eq!(game.winner(), Some(Player::Two));
}

#[test]
fn no_window_opens_after_a_move_that_ends_the_game() {
    // Sans la règle d'or, une ligne faite en posant gagne aussitôt.
    let config = GameConfig {
        board_size: 3,
        align_to_win: 2,
        pions_per_player: 4,
        starting_banked_actions: [0, 2],
        starting_attacks: [Vec::new(), vec![sentinel(Rarity::Common)]],
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0), Action::Pass]);
    play(&mut game, &[place(2, 2), Action::EndTurn]);
    play(&mut game, &[place(1, 0)]);
    assert_eq!(game.winner(), Some(Player::One));
    assert_eq!(game.pending_decision(), None);
}

fn lizard(rarity: Rarity) -> Attack {
    card(AttackKind::QueueDuLezard, rarity)
}

fn parrot(rarity: Rarity) -> Attack {
    card(AttackKind::Perroquet, rarity)
}

/// J1 (cette carte) pose E5, J2 (ces cartes) pose F5, chacun garde ses points ; au tour 3,
/// J1 a 2 points et J2 en a 2 en banque.
fn face_to_face(player_one: Attack, player_two: Vec<Attack>) -> Game {
    let mut game = game_with(rules(vec![player_one], player_two));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    assert_eq!(game.actions_left(), 2);
    assert_eq!(game.banked_actions(Player::Two), 2);
    game
}

/// L'Ours Rare de J1 tue F5 depuis E5.
const KILL_F5: [Action; 3] = [
    Action::UseAttack(0),
    Action::Choose(Position { row: 4, col: 4 }),
    Action::Choose(Position { row: 4, col: 5 }),
];

#[test]
fn the_lizard_places_next_to_its_dead_pion_but_never_on_its_cell() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    let mut game = face_to_face(ours, vec![lizard(Rarity::Common)]);
    play(&mut game, &KILL_F5);
    assert_eq!(game.board().owner(at(4, 5)), None);
    assert!(two_may_react(&game));

    play(&mut game, &[Action::UseAttack(0)]);
    // Le lanceur est la case du pion mort, vide.
    assert_eq!(game.legal_actions(), [choose(4, 5)]);
    play(&mut game, &[choose(4, 5)]);
    assert!(!game.legal_actions().contains(&choose(4, 5)));
    play(&mut game, &[choose(3, 6)]);

    assert_eq!(game.board().owner(at(3, 6)), Some(Player::Two));
    assert_eq!(game.board().owner(at(4, 5)), None);
    assert_eq!(game.banked_actions(Player::Two), 1);
}

#[test]
fn the_rare_lizard_places_up_to_two_pions() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    let mut game = face_to_face(ours, vec![lizard(Rarity::Rare)]);
    play(&mut game, &KILL_F5);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(3, 6)],
    );
    assert!(game.legal_actions().contains(&Action::FinishAttack));
    play(&mut game, &[choose(5, 6)]);
    assert_eq!(game.board().owner(at(3, 6)), Some(Player::Two));
    assert_eq!(game.board().owner(at(5, 6)), Some(Player::Two));
    assert_eq!(game.banked_actions(Player::Two), 0);
}

#[test]
fn the_parrot_replays_the_attack_with_its_own_launcher_and_targets() {
    let hare = card(AttackKind::Lievre, Rarity::Common);
    let mut game = face_to_face(hare, vec![parrot(Rarity::Rare)]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(3, 3),
            Action::FinishAttack,
        ],
    );
    assert!(two_may_react(&game));

    let events = play(&mut game, &[Action::UseAttack(0)]);
    assert!(events.is_empty());
    assert_eq!(
        game.pending_decision(),
        Some(Decision::ChooseLauncher {
            player: Player::Two,
            attack: hare
        })
    );
    let events = play(&mut game, &[choose(4, 5), choose(4, 6), choose(4, 7)]);
    assert_eq!(game.board().owner(at(4, 7)), Some(Player::Two));
    // C'est le Perroquet qui est joué, payé à son prix (2, pour une attaque de 1) et
    // rechargé.
    assert!(events.contains(&Event::AttackUsed {
        player: Player::Two,
        attack: parrot(Rarity::Rare),
        launcher: at(4, 5),
    }));
    assert_eq!(game.banked_actions(Player::Two), 0);
    assert_eq!(game.arsenal(Player::Two)[0].cooldown_left(), 1);
    assert_eq!(game.current_player(), Player::One);
}

#[test]
fn the_parrot_only_replays_an_attack_within_its_cost() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    // F5 meurt : seule la Queue du lézard peut répondre, le Perroquet Commune ne rejoue
    // pas une attaque de 2 points.
    let mut game = face_to_face(ours, vec![parrot(Rarity::Common), lizard(Rarity::Common)]);
    play(&mut game, &KILL_F5);
    assert_eq!(game.legal_actions(), [Action::UseAttack(1), Action::Pass]);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NotTriggered(parrot(Rarity::Common)))
    );
}

#[test]
fn the_rare_parrot_replays_a_kill_on_the_attacker_side() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    let mut game = game_with(rules(vec![ours], vec![parrot(Rarity::Rare)]));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(&mut game, &[place(3, 3), Action::EndTurn]);
    play(&mut game, &[place(5, 5), Action::EndTurn]);
    // Tour 5 : J1 tue F5 ; J2 rejoue l'Ours Rare depuis F6 sur E5.
    play(&mut game, &KILL_F5);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(5, 5), choose(4, 4)],
    );
    assert_eq!(game.board().owner(at(4, 4)), None);
    assert_eq!(game.reserve_pions(Player::One), 13);
}

fn feint(rarity: Rarity) -> Attack {
    card(AttackKind::Feintaille, rarity)
}

#[test]
fn the_feint_dodges_the_attack_before_it_applies() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    let mut game = face_to_face(ours, vec![feint(Rarity::Common)]);
    play(&mut game, &KILL_F5);
    // L'attaque attend : F5 est encore là, rien n'est payé.
    assert!(two_may_react(&game));
    assert_eq!(game.board().owner(at(4, 5)), Some(Player::Two));
    assert_eq!(game.actions_left(), 2);

    play(&mut game, &[Action::UseAttack(0)]);
    assert_eq!(game.legal_actions().first(), Some(&choose(4, 5)));
    let events = play(&mut game, &[choose(4, 5), choose(4, 6)]);
    // F5 a esquivé en G5 ; l'Ours de J1 frappe une case vide, et il est payé quand même.
    assert_eq!(game.board().owner(at(4, 6)), Some(Player::Two));
    assert_eq!(game.board().owner(at(4, 5)), None);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::PionKilled { .. }))
    );
    assert!(events.contains(&Event::AttackUsed {
        player: Player::One,
        attack: ours,
        launcher: at(4, 4),
    }));
    assert_eq!(game.actions_left(), 0);
    assert_eq!(game.banked_actions(Player::Two), 1);
}

#[test]
fn passing_lets_the_waiting_attack_apply() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    let mut game = face_to_face(ours, vec![feint(Rarity::Common)]);
    play(&mut game, &KILL_F5);
    let events = play(&mut game, &[Action::Pass]);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::PionKilled { .. }))
    );
    assert_eq!(game.board().owner(at(4, 5)), None);
}

#[test]
fn the_common_feint_does_not_dodge_an_attack_above_two_points() {
    let ours = card(AttackKind::Ours, Rarity::Epic);
    let config = GameConfig {
        starting_banked_actions: [0, 3],
        ..rules(vec![ours], vec![feint(Rarity::Common), feint(Rarity::Rare)])
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    // Tour 5 : J1 a 3 points, l'Ours Épique (3 points) vise F5.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );
    assert!(two_may_react(&game));
    assert_eq!(game.legal_actions(), [Action::UseAttack(1), Action::Pass]);
}

#[test]
fn an_immobilized_pion_cannot_dodge() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    let config = GameConfig {
        starting_banked_actions: [1, 3],
        ..rules(
            vec![ours, sentinel(Rarity::Rare)],
            vec![feint(Rarity::Common)],
        )
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    // Tour 2 : la Sentinelle Rare de J1 répond à F5 et l'immobilise.
    play(
        &mut game,
        &[
            place(4, 5),
            Action::UseAttack(1),
            choose(4, 5),
            choose(4, 6),
            Action::EndTurn,
        ],
    );
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    // Tour 5 : l'Ours Rare de J1 vise F5, immobilisé : il ne peut pas esquiver.
    assert!(game.has_effect(at(4, 5), EffectKind::Immobilized));
    play(&mut game, &KILL_F5);
    assert_eq!(game.board().owner(at(4, 5)), None);
    assert!(!two_may_react(&game));
}

/// Les règles actuelles sans mur, J1 et J2 avec ces cartes et 3 points en banque au départ :
/// J1 a 4 points à son premier tour, et au moins 4 ensuite s'il économise.
fn rich(player_one: Vec<Attack>, player_two: Vec<Attack>) -> GameConfig {
    GameConfig {
        starting_banked_actions: [3, 3],
        ..rules(player_one, player_two)
    }
}

/// Les choix que la décision en cours propose, triés.
fn launchers(game: &Game) -> Vec<Action> {
    let mut actions = game.legal_actions();
    actions.sort_by_key(|action| format!("{action:?}"));
    actions
}

#[test]
fn the_sentinel_answers_one_of_the_pions_a_card_places() {
    let friends = card(AttackKind::MeilleursAmis, Rarity::Epic);
    let mut game = game_with(rich(vec![friends], vec![sentinel(Rarity::Common)]));
    play(&mut game, &[place(4, 4), Action::Pass, Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    // Tour 3 : Meilleurs amis pose G7 et I7 ; une seule fenêtre, pour l'un ou l'autre.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(6, 6),
            choose(6, 8),
        ],
    );
    assert!(two_may_react(&game));
    play(&mut game, &[Action::UseAttack(0)]);
    assert_eq!(launchers(&game), [choose(6, 6), choose(6, 8)]);
}

#[test]
fn the_lizard_chooses_around_which_dead_pion_it_answers() {
    let ours = card(AttackKind::Ours, Rarity::Epic);
    let mut game = game_with(rich(vec![ours], vec![lizard(Rarity::Common)]));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn]);
    play(&mut game, &[place(5, 5), Action::EndTurn]);
    // Tour 5 : l'Ours Épique tue F5 et F6 d'un coup.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
            choose(5, 5),
        ],
    );
    assert!(two_may_react(&game));
    play(&mut game, &[Action::UseAttack(0)]);
    assert_eq!(launchers(&game), [choose(4, 5), choose(5, 5)]);
}

#[test]
fn the_lizard_answers_a_guard_shot_but_the_parrot_does_not() {
    let queen = card(AttackKind::Reine, Rarity::Epic);
    let mut game = game_with(rich(
        vec![queen],
        vec![parrot(Rarity::Rare), lizard(Rarity::Common)],
    ));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(2, 6), Action::EndTurn]);
    // Tour 3 : J1 couronne une reine gardée en C3 (3 points : le Perroquet ne la rejoue pas).
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(2, 2)],
    );
    assert!(!two_may_react(&game));
    play(&mut game, &[Action::EndTurn, Action::EndTurn]);
    // Tour 5 : le garde tire sur G3 ; seule la Queue du lézard répond.
    play(&mut game, &[Action::Shoot(at(2, 6))]);
    assert_eq!(game.board().owner(at(2, 6)), None);
    assert!(two_may_react(&game));
    assert_eq!(game.legal_actions(), [Action::UseAttack(1), Action::Pass]);
}

#[test]
fn the_rare_sentinel_is_not_playable_under_scorched_earth() {
    let scorched_earth = card(AttackKind::Attila, Rarity::Legendary);
    let mut game = game_with(rich(vec![scorched_earth], vec![sentinel(Rarity::Rare)]));
    play(&mut game, &[place(4, 4), Action::Pass, Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    // Tour 3 : Terre brûlée sur J2, puis J1 pose : la Sentinelle Rare immobiliserait.
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert_eq!(game.scorched_until(Player::Two), Some(4));
    play(&mut game, &[place(6, 6)]);
    // Pas de fenêtre : J1, sans plus rien à jouer, a fini son tour.
    assert!(!two_may_react(&game));
    assert_eq!(game.current_player(), Player::Two);
    assert_eq!(game.reserve_pions(Player::Two), 14);
}

#[test]
fn a_charged_pion_is_not_targeted() {
    let bull = card(AttackKind::CoupDuTaureau, Rarity::Common);
    let mut game = game_with(rules(vec![bull], vec![feint(Rarity::Common)]));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );
    assert!(!two_may_react(&game));
    assert_eq!(game.board().owner(at(4, 6)), Some(Player::Two));
    assert_eq!(game.board().owner(at(4, 5)), Some(Player::One));
}

#[test]
fn the_owl_answers_a_swap() {
    let swap = card(AttackKind::Interchange, Rarity::Epic);
    let mut game = game_with(rich(vec![swap], vec![owl(Rarity::Common)]));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(0, 0)],
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::PionsSwapped { .. }))
    );
    assert!(two_may_react(&game));
}

#[test]
fn a_parrot_that_replays_scorched_earth_scorches_the_attacker() {
    let scorched_earth = card(AttackKind::Attila, Rarity::Legendary);
    let mut config = rules(vec![scorched_earth], vec![parrot(Rarity::Rare)]);
    // Terre brûlée à 2 points, pour que le Perroquet Rare puisse la rejouer.
    for entry in &mut config.attack_catalog {
        if entry.attack == scorched_earth {
            entry.stats.cost = 2;
        }
    }
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(0, 0), Action::EndTurn]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]);
    assert_eq!(game.scorched_until(Player::Two), Some(4));
    assert!(two_may_react(&game));
    play(&mut game, &[Action::UseAttack(0), choose(0, 0)]);
    // J1 est sous Terre brûlée pendant son prochain tour, le tour 5.
    assert_eq!(game.scorched_until(Player::One), Some(5));
    assert_eq!(game.scorched_until(Player::Two), Some(4));
}

#[test]
fn after_a_feint_passed_another_reaction_may_answer_the_same_attack() {
    let ours = card(AttackKind::Ours, Rarity::Rare);
    let mut game = face_to_face(ours, vec![feint(Rarity::Common), lizard(Rarity::Common)]);
    play(&mut game, &KILL_F5);
    assert!(two_may_react(&game));
    // J2 passe la Feintaille : F5 meurt, puis la Queue du lézard peut répondre à sa mort.
    play(&mut game, &[Action::Pass]);
    assert_eq!(game.board().owner(at(4, 5)), None);
    assert!(two_may_react(&game));
    assert_eq!(game.legal_actions(), [Action::UseAttack(1), Action::Pass]);
}

#[test]
fn after_a_feint_played_no_other_reaction_this_turn() {
    let ours = card(AttackKind::Ours, Rarity::Epic);
    let mut game = game_with(rich(
        vec![ours],
        vec![feint(Rarity::Rare), lizard(Rarity::Common)],
    ));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn]);
    play(&mut game, &[place(5, 5), Action::EndTurn]);
    // F5 esquive, F6 meurt : la Queue du lézard pourrait répondre, mais J2 a déjà réagi.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
            choose(5, 5),
        ],
    );
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(3, 6)],
    );
    assert_eq!(game.board().owner(at(5, 5)), None);
    assert!(!two_may_react(&game));
}

#[test]
fn the_other_targeted_pions_are_hit_normally() {
    let ours = card(AttackKind::Ours, Rarity::Epic);
    let mut game = game_with(rich(vec![ours], vec![feint(Rarity::Rare)]));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn]);
    play(&mut game, &[place(5, 5), Action::EndTurn]);
    // Tour 5 : l'Ours Épique vise F5 et F6 ; F5 esquive en G4, F6 meurt.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 5),
            choose(5, 5),
        ],
    );
    assert!(two_may_react(&game));
    play(&mut game, &[Action::UseAttack(0)]);
    assert_eq!(launchers(&game), [choose(4, 5), choose(5, 5)]);
    play(&mut game, &[choose(4, 5), choose(3, 6)]);
    assert_eq!(game.board().owner(at(3, 6)), Some(Player::Two));
    assert_eq!(game.board().owner(at(5, 5)), None);
}

#[test]
fn a_dodged_seduction_places_nothing() {
    let seduction = card(AttackKind::BeauteFatale, Rarity::Epic);
    let mut game = game_with(rich(vec![seduction], vec![feint(Rarity::Rare)]));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[place(4, 5), Action::EndTurn]);
    // Tour 3 : la Beauté fatale Épique de J1 séduit F5 ; F5 esquive en G5.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    );
    assert!(two_may_react(&game));
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 5), choose(4, 6)],
    );
    assert_eq!(game.board().owner(at(4, 6)), Some(Player::Two));
    assert_eq!(
        game.board().owner(at(4, 5)),
        None,
        "rien n'est posé à sa place"
    );
    assert_eq!(game.reserve_pions(Player::One), 14);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::PionPlaced { .. }))
    );
}
