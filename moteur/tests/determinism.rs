//! Déterminisme, replay et invariant du miroir d'événements (principes 1, 2 et 5).

mod common;

use common::{SEED, game_with, new_game, place, random_config};
use fight_of_pions::{
    Action, ActiveEffect, AttackKind, EffectKind, Event, Game, GameConfig, GameEnd, GameError,
    GameRecord, Patient, Pion, PionId, Player, Position, Rarity, ReplayError, Rng, TokenPile,
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
    /// Les poses restantes de chaque joueur, telles qu'annoncées.
    placements: [u32; 2],
    /// Les pions posés depuis le début du tour en cours (un fait, pas une règle).
    arrived: Vec<PionId>,
    summoning_sickness: bool,
    /// Les effets continus en jeu, dans l'ordre de leur création.
    effects: Vec<ActiveEffect>,
    /// Les pions à l'infirmerie, dans l'ordre d'arrivée.
    infirmary: Vec<Patient>,
    /// La tombe de chaque case, s'il y en a une.
    tombs: Vec<Option<Pion>>,
    /// Les piles de jetons, dans l'ordre où elles ont été posées.
    token_piles: Vec<TokenPile>,
    /// Les pions au cimetière, dans l'ordre d'arrivée.
    graveyard: Vec<Pion>,
    /// Le dernier tour de Terre brûlée de chaque joueur, s'il y est.
    scorched_until: [Option<u32>; 2],
    pions_per_player: u32,
}

impl EventMirror {
    /// L'état de départ, connu par la configuration. Seules les poses restantes du premier
    /// tour se lisent sur la partie qui commence, comme un client le ferait une fois : les
    /// déduire de la configuration reviendrait à recoder la règle.
    fn new(config: &GameConfig, game: &Game) -> EventMirror {
        let ready = |player: Player| vec![0; config.starting_attacks[player.index()].len()];
        let [first_bank, second_bank] = config.starting_banked_actions;
        EventMirror {
            size: config.board_size,
            cells: vec![None; config.board_size * config.board_size],
            reserves: [config.pions_per_player; 2],
            actions_left: [config.first_turn_actions + first_bank, 0],
            banked: [0, second_bank],
            cooldowns: [ready(Player::One), ready(Player::Two)],
            current_player: Player::One,
            end: None,
            placements: [game.placements_left(), 0],
            arrived: Vec::new(),
            summoning_sickness: config.summoning_sickness,
            effects: Vec::new(),
            infirmary: Vec::new(),
            tombs: vec![None; config.board_size * config.board_size],
            token_piles: Vec::new(),
            graveyard: Vec::new(),
            scorched_until: [None, None],
            pions_per_player: config.pions_per_player,
        }
    }

    fn is_on_board(&self, pion: PionId) -> bool {
        self.cells.iter().flatten().any(|cell| cell.id() == pion)
    }

    fn cell(&mut self, at: Position) -> &mut Option<Pion> {
        &mut self.cells[at.row * self.size + at.col]
    }

