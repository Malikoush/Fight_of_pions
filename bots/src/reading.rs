//! Ce qu'un bot lit dans la partie, sans recopier de règle : la légalité vient de
//! `legal_actions`, les poses restantes de `placements_left`, l'effet d'une attaque ou
//! d'une fin de tour d'une copie de la partie où on les a joués.

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
