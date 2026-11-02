//! Règles de fin de partie (carnet de règles, « Fin de partie »).

mod common;

use common::{game_with, new_game, place, play, random_config};
use fight_of_pions::{
    Action, Attack, AttackKind, Decision, Direction, EffectKind, EndReason, Event, Game,
    GameConfig, GameEnd, Outcome, Player, Position, Rarity, Rng,
};

fn draw(reason: EndReason) -> Event {
    Event::GameEnded(GameEnd {
        outcome: Outcome::Draw,
        reason,
    })
}

/// Vérifie qu'une réponse est exactement « un pion posé, sa réserve et ses actions
/// décomptées, plus aucune pose, puis la fin » : aucun `TurnEnded` ne doit se glisser
/// avant la fin, même si la pose a vidé les actions du tour.
fn assert_placed_then(events: &[Event], end: Event) {
    assert_eq!(events.len(), 5, "événements inattendus : {events:?}");
    assert!(matches!(events[0], Event::PionPlaced { .. }));
    assert!(matches!(events[1], Event::ReserveChanged { .. }));
    assert!(matches!(events[2], Event::ActionsChanged { .. }));
    assert!(matches!(
        events[3],
        Event::PlacementsChanged {
            placements_left: 0,
            ..
        }
    ));
    assert_eq!(events[4], end);
}

/// La définition du carnet, écrite indépendamment du moteur : une partie est figée quand
/// rien n'est programmé (aucun effet continu qui s'arrêtera à un tour donné, personne à
/// l'infirmerie ; un pacte, une couronne ou un enfant, sans durée, ne programment rien) et
/// qu'aucun des deux joueurs ne peut plus ni poser (plateau plein ou réserve vide), ni
/// lancer une attaque qui a une cible, ni faire agir sa reine (un pas vers une case libre
/// collée, ou un tir s'il lui reste un garde et un ennemi au bout d'une ligne droite).
fn frozen_by_the_rules(game: &Game) -> bool {
    // Une durée ou un retour d'Interchange changeront la partie tout seuls.
    let timed_effect = game.active_effects().iter().any(|effect| {
        effect.last_turn().is_some() || matches!(effect.kind(), EffectKind::Return { .. })
    });
    if timed_effect || !game.infirmary().is_empty() {
        return false;
    }
    let board_full = game
        .board()
        .positions()
        .all(|pos| game.board().owner(pos).is_some());
    let can_act = |player: Player| {
        let can_place = !board_full && game.reserve_pions(player) > 0;
        can_place
            || queen_can_act(game, player)
            || hare_can_act(game, player)
            || game
                .arsenal(player)
                .iter()
                .any(|slot| has_a_target_without_effects(game, player, slot.attack()))
    };
    !can_act(Player::One) && !can_act(Player::Two)
}

/// Vrai si ce pion est protégé : sans effet à durée, seul un enfant (Familia) peut l'être.
fn protected(game: &Game, pos: Position) -> bool {
    game.has_effect(pos, EffectKind::Protected)
}

/// Vrai si le pion de cette case est un enfant (Familia).
fn child(game: &Game, pos: Position) -> bool {
    game.board()
        .get(pos)
        .and_then(|cell| cell.pion())
        .is_some_and(|pion| {
            game.active_effects()
                .iter()
                .any(|effect| effect.pion() == pion.id() && effect.framed_by().is_some())
        })
}

/// Vrai si la reine de ce joueur peut encore agir, sans statut à durée sur le plateau : une
/// case libre collée à elle, ou un garde et un ennemi non protégé comme premier pion d'une
/// ligne droite.
fn queen_can_act(game: &Game, player: Player) -> bool {
    let board = game.board();
    let Some(queen) = game.queen_of(player) else {
        return false;
    };
    let free_step = Direction::ALL
        .iter()
        .filter_map(|&dir| board.neighbor(queen, dir))
        .any(|pos| board.owner(pos).is_none());
    let guards = board
        .get(queen)
        .and_then(|cell| cell.pion())
        .map_or(0, |pion| game.guards_of(pion.id()));
    let enemy_in_a_line = Direction::ALL.iter().any(|&dir| {
        let mut pos = queen;
        while let Some(next) = board.neighbor(pos, dir) {
            match board.owner(next) {
                Some(owner) => return owner == player.opponent() && !protected(game, next),
                None => pos = next,
            }
        }
        false
    });
    free_step || (guards > 0 && enemy_in_a_line)
}

/// Vrai si un pion de ce joueur porte des jetons de pas (Lièvre Épique) et a une case
/// libre collée.
fn hare_can_act(game: &Game, player: Player) -> bool {
    let board = game.board();
    board
        .positions()
        .filter(|&pos| board.owner(pos) == Some(player))
        .any(|pos| {
            game.has_effect(pos, EffectKind::StepTokens)
                && Direction::ALL.iter().any(|&dir| {
                    board
                        .neighbor(pos, dir)
                        .is_some_and(|next| board.owner(next).is_none())
                })
        })
}

