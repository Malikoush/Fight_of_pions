//! Le Coup du taureau et le mur (B7, règles précisées le 2026-10-30 et le 2026-11-03 dans
//! le carnet) : une charge en ligne droite qui pousse, écrase ou tue, et creuse le mur.
//!
//! Configuration : les premières règles (sans infirmerie, une pose coûte 1 action), avec
//! 8 actions à chaque tour, aucune action gardée et 2 rangées de mur. La grille fait
//! 14 × 14 : le plateau de jeu va des rangées et colonnes 2 à 11 (C3 à L12).

mod common;

use common::{choose, config_with_attacks, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, Event, Game, GameConfig, GameError, Player, Position, Rarity,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn bull(rarity: Rarity) -> Attack {
    Attack {
        kind: AttackKind::CoupDuTaureau,
        rarity,
    }
}

/// Chaque joueur commence avec ces attaques, a 8 actions à chaque tour, et le plateau a
/// `wall` rangées de mur.
fn config(player_one: Vec<Attack>, player_two: Vec<Attack>, wall: usize) -> GameConfig {
    GameConfig {
        first_turn_actions: 8,
        actions_per_turn: 8,
        max_banked_actions: 0,
        wall_thickness: wall,
        ..config_with_attacks(player_one, player_two)
    }
}

/// J1 pose ses pions au tour 1, J2 les siens au tour 2 : c'est au tour de J1 (tour 3), qui
/// a le Coup du taureau dans cette rareté.
fn charge_ready(rarity: Rarity, ones: &[Position], twos: &[Position]) -> Game {
    let mut game = game_with(config(vec![bull(rarity)], Vec::new(), 2));
    for &pos in ones {
        play(&mut game, &[Action::Place(pos)]);
    }
    play(&mut game, &[Action::EndTurn]);
    for &pos in twos {
        play(&mut game, &[Action::Place(pos)]);
    }
    play(&mut game, &[Action::EndTurn]);
    game
}

/// Les directions proposées au lanceur : les cases à choisir.
fn directions(game: &mut Game, launcher: Position) -> Vec<Position> {
    play(game, &[Action::UseAttack(0), Action::Choose(launcher)]);
    let mut cells: Vec<Position> = game
        .legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::Choose(pos) => Some(pos),
            _ => None,
        })
        .collect();
    cells.sort_by_key(|pos| (pos.row, pos.col));
    cells
}

fn owner(game: &Game, row: usize, col: usize) -> Option<Player> {
    game.board().owner(at(row, col))
}

#[test]
fn the_wall_is_not_part_of_the_board() {
    let mut game = game_with(config(Vec::new(), Vec::new(), 2));
    assert_eq!(game.board().size(), 14);
    assert!(game.board().is_wall(at(0, 0)) && game.board().is_wall(at(2, 12)));
    assert!(!game.board().contains(at(1, 5)));
    assert_eq!(game.board().positions().count(), 100);
    assert!(!game.legal_actions().contains(&place(1, 5)));
    assert_eq!(
        game.apply(place(1, 5)),
        Err(GameError::OutOfBounds(at(1, 5)))
    );
    // Le premier coin du plateau de jeu s'appelle C3.
    assert_eq!(at(2, 2).to_string(), "C3");
}

#[test]
fn the_wall_stops_a_jump() {
    let wolf = Attack {
        kind: AttackKind::SautDuLoup,
        rarity: Rarity::Rare,
    };
    let mut game = game_with(config(vec![wolf], Vec::new(), 2));
    play(&mut game, &[place(2, 3), place(2, 2)]);
    // Depuis D3, sauter par-dessus C3 atterrirait dans le mur : pas proposé.
    play(&mut game, &[Action::UseAttack(0), choose(2, 3)]);
    assert!(!game.legal_actions().contains(&choose(2, 1)));
    assert!(game.legal_actions().contains(&choose(4, 3)));
}

