//! Le bot glouton : il regarde une seule décision à la fois, dans cet ordre.
//!
//! 1. **Gagner** : finir une fenêtre en posant, ou après une attaque jouée sur une copie.
//! 2. **Se défendre** : bloquer les **menaces**, les fenêtres que l'adversaire pourrait
//!    finir en posant à son prochain tour, avec la pose ou l'attaque qui en laisse le moins.
//! 3. **Garder des actions**, s'il est réglé pour.
//! 4. **Se développer** : poser sur une des meilleures cases selon les fenêtres.
//!
//! Il ne voit pas les attaques de l'adversaire dans ses menaces : une ligne bloquée qu'un
//! Ours pourrait rouvrir ne l'inquiète pas.

use crate::reading::{AttackPlan, attack_plans, is_choosing, placeable, placement_budget};
use crate::windows::WindowContent;
use crate::{Bot, Windows};
use fight_of_pions::{Action, Game, Player, Position, Rng};
use std::cell::OnceCell;
use std::cmp::Reverse;

/// Ce qu'une fenêtre ouverte rapporte : multiplié par cette base à chaque pion dedans.
const WINDOW_GROWTH: u64 = 5;
/// Le poids d'une fenêtre à soi, face à celui d'une fenêtre adverse à gêner : 5 contre 4.
const OWN_WEIGHT: u64 = 5;
const OPPONENT_WEIGHT: u64 = 4;
/// Pour varier les parties, le bot tire au hasard parmi ses meilleures cases : celles qui
/// valent au moins ce pourcentage de la meilleure, et au plus ce nombre de cases.
const NEAR_BEST_PERCENT: u64 = 90;
const NEAR_BEST_COUNT: usize = 3;

/// Ce que le bot fait des actions qu'il ne dépense pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Banking {
    /// Il dépense toutes ses actions à chaque tour.
    SpendAll,
    /// Il garde autant d'actions que la configuration le permet, pour un tour plus fort ;
    /// il ne les dépense que pour gagner ou se défendre.
    KeepMax,
}

/// Le bot glouton (voir le module).
pub struct GreedyBot {
    banking: Banking,
}

impl GreedyBot {
    /// Un bot glouton qui gère ainsi ses actions non dépensées.
    pub const fn new(banking: Banking) -> GreedyBot {
        GreedyBot { banking }
    }

    /// Finir son tour pour garder ses actions, s'il est réglé pour et qu'il peut toutes
    /// les garder.
    fn keep_actions(&self, game: &Game) -> Option<Action> {
        let keep = match self.banking {
            Banking::SpendAll => 0,
            Banking::KeepMax => game.config().max_banked_actions,
        };
        (keep > 0 && game.actions_left() <= keep).then_some(Action::EndTurn)
    }
}

impl Bot for GreedyBot {
    fn name(&self) -> &'static str {
        match self.banking {
            Banking::SpendAll => "glouton",
            Banking::KeepMax => "banquier",
        }
    }

    fn choose(&self, game: &Game, rng: &mut Rng) -> Option<Action> {
        let me = game.pending_decision()?.player();
        let legal = game.legal_actions();
        let windows = Windows::new(game.board(), game.config().align_to_win);
        let view = View {
            game,
            me,
            legal: &legal,
            windows: &windows,
            plans: OnceCell::new(),
        };
        // Chaque étape ne propose que des actions légales : une erreur ici doit apparaître
        // comme un refus dans le bilan de l'arène, pas être rattrapée en silence.
        if is_choosing(game) {
            // La suite de l'attaque commencée, choisie comme au début : gagner d'abord,
            // sinon laisser le moins de menaces.
            view.winning_plan()
                .or_else(|| view.safest_plan().map(|(plan, _)| plan))
                .map(|plan| plan.first)
        } else {
            view.winning_move()
                .or_else(|| view.defensive_move())
                .or_else(|| self.keep_actions(game))
                .or_else(|| view.developing_move(rng))
        }
    }
}

/// La partie vue par le bot pour une décision.
struct View<'a> {
    game: &'a Game,
    me: Player,
    legal: &'a [Action],
    windows: &'a Windows,
    /// Les attaques possibles, calculées seulement si une étape en a besoin.
    plans: OnceCell<Vec<AttackPlan>>,
}

