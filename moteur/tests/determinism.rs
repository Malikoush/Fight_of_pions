//! Déterminisme, replay et invariant du miroir d'événements (principes 1, 2 et 5).

mod common;

use common::{SEED, game_with, new_game, place, random_config};
use fight_of_pions::{
    Action, Event, Game, GameConfig, GameEnd, GameError, GameRecord, Pion, Player, Position,
    Rarity, ReplayError, Rng,
};

/// Ce qu'un client sait de la partie s'il ne suit **que** les événements, en partant de
/// la configuration. S'il diverge un jour du moteur, un client (labo, Godot, bot) se
/// désynchronise : c'est ce que ces tests interdisent. Chaque annonce doit aussi changer
/// quelque chose : une annonce inutile est refusée.
struct EventMirror {
    size: usize,
    cells: Vec<Option<Pion>>,
    reserves: [u32; 2],
    actions_left: [u32; 2],
    banked: [u32; 2],
    cooldowns: [Vec<u32>; 2],
    current_player: Player,
    end: Option<GameEnd>,
}

impl EventMirror {
    /// L'état de départ, connu par la seule configuration.
    fn new(config: &GameConfig) -> EventMirror {
        let ready = |player: Player| vec![0; config.starting_attacks[player.index()].len()];
        EventMirror {
            size: config.board_size,
            cells: vec![None; config.board_size * config.board_size],
            reserves: [config.pions_per_player; 2],
            actions_left: [config.first_turn_actions, 0],
            banked: [0, 0],
            cooldowns: [ready(Player::One), ready(Player::Two)],
            current_player: Player::One,
            end: None,
        }
    }

    fn cell(&mut self, at: Position) -> &mut Option<Pion> {
        &mut self.cells[at.row * self.size + at.col]
    }

    fn follow(&mut self, event: &Event) {
        match *event {
            Event::PionPlaced { pion, at } => {
                assert_eq!(
                    *self.cell(at),
                    None,
                    "pose sur une case occupée : {event:?}"
                );
                *self.cell(at) = Some(pion);
            }
            Event::PionKilled { pion, at, .. } => {
                assert_eq!(*self.cell(at), Some(pion), "pion tué absent : {event:?}");
                *self.cell(at) = None;
            }
            Event::ReserveChanged { player, pions } => {
                let reserve = &mut self.reserves[player.index()];
                assert_ne!(*reserve, pions, "annonce inutile : {event:?}");
                *reserve = pions;
            }
            Event::ActionsChanged {
                player,
                actions_left,
                banked,
            } => {
                let i = player.index();
                let unchanged = (self.actions_left[i], self.banked[i]) == (actions_left, banked);
                assert!(!unchanged, "annonce inutile : {event:?}");
                self.actions_left[i] = actions_left;
                self.banked[i] = banked;
            }
            Event::CooldownChanged {
                player,
                slot,
                turns_left,
            } => {
                let cooldown = &mut self.cooldowns[player.index()][slot];
                assert_ne!(*cooldown, turns_left, "annonce inutile : {event:?}");
                *cooldown = turns_left;
            }
            // L'attaque elle-même ne change rien : ses effets sont annoncés à part.
            Event::AttackUsed { .. } => {}
            Event::TurnEnded { next } => self.current_player = next,
            Event::GameEnded(end) => self.end = Some(end),
        }
    }

    fn assert_matches(&self, game: &Game) {
        for pos in game.board().positions() {
            let in_game = game.board().get(pos).and_then(|cell| cell.pion());
            let mirrored = self.cells[pos.row * self.size + pos.col];
            assert_eq!(mirrored, in_game, "case {pos} différente");
        }
        for player in [Player::One, Player::Two] {
            let i = player.index();
            assert_eq!(self.reserves[i], game.reserve_pions(player));
            assert_eq!(self.banked[i], game.banked_actions(player));
            let expected_actions = if player == game.current_player() {
                game.actions_left()
            } else {
                0
            };
            assert_eq!(
                self.actions_left[i], expected_actions,
                "actions de {player:?}"
            );
            let cooldowns: Vec<u32> = game
                .arsenal(player)
                .iter()
                .map(|slot| slot.cooldown_left())
                .collect();
            assert_eq!(self.cooldowns[i], cooldowns, "recharges de {player:?}");
        }
        assert_eq!(self.current_player, game.current_player());
        assert_eq!(self.end, game.end());
    }
}

/// Une partie jouée au hasard : son enregistrement, et pour chaque action, les événements
/// produits et l'état obtenu juste après.
struct PlayedGame {
    record: GameRecord,
    steps: Vec<(Vec<Event>, Game)>,
}

