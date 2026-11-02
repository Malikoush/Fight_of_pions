//! Le Massacre divin (B6b, règles précisées le 2026-11-02 dans le carnet) : des piles de
//! jetons devant le joueur, usées par ses prochains kills ; des points rendus, le
//! cimetière, la défaite par manque de pions, et Attila qui annule les jetons.
//!
//! Configuration : les premières règles (sans infirmerie, une pose coûte 1 action), avec
//! 8 actions à chaque tour et aucune action gardée, sauf mention contraire.

mod common;

use common::{choose, config_with_attacks, game_with, place, play};
use fight_of_pions::{
    Action, Attack, AttackKind, EffectKind, EndReason, Event, Game, GameConfig, GameEnd, GameError,
    Outcome, Player, Position, Rarity, TokenReward,
};

fn at(row: usize, col: usize) -> Position {
    Position { row, col }
}

fn card(kind: AttackKind, rarity: Rarity) -> Attack {
    Attack { kind, rarity }
}

fn massacre(rarity: Rarity) -> Attack {
    card(AttackKind::MassacreDivin, rarity)
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

/// Les jetons qui restent dans les piles de ce joueur, dans l'ordre des piles.
fn tokens_of(game: &Game, player: Player) -> Vec<u32> {
    game.token_piles()
        .iter()
        .filter(|pile| pile.owner() == player)
        .map(|pile| pile.tokens())
        .collect()
}

/// Lance l'attaque de cet emplacement depuis ce pion, sans cible (le Massacre divin).
fn launch(slot: usize, from: Position) -> [Action; 2] {
    [Action::UseAttack(slot), Action::Choose(from)]
}

/// Au tour 3, J1 a un pion en A1, un autre en F6, et J2 un pion collé en G6. J1 a posé
/// ses piles au tour 1 avec les emplacements `piles` ; son Ours Rare est le dernier de
/// son arsenal.
fn ready_to_kill(
    mut attacks: Vec<Attack>,
    piles: &[usize],
    config_of: impl Fn(GameConfig) -> GameConfig,
) -> Game {
    let bear = attacks.len();
    attacks.push(card(AttackKind::Ours, Rarity::Rare));
    let mut game = game_with(config_of(config(attacks, Vec::new())));
    play(&mut game, &[place(0, 0), place(5, 5)]);
    for &slot in piles {
        play(&mut game, &launch(slot, at(0, 0)));
    }
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(&mut game, &[place(5, 6), Action::EndTurn]); // tour 2
    assert_eq!(
        game.arsenal(Player::One)[bear].attack().kind,
        AttackKind::Ours
    );
    game
}

/// L'Ours Rare de J1 (dernier emplacement) tue G6 depuis F6.
fn kill_g6(game: &mut Game) -> Vec<Event> {
    let bear = game.arsenal(Player::One).len() - 1;
    play(game, &[Action::UseAttack(bear), choose(5, 5), choose(5, 6)])
}

#[test]
fn each_rarity_lays_its_own_pile() {
    let all = vec![
        massacre(Rarity::Common),
        massacre(Rarity::Rare),
        massacre(Rarity::Legendary),
        massacre(Rarity::Epic),
    ];
    let mut game = game_with(config(all, Vec::new()));
    play(&mut game, &[place(0, 0)]);
    for slot in 0..3 {
        play(&mut game, &launch(slot, at(0, 0)));
    } // tour 1 : 1 + 1 + 2 + 4 actions, le tour finit seul
    play(&mut game, &[Action::EndTurn]); // tour 2
    play(&mut game, &launch(3, at(0, 0))); // tour 3
    let piles: Vec<(TokenReward, u32)> = game
        .token_piles()
        .iter()
        .map(|pile| (pile.reward(), pile.tokens()))
        .collect();
    assert_eq!(
        piles,
        [
            (TokenReward::Points(2), 1),
            (TokenReward::Points(1), 4),
            (TokenReward::Graveyard, 3),
            (TokenReward::Graveyard, 2),
        ]
    );
}

#[test]
fn the_common_gives_two_points_to_the_next_kill_only() {
    let mut game = ready_to_kill(vec![massacre(Rarity::Common)], &[0], |config| config);
    kill_g6(&mut game); // tour 3 : l'Ours coûte 2, le kill en rend 2
    assert_eq!(game.actions_left(), 8);
    assert_eq!(tokens_of(&game, Player::One), Vec::<u32>::new());
    // Un kill ordinaire : sans infirmerie, le pion retourne dans la réserve.
    assert!(game.graveyard().is_empty());
    assert_eq!(game.reserve_pions(Player::Two), 15);
}

#[test]
fn the_rare_gives_one_point_for_each_kill_even_in_the_same_attack() {
    let config = config(
        vec![massacre(Rarity::Rare), card(AttackKind::Ours, Rarity::Epic)],
        Vec::new(),
    );
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0), place(5, 5)]);
    play(&mut game, &launch(0, at(0, 0)));
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(&mut game, &[place(4, 4), place(4, 5), Action::EndTurn]); // tour 2
    let events = play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(5, 5),
            choose(4, 4),
            choose(4, 5),
        ],
    ); // tour 3 : deux kills d'un coup
    let used = events
        .iter()
        .filter(|event| matches!(event, Event::TokenUsed(_)))
        .count();
    assert_eq!(used, 2);
    assert_eq!(tokens_of(&game, Player::One), [2]);
    assert_eq!(game.actions_left(), 8 - 3 + 2);
}