#[test]
fn the_common_pushes_one_cell_and_takes_the_place() {
    let mut game = charge_ready(Rarity::Common, &[at(6, 6)], &[at(6, 7)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    assert_eq!(owner(&game, 6, 6), None);
    assert_eq!(owner(&game, 6, 7), Some(Player::One));
    assert_eq!(owner(&game, 6, 8), Some(Player::Two));
    let moves: Vec<(Position, Position)> = events
        .iter()
        .filter_map(|event| match event {
            Event::PionMoved { from, to, .. } => Some((*from, *to)),
            _ => None,
        })
        .collect();
    // Le pion adverse recule d'abord, puis le lanceur prend sa place.
    assert_eq!(moves, vec![(at(6, 7), at(6, 8)), (at(6, 6), at(6, 7))]);
}

#[test]
fn the_common_cannot_push_against_a_pion_or_the_wall() {
    // G7 ne peut pas reculer (H7 est à J2) ; L8 est contre le mur ; J9 est à J1.
    let ones = [at(6, 5), at(7, 10), at(8, 9)];
    let twos = [at(6, 6), at(6, 7), at(7, 11)];
    let mut game = charge_ready(Rarity::Common, &ones, &twos);
    assert!(!directions(&mut game, at(6, 5)).contains(&at(6, 6)));
    let mut game = charge_ready(Rarity::Common, &ones, &twos);
    let offered = directions(&mut game, at(7, 10));
    assert!(!offered.contains(&at(7, 11)));
    assert_eq!(
        game.apply(choose(7, 11)),
        Err(GameError::InvalidChoice(at(7, 11)))
    );
    // Une case libre laisse passer la charge ; un de tes pions l'arrête.
    assert!(offered.contains(&at(6, 10)));
    assert!(!offered.contains(&at(8, 9)));
}

#[test]
fn the_launcher_digs_the_wall_and_stops_at_the_edge_of_the_grid() {
    let mut game = charge_ready(Rarity::Legendary, &[at(6, 11)], &[at(9, 9)]);
    // Vers l'est, la charge entre dans le mur : la case M7 est proposée.
    assert!(directions(&mut game, at(6, 11)).contains(&at(6, 12)));
    let events = play(&mut game, &[choose(6, 12)]);
    // Charge de 3 cases, mais seulement 2 rangées de mur : elle s'arrête au bord de la
    // grille, sur la dernière case creusée.
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::WallDug { .. }))
            .count(),
        2
    );
    assert!(events.contains(&Event::WallDug { at: at(6, 12) }));
    assert_eq!(owner(&game, 6, 13), Some(Player::One));
    // Les cases creusées sont des cases normales, pour toujours : J2 peut y poser.
    for col in [12, 13] {
        assert!(!game.board().is_wall(at(6, col)) && game.board().contains(at(6, col)));
    }
    play(&mut game, &[Action::EndTurn]);
    play(&mut game, &[place(6, 12)]);
    assert_eq!(owner(&game, 6, 12), Some(Player::Two));
}

#[test]
fn a_pushed_pion_does_not_dig() {
    // Rare : K7 ne recule que d'une case avant le mur, il est écrasé.
    let mut game = charge_ready(Rarity::Rare, &[at(6, 9)], &[at(6, 10)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 9), choose(6, 10)],
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::WallDug { .. }))
    );
    assert!(events.iter().any(
        |event| matches!(event, Event::PionKilled { at: killed, .. } if *killed == at(6, 10))
    ));
    assert_eq!(owner(&game, 6, 10), Some(Player::One));
    assert!(game.board().is_wall(at(6, 12)));
}

#[test]
fn the_rare_pushes_two_cells() {
    let mut game = charge_ready(Rarity::Rare, &[at(6, 6)], &[at(6, 7)]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    assert_eq!(owner(&game, 6, 7), Some(Player::One));
    assert_eq!(owner(&game, 6, 8), None);
    assert_eq!(owner(&game, 6, 9), Some(Player::Two));
}

#[test]
fn the_rare_crushes_a_pion_that_cannot_back_off_two_cells() {
    let mut game = charge_ready(Rarity::Rare, &[at(6, 6)], &[at(6, 7), at(6, 9)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    assert!(
        events.iter().any(
            |event| matches!(event, Event::PionKilled { at: killed, .. } if *killed == at(6, 7))
        )
    );
    assert_eq!(owner(&game, 6, 7), Some(Player::One));
    assert_eq!(owner(&game, 6, 8), None);
    assert_eq!(owner(&game, 6, 9), Some(Player::Two));
}

/// J2 protège H7 avec le Garde royal (Reine Commune) au tour 2 ; ses autres pions sont
/// posés avant.
fn protected_h7(rarity: Rarity, others: &[Position]) -> Game {
    let royal_guard = Attack {
        kind: AttackKind::Reine,
        rarity: Rarity::Common,
    };
    let mut game = game_with(config(vec![bull(rarity)], vec![royal_guard], 2));
    play(&mut game, &[place(6, 6), Action::EndTurn]);
    play(&mut game, &[place(6, 7)]);
    for &pos in others {
        play(&mut game, &[Action::Place(pos)]);
    }
    play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 7), Action::EndTurn],
    );
    game
}

#[test]
fn the_rare_pushes_a_protected_pion_as_far_as_it_can() {
    let mut game = protected_h7(Rarity::Rare, &[at(6, 9)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::PionKilled { .. }))
    );
    assert_eq!(owner(&game, 6, 7), Some(Player::One));
    assert_eq!(owner(&game, 6, 8), Some(Player::Two));
    // Sans aucune place pour reculer, la charge est impossible dans cette direction.
    let mut game = protected_h7(Rarity::Rare, &[at(6, 8)]);
    assert!(!directions(&mut game, at(6, 6)).contains(&at(6, 7)));
}

