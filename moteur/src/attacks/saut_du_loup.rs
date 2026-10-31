//! Le Saut du loup (fiche n°02) : le lanceur bondit comme aux dames et frappe le pion
//! adverse qu'il survole.
//!
//! - Commune : 1 saut ; un pion adverse survolé est immobilisé 2 tours.
//! - Rare : 1 saut meurtrier.
//! - Épique : jusqu'à 2 sauts meurtriers.
//! - Légendaire : jusqu'à 3 sauts meurtriers.
//!
//! Un saut fait toujours 2 cases en ligne droite (8 directions) et atterrit sur une case
//! libre, par-dessus n'importe quoi : du vide, un de tes pions, un pion adverse. Chaque
//! saut part de la case d'arrivée du précédent ; un même pion ne peut pas être survolé
//! deux fois, mais le loup peut repasser par une case, y compris sa case de départ. Un
//! saut meurtrier tue tout de suite le pion adverse survolé (sa case est libre pour la
//! suite), sauf s'il est protégé (carnet, 2026-10-30).
//!
//! Les cibles choisies sont les cases d'arrivée, dans l'ordre des sauts.

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{Board, Cell, Direction, EffectKind, PionId, Position, Rarity};

/// La durée de l'immobilisation de la Commune.
const HOWL_TURNS: u32 = 2;

pub(crate) struct SautDuLoup;

impl AttackRules for SautDuLoup {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Common | Rarity::Rare => 1..=1,
            Rarity::Epic => 1..=2,
            Rarity::Legendary => 1..=3,
        }
    }

    /// Les cases où le loup peut atterrir au prochain saut.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        Walk::replay(context, chosen).map_or_else(Vec::new, |walk| {
            walk.hops().map(|(_, landing)| landing).collect()
        })
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        Walk::replay(context, targets).map_or_else(Vec::new, |walk| walk.effects)
    }
}

/// Le chemin du loup après les sauts choisis, rejoué sur une copie du plateau.
struct Walk {
    board: Board,
    /// La case actuelle du loup.
    at: Position,
    /// Les pions déjà survolés.
    jumped: Vec<PionId>,
    effects: Vec<Effect>,
}

impl Walk {
    /// Rejoue ces sauts depuis le lanceur ; `None` si l'un d'eux n'est pas permis.
    fn replay(context: &AttackContext, landings: &[Position]) -> Option<Walk> {
        let mut walk = Walk {
            board: context.board.clone(),
            at: context.launcher,
            jumped: Vec::new(),
            effects: Vec::new(),
        };
        for &landing in landings {
            let (over, _) = walk.hops().find(|&(_, to)| to == landing)?;
            walk.jump(context, over, landing)?;
        }
        Some(walk)
    }

    /// Les sauts possibles depuis la case actuelle : (case survolée, case d'arrivée).
    fn hops(&self) -> impl Iterator<Item = (Position, Position)> + '_ {
        Direction::ALL.iter().filter_map(move |&dir| {
            let over = self.board.neighbor(self.at, dir)?;
            let landing = self.board.neighbor(over, dir)?;
            let free = self.board.is_free(landing);
            let not_jumped_yet = self
                .board
                .get(over)
                .and_then(Cell::pion)
                .is_none_or(|pion| !self.jumped.contains(&pion.id()));
            (free && not_jumped_yet).then_some((over, landing))
        })
    }

    /// Un saut par-dessus `over` jusqu'à `landing`, et ce qu'il fait au pion survolé.
    fn jump(&mut self, context: &AttackContext, over: Position, landing: Position) -> Option<()> {
        if let Some(pion) = self.board.get(over).and_then(Cell::pion) {
            self.jumped.push(pion.id());
            // Les pions adverses ne bougent pas pendant les sauts : la partie les voit
            // toujours à leur case.
            if context.is_enemy(over) {
                if context.rarity == Rarity::Common {
                    self.effects.push(Effect::Apply {
                        at: over,
                        kind: EffectKind::Immobilized,
                        turns: HOWL_TURNS,
                    });
                } else if context.can_kill(over) {
                    self.effects.push(Effect::Kill { at: over });
                    self.board.remove_pion(over)?;
                }
            }
        }
        self.effects.push(Effect::Move {
            from: self.at,
            to: landing,
        });
        self.board.move_pion(self.at, landing).ok()?;
        self.at = landing;
        Some(())
    }
}