#[test]
fn the_epic_sends_the_killed_pion_to_the_graveyard_for_good() {
    for infirmary_turns in [0, 2] {
        let mut game = ready_to_kill(vec![massacre(Rarity::Epic)], &[0], |config| GameConfig {
            infirmary_turns,
            ..config
        });
        let events = kill_g6(&mut game); // tour 3
        let dead = game.graveyard();
        assert_eq!(dead.len(), 1, "infirmerie {infirmary_turns}");
        assert_eq!(dead[0].owner(), Player::Two);
        assert!(events.contains(&Event::GraveyardEntered(dead[0])));
        // Ni infirmerie, ni tombe, ni retour dans la réserve.
        assert!(game.infirmary().is_empty());
        assert_eq!(game.board().tomb(at(5, 6)), None);
        assert_eq!(game.reserve_pions(Player::Two), 14);
        assert_eq!(game.living_pions(Player::Two), 14);
        assert_eq!(tokens_of(&game, Player::One), [1]);
        // Il ne revient jamais, même après les tours d'infirmerie.
        for _ in 0..4 {
            play(&mut game, &[Action::EndTurn]);
        }
        assert_eq!(game.reserve_pions(Player::Two), 14);
    }
}

#[test]
fn piles_add_up_and_each_kill_uses_a_token_of_each() {
    let piles = vec![massacre(Rarity::Common), massacre(Rarity::Epic)];
    let mut game = ready_to_kill(piles, &[0, 1], |config| config);
    kill_g6(&mut game); // tour 3
    // 2 points et le cimetière, pour le même kill.
    assert_eq!(game.actions_left(), 8);
    assert_eq!(game.graveyard().len(), 1);
    assert_eq!(tokens_of(&game, Player::One), [1]);
}

#[test]
fn a_kill_during_the_enemy_turn_gives_points_to_a_capped_bank() {
    let config = GameConfig {
        max_banked_actions: 2,
        ..config(
            vec![
                massacre(Rarity::Common),
                card(AttackKind::MeilleursAmis, Rarity::Epic),
            ],
            vec![card(AttackKind::Ours, Rarity::Rare)],
        )
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &launch(0, at(0, 0)));
    play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(0, 0),
            choose(5, 5),
            choose(9, 9),
        ],
    );
    play(&mut game, &[place(2, 2), place(2, 3), Action::EndTurn]); // tour 1 : 1 action gardée
    assert_eq!(game.banked_actions(Player::One), 1);
    // Tour 2 : J2 tue l'ami en F6 ; l'ami en J10 le venge, un kill de J1.
    play(
        &mut game,
        &[
            place(5, 6),
            Action::UseAttack(0),
            choose(5, 6),
            choose(5, 5),
        ],
    );
    assert_eq!(game.board().owner(at(5, 6)), None);
    assert_eq!(game.current_player(), Player::Two);
    assert_eq!(game.banked_actions(Player::One), 2);
    assert_eq!(tokens_of(&game, Player::One), Vec::<u32>::new());
}

