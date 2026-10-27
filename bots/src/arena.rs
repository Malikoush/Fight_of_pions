//! L'arène : des parties complètes entre deux bots, et leur bilan.
//!
//! Elle sert à voir l'effet d'un réglage avant de le changer pour de bon : on joue les
//! mêmes paires de bots sur plusieurs configurations, et on compare les bilans.

use crate::{Bot, BotKind};
use fight_of_pions::{
    Action, Attack, EndReason, Event, Game, GameConfig, GameError, GameRecord, Outcome, Player, Rng,
};
use std::collections::BTreeMap;
use std::{fmt, panic, thread};

/// Joue `games` parties entre ces deux bots, avec les graines 0 à `games - 1` ; `bots[0]`
/// est toujours le joueur 1. Le moteur et les bots étant déterministes, le même appel
/// donne toujours le même bilan. Erreur si la configuration est invalide.
pub fn run(config: &GameConfig, bots: [&dyn Bot; 2], games: u64) -> Result<Report, GameError> {
    config.validate()?;
    let mut report = Report::new([bots[0].name(), bots[1].name()]);
    for seed in 0..games {
        let (game, rng) = start_game(config, seed)?;
        report.play(game, rng, bots);
    }
    Ok(report)
}

/// [`run`] pour chacune de ces paires, en parallèle (une paire par fil). Les bilans sont
/// rendus dans l'ordre des paires, et ne dépendent que des graines.
pub fn run_pairs(
    config: &GameConfig,
    pairs: &[[BotKind; 2]],
    games: u64,
) -> Result<Vec<Report>, GameError> {
    thread::scope(|scope| {
        let handles: Vec<_> = pairs
            .iter()
            .map(|&[first, second]| {
                scope.spawn(move || run(config, [first.bot(), second.bot()], games))
            })
            .collect();
        // Un bot qui plante est un bug : sa panique remonte telle quelle à l'appelant.
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|cause| panic::resume_unwind(cause))
            })
            .collect()
    })
}

/// La partie que [`run`] joue avec cette graine, enregistrée : la référence qui vérifie
/// qu'un client (le labo) revoit bien la même partie, et de quoi reproduire un refus.
/// Erreur si la configuration est invalide ou si un bot propose une action refusée.
pub fn record_game(
    config: &GameConfig,
    seed: u64,
    bots: [&dyn Bot; 2],
) -> Result<GameRecord, String> {
    let (game, rng) = start_game(config, seed).map_err(|error| error.to_string())?;
    let mut record = GameRecord::new(config.clone(), seed);
    play_game(game, rng, bots, |_, action, _, _| {
        record.actions.push(action)
    })?;
    Ok(record)
}

/// Le début d'une partie de l'arène : la partie, et le hasard de ses bots, tous deux
/// initialisés avec la même graine. Un client qui commence ainsi (le labo) et fait jouer
/// les mêmes bots revoit exactement les parties de l'arène.
pub fn start_game(config: &GameConfig, seed: u64) -> Result<(Game, Rng), GameError> {
    Ok((Game::new(config.clone(), seed)?, Rng::new(seed)))
}

/// Joue la partie jusqu'au bout et la rend finie. `on_action` voit chaque action
/// acceptée : le joueur qui l'a choisie, l'action, ses événements et la partie après.
/// Erreur : le refus, décrit avec sa graine et son tour.
fn play_game(
    mut game: Game,
    mut rng: Rng,
    bots: [&dyn Bot; 2],
    mut on_action: impl FnMut(Player, Action, Vec<Event>, &Game),
) -> Result<Game, String> {
    while let Some(decision) = game.pending_decision() {
        let player = decision.player();
        let bot = bots[player.index()];
        let (seed, turn) = (game.seed(), game.turn());
        let action = bot.choose(&game, &mut rng).ok_or_else(|| {
            format!(
                "graine {seed}, tour {turn} : {} n'a proposé aucune action",
                bot.name()
            )
        })?;
        let events = game.apply(action).map_err(|error| {
            format!(
                "graine {seed}, tour {turn} : {} : {action:?} refusée ({error})",
                bot.name()
            )
        })?;
        on_action(player, action, events, &game);
    }
    Ok(game)
}

