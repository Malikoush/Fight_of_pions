//! Le Lièvre (fiche n°10) : le mouvement pur, pas à pas.
//!
//! - Commune : 1 de tes pions avance de 2 pas.
//! - Rare : 2 de tes pions avancent de 2 pas chacun.
//! - Légendaire : 3 de tes pions avancent de 3 pas chacun.
//!
//! Un pas va vers une case libre collée (8 directions) ; on peut tourner en chemin. Chaque
//! pion avance **jusqu'à** son nombre de pas et finit sur une autre case que sa case de
//! départ ; on peut aussi s'arrêter avant le nombre de pions (carnet, 2026-10-30). Les
//! pions bougent l'un après l'autre, et chacun est un lanceur : posé avant ce tour et non
//! immobilisé. Le premier est le lanceur choisi.
//!
//! Les cibles choisies mêlent les pas (une case libre) et les pions suivants (une case
//! occupée par un de tes pions) : les deux ne se confondent jamais.
//!
//! - Épique : 3 jetons de pas sur le lanceur ; à chacun de tes tours, il peut en dépenser
//!   1 pour avancer de jusqu'à 2 pas (un geste gratuit, `game/hare.rs`). Pas jouable
//!   depuis un pion qui a encore des jetons (carnet, 2026-11-02).

use std::ops::RangeInclusive;

use super::Effect;
use super::reach::around;
use super::rules::{AttackContext, AttackRules};
use crate::{Board, Cell, EffectEnd, EffectKind, PionId, Position, Rarity};

/// Les jetons de pas de l'Épique.
const HARE_TOKENS: u32 = 3;
/// Les pas que permet un jeton de l'Épique, au plus.
pub(crate) const HARE_TOKEN_STEPS: u32 = 2;

pub(crate) struct Lievre;

/// Combien de pions avancent, et de combien de pas chacun, au plus. L'Épique ne passe
/// jamais par ici : elle ne choisit aucune cible.
fn limits(rarity: Rarity) -> (usize, usize) {
    match rarity {
        Rarity::Common | Rarity::Epic => (1, 2),
        Rarity::Rare => (2, 2),
        Rarity::Legendary => (3, 3),
    }
}

impl AttackRules for Lievre {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    /// Au plus : les pas du lanceur, puis pour chaque pion suivant, son choix et ses pas.
    /// L'Épique pose ses jetons sur le lanceur, sans cible.
    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        if rarity == Rarity::Epic {
            return 0..=0;
        }
        let (pions, steps) = limits(rarity);
        1..=steps + (pions - 1) * (1 + steps)
    }

    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        if context.rarity == Rarity::Epic {
            return Vec::new();
        }
        let Some(walk) = Walk::replay(context, chosen) else {
            return Vec::new();
        };
        let (pions, steps) = limits(context.rarity);
        let mut cells = Vec::new();
        if walk.current.steps < steps {
            cells.extend(walk.free_steps());
        }
        if walk.current.is_done() && walk.pions() < pions {
            cells.extend(walk.next_pions(context));
        }
        cells
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        if context.rarity == Rarity::Epic {
            if context.has(context.launcher, EffectKind::StepTokens) {
                return Vec::new();
            }
            return vec![Effect::Attach {
                at: context.launcher,
                kind: EffectKind::StepTokens,
                end: EffectEnd::Tokens(HARE_TOKENS),
            }];
        }
        Walk::replay(context, targets).map_or_else(Vec::new, |walk| {
            walk.legs()
                .filter(|leg| leg.is_done())
                .map(|leg| Effect::Move {
                    from: leg.start,
                    to: leg.at,
                })
                .collect()
        })
    }

    /// Le dernier pion choisi doit avoir bougé ; l'Épique, qui ne fait bouger personne,
    /// s'arrête toujours.
    fn can_stop(&self, context: &AttackContext, chosen: &[Position]) -> bool {
        context.rarity == Rarity::Epic
            || Walk::replay(context, chosen).is_some_and(|walk| walk.current.is_done())
    }
}

/// Le trajet d'un pion.
struct Leg {
    pion: PionId,
    start: Position,
    at: Position,
    steps: usize,
}

impl Leg {
    /// Le trajet d'un pion qui n'a pas encore bougé.
    fn at_start(pion: PionId, start: Position) -> Leg {
        Leg {
            pion,
            start,
            at: start,
            steps: 0,
        }
    }

    /// Vrai si le pion a bougé et ne finit pas sur sa case de départ.
    fn is_done(&self) -> bool {
        self.steps > 0 && self.at != self.start
    }
}

/// Les trajets après les choix déjà faits, rejoués sur une copie du plateau.
struct Walk {
    board: Board,
    /// Les trajets finis des pions choisis avant.
    done: Vec<Leg>,
    /// Le trajet en cours, celui du dernier pion choisi (d'abord le lanceur).
    current: Leg,
}

impl Walk {
    /// Rejoue ces choix depuis le lanceur ; `None` si l'un d'eux n'est pas permis.
    fn replay(context: &AttackContext, chosen: &[Position]) -> Option<Walk> {
        let launcher = context.board.get(context.launcher).and_then(Cell::pion)?;
        let mut walk = Walk {
            board: context.board.clone(),
            done: Vec::new(),
            current: Leg::at_start(launcher.id(), context.launcher),
        };
        let (pions, steps) = limits(context.rarity);
        for &pos in chosen {
            if walk.current.steps < steps && walk.free_steps().any(|cell| cell == pos) {
                walk.board.move_pion(walk.current.at, pos).ok()?;
                walk.current.at = pos;
                walk.current.steps += 1;
            } else if walk.current.is_done()
                && walk.pions() < pions
                && walk.next_pions(context).any(|cell| cell == pos)
            {
                let pion = walk.board.get(pos).and_then(Cell::pion)?;
                let finished = std::mem::replace(&mut walk.current, Leg::at_start(pion.id(), pos));
                walk.done.push(finished);
            } else {
                return None;
            }
        }
        Some(walk)
    }

    /// Tous les trajets, dans l'ordre : ceux des pions choisis avant, puis l'actuel.
    fn legs(&self) -> impl Iterator<Item = &Leg> {
        self.done.iter().chain(std::iter::once(&self.current))
    }

    /// Le nombre de pions choisis, lanceur compris.
    fn pions(&self) -> usize {
        self.done.len() + 1
    }

    /// Les cases libres collées au pion en cours.
    fn free_steps(&self) -> impl Iterator<Item = Position> + '_ {
        around(&self.board, self.current.at).filter(|&cell| self.board.is_free(cell))
    }

    /// Les pions qui peuvent avancer ensuite : à toi, prêts à agir, pas encore choisis.
    fn next_pions<'a>(&'a self, context: &'a AttackContext) -> impl Iterator<Item = Position> + 'a {
        self.board.positions().filter(move |&pos| {
            self.board
                .get(pos)
                .and_then(Cell::pion)
                .is_some_and(|pion| {
                    context.can_act(pion) && self.legs().all(|leg| leg.pion != pion.id())
                })
        })
    }
}
