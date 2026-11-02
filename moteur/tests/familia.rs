//! La Familia (B5d, règles précisées le 2026-11-02 dans le carnet) : parents posés depuis la
//! réserve, enfants protégés tant que les deux mêmes membres de leur famille les encadrent.
//!
//! Configuration : les premières règles, avec 4 actions à chaque tour (assez pour la
//! Légendaire) et aucune action gardée ; une pose coûte 1 action.

mod common;

use common::{choose, config_with_attacks, game_with, ours, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, EffectKind, Event, Game, GameConfig, GameError, PionId, Position,
    Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn familia(rarity: Rarity) -> Attack {
    card(AttackKind::Familia, rarity)
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

fn pion_at(game: &Game, pos: Position) -> PionId {
    game.board()
        .get(pos)
        .and_then(|cell| cell.pion())
        .unwrap()
        .id()
}

/// Les deux membres qui encadrent l'enfant de cette case, s'il en est un.
fn framed_by_at(game: &Game, pos: Position) -> Option<[PionId; 2]> {
    let pion = game.board().get(pos).and_then(|cell| cell.pion())?.id();
    game.active_effects()
        .iter()
        .find(|effect| effect.pion() == pion)
        .and_then(|effect| effect.framed_by())
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

/// J1 pose D5, E5 et F5 (E5 est encadré en ligne), J2 pose E6, collé aux trois.
fn d5_e5_f5_against_e6(player_one: Vec<Attack>, player_two: Vec<Attack>) -> Game {
    let mut game = game_with(config(player_one, player_two));
    play(
        &mut game,
        &[place(4, 3), place(4, 4), place(4, 5), Action::EndTurn],
    );
    play(&mut game, &[place(5, 4), Action::EndTurn]);
    game
}

#[test]
fn the_common_makes_the_launcher_a_child_of_the_two_pions_framing_it() {
    let mut game = d5_e5_f5_against_e6(vec![familia(Rarity::Common)], vec![ours(Rarity::Rare)]);
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]); // tour 3
    // La famille se choisit par un des deux pions qui encadrent.
    assert_eq!(choices(&game), [at(4, 3), at(4, 5)]);
    play(&mut game, &[choose(4, 3)]);

    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
    let framed_by = [pion_at(&game, at(4, 3)), pion_at(&game, at(4, 5))];
    assert_eq!(framed_by_at(&game, at(4, 4)), Some(framed_by));
    let child = game
        .active_effects()
        .iter()
        .find(|effect| effect.framed_by().is_some())
        .copied()
        .unwrap();
    assert_eq!(child.last_turn(), None);

    // L'Ours Rare adverse ne peut pas choisir l'enfant.
    play(
        &mut game,
        &[Action::EndTurn, Action::UseAttack(0), choose(5, 4)],
    ); // tour 4
    assert_eq!(choices(&game), [at(4, 3), at(4, 5)]);
}

#[test]
fn the_protection_falls_for_good_when_a_parent_is_killed() {
    let mut game = d5_e5_f5_against_e6(vec![familia(Rarity::Common)], vec![ours(Rarity::Rare)]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 3),
            Action::EndTurn,
        ],
    ); // tour 3
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(5, 4), choose(4, 3)],
    ); // tour 4
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EffectEnded(effect) if effect.framed_by().is_some()
    )));
    assert!(!game.has_effect(at(4, 4), EffectKind::Protected));

    // Tour 5 : un nouveau pion en D5 ne refait pas la famille.
    play(&mut game, &[Action::EndTurn, place(4, 3)]);
    assert!(!game.has_effect(at(4, 4), EffectKind::Protected));
}

#[test]
fn the_protection_falls_when_the_child_moves() {
    let hare = card(AttackKind::Lievre, Rarity::Common);
    let mut game = d5_e5_f5_against_e6(vec![familia(Rarity::Common), hare], Vec::new());
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 3),
            Action::EndTurn,
        ],
    ); // tour 3
    play(&mut game, &[Action::EndTurn]); // tour 4
    // Tour 5 : l'enfant fait un pas en E4, hors de sa famille.
    play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(4, 4),
            choose(3, 4),
            Action::FinishAttack,
        ],
    );
    assert!(!game.has_effect(at(3, 4), EffectKind::Protected));
    assert_eq!(framed_by_at(&game, at(3, 4)), None);
}

#[test]
fn the_rare_places_the_missing_parent_from_the_reserve() {
    let mut game = game_with(config(vec![familia(Rarity::Rare)], Vec::new()));
    play(&mut game, &[place(4, 3), place(4, 4), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn]);
    let reserve = game.reserve_pions(game.current_player());
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]); // tour 3
    // Seule F5 finit d'encadrer E5 avec D5.
    assert_eq!(choices(&game), [at(4, 5)]);
    play(&mut game, &[choose(4, 5)]);

    assert_eq!(game.reserve_pions(game.current_player()), reserve - 1);
    let framed_by = [pion_at(&game, at(4, 5)), pion_at(&game, at(4, 3))];
    assert_eq!(framed_by_at(&game, at(4, 4)), Some(framed_by));
}

