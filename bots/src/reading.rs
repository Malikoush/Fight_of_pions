//! Ce qu'un bot lit dans la partie, sans recopier de règle : la légalité vient de
//! `legal_actions`, les poses restantes de `placements_left`, l'effet d'une attaque ou
//! d'une fin de tour d'une copie de la partie où on les a joués.

use fight_of_pions::{Action, Game, Player, Position};
use std::collections::HashSet;

/// Les cases où l'on peut poser maintenant.
pub(crate) fn placeable(legal: &[Action]) -> Vec<Position> {
    legal
        .iter()
        .filter_map(|action| match action {
            Action::Place(pos) => Some(*pos),
            _ => None,
        })
        .collect()
}

/// Combien de pions `player` peut poser dans son tour : le tour en cours si c'est à lui,
/// sinon le prochain, lu sur une copie où le joueur courant a fini son tour (actions
/// gardées, poses permises et limite de tours comprises). 0 s'il n'aura plus de tour.
pub(crate) fn placement_budget(game: &Game, player: Player) -> usize {
    let placements = if game.current_player() == player {
        game.placements_left()
    } else {
        let mut copy = game.clone();
        copy.apply(Action::EndTurn)
            .map_or(0, |_| copy.placements_left())
    };
    usize::try_from(placements).unwrap_or(usize::MAX)
}

/// Une attaque jouée jusqu'au bout sur une copie de la partie.
pub(crate) struct AttackPlan {
    /// La prochaine action à envoyer au moteur pour la jouer.
    pub first: Action,
    /// La partie telle que l'attaque la laisse.
    pub after: Game,
}

/// Le plus d'attaques complètes différentes que [`attack_plans`] garde à partir d'une même
/// première action. Sans limite, certaines cartes en ont des milliards (le Lièvre
/// Légendaire : 3 pions, jusqu'à 3 pas chacun, 8 directions par pas) : le bot ne
/// déciderait jamais.
pub(crate) const PLANS_PER_FIRST_ACTION: usize = 64;
/// Le plus de copies jouées par attaque gardée : borne le travail quand beaucoup de suites
/// mènent à la même partie (deux cibles choisies dans un ordre ou dans l'autre).
const COPIES_PER_PLAN: usize = 3;

/// Les attaques possibles maintenant, jouées jusqu'au bout sur des copies, en essayant les
/// choix de chaque étape dans l'ordre de `legal_actions` : voir [`attack_plans_in_order`].
pub(crate) fn attack_plans(game: &Game) -> Vec<AttackPlan> {
    attack_plans_in_order(game, PLANS_PER_FIRST_ACTION, |_| {})
}

/// Les attaques possibles maintenant, jouées jusqu'au bout sur des copies : depuis chaque
/// carte utilisable si aucune attaque n'est en cours, sinon la suite de l'attaque
/// commencée. Le moteur ne propose jamais d'impasse : chaque branche va au bout.
///
/// Au plus `plans_per_first_action` attaques différentes par première action : `order`
/// range les choix de chaque étape, les premiers sont essayés d'abord. Deux suites qui
/// laissent la même partie n'en font qu'une.
pub(crate) fn attack_plans_in_order(
    game: &Game,
    plans_per_first_action: usize,
    mut order: impl FnMut(&mut [Action]),
) -> Vec<AttackPlan> {
    let choosing = game.attack_in_progress();
    let mut plans = Vec::new();
    for first in game.legal_actions() {
        if choosing || matches!(first, Action::UseAttack(_)) {
            let mut search = Search {
                first,
                plans_left: plans_per_first_action,
                copies_left: plans_per_first_action.saturating_mul(COPIES_PER_PLAN),
                seen: HashSet::new(),
            };
            search.play_to_the_end(game, first, &mut order, &mut plans);
        }
    }
    plans
}

/// La recherche des attaques complètes qui commencent par une même première action.
struct Search {
    first: Action,
    plans_left: usize,
    copies_left: usize,
    /// Les parties déjà atteintes. Seule l'appartenance est lue, jamais l'ordre : le
    /// résultat reste le même d'une exécution à l'autre.
    seen: HashSet<Game>,
}

impl Search {
    fn play_to_the_end(
        &mut self,
        game: &Game,
        next: Action,
        order: &mut impl FnMut(&mut [Action]),
        plans: &mut Vec<AttackPlan>,
    ) {
        if self.plans_left == 0 || self.copies_left == 0 {
            return;
        }
        self.copies_left -= 1;
        let mut copy = game.clone();
        // `next` vient de `legal_actions` : le moteur ne peut pas la refuser.
        if copy.apply(next).is_err() {
            return;
        }
        if copy.attack_in_progress() {
            let mut choices = copy.legal_actions();
            order(&mut choices);
            for action in choices {
                self.play_to_the_end(&copy, action, order, plans);
            }
        } else if !self.seen.contains(&copy) {
            self.seen.insert(copy.clone());
            self.plans_left -= 1;
            plans.push(AttackPlan {
                first: self.first,
                after: copy,
            });
        }
    }
}