impl View<'_> {
    fn winning_move(&self) -> Option<Action> {
        let by_placing = best_finishable_now(self.game, self.windows, self.me)
            .and_then(|window| window.empty.first().copied())
            .map(Action::Place);
        by_placing.or_else(|| self.winning_plan().map(|plan| plan.first))
    }

    fn defensive_move(&self) -> Option<Action> {
        let threats = threats(self.game, self.windows, self.me);
        if threats.is_empty() {
            return None;
        }
        let by_placing = placeable(self.legal)
            .into_iter()
            .map(|pos| {
                (
                    pos,
                    threats.iter().filter(|w| w.empty.contains(&pos)).count(),
                )
            })
            .filter(|&(_, blocked)| blocked > 0)
            .max_by_key(|&(_, blocked)| blocked)
            .map(|(pos, blocked)| (Action::Place(pos), threats.len() - blocked));
        let by_attacking = self
            .safest_plan()
            .map(|(plan, threats_left)| (plan.first, threats_left));
        // À menaces restantes égales, la pose l'emporte : elle coûte moins cher.
        match (by_placing, by_attacking) {
            (Some(placing), Some(attacking)) if attacking.1 < placing.1 => Some(attacking.0),
            (Some(placing), _) => Some(placing.0),
            (None, Some(attacking)) if attacking.1 < threats.len() => Some(attacking.0),
            _ => None,
        }
    }

    fn developing_move(&self, rng: &mut Rng) -> Option<Action> {
        let scores = self.cell_scores();
        let score_of = |pos: &Position| scores.get(cell_index(self.game, *pos)).copied();
        let mut cells = placeable(self.legal);
        cells.sort_by_key(|pos| Reverse(score_of(pos)));
        let Some(best) = cells.first().and_then(score_of) else {
            return Some(Action::EndTurn);
        };
        let threshold = best.saturating_mul(NEAR_BEST_PERCENT) / 100;
        let near_best: Vec<Position> = cells
            .into_iter()
            .take_while(|pos| score_of(pos).is_some_and(|score| score >= threshold))
            .take(NEAR_BEST_COUNT)
            .collect();
        rng.pick(&near_best).copied().map(Action::Place)
    }

    /// La valeur de chaque case libre : la somme des fenêtres qu'elle ferait avancer pour
    /// le bot, et de celles qu'elle gênerait chez l'adversaire.
    fn cell_scores(&self) -> Vec<u64> {
        let board = self.game.board();
        let mut scores = vec![0_u64; board.size() * board.size()];
        for window in self.windows.iter() {
            let content = WindowContent::of(board, window, self.me);
            let value = window_value(&content);
            for &pos in &content.empty {
                if let Some(score) = scores.get_mut(cell_index(self.game, pos)) {
                    *score = score.saturating_add(value);
                }
            }
        }
        scores
    }

    fn plans(&self) -> &[AttackPlan] {
        self.plans.get_or_init(|| attack_plans(self.game))
    }

    /// La première attaque après laquelle le bot gagne, ou peut gagner en posant.
    fn winning_plan(&self) -> Option<&AttackPlan> {
        self.plans().iter().find(|plan| {
            plan.after.winner() == Some(self.me)
                || best_finishable_now(&plan.after, self.windows, self.me).is_some()
        })
    }

    /// L'attaque qui laisse le moins de menaces, et combien elle en laisse.
    fn safest_plan(&self) -> Option<(&AttackPlan, usize)> {
        self.plans()
            .iter()
            .map(|plan| (plan, threats(&plan.after, self.windows, self.me).len()))
            .min_by_key(|&(_, threats_left)| threats_left)
    }
}

/// La fenêtre que `player` peut finir tout de suite en posant, la plus rapide à finir ;
/// `None` si ce n'est pas à lui de poser.
fn best_finishable_now(game: &Game, windows: &Windows, player: Player) -> Option<WindowContent> {
    if game.end().is_some() || game.current_player() != player {
        return None;
    }
    windows
        .finishable(game.board(), player, placement_budget(game, player))
        .into_iter()
        .min_by_key(|window| window.empty.len())
}

/// Les menaces contre `defender` : les fenêtres que son adversaire pourrait finir en
/// posant à son prochain tour (ou dans le tour en cours, s'il a déjà commencé).
fn threats(game: &Game, windows: &Windows, defender: Player) -> Vec<WindowContent> {
    if game.end().is_some() {
        return Vec::new();
    }
    let attacker = defender.opponent();
    windows.finishable(game.board(), attacker, placement_budget(game, attacker))
}

/// Ce que vaut une fenêtre pour qui veut y poser : la sienne si l'adversaire n'y est pas,
/// celle de l'adversaire à gêner si le joueur n'y est pas. Une fenêtre mixte ne vaut rien.
fn window_value(content: &WindowContent) -> u64 {
    let grown =
        |pions: usize| WINDOW_GROWTH.saturating_pow(u32::try_from(pions).unwrap_or(u32::MAX));
    let own = if content.opponent == 0 {
        OWN_WEIGHT.saturating_mul(grown(content.own))
    } else {
        0
    };
    let opponent = if content.own == 0 {
        OPPONENT_WEIGHT.saturating_mul(grown(content.opponent))
    } else {
        0
    };
    own.saturating_add(opponent)
}

fn cell_index(game: &Game, pos: Position) -> usize {
    pos.row * game.board().size() + pos.col
}
