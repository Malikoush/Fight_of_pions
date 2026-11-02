//! Familia (fiche n°11) : des familles de pions en ligne, où l'enfant encadré par sa
//! famille est protégé.
//!
//! - Commune : le lanceur, déjà encadré par deux de tes pions, devient enfant.
//! - Rare : pose 1 parent pour finir d'encadrer le lanceur, qui devient enfant.
//! - Épique : pose 2 parents de part et d'autre du lanceur, qui devient enfant.
//! - Légendaire : pose 2 parents de part et d'autre du lanceur et d'un de tes pions collé à
//!   lui en ligne : les 2 du milieu deviennent enfants.
//!
//! « En ligne droite » : les 8 directions. Le joueur choisit une seule case : un des deux
//! pions qui encadrent (Commune), la case du parent (Rare, et l'un des deux pour l'Épique),
//! ou le second enfant (Légendaire, dont les parents se posent tout seuls aux deux bouts).
//! Les pions qui encadrent peuvent être n'importe lesquels des tiens ; le second enfant de
//! la Légendaire est un lanceur aussi. Un enfant ne le redevient pas tant qu'il l'est, et la
//! carte pose ses
//! parents ou rien : il faut assez de pions en réserve (carnet, 2026-11-02).
//!
//! L'enfant est protégé (`EffectKind::Protected`), sans durée, tant que les deux mêmes
//! membres l'encadrent ; la partie retire cette protection pour de bon dès que ce n'est
//! plus le cas (`Game::end_unframed_children`).

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::both_sides;
use super::rules::{AttackContext, AttackRules};
use crate::{Cell, Direction, Position, Rarity};

pub(crate) struct Familia;

/// Ce que fait la carte dans une direction depuis le lanceur.
struct Plan {
    /// La case à choisir.
    target: Position,
    /// Les cases libres où se posent les parents.
    parents: Vec<Position>,
    /// Chaque enfant, avec les deux cases qui l'encadrent.
    children: Vec<(Position, [Position; 2])>,
}

/// Ce que la carte ferait dans cette direction (la cible choisie y est collée au lanceur),
/// si c'est permis, le lanceur n'étant pas déjà un enfant.
fn plan(context: &AttackContext, dir: Direction) -> Option<Plan> {
    let board = context.board;
    let launcher = context.launcher;
    let mine = |pos: Position| board.owner(pos) == Some(context.player);
    let (ahead, behind) = both_sides(board, launcher, dir)?;
    let plan = match context.rarity {
        Rarity::Common => (mine(ahead) && mine(behind)).then(|| Plan {
            target: ahead,
            parents: Vec::new(),
            children: vec![(launcher, [ahead, behind])],
        }),
        Rarity::Rare => (board.is_free(ahead) && mine(behind)).then(|| Plan {
            target: ahead,
            parents: vec![ahead],
            children: vec![(launcher, [ahead, behind])],
        }),
        Rarity::Epic => (board.is_free(ahead) && board.is_free(behind)).then(|| Plan {
            target: ahead,
            parents: vec![ahead, behind],
            children: vec![(launcher, [ahead, behind])],
        }),
        Rarity::Legendary => {
            let beyond = board.neighbor(ahead, dir)?;
            let sibling_ready = board
                .get(ahead)
                .and_then(Cell::pion)
                .is_some_and(|pion| context.can_act(pion));
            (sibling_ready
                && !context.is_child(ahead)
                && board.is_free(behind)
                && board.is_free(beyond))
            .then(|| Plan {
                target: ahead,
                parents: vec![behind, beyond],
                children: vec![(launcher, [behind, ahead]), (ahead, [launcher, beyond])],
            })
        }
    }?;
    let affordable = u32::try_from(plan.parents.len()).is_ok_and(|n| n <= context.reserve);
    affordable.then_some(plan)
}

/// Ce que la carte peut faire depuis le lanceur, au plus une fois par direction. Rien si le
/// lanceur est déjà un enfant : il ne le redevient pas.
fn plans<'a>(context: &'a AttackContext) -> impl Iterator<Item = Plan> + 'a {
    let directions: &[Direction] = if context.is_child(context.launcher) {
        &[]
    } else {
        &Direction::ALL
    };
    directions.iter().filter_map(move |&dir| plan(context, dir))
}

impl AttackRules for Familia {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    fn target_count(&self, _rarity: Rarity) -> RangeInclusive<usize> {
        1..=1
    }

    fn candidate_targets(&self, context: &AttackContext, _chosen: &[Position]) -> Vec<Position> {
        plans(context).map(|plan| plan.target).collect()
    }

    /// Les parents d'abord, puis les enfants : un enfant n'est encadré qu'une fois ses
    /// parents posés.
    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let [target] = targets else {
            return Vec::new();
        };
        plans(context)
            .find(|plan| plan.target == *target)
            .map_or_else(Vec::new, |plan| {
                let parents = plan.parents.into_iter().map(|at| Effect::Place { at });
                let children = plan
                    .children
                    .into_iter()
                    .map(|(at, framed_by)| Effect::MakeChild { at, framed_by });
                parents.chain(children).collect()
            })
    }
}