#[test]
fn a_queen_saved_by_her_guard_uses_no_token() {
    let mut attacks = vec![massacre(Rarity::Common)];
    attacks.push(card(AttackKind::Ours, Rarity::Rare));
    let queen = card(AttackKind::Reine, Rarity::Epic);
    let mut game = game_with(config(attacks, vec![queen]));
    play(&mut game, &[place(0, 0), place(5, 5)]);
    play(&mut game, &launch(0, at(0, 0)));
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(
        &mut game,
        &[
            place(9, 9),
            Action::UseAttack(0),
            choose(9, 9),
            choose(5, 6),
            Action::EndTurn,
        ],
    ); // tour 2 : une reine gardée en G6
    play(
        &mut game,
        &[Action::UseAttack(1), choose(5, 5), choose(5, 6)],
    ); // tour 3
    assert_eq!(game.queen_of(Player::Two), Some(at(5, 6)));
    assert_eq!(tokens_of(&game, Player::One), [1]);
    assert_eq!(game.actions_left(), 8 - 2);
}

#[test]
fn falling_under_six_pions_through_the_graveyard_loses_at_once() {
    let mut game = ready_to_kill(vec![massacre(Rarity::Epic)], &[0], |config| GameConfig {
        pions_per_player: 6,
        ..config
    });
    let events = kill_g6(&mut game); // tour 3 : J2 n'a plus que 5 pions
    assert_eq!(game.living_pions(Player::Two), 5);
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded(GameEnd {
            outcome: Outcome::Won(Player::One),
            reason: EndReason::TooFewPions,
        }))
    );
}

#[test]
fn both_players_under_six_pions_in_the_same_attack_is_a_draw() {
    let config = GameConfig {
        pions_per_player: 6,
        ..config(
            vec![massacre(Rarity::Epic), card(AttackKind::Ours, Rarity::Rare)],
            vec![
                massacre(Rarity::Epic),
                card(AttackKind::MeilleursAmis, Rarity::Epic),
            ],
        )
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0), place(5, 5)]);
    play(&mut game, &launch(0, at(0, 0)));
    play(&mut game, &[Action::EndTurn]); // tour 1
    play(&mut game, &[place(9, 9)]);
    play(&mut game, &launch(0, at(9, 9)));
    play(
        &mut game,
        &[
            Action::UseAttack(1),
            choose(9, 9),
            choose(5, 6),
            choose(0, 9),
            Action::EndTurn,
        ],
    ); // tour 2 : les amis de J2 en G6 et J1
    // Tour 3 : J1 tue l'ami en G6 (cimetière) ; l'ami en J1 tue F6 (cimetière aussi).
    let events = play(
        &mut game,
        &[Action::UseAttack(1), choose(5, 5), choose(5, 6)],
    );
    assert_eq!(game.graveyard().len(), 2);
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded(GameEnd {
            outcome: Outcome::Draw,
            reason: EndReason::TooFewPions,
        }))
    );
}

#[test]
fn attila_rare_cancels_every_enemy_pile_but_the_common_cannot() {
    let mut game = game_with(config(
        vec![
            card(AttackKind::Attila, Rarity::Common),
            card(AttackKind::Attila, Rarity::Rare),
        ],
        vec![massacre(Rarity::Common), massacre(Rarity::Epic)],
    ));
    play(&mut game, &[place(0, 0), Action::EndTurn]); // tour 1
    play(&mut game, &[place(9, 9)]);
    play(&mut game, &launch(0, at(9, 9)));
    play(&mut game, &launch(1, at(9, 9)));
    play(&mut game, &[Action::EndTurn]); // tour 2
    assert_eq!(tokens_of(&game, Player::Two), [1, 2]);
    // Les jetons ne sont pas des statuts : la Commune n'a rien à retirer.
    assert_eq!(
        game.apply(Action::UseAttack(0)),
        Err(GameError::NoTarget(card(
            AttackKind::Attila,
            Rarity::Common
        )))
    );
    play(&mut game, &[Action::UseAttack(1), choose(0, 0)]);
    assert!(game.legal_actions().contains(&Action::ChooseEnemyTokens));
    play(&mut game, &[Action::ChooseEnemyTokens]); // tour 3 : l'attaque est jouée
    assert!(!game.attack_in_progress());
    assert!(game.token_piles().is_empty());
}