#[test]
fn the_epic_places_two_parents_in_any_of_the_eight_directions() {
    let mut game = game_with(config(vec![familia(Rarity::Epic)], Vec::new()));
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn]);
    let reserve = game.reserve_pions(game.current_player());
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]); // tour 3
    assert_eq!(choices(&game).len(), 8);
    play(&mut game, &[choose(3, 3)]); // D4 : la diagonale D4-F6

    assert_eq!(game.reserve_pions(game.current_player()), reserve - 2);
    let framed_by = [pion_at(&game, at(3, 3)), pion_at(&game, at(5, 5))];
    assert_eq!(framed_by_at(&game, at(4, 4)), Some(framed_by));
}

#[test]
fn a_card_that_lacks_parents_in_reserve_cannot_be_played() {
    let config = GameConfig {
        pions_per_player: 2,
        ..config(
            vec![familia(Rarity::Epic), familia(Rarity::Rare)],
            Vec::new(),
        )
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]); // réserve : 1 pion
    play(&mut game, &[Action::EndTurn]);
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(familia(Rarity::Epic)))
    );
    // Tour 3 : D5 posé, F5 finirait d'encadrer E5, mais la réserve est vide.
    play(&mut game, &[place(4, 3)]);
    assert_eq!(game.reserve_pions(game.current_player()), 0);
    assert_eq!(
        game.apply(Action::UseAttack(1)),
        Err(GameError::NoTarget(familia(Rarity::Rare)))
    );
}

#[test]
fn the_legendary_makes_two_children_who_frame_each_other() {
    let mut game = game_with(config(
        vec![familia(Rarity::Legendary)],
        vec![ours(Rarity::Rare)],
    ));
    play(&mut game, &[place(4, 4), place(4, 5), Action::EndTurn]); // E5, F5
    play(&mut game, &[place(5, 7), Action::EndTurn]); // H6, collé à G5
    play(&mut game, &[Action::UseAttack(0), choose(4, 4)]); // tour 3
    assert_eq!(choices(&game), [at(4, 5)]);
    play(&mut game, &[choose(4, 5)]); // les parents en D5 et G5

    let [d5, e5, f5, g5] = [3, 4, 5, 6].map(|col| pion_at(&game, at(4, col)));
    assert_eq!(framed_by_at(&game, at(4, 4)), Some([d5, f5]));
    assert_eq!(framed_by_at(&game, at(4, 5)), Some([e5, g5]));

    // Tour 4 : G5 tué, F5 n'est plus encadré ; E5 l'est encore, par D5 et son frère.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(5, 7), choose(4, 6)],
    );
    assert!(!game.has_effect(at(4, 5), EffectKind::Protected));
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
}

#[test]
fn the_second_child_of_the_legendary_must_be_able_to_act() {
    // Une pose gratuite par tour : F5 se pose sans entamer les 4 actions de la carte.
    let config = GameConfig {
        summoning_sickness: true,
        place_cost: 0,
        max_placements_per_turn: Some(1),
        ..config(vec![familia(Rarity::Legendary)], Vec::new())
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]); // E5
    play(&mut game, &[Action::EndTurn]);
    play(&mut game, &[place(4, 5)]); // tour 3 : F5 vient d'arriver
    // F5 pourra agir plus tard : la carte attend un lanceur prêt, comme le Lièvre.
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoReadyLauncher(familia(Rarity::Legendary)))
    );
    play(&mut game, &[Action::EndTurn, Action::EndTurn]); // tours 3 et 4
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 5)],
    ); // tour 5
    assert!(game.has_effect(at(4, 5), EffectKind::Protected));
}

#[test]
fn a_child_does_not_become_a_child_again() {
    let mut game = d5_e5_f5_against_e6(vec![familia(Rarity::Common)], Vec::new());
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 3)],
    ); // tour 3
    // D4 et F6 encadrent aussi E5, en diagonale : c'est déjà un enfant, rien à faire.
    play(&mut game, &[place(3, 3), place(5, 5), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn]); // tour 4
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(familia(Rarity::Common)))
    );
}

#[test]
fn attila_common_removes_the_protection_of_an_enemy_child() {
    let attila = card(AttackKind::Attila, Rarity::Common);
    let mut game = d5_e5_f5_against_e6(vec![familia(Rarity::Common)], vec![attila]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 3),
            Action::EndTurn,
        ],
    ); // tour 3
    play(
        &mut game,
        &[Action::UseAttack(0), choose(5, 4), choose(4, 4)],
    ); // tour 4
    assert!(!game.has_effect(at(4, 4), EffectKind::Protected));

    // Tour 5 : E5 n'est plus un enfant, il peut le redevenir.
    play(
        &mut game,
        &[
            Action::EndTurn,
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 3),
        ],
    );
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
}

#[test]
fn the_pions_framing_the_common_child_may_have_just_arrived() {
    let config = GameConfig {
        summoning_sickness: true,
        ..config(vec![familia(Rarity::Common)], Vec::new())
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]); // E5
    play(&mut game, &[Action::EndTurn]);
    // Tour 3 : D5 et F5 viennent d'arriver, E5 devient quand même leur enfant.
    play(
        &mut game,
        &[
            place(4, 3),
            place(4, 5),
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 3),
        ],
    );
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
}