    fn follow(&mut self, event: &Event) {
        // La fin de partie est toujours le dernier événement.
        assert_eq!(
            self.end, None,
            "événement après la fin de partie : {event:?}"
        );
        match *event {
            Event::PionPlaced { pion, at } => {
                assert_eq!(
                    *self.cell(at),
                    None,
                    "pose sur une case occupée : {event:?}"
                );
                *self.cell(at) = Some(pion);
                self.arrived.push(pion.id());
            }
            Event::PionKilled { pion, at, .. } => {
                assert_eq!(*self.cell(at), Some(pion), "pion tué absent : {event:?}");
                *self.cell(at) = None;
            }
            Event::PionMoved { pion, from, to } => {
                assert_eq!(
                    *self.cell(from),
                    Some(pion),
                    "pion déplacé absent : {event:?}"
                );
                assert_eq!(*self.cell(to), None, "arrivée occupée : {event:?}");
                *self.cell(from) = None;
                *self.cell(to) = Some(pion);
            }
            Event::PionsSwapped {
                first,
                first_at,
                second,
                second_at,
            } => {
                assert_eq!(*self.cell(first_at), Some(first), "{event:?}");
                assert_eq!(*self.cell(second_at), Some(second), "{event:?}");
                *self.cell(first_at) = Some(second);
                *self.cell(second_at) = Some(first);
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
            Event::PlacementsChanged {
                player,
                placements_left,
            } => {
                let placements = &mut self.placements[player.index()];
                assert_ne!(*placements, placements_left, "annonce inutile : {event:?}");
                *placements = placements_left;
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
            Event::EffectStarted(effect) => {
                assert!(
                    self.is_on_board(effect.pion()),
                    "effet sur un pion absent : {event:?}"
                );
                assert!(
                    self.effects.iter().all(|known| known.id() != effect.id()),
                    "identité d'effet réutilisée : {event:?}"
                );
                self.effects.push(effect);
            }
            Event::EffectChanged(effect) => {
                let known = self
                    .effects
                    .iter_mut()
                    .find(|known| known.id() == effect.id());
                let known = known.unwrap_or_else(|| panic!("effet inconnu : {event:?}"));
                assert_ne!(*known, effect, "annonce inutile : {event:?}");
                *known = effect;
            }
            Event::EffectEnded(effect) => {
                let index = self.effects.iter().position(|known| *known == effect);
                let index = index.unwrap_or_else(|| panic!("effet inconnu : {event:?}"));
                self.effects.remove(index);
            }
            Event::InfirmaryEntered(patient) => {
                assert!(
                    !self.is_on_board(patient.pion().id()),
                    "pion encore sur le plateau : {event:?}"
                );
                self.infirmary.push(patient);
            }
            Event::TombAdded { pion, at } => {
                let tomb = &mut self.tombs[at.row * self.size + at.col];
                assert_eq!(*tomb, None, "deux tombes sur une case : {event:?}");
                *tomb = Some(pion);
            }
            Event::TombRemoved { pion, at } => {
                let tomb = &mut self.tombs[at.row * self.size + at.col];
                assert_eq!(*tomb, Some(pion), "tombe absente : {event:?}");
                *tomb = None;
            }
            Event::InfirmaryLeft(patient) => {
                let index = self.infirmary.iter().position(|known| *known == patient);
                let index = index.unwrap_or_else(|| panic!("patient inconnu : {event:?}"));
                self.infirmary.remove(index);
            }
            Event::GraveyardEntered(pion) => {
                assert!(
                    !self.is_on_board(pion.id()),
                    "pion encore sur le plateau : {event:?}"
                );
                self.graveyard.push(pion);
            }
            Event::TokenPileStarted(pile) => {
                assert!(pile.tokens() > 0, "pile vide : {event:?}");
                assert!(
                    self.token_piles
                        .iter()
                        .all(|known| known.play() != pile.play()),
                    "deux piles pour un jeu : {event:?}"
                );
                self.token_piles.push(pile);
            }
            Event::TokenUsed(pile) => {
                let known = self
                    .token_piles
                    .iter_mut()
                    .find(|known| known.play() == pile.play());
                let known = known.unwrap_or_else(|| panic!("pile inconnue : {event:?}"));
                assert_eq!(known.tokens(), pile.tokens() + 1, "{event:?}");
                assert!(pile.tokens() > 0, "pile vide encore là : {event:?}");
                *known = pile;
            }
            Event::TokenPileEnded(pile) => {
                let index = self
                    .token_piles
                    .iter()
                    .position(|known| known.play() == pile.play());
                let index = index.unwrap_or_else(|| panic!("pile inconnue : {event:?}"));
                self.token_piles.remove(index);
            }
            Event::ScorchedEarthStarted { player, last_turn } => {
                let scorched = &mut self.scorched_until[player.index()];
                assert_ne!(*scorched, Some(last_turn), "annonce inutile : {event:?}");
                *scorched = Some(last_turn);
            }
            Event::ScorchedEarthEnded { player } => {
                let scorched = &mut self.scorched_until[player.index()];
                assert!(scorched.is_some(), "Terre brûlée inconnue : {event:?}");
                *scorched = None;
            }
            // L'attaque elle-même ne change rien : ses effets sont annoncés à part.
            Event::AttackUsed { .. } | Event::QueenShot { .. } => {}
            Event::TurnEnded { next } => {
                self.current_player = next;
                self.arrived.clear();
            }
            Event::GameEnded(end) => self.end = Some(end),
        }
    }

    fn assert_matches(&self, game: &Game) {
        for pos in game.board().positions() {
            let in_game = game.board().get(pos).and_then(|cell| cell.pion());
            let mirrored = self.cells[pos.row * self.size + pos.col];
            assert_eq!(mirrored, in_game, "case {pos} différente");
            let sick = self.summoning_sickness
                && mirrored.is_some_and(|pion| self.arrived.contains(&pion.id()));
            assert_eq!(sick, game.is_summoning_sick(pos), "case {pos}");
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
            let expected_placements = if player == game.current_player() {
                game.placements_left()
            } else {
                0
            };
            assert_eq!(
                self.placements[i], expected_placements,
                "poses restantes de {player:?}"
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
        assert_eq!(self.effects, game.active_effects(), "effets continus");
        assert_eq!(self.infirmary, game.infirmary(), "infirmerie");
        assert_eq!(self.token_piles, game.token_piles(), "piles de jetons");
        assert_eq!(self.graveyard, game.graveyard(), "cimetière");
        for player in [Player::One, Player::Two] {
            assert_eq!(
                self.scorched_until[player.index()],
                game.scorched_until(player),
                "Terre brûlée de {player:?}"
            );
        }
        for pos in game.board().positions() {
            let mirrored = self.tombs[pos.row * self.size + pos.col];
            assert_eq!(mirrored, game.board().tomb(pos), "tombe en {pos}");
            // Une tombe appartient toujours à un pion encore à l'infirmerie.
            if let Some(dead) = mirrored {
                assert!(self.infirmary.iter().any(|patient| patient.pion() == dead));
            }
        }
        // Aucun effet ne survit à son pion.
        for effect in &self.effects {
            assert!(self.is_on_board(effect.pion()), "{effect:?}");
        }
        // Un pacte lie toujours deux amis sur le plateau, chacun portant celui de l'autre.
        for effect in &self.effects {
            if let EffectKind::Pact { friend } = effect.kind() {
                let mutual = EffectKind::Pact {
                    friend: effect.pion(),
                };
                let reciprocal = self
                    .effects
                    .iter()
                    .any(|other| other.pion() == friend && other.kind() == mutual);
                assert!(reciprocal, "pacte sans réciproque : {effect:?}");
            }
        }
        // Aucun pion ne se perd : plateau, réserve, infirmerie et cimetière font toujours le
        // compte.
        for player in [Player::One, Player::Two] {
            let on_board = self.cells.iter().flatten();
            let on_board = on_board.filter(|pion| pion.owner() == player).count() as u32;
            let wounded = self.infirmary.iter();
            let wounded = wounded
                .filter(|patient| patient.pion().owner() == player)
                .count() as u32;
            assert_eq!(
                on_board + self.reserves[player.index()] + wounded,
                game.living_pions(player)
            );
            let dead = self.graveyard.iter();
            let dead = dead.filter(|pion| pion.owner() == player).count() as u32;
            assert_eq!(
                on_board + self.reserves[player.index()] + wounded + dead,
                self.pions_per_player,
                "pions de {player:?}"
            );
        }
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
    let mut mirror = EventMirror::new(&config, &game);
    let mut record = GameRecord::new(config, seed);
    let mut steps = Vec::new();
    // L'état de départ, déduit de la configuration (et des poses restantes lues une fois
    // sur la partie), doit déjà être juste.
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
    let (mut effects_ended, mut healed, mut moved, mut swapped) = (0, 0, 0, 0);
    let (mut tokens_used, mut buried, mut tokens_spent, mut returns) = (0, 0, 0, 0);
    let mut scorched = 0;
    let mut kinds_played = Vec::new();
    for seed in 0..500 {
        let mut rng = Rng::new(seed);
        let config = if seed % 2 == 0 {
            GameConfig::first_rules()
        } else {
            random_config(&mut rng)
        };
        let played = play_random_game(config, seed, &mut rng);
        attacks += played.count(|event| matches!(event, Event::AttackUsed { .. }));
        epics += played.count(|event| {
            matches!(event, Event::AttackUsed { attack, .. } if attack.rarity == Rarity::Epic)
        });
        kills += played.count(|event| matches!(event, Event::PionKilled { .. }));
        effects_ended += played.count(|event| matches!(event, Event::EffectEnded(_)));
        healed += played.count(|event| matches!(event, Event::InfirmaryLeft(_)));
        moved += played.count(|event| matches!(event, Event::PionMoved { .. }));
        swapped += played.count(|event| matches!(event, Event::PionsSwapped { .. }));
        tokens_used +=
            played.count(|event| matches!(event, Event::TokenUsed(_) | Event::TokenPileEnded(_)));
        buried += played.count(|event| matches!(event, Event::GraveyardEntered(_)));
        tokens_spent += played.count(|event| matches!(event, Event::EffectChanged(_)));
        scorched += played.count(|event| {
            matches!(
                event,
                Event::ScorchedEarthStarted { .. } | Event::ScorchedEarthEnded { .. }
            )
        });
        returns += played.count(|event| {
            matches!(event, Event::EffectEnded(effect) if matches!(effect.kind(), EffectKind::Return { .. }))
        });
        for (events, _) in &played.steps {
            for event in events {
                if let Event::AttackUsed { attack, .. } = event
                    && !kinds_played.contains(&attack.kind)
                {
                    kinds_played.push(attack.kind);
                }
            }
        }
    }
    // Sans attaques jouées, ce test ne prouverait rien sur elles.
    assert!(attacks > 100, "seulement {attacks} attaques jouées");
    assert!(epics > 20, "seulement {epics} Épiques jouées");
    assert!(moved > 50, "seulement {moved} pions déplacés");
    assert!(swapped > 10, "seulement {swapped} échanges");
    assert!(kills > 100, "seulement {kills} pions tués");
    assert!(effects_ended > 50, "seulement {effects_ended} effets finis");
    assert!(
        healed > 20,
        "seulement {healed} pions sortis de l'infirmerie"
    );
    assert!(tokens_used > 20, "seulement {tokens_used} jetons usés");
    // Rare au hasard : il faut un Massacre Épique ou Légendaire, puis un kill.
    // Rare au hasard, d'autant que les cartes sont nombreuses : il faut un Massacre Épique
    // ou Légendaire, puis un kill. Les tests de règles couvrent le cimetière.
    assert!(buried > 0, "aucun pion au cimetière");
    assert!(returns > 5, "seulement {returns} retours d'Interchange");
    assert!(
        scorched > 5,
        "seulement {scorched} débuts ou fins de Terre brûlée"
    );
    assert!(
        tokens_spent > 5,
        "seulement {tokens_spent} jetons de pas dépensés"
    );
    for kind in [
        AttackKind::BeauteFatale,
        AttackKind::SautDuLoup,
        AttackKind::Reine,
        AttackKind::Attila,
        AttackKind::Medecin,
        AttackKind::Interchange,
        AttackKind::MassacreDivin,
        AttackKind::Ours,
        AttackKind::Lievre,
        AttackKind::MeilleursAmis,
        AttackKind::Familia,
    ] {
        assert!(kinds_played.contains(&kind), "{kind} jamais jouée");
    }
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
    let config = GameConfig::first_rules();
    assert_ne!(
        Game::new(config.clone(), 1).unwrap(),
        Game::new(config, 2).unwrap()
    );
}

#[test]
fn replay_reports_the_refused_action() {
    let mut record = GameRecord::new(GameConfig::first_rules(), SEED);
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
        ..GameConfig::first_rules()
    };
    let record = GameRecord::new(config, SEED);
    assert!(matches!(record.replay(), Err(ReplayError::Start(_))));
}

#[test]
fn an_empty_record_replays_to_a_new_game() {
    let record = GameRecord::new(GameConfig::first_rules(), SEED);
    assert_eq!(record.replay(), Ok(game_with(GameConfig::first_rules())));
}