#[test]
fn attila_epic_cancels_the_enemy_tokens_and_a_pion() {
    let mut game = game_with(config(
        vec![card(AttackKind::Attila, Rarity::Epic)],
        vec![
            massacre(Rarity::Common),
            card(AttackKind::Ours, Rarity::Common),
        ],
    ));
    play(&mut game, &[place(0, 0), place(5, 5), Action::EndTurn]); // tour 1
    play(&mut game, &[place(5, 6)]);
    play(&mut game, &launch(0, at(5, 6)));
    play(
        &mut game,
        &[Action::UseAttack(1), choose(5, 6), Action::EndTurn],
    ); // tour 2 : F6 grogné
    assert!(game.has_effect(at(5, 5), EffectKind::Immobilized));
    play(
        &mut game,
        &[
            Action::UseAttack(0),
            choose(0, 0),
            Action::ChooseEnemyTokens,
        ],
    );
    // Les jetons comptent pour une cible : le second pion reste à choisir.
    assert!(game.attack_in_progress());
    // Une seule fois par attaque.
    refused(&mut game, Action::ChooseEnemyTokens);
    play(&mut game, &[choose(5, 5)]); // tour 3
    assert!(game.token_piles().is_empty());
    assert!(!game.has_effect(at(5, 5), EffectKind::Immobilized));
}

/// Refusé, et la partie ne change pas.
fn refused(game: &mut Game, action: Action) {
    let before = game.clone();
    assert_eq!(
        game.apply(action),
        Err(GameError::UnexpectedAction),
        "{action:?}"
    );
    assert_eq!(*game, before);
}

#[test]
fn the_enemy_tokens_can_only_be_chosen_by_attila_rare_or_epic_when_there_are_some() {
    let mut game = game_with(config(
        vec![
            card(AttackKind::Attila, Rarity::Rare),
            card(AttackKind::Attila, Rarity::Epic),
            card(AttackKind::Attila, Rarity::Common),
            card(AttackKind::Ours, Rarity::Rare),
        ],
        vec![
            massacre(Rarity::Common),
            card(AttackKind::Ours, Rarity::Common),
        ],
    ));
    play(
        &mut game,
        &[place(0, 0), place(5, 5), place(4, 6), Action::EndTurn],
    ); // tour 1
    play(&mut game, &[place(5, 6)]);
    play(&mut game, &launch(0, at(5, 6)));
    play(
        &mut game,
        &[Action::UseAttack(1), choose(5, 6), Action::EndTurn],
    ); // tour 2 : F6 et G5 grognés, une pile adverse
    // Tour 3. Hors attaque, puis avant le lanceur.
    refused(&mut game, Action::ChooseEnemyTokens);
    play(&mut game, &[Action::UseAttack(0)]);
    refused(&mut game, Action::ChooseEnemyTokens);
    play(&mut game, &[choose(0, 0), Action::ChooseEnemyTokens]); // Attila Rare
    assert!(game.token_piles().is_empty());
    // Attila Commune ne vise que des statuts.
    play(&mut game, &[Action::UseAttack(2), choose(0, 0)]);
    refused(&mut game, Action::ChooseEnemyTokens);
    play(&mut game, &[choose(5, 5)]);
    // L'adversaire n'a plus de pile : l'Épique ne peut viser que G5.
    play(&mut game, &[Action::UseAttack(1), choose(0, 0)]);
    refused(&mut game, Action::ChooseEnemyTokens);
    play(&mut game, &[choose(4, 6)]);
    // Une autre famille ne vise jamais de jetons.
    play(&mut game, &[Action::UseAttack(3), choose(5, 5)]);
    refused(&mut game, Action::ChooseEnemyTokens);
    play(&mut game, &[choose(5, 6)]);
}

#[test]
fn points_beyond_the_bank_are_lost_only_at_the_end_of_the_turn() {
    let mut game = ready_to_kill(vec![massacre(Rarity::Common)], &[0], |config| GameConfig {
        max_banked_actions: 3,
        ..config
    }); // J1 a gardé 3 actions au tour 1
    kill_g6(&mut game); // tour 3 : 8 + 3 - 2 + 2
    assert_eq!(game.actions_left(), 11);
    play(&mut game, &[Action::EndTurn]);
    assert_eq!(game.banked_actions(Player::One), 3);
}