/// Le bilan de parties jouées entre deux bots, toujours dans les mêmes places.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Le nom du bot joueur 1, puis du bot joueur 2.
    pub bots: [&'static str; 2],
    /// Les parties allées jusqu'au bout. Tous les compteurs qui suivent, sauf les refus,
    /// ne portent que sur elles.
    pub games: u64,
    /// Les victoires du joueur 1, puis du joueur 2.
    pub wins: [u64; 2],
    /// Les nuls par partie figée.
    pub frozen: u64,
    /// Les nuls par limite de tours : des parties qui n'ont rien donné.
    pub turn_limit: u64,
    /// Le total des tours joués.
    pub turns: u64,
    /// Combien de fois chaque attaque a été lancée.
    pub attacks: BTreeMap<Attack, u64>,
    /// Le total des pions tués.
    pub kills: u64,
    /// Les tours finis après avoir agi, en gardant des actions pour le tour suivant.
    pub turns_keeping_actions: u64,
    /// Les tours finis sans aucune action (qu'il en reste à garder ou non).
    pub passes: u64,
    /// Les parties abandonnées parce qu'un bot a proposé une action refusée, ou aucune :
    /// toujours un bug du bot ou du moteur. Elles ne laissent rien d'autre dans le bilan.
    pub refused: u64,
    /// Le premier refus, avec sa graine et son tour, pour le reproduire.
    pub first_refusal: Option<String>,
}

impl Report {
    /// L'en-tête des colonnes affichées par une ligne de bilan.
    pub const HEADER: &'static str = "joueur 1    joueur 2    |  J1 gagne  J2 gagne  nul figée  nul limite | tours  attaques  tués |  garde  passe | refus";

    fn new(bots: [&'static str; 2]) -> Report {
        Report {
            bots,
            games: 0,
            wins: [0, 0],
            frozen: 0,
            turn_limit: 0,
            turns: 0,
            attacks: BTreeMap::new(),
            kills: 0,
            turns_keeping_actions: 0,
            passes: 0,
            refused: 0,
            first_refusal: None,
        }
    }

    /// Joue une partie jusqu'au bout et l'ajoute au bilan. Une partie abandonnée sur un
    /// refus n'y laisse que ce refus, pour ne pas mêler ses compteurs à ceux des autres.
    fn play(&mut self, game: Game, rng: Rng, bots: [&dyn Bot; 2]) {
        let before = self.clone();
        if let Err(refusal) = self.play_to_the_end(game, rng, bots) {
            *self = before;
            self.refused += 1;
            self.first_refusal.get_or_insert(refusal);
        }
    }

    /// Joue la partie en comptant au fur et à mesure ; le refus, s'il y en a un, décrit
    /// l'action en cause.
    fn play_to_the_end(&mut self, game: Game, rng: Rng, bots: [&dyn Bot; 2]) -> Result<(), String> {
        let mut acted_this_turn = false;
        let end = play_game(game, rng, bots, |player, action, events, game| {
            if action == Action::EndTurn {
                self.count_turn_end(game, player, acted_this_turn);
            }
            acted_this_turn = action != Action::EndTurn;
            for event in events {
                match event {
                    Event::AttackUsed { attack, .. } => {
                        *self.attacks.entry(attack).or_default() += 1
                    }
                    Event::PionKilled { .. } => self.kills += 1,
                    Event::TurnEnded { .. } => acted_this_turn = false,
                    _ => {}
                }
            }
        })?;
        self.count_end(&end);
        Ok(())
    }

    /// Un tour fini volontairement : une passe s'il n'a rien fait, sinon un tour où il
    /// garde des actions, si le moteur lui en a gardé.
    fn count_turn_end(&mut self, game: &Game, player: Player, acted_this_turn: bool) {
        if !acted_this_turn {
            self.passes += 1;
        } else if game.banked_actions(player) > 0 {
            self.turns_keeping_actions += 1;
        }
    }

    fn count_end(&mut self, game: &Game) {
        let Some(end) = game.end() else {
            return;
        };
        self.games += 1;
        self.turns += u64::from(game.turn());
        match end.outcome {
            Outcome::Won(player) => self.wins[player.index()] += 1,
            Outcome::Draw if end.reason == EndReason::TurnLimit => self.turn_limit += 1,
            // Un alignement a toujours un vainqueur : l'autre nul est la partie figée.
            Outcome::Draw => self.frozen += 1,
        }
    }
}

/// Une ligne sous [`Report::HEADER`], puis les fourchettes à 95 % des victoires et des
/// nuls, puis le détail des attaques s'il y en a eu.
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let percent = |count: u64| ratio(count, self.games) * 100.0;
        let per_game = |count: u64| ratio(count, self.games);
        write!(
            f,
            "{:<11} {:<11} | {:>7.1} % {:>7.1} % {:>8.1} % {:>9.1} % | {:>5.1} {:>9.2} {:>5.2} | {:>4.0} % {:>4.0} % | {:>5}",
            self.bots[0],
            self.bots[1],
            percent(self.wins[0]),
            percent(self.wins[1]),
            percent(self.frozen),
            percent(self.turn_limit),
            per_game(self.turns),
            per_game(self.attacks.values().sum()),
            per_game(self.kills),
            ratio(self.turns_keeping_actions, self.turns) * 100.0,
            ratio(self.passes, self.turns) * 100.0,
            self.refused,
        )?;
        if self.games > 0 {
            let range = |count: u64| {
                let (low, high) = wilson_interval(count, self.games);
                format!("{:.0}–{:.0} %", low * 100.0, high * 100.0)
            };
            write!(
                f,
                "
    fourchettes à 95 % : J1 gagne {}, J2 gagne {}, nul figée {}, nul limite {}",
                range(self.wins[0]),
                range(self.wins[1]),
                range(self.frozen),
                range(self.turn_limit),
            )?;
        }
        for (attack, count) in &self.attacks {
            write!(f, "\n    {attack} : {:.2} par partie", per_game(*count))?;
        }
        if let Some(refusal) = &self.first_refusal {
            write!(f, "\n    premier refus : {refusal}")?;
        }
        Ok(())
    }
}