/// Vrai si l'attaque a une cible depuis un des pions du joueur, sur un plateau où aucun
/// pion n'a de statut à durée (seuls les enfants peuvent être protégés, et personne n'est
/// immobilisé). Écrit sans réutiliser le moteur, d'après les fiches :
/// - l'Ours vise un ennemi collé au lanceur, non protégé pour la Rare et l'Épique (elles
///   tuent) ;
/// - la Beauté fatale vise par le regard (2 cases en ligne droite, sans pion entre) :
///   1 ennemi pour la Commune, 2 pour la Rare ; l'Épique séduit dans le regard et la
///   Légendaire n'importe où, un ennemi non protégé, s'il reste un pion en réserve ;
/// - la Familia part d'un lanceur qui n'est pas déjà enfant ; dans une ligne droite, de
///   part et d'autre de lui : deux de tes pions (Commune), une case libre et un de tes pions
///   (Rare, 1 pion en réserve), deux cases libres (Épique, 2 pions) ; la Légendaire (2 pions)
///   demande un de tes pions collé, pas enfant, et une case libre à chaque bout ;
/// - Meilleurs amis pose 2 pions de la réserve sur 2 cases libres ;
/// - le Garde royal (Reine Commune) protège le lanceur : toujours possible ; les autres
///   Reines posent une reine, s'il reste un pion en réserve, une case libre, et pas de reine ;
/// - Attila vise un pion qui porte un effet adverse (sans durée : un pacte, des gardes, un
///   enfant) : un statut pour la Commune (un enfant), tout sauf la couronne pour la Rare et
///   l'Épique (son second pion est facultatif), qui visent aussi les jetons adverses ;
/// - le Massacre divin pose ses jetons depuis n'importe lequel de tes pions : toujours
///   possible ;
/// - le Médecin ressuscite sur une tombe : personne à l'infirmerie, donc aucune tombe ;
/// - le Saut du loup a besoin d'une case libre à 2 cases en ligne droite ;
/// - l'Interchange Commune téléporte sur une case libre (sans retour en attente, puisque
///   rien n'est programmé) ; la Rare vise un ennemi à 1 ou 2 cases en ligne droite, même
///   derrière un pion ; l'Épique, n'importe quel ennemi ; la Légendaire, deux ennemis collés,
///   avec un second de tes pions ;
/// - le Lièvre a besoin d'une case libre collée au lanceur ; l'Épique, d'un lanceur sans
///   jetons de pas.
fn has_a_target_without_effects(game: &Game, player: Player, attack: Attack) -> bool {
    let board = game.board();
    let enemy = |pos: Position| board.owner(pos) == Some(player.opponent());
    let in_sight = |launcher: Position, target: &dyn Fn(Position) -> bool| -> usize {
        Direction::ALL
            .iter()
            .filter_map(|&dir| {
                let first = board.neighbor(launcher, dir)?;
                if board.owner(first).is_some() {
                    return Some(first);
                }
                board.neighbor(first, dir)
            })
            .filter(|&pos| target(pos))
            .count()
    };
    let killable = |pos: Position| enemy(pos) && !protected(game, pos);
    let adjacent = |launcher: Position, target: &dyn Fn(Position) -> bool| {
        Direction::ALL
            .iter()
            .filter_map(|&dir| board.neighbor(launcher, dir))
            .any(target)
    };
    let mine = |pos: Position| board.owner(pos) == Some(player);
    let free = |pos: Position| board.owner(pos).is_none();
    let free_two_away = |launcher: Position| {
        Direction::ALL.iter().any(|&dir| {
            board
                .neighbor(launcher, dir)
                .and_then(|over| board.neighbor(over, dir))
                .is_some_and(free)
        })
    };
    let enemy_in_line = |launcher: Position| {
        Direction::ALL.iter().any(|&dir| {
            let first = board.neighbor(launcher, dir);
            let second = first.and_then(|pos| board.neighbor(pos, dir));
            first.is_some_and(enemy) || second.is_some_and(enemy)
        })
    };
    let free_neighbor = |launcher: Position| {
        Direction::ALL
            .iter()
            .filter_map(|&dir| board.neighbor(launcher, dir))
            .any(free)
    };
    let any_enemy = board.positions().any(enemy);
    let enemy_pair = board.positions().any(|pos| {
        enemy(pos)
            && Direction::ALL
                .iter()
                .any(|&dir| board.neighbor(pos, dir).is_some_and(enemy))
    });
    let own_pions = board
        .positions()
        .filter(|&pos| board.owner(pos) == Some(player))
        .count();
    let any_killable = board.positions().any(killable);
    let reserve = game.reserve_pions(player);
    let free_cells = board.positions().filter(|&pos| free(pos)).count();
    // Attila : pour chaque pion, les jeux de carte adverses qui l'ont touché.
    let enemy_effects = |pos: Position| -> Vec<fight_of_pions::ActiveEffect> {
        let Some(pion) = board.get(pos).and_then(|cell| cell.pion()) else {
            return Vec::new();
        };
        game.active_effects()
            .iter()
            .filter(|effect| effect.pion() == pion.id() && effect.creator() == player.opponent())
            .copied()
            .collect()
    };
    let enemy_statuses: Vec<Position> = board
        .positions()
        .filter(|&pos| {
            enemy_effects(pos)
                .iter()
                .any(|effect| effect.kind().is_status())
        })
        .collect();
    let plays_at = |pos: Position| -> Vec<fight_of_pions::PlayId> {
        enemy_effects(pos)
            .iter()
            .filter(|effect| effect.kind() != EffectKind::Crown)
            .map(|effect| effect.play())
            .collect()
    };
    let enemy_tokens = game
        .token_piles()
        .iter()
        .any(|pile| pile.owner() == player.opponent());
    let enemy_plays: Vec<Position> = board
        .positions()
        .filter(|&pos| !plays_at(pos).is_empty())
        .collect();
    let familia = |launcher: Position, rarity: Rarity| {
        let parents = match rarity {
            Rarity::Common => 0,
            Rarity::Rare => 1,
            Rarity::Epic | Rarity::Legendary => 2,
        };
        !child(game, launcher)
            && reserve >= parents
            && Direction::ALL.iter().any(|&dir| {
                let ahead = board.neighbor(launcher, dir);
                let behind = board.neighbor(launcher, dir.opposite());
                let (Some(ahead), Some(behind)) = (ahead, behind) else {
                    return false;
                };
                match rarity {
                    Rarity::Common => mine(ahead) && mine(behind),
                    Rarity::Rare => free(ahead) && mine(behind),
                    Rarity::Epic => free(ahead) && free(behind),
                    Rarity::Legendary => {
                        mine(ahead)
                            && !child(game, ahead)
                            && free(behind)
                            && board.neighbor(ahead, dir).is_some_and(free)
                    }
                }
            })
    };
    board
        .positions()
        .filter(|&pos| board.owner(pos) == Some(player))
        .any(|launcher| match (attack.kind, attack.rarity) {
            (AttackKind::Ours, Rarity::Rare | Rarity::Epic) => adjacent(launcher, &killable),
            (AttackKind::Ours, _) => adjacent(launcher, &enemy),
            (AttackKind::BeauteFatale, Rarity::Rare) => in_sight(launcher, &enemy) >= 2,
            (AttackKind::BeauteFatale, Rarity::Epic) => {
                reserve > 0 && in_sight(launcher, &killable) >= 1
            }
            (AttackKind::BeauteFatale, Rarity::Legendary) => reserve > 0 && any_killable,
            (AttackKind::BeauteFatale, _) => in_sight(launcher, &enemy) >= 1,
            (AttackKind::MeilleursAmis, _) => reserve >= 2 && free_cells >= 2,
            (AttackKind::Reine, Rarity::Common) => true,
            (AttackKind::Reine, _) => {
                reserve > 0 && free_cells > 0 && game.queen_of(player).is_none()
            }
            (AttackKind::Attila, Rarity::Common) => !enemy_statuses.is_empty(),
            (AttackKind::Attila, _) => !enemy_plays.is_empty() || enemy_tokens,
            (AttackKind::MassacreDivin, _) => true,
            (AttackKind::Medecin, _) => false,
            (AttackKind::SautDuLoup, _) => free_two_away(launcher),
            (AttackKind::Interchange, Rarity::Common) => free_cells > 0,
            (AttackKind::Interchange, Rarity::Rare) => enemy_in_line(launcher),
            (AttackKind::Interchange, Rarity::Epic) => any_enemy,
            (AttackKind::Interchange, Rarity::Legendary) => enemy_pair && own_pions >= 2,
            (AttackKind::Lievre, Rarity::Epic) => {
                !game.has_effect(launcher, EffectKind::StepTokens)
            }
            (AttackKind::Lievre, _) => free_neighbor(launcher),
            (AttackKind::Familia, rarity) => familia(launcher, rarity),
        })
}