impl PlayedGame {
    /// Combien d'événements de cette sorte la partie a produits.
    fn count(&self, matches: impl Fn(&Event) -> bool) -> usize {
        self.steps
            .iter()
            .flat_map(|(events, _)| events)
            .filter(|event| matches(event))
            .count()
    }
}

/// Joue une partie au hasard jusqu'à la fin, en vérifiant le miroir après chaque action.
fn play_random_game(config: GameConfig, seed: u64, rng: &mut Rng) -> PlayedGame {
    let mut game = Game::new(config.clone(), seed).unwrap();
    let mut mirror = EventMirror::new(&config);
    let mut record = GameRecord::new(config, seed);
    let mut steps = Vec::new();
    // L'état de départ, déduit de la seule configuration, doit déjà être juste.
    mirror.assert_matches(&game);

    while game.end().is_none() {
        let action = *rng.pick(&game.legal_actions()).unwrap();
        let events = game.apply(action).unwrap();
        for event in &events {
            mirror.follow(event);
        }
        mirror.assert_matches(&game);
        record.actions.push(action);
        steps.push((events, game.clone()));
    }
    PlayedGame { record, steps }
}

#[test]
fn events_are_enough_to_follow_the_board() {
    let (mut attacks, mut epics, mut kills) = (0, 0, 0);
    for seed in 0..300 {
        let mut rng = Rng::new(seed);
        let config = if seed % 2 == 0 {
            GameConfig::default()
        } else {
            random_config(&mut rng)
        };
        let played = play_random_game(config, seed, &mut rng);
        attacks += played.count(|event| matches!(event, Event::AttackUsed { .. }));
        epics += played.count(|event| {
            matches!(event, Event::AttackUsed { attack, .. } if attack.rarity == Rarity::Epic)
        });
        kills += played.count(|event| matches!(event, Event::PionKilled { .. }));
    }
    // Sans attaques jouées, ce test ne prouverait rien sur elles.
    assert!(attacks > 100, "seulement {attacks} attaques jouées");
    assert!(epics > 20, "seulement {epics} Ours Épique joués");
    assert!(kills > 100, "seulement {kills} pions tués");
}

#[test]
fn replaying_a_record_rebuilds_the_same_game() {
    for seed in 0..100 {
        let mut rng = Rng::new(seed);
        let config = random_config(&mut rng);
        let played = play_random_game(config, seed, &mut rng);
        let finished = &played.steps.last().unwrap().1;

        assert_eq!(played.record.replay().as_ref(), Ok(finished));
        assert_eq!(finished.seed(), seed);
    }
}

#[test]
fn same_seed_and_actions_give_same_events_at_every_step() {
    for seed in 0..100 {
        let mut rng = Rng::new(seed);
        let config = random_config(&mut rng);
        let played = play_random_game(config, seed, &mut rng);

        // Rejoue action par action : à chaque coup, mêmes événements et même état.
        let mut again = Game::new(played.record.config.clone(), seed).unwrap();
        for (action, (events, state)) in played.record.actions.iter().zip(&played.steps) {
            assert_eq!(&again.apply(*action).unwrap(), events);
            assert_eq!(&again, state);
        }
    }
}

#[test]
fn same_seed_and_actions_give_same_events_and_state() {
    let actions = [place(4, 4), place(0, 0), Action::EndTurn, place(9, 9)];
    let mut a = new_game();
    let mut b = new_game();
    for action in actions {
        assert_eq!(a.apply(action), b.apply(action));
        assert_eq!(a, b);
    }
}

#[test]
fn the_seed_is_part_of_the_game() {
    let config = GameConfig::default();
    assert_ne!(
        Game::new(config.clone(), 1).unwrap(),
        Game::new(config, 2).unwrap()
    );
}

#[test]
fn replay_reports_the_refused_action() {
    let mut record = GameRecord::new(GameConfig::default(), SEED);
    // Le joueur 2 rejoue sur la case que le joueur 1 vient de prendre.
    record.actions = vec![place(0, 0), place(0, 0)];
    assert_eq!(
        record.replay(),
        Err(ReplayError::RefusedAction {
            index: 1,
            error: GameError::CellOccupied(Position { row: 0, col: 0 }),
        })
    );
}

#[test]
fn replay_reports_an_invalid_config() {
    let config = GameConfig {
        board_size: 0,
        ..GameConfig::default()
    };
    let record = GameRecord::new(config, SEED);
    assert!(matches!(record.replay(), Err(ReplayError::Start(_))));
}

#[test]
fn an_empty_record_replays_to_a_new_game() {
    let record = GameRecord::new(GameConfig::default(), SEED);
    assert_eq!(record.replay(), Ok(game_with(GameConfig::default())));
}