#[test]
fn the_common_pushes_a_protected_pion_too() {
    let mut game = protected_h7(Rarity::Common, &[]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    assert_eq!(owner(&game, 6, 7), Some(Player::One));
    assert_eq!(owner(&game, 6, 8), Some(Player::Two));
}

#[test]
fn the_epic_kills_every_enemy_on_its_way() {
    let mut game = charge_ready(Rarity::Epic, &[at(6, 6)], &[at(6, 7), at(6, 8)]);
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    let killed = events
        .iter()
        .filter(|event| matches!(event, Event::PionKilled { .. }))
        .count();
    assert_eq!(killed, 2);
    assert_eq!(owner(&game, 6, 8), Some(Player::One));
    assert_eq!(game.reserve_pions(Player::Two), 15);
}

#[test]
fn the_legendary_charges_three_cells() {
    let twos = [at(6, 7), at(6, 8), at(6, 9), at(6, 10)];
    let mut game = charge_ready(Rarity::Legendary, &[at(6, 6)], &twos);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    assert_eq!(owner(&game, 6, 9), Some(Player::One));
    // Au-delà de 3 cases, le pion adverse n'est pas touché.
    assert_eq!(owner(&game, 6, 10), Some(Player::Two));
    assert_eq!(game.reserve_pions(Player::Two), 14);
}

#[test]
fn the_epic_stops_before_a_protected_pion_or_its_own() {
    // H7 protégé arrête la charge dès la première case : direction impossible.
    let mut game = protected_h7(Rarity::Epic, &[]);
    assert!(!directions(&mut game, at(6, 6)).contains(&at(6, 7)));
    // Ton propre pion en I7 : le lanceur s'arrête devant, sur H7.
    let mut game = charge_ready(Rarity::Legendary, &[at(6, 6), at(6, 8)], &[at(6, 7)]);
    play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    assert_eq!(owner(&game, 6, 7), Some(Player::One));
    assert_eq!(owner(&game, 6, 8), Some(Player::One));
    assert_eq!(owner(&game, 6, 6), None);
}

#[test]
fn a_guard_saves_the_queen_and_stops_the_charge() {
    let queen = Attack {
        kind: AttackKind::Reine,
        rarity: Rarity::Epic,
    };
    let mut game = game_with(config(vec![bull(Rarity::Legendary)], vec![queen], 2));
    play(&mut game, &[place(6, 6), Action::EndTurn]);
    // J2 couronne une reine gardée en H7, avec un pion derrière elle en I7.
    play(&mut game, &[place(9, 9), place(6, 8)]);
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(9, 9),
            choose(6, 7),
            Action::EndTurn,
        ],
    );
    assert_eq!(game.queen_of(Player::Two), Some(at(6, 7)));
    let events = play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 6), choose(6, 7)],
    );
    // La reine survit, la charge s'arrête : le lanceur ne bouge pas, I7 n'est pas touché.
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::PionKilled { .. } | Event::PionMoved { .. }))
    );
    assert_eq!(game.queen_of(Player::Two), Some(at(6, 7)));
    assert_eq!(owner(&game, 6, 6), Some(Player::One));
    assert_eq!(owner(&game, 6, 8), Some(Player::Two));
}

#[test]
fn without_a_wall_the_edge_stops_the_charge() {
    let mut game = game_with(config(vec![bull(Rarity::Common)], Vec::new(), 0));
    play(&mut game, &[place(0, 5)]);
    let offered = directions(&mut game, at(0, 5));
    assert_eq!(
        offered,
        vec![at(0, 4), at(0, 6), at(1, 4), at(1, 5), at(1, 6)]
    );
}

#[test]
fn a_tomb_does_not_stop_the_charge() {
    let config = GameConfig {
        infirmary_turns: 3,
        ..config(vec![bull(Rarity::Legendary)], Vec::new(), 2)
    };
    let mut game = game_with(config);
    play(&mut game, &[place(6, 6), Action::EndTurn]);
    play(&mut game, &[place(6, 7), Action::EndTurn]);
    // Tour 3 : la charge tue H7, qui y laisse sa tombe, et finit en J7.
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(6, 6),
            choose(6, 7),
            Action::EndTurn,
        ],
    );
    assert!(game.board().tomb(at(6, 7)).is_some());
    assert_eq!(owner(&game, 6, 9), Some(Player::One));
    play(&mut game, &[Action::EndTurn]); // tour 4
    // Tour 5 : la charge revient vers l'ouest, par-dessus la tombe, jusqu'en G7.
    play(
        &mut game,
        &[Action::UseAttack(0), choose(6, 9), choose(6, 8)],
    );
    assert_eq!(owner(&game, 6, 6), Some(Player::One));
    assert!(game.board().tomb(at(6, 7)).is_some());
}