#[test]
fn both_reserves_empty_freezes_the_game_into_a_draw() {
    let config = GameConfig {
        pions_per_player: 1,
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);

    play(&mut game, &[place(0, 0)]);
    // Une seule réserve vide : le joueur 2 peut encore poser, la partie continue.
    assert_eq!(game.end(), None);

    let events = play(&mut game, &[place(9, 9)]);
    assert_eq!(events.last(), Some(&draw(EndReason::Frozen)));
    assert_eq!(game.winner(), None);
    assert_eq!(game.pending_decision(), None);
    assert!(game.legal_actions().is_empty());
}

#[test]
fn full_board_freezes_the_game_into_a_draw() {
    let config = GameConfig {
        board_size: 3,
        align_to_win: 3,
        pions_per_player: 5,
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);

    // Une grille de morpion remplie sans ligne de 3 :
    //   X O X
    //   X O O
    //   O X X
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(0, 1), place(1, 1)]);
    play(&mut game, &[place(0, 2), place(1, 0)]);
    play(&mut game, &[place(1, 2), place(2, 0)]);
    play(&mut game, &[place(2, 1)]);
    // Cette dernière pose remplit le plateau et vide aussi les actions du tour.
    let events = play(&mut game, &[place(2, 2)]);

    assert_placed_then(&events, draw(EndReason::Frozen));
}