/// Les valeurs plausibles, à 95 %, de la proportion dont on a observé `count` sur `total`
/// (intervalle de Wilson). Contrairement à « p ± 1,96 × écart type », il reste juste avec
/// peu de parties et à 0 ou 100 % : 10 victoires sur 10 donnent 72 à 100 %, pas 100 % pile.
/// `total` doit être non nul.
fn wilson_interval(count: u64, total: u64) -> (f64, f64) {
    /// Le quantile de la loi normale pour 95 %.
    const Z: f64 = 1.96;
    let n = total as f64;
    let p = ratio(count, total);
    let z2 = Z * Z;
    let denominator = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denominator;
    let half_width = Z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denominator;
    (
        (center - half_width).max(0.0),
        (center + half_width).min(1.0),
    )
}

/// `count / total`, ou 0 s'il n'y a rien à diviser.
fn ratio(count: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close((low, high): (f64, f64), expected: (f64, f64)) {
        assert!(
            (low - expected.0).abs() < 1e-3,
            "{low} au lieu de {}",
            expected.0
        );
        assert!(
            (high - expected.1).abs() < 1e-3,
            "{high} au lieu de {}",
            expected.1
        );
    }

    #[test]
    fn wilson_interval_matches_known_values() {
        assert_close(wilson_interval(5, 10), (0.2366, 0.7634));
        assert_close(wilson_interval(7, 10), (0.3968, 0.8922));
        assert_close(wilson_interval(50, 100), (0.4038, 0.5962));
    }

    #[test]
    fn wilson_interval_stays_honest_at_the_extremes() {
        assert_close(wilson_interval(0, 10), (0.0, 0.2775));
        assert_close(wilson_interval(10, 10), (0.7225, 1.0));
    }

    #[test]
    fn the_report_shows_its_ranges_only_once_games_are_played() {
        let mut report = Report::new(["a", "b"]);
        assert!(!report.to_string().contains("fourchettes"));
        report.games = 10;
        report.wins = [7, 3];
        assert!(
            report
                .to_string()
                .contains("J1 gagne 40–89 %, J2 gagne 11–60 %")
        );
    }
}
