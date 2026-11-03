//! Le Coup du taureau (fiche n°07) : une charge en ligne droite, assez puissante pour percer
//! le mur du plateau.
//!
//! - Commune : charge d'1 case ; le pion adverse qui s'y trouve recule d'1 case et le
//!   lanceur prend sa place. S'il ne peut pas reculer, la charge est impossible.
//! - Rare : charge d'1 case ; le pion adverse recule de 2 cases et le lanceur prend sa
//!   place. S'il ne peut pas reculer des 2 cases, il est écrasé (kill) ; protégé, il recule
//!   aussi loin que possible et n'est jamais tué, et s'il ne peut pas reculer du tout, la
//!   charge est impossible.
//! - Épique : charge de 2 cases, Légendaire : de 3 cases ; chaque pion adverse sur le
//!   passage est tué. Un pion adverse protégé arrête la charge.
//!
//! Le joueur choisit seulement la direction (8 directions), en désignant la case collée au
//! lanceur, même dans le mur ; la charge va aussi loin que sa rareté le permet (carnet,
//! 2026-10-30). Le lanceur creuse le mur qu'il atteint et continue dans la case creusée ;
//! un pion poussé ne creuse pas, le mur le bloque. Tes propres pions arrêtent la charge,
//! une tombe non ; au-delà du mur, la grille l'arrête. La charge avance d'au moins une case
//! (carnet, 2026-11-03).

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{Direction, Position, Rarity};

/// La longueur de la charge de la Commune et de la Rare.
const SHORT_CHARGE: usize = 1;
/// La longueur de la charge de l'Épique.
const EPIC_CHARGE: usize = 2;
/// La longueur de la charge de la Légendaire.
const LEGENDARY_CHARGE: usize = 3;
/// Le recul du pion adverse poussé par la Commune.
const COMMON_RECOIL: usize = 1;
/// Le recul du pion adverse poussé par la Rare.
const RARE_RECOIL: usize = 2;

pub(crate) struct CoupDuTaureau;

impl AttackRules for CoupDuTaureau {
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

    /// La case choisie donne la direction : un pion chargé n'est pas visé (carnet, B8c).
    fn aims_at_targets(&self) -> bool {
        false
    }

    /// Les cases collées au lanceur dans les directions où la charge avance.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        if !chosen.is_empty() {
            return Vec::new();
        }
        Direction::ALL
            .iter()
            .filter(|&&dir| !charge(context, dir).is_empty())
            .filter_map(|&dir| context.launcher.step(dir))
            .collect()
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let &[first] = targets else {
            return Vec::new();
        };
        Direction::ALL
            .into_iter()
            .find(|&dir| context.launcher.step(dir) == Some(first))
            .map_or_else(Vec::new, |dir| charge(context, dir))
    }
}

/// La charge dans cette direction, case par case ; vide si elle n'avance pas.
fn charge(context: &AttackContext, dir: Direction) -> Vec<Effect> {
    let board = context.board;
    let length = match context.rarity {
        Rarity::Common | Rarity::Rare => SHORT_CHARGE,
        Rarity::Epic => EPIC_CHARGE,
        Rarity::Legendary => LEGENDARY_CHARGE,
    };
    let mut effects = Vec::new();
    let mut at = context.launcher;
    for _ in 0..length {
        let Some(next) = at.step(dir) else {
            break;
        };
        if board.is_wall(next) {
            effects.push(Effect::Dig { at: next });
        } else if context.is_enemy(next) {
            let Some(clearing) = clear(context, next, dir) else {
                break;
            };
            effects.extend(clearing);
        } else if !board.is_free(next) {
            // Un de tes pions, ou le bord de la grille.
            break;
        }
        effects.push(Effect::Move { from: at, to: next });
        at = next;
    }
    effects
}

/// Ce que la charge fait au pion adverse de la case `at` pour y entrer : le pousser
/// (Commune, Rare) ou le tuer (Épique, Légendaire). `None` s'il l'arrête.
fn clear(context: &AttackContext, at: Position, dir: Direction) -> Option<Vec<Effect>> {
    let recoil = match context.rarity {
        Rarity::Common => COMMON_RECOIL,
        Rarity::Rare => RARE_RECOIL,
        Rarity::Epic | Rarity::Legendary => {
            return context.can_kill(at).then(|| vec![Effect::Kill { at }]);
        }
    };
    // Les cases libres derrière lui, jusqu'à son recul : le mur le bloque.
    let behind: Vec<Position> = std::iter::successors(at.step(dir), |pos| pos.step(dir))
        .take(recoil)
        .take_while(|&pos| context.board.is_free(pos))
        .collect();
    let push = |to: &Position| vec![Effect::Move { from: at, to: *to }];
    match behind.last() {
        Some(to) if behind.len() == recoil => Some(push(to)),
        // Bloqué avant la fin de son recul : écrasé, ou poussé aussi loin que possible
        // s'il est protégé (Rare) ; la Commune ne passe pas.
        _ if context.rarity == Rarity::Common => None,
        _ if context.can_kill(at) => Some(vec![Effect::Kill { at }]),
        Some(to) => Some(push(to)),
        None => None,
    }
}