#[test]
fn turn_limit_reached_by_placing_ends_without_starting_a_new_turn() {
    let config = GameConfig {
        max_turns: 1,
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);
    // Le premier tour n'a qu'une action : la pose finit le tour, qui était le dernier.
    let events = play(&mut game, &[place(0, 0)]);

    assert_placed_then(&events, draw(EndReason::TurnLimit));
    assert_eq!(game.turn(), 1);
}

#[test]
fn random_games_freeze_exactly_when_the_rules_say_so() {
    for seed in 0..400 {
        let mut rng = Rng::new(seed);
        let mut game = game_with(random_config(&mut rng));

        while game.end().is_none() {
            let legal = game.legal_actions();
            game.apply(*rng.pick(&legal).unwrap()).unwrap();

            match game.end().map(|end| end.reason) {
                // Une victoire peut arriver sur une partie qui se fige au même coup.
                Some(EndReason::Alignment) => {}
                Some(EndReason::Frozen) => assert!(
                    frozen_by_the_rules(&game),
                    "partie {seed} déclarée figée à tort : {:?}",
                    game.config()
                ),
                Some(EndReason::TooFewPions) => assert!(
                    [Player::One, Player::Two].into_iter().any(|player| {
                        game.graveyard().iter().any(|pion| pion.owner() == player)
                            && (game.living_pions(player) as usize) < game.config().align_to_win
                    }),
                    "partie {seed} perdue par manque de pions à tort"
                ),
                Some(EndReason::TurnLimit) | None => assert!(
                    !frozen_by_the_rules(&game),
                    "partie {seed} figée mais pas arrêtée : {:?}",
                    game.config()
                ),
            }
        }
        assert!(game.turn() <= game.config().max_turns);
    }
}

#[test]
fn alignment_wins_even_when_the_game_freezes_at_the_same_time() {
    let config = GameConfig {
        board_size: 3,
        align_to_win: 2,
        pions_per_player: 2,
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]);
    play(&mut game, &[place(2, 2), place(0, 2)]);

    // Ce pion aligne 2 et vide la dernière réserve : la victoire passe avant.
    let events = play(&mut game, &[place(0, 1)]);
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded(GameEnd {
            outcome: Outcome::Won(Player::One),
            reason: EndReason::Alignment,
        }))
    );
}

#[test]
fn turn_limit_ends_the_game_in_a_draw() {
    let config = GameConfig {
        max_turns: 3,
        ..GameConfig::first_rules()
    };
    let mut game = game_with(config);
    play(&mut game, &[place(0, 0)]); // fin du tour 1
    play(&mut game, &[Action::EndTurn]); // fin du tour 2

    let events = play(&mut game, &[Action::EndTurn]); // fin du tour 3 : limite
    // Le joueur 1 pouvait encore poser ; la partie finie, il ne le peut plus.
    assert_eq!(
        events,
        vec![
            Event::PlacementsChanged {
                player: Player::One,
                placements_left: 0,
            },
            draw(EndReason::TurnLimit),
        ]
    );
    assert_eq!(game.turn(), 3);
    assert_eq!(game.pending_decision(), None);
}

#[test]
fn players_who_only_pass_reach_the_turn_limit() {
    let mut game = new_game();
    while game.end().is_none() {
        play(&mut game, &[Action::EndTurn]);
    }
    assert_eq!(game.end().map(|end| end.reason), Some(EndReason::TurnLimit));
    assert_eq!(game.turn(), game.config().max_turns);
}

#[test]
fn pending_decision_follows_the_current_player() {
    let mut game = new_game();
    assert_eq!(
        game.pending_decision(),
        Some(Decision::MainAction {
            player: Player::One
        })
    );
    play(&mut game, &[place(0, 0)]);
    assert_eq!(
        game.pending_decision().map(Decision::player),
        Some(Player::Two)
    );
}