#[test]
fn the_parents_do_not_use_the_normal_placement() {
    // Une pose gratuite par tour.
    let config = GameConfig {
        place_cost: 0,
        max_placements_per_turn: Some(1),
        ..config(vec![familia(Rarity::Epic)], Vec::new())
    };
    let mut game = game_with(config);
    play(&mut game, &[place(4, 4), Action::EndTurn]);
    play(&mut game, &[Action::EndTurn]);
    assert_eq!(game.placements_left(), 1); // tour 3
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(3, 4)],
    );
    assert_eq!(game.placements_left(), 1);
}

#[test]
fn the_legendary_never_takes_a_child_as_its_second_child() {
    let mut game = game_with(config(
        vec![familia(Rarity::Common), familia(Rarity::Legendary)],
        Vec::new(),
    ));
    // Tour 1 : E4, E5, E6 en colonne, et D5.
    play(
        &mut game,
        &[place(3, 4), place(4, 4), place(5, 4), place(4, 3)],
    );
    play(&mut game, &[Action::EndTurn]); // tour 2
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(3, 4),
            Action::EndTurn,
        ],
    ); // tour 3 : E5 devient enfant d'E4 et E6
    play(&mut game, &[Action::EndTurn]); // tour 4
    // Tour 5 : depuis D5, E5 (déjà enfant) n'est pas proposé ; E4 et E6 le sont.
    play(&mut game, &[Action::UseAttack(1), choose(4, 3)]);
    assert_eq!(choices(&game), [at(3, 4), at(5, 4)]);
}

#[test]
fn a_queen_parent_that_steps_away_breaks_the_family() {
    let queen = card(AttackKind::Reine, Rarity::Rare);
    let mut game = game_with(config(vec![queen, familia(Rarity::Common)], Vec::new()));
    play(&mut game, &[place(4, 4), place(4, 5), Action::EndTurn]); // E5, F5
    play(&mut game, &[Action::EndTurn]);
    // Tour 3 : la reine en D5, puis E5 devient l'enfant de la reine et de F5.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(4, 3),
            Action::UseAttack(1),
            choose(4, 4),
            choose(4, 3),
            Action::EndTurn,
        ],
    );
    assert!(game.has_effect(at(4, 4), EffectKind::Protected));
    play(&mut game, &[Action::EndTurn]); // tour 4
    play(&mut game, &[Action::StepQueen(at(3, 3))]); // tour 5
    assert!(!game.has_effect(at(4, 4), EffectKind::Protected));
}

#[test]
fn a_child_who_leaves_its_family_does_not_escape_a_vengeance() {
    let wolf = card(AttackKind::SautDuLoup, Rarity::Rare);
    let friends = card(AttackKind::MeilleursAmis, Rarity::Epic);
    let mut game = game_with(config(vec![familia(Rarity::Common), wolf], vec![friends]));
    play(
        &mut game,
        &[place(4, 3), place(4, 4), place(4, 5), Action::EndTurn],
    );
    // Tour 2 : J2 lie par un pacte E6 et J1 (A10 reste son lanceur).
    play(
        &mut game,
        &[
            place(9, 0),
            Action::UseAttack(0),
            choose(9, 0),
            choose(5, 4),
            choose(0, 9),
        ],
    );
    // Tour 3 : E5 devient enfant, puis saute par-dessus E6 (tué) jusqu'en E7, hors de sa
    // famille. L'ami en J1 le venge : E5 n'est plus protégé.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(4, 4), choose(4, 3)],
    );
    let events = play(
        &mut game,
        &[Action::UseAttack(1), choose(4, 4), choose(6, 4)],
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::PionKilled { at: killed_at, .. } if *killed_at == at(6, 4)
    )));
    assert_eq!(game.board().owner(at(6, 4)), None);
}

#[test]
fn during_an_attack_a_child_keeps_its_protection_until_the_attack_ends() {
    let wolf = card(AttackKind::SautDuLoup, Rarity::Epic);
    let mut game = game_with(config(vec![familia(Rarity::Common)], vec![wolf]));
    // Tour 1 : E4, E5, E6 en colonne.
    play(
        &mut game,
        &[place(3, 4), place(4, 4), place(5, 4), Action::EndTurn],
    );
    play(&mut game, &[place(3, 5), Action::EndTurn]); // tour 2 : F4
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(4, 4),
            choose(3, 4),
            Action::EndTurn,
        ],
    ); // tour 3
    // Tour 4 : le loup survole E4 (tué) jusqu'en D4, puis E5 jusqu'en F6. L'attaque est
    // calculée sur la situation du début : E5, encore protégé, survit ; sa protection tombe
    // à la fin de l'attaque.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(3, 5),
            choose(3, 3),
            choose(5, 5),
        ],
    );
    assert_eq!(game.board().owner(at(3, 4)), None);
    assert!(game.board().owner(at(4, 4)).is_some());
    assert!(!game.has_effect(at(4, 4), EffectKind::Protected));
}
