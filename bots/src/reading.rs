//! Ce qu'un bot lit dans la partie, sans recopier de règle : la légalité vient de
//! `legal_actions`, l'effet d'une attaque ou d'une fin de tour d'une copie de la partie
//! où on les a joués. Seul le coût d'une pose est lu dans la configuration.

use fight_of_pions::{Action, Decision, Game, Player, Position};

/// Vrai si la partie attend le lanceur ou une cible d'une attaque commencée.
pub(crate) fn is_choosing(game: &Game) -> bool {
    matches!(
        game.pending_decision(),
        Some(Decision::ChooseLauncher { .. } | Decision::ChooseTarget { .. })
    )
}

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
/// sinon le prochain. C'est la seule lecture de règle des bots : poser coûte
/// `place_cost` actions et un pion de la réserve.
pub(crate) fn placement_budget(game: &Game, player: Player) -> usize {
    let placements = actions_of_turn(game, player)
        .checked_div(game.config().place_cost)
        .unwrap_or(0)
        .min(game.reserve_pions(player));
    usize::try_from(placements).unwrap_or(usize::MAX)
}

/// Les actions de `player` pour son tour : celles qui lui restent si c'est à lui, sinon
/// celles que le moteur lui donne quand le joueur courant finit son tour, lues sur une
/// copie (actions gardées et limite de tours comprises). 0 s'il n'aura plus de tour.
fn actions_of_turn(game: &Game, player: Player) -> u32 {
    if game.end().is_some() {
        return 0;
    }
    if game.current_player() == player {
        return game.actions_left();
    }
    let mut copy = game.clone();
    match copy.apply(Action::EndTurn) {
        Ok(_) if copy.end().is_none() => copy.actions_left(),
        _ => 0,
    }
}

/// Une attaque jouée jusqu'au bout sur une copie de la partie.
pub(crate) struct AttackPlan {
    /// La prochaine action à envoyer au moteur pour la jouer.
    pub first: Action,
    /// La partie telle que l'attaque la laisse.
    pub after: Game,
}

/// Toutes les attaques possibles maintenant, jouées jusqu'au bout sur des copies : depuis
/// chaque carte utilisable si aucune attaque n'est en cours, sinon la suite de l'attaque
/// commencée. Le moteur ne propose jamais d'impasse : chaque branche va au bout.
pub(crate) fn attack_plans(game: &Game) -> Vec<AttackPlan> {
    let choosing = is_choosing(game);
    let mut plans = Vec::new();
    for first in game.legal_actions() {
        if choosing || matches!(first, Action::UseAttack(_)) {
            play_to_the_end(game, first, first, &mut plans);
        }
    }
    plans
}

fn play_to_the_end(game: &Game, first: Action, next: Action, plans: &mut Vec<AttackPlan>) {
    let mut copy = game.clone();
    // `next` vient de `legal_actions` : le moteur ne peut pas la refuser.
    if copy.apply(next).is_err() {
        return;
    }
    if is_choosing(&copy) {
        for action in copy.legal_actions() {
            play_to_the_end(&copy, first, action, plans);
        }
    } else {
        plans.push(AttackPlan { first, after: copy });
    }
}
