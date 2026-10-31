//! Le Médecin (fiche n°05) : il ramène tes pions tombés, là où ils sont tombés.
//!
//! - Commune : ressuscite 1 pion sur sa tombe, si elle est libre.
//! - Rare : ressuscite jusqu'à 2 pions, sur des tombes libres.
//! - Épique : ressuscite 1 pion, même sur une tombe occupée par un pion adverse (tué) ; le
//!   pion ressuscité est protégé 2 tours.
//! - Légendaire : ressuscite tous tes pions qui peuvent revenir, même sur des tombes
//!   occupées par un pion adverse ; il en faut au moins un.
//!
//! Une **tombe** est laissée par un pion tué qui part à l'infirmerie ; elle disparaît quand
//! il en sort. On ne ressuscite que sur la tombe du pion lui-même. Une tombe occupée par
//! ton propre pion, ou par un pion adverse protégé, ne permet pas de revenir (carnet,
//! 2026-10-30). Sans infirmerie, il n'y a pas de tombe : le Médecin n'est pas jouable.
//!
//! Les cibles choisies sont les cases des tombes.

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::{EffectKind, Position, Rarity};

/// La protection du pion ressuscité par l'Épique.
const REVIVED_GUARD_TURNS: u32 = 2;

pub(crate) struct Medecin;

impl AttackRules for Medecin {
    fn rarities(&self) -> &'static [Rarity] {
        &[
            Rarity::Common,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }

    fn ignores_launcher(&self, _rarity: Rarity) -> bool {
        true
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Common | Rarity::Epic => 1..=1,
            Rarity::Rare => 1..=2,
            // Tous les pions qui peuvent revenir, sans choix.
            Rarity::Legendary => 0..=0,
        }
    }

    /// Les tombes de tes pions où ils peuvent revenir, pas encore choisies.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        if context.rarity == Rarity::Legendary {
            return Vec::new();
        }
        reachable_tombs(context)
            .filter(|pos| !chosen.contains(pos))
            .collect()
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        let tombs: Vec<Position> = if context.rarity == Rarity::Legendary {
            reachable_tombs(context).collect()
        } else {
            targets.to_vec()
        };
        tombs
            .into_iter()
            .flat_map(|at| revive(context, at))
            .collect()
    }
}

/// Les cases des tombes de tes pions où ils peuvent revenir, rangée par rangée : libres
/// pour la Commune et la Rare ; libres, ou occupées par un pion adverse qu'on peut tuer,
/// pour l'Épique et la Légendaire.
fn reachable_tombs<'a>(context: &'a AttackContext) -> impl Iterator<Item = Position> + 'a {
    let through_enemies = matches!(context.rarity, Rarity::Epic | Rarity::Legendary);
    context.board.positions().filter(move |&pos| {
        let ours = context
            .board
            .tomb(pos)
            .is_some_and(|dead| dead.owner() == context.player);
        let open = context.board.is_free(pos) || (through_enemies && context.can_kill(pos));
        ours && open
    })
}

/// Ressusciter le pion de la tombe de cette case : tuer d'abord l'occupant adverse s'il y
/// en a un, et protéger le pion revenu avec l'Épique.
fn revive(context: &AttackContext, at: Position) -> Vec<Effect> {
    let Some(dead) = context.board.tomb(at) else {
        return Vec::new();
    };
    let mut effects = Vec::new();
    if !context.board.is_free(at) {
        effects.push(Effect::Kill { at });
    }
    effects.push(Effect::Resurrect {
        pion: dead.id(),
        at,
    });
    if context.rarity == Rarity::Epic {
        effects.push(Effect::Apply {
            at,
            kind: EffectKind::Protected,
            turns: REVIVED_GUARD_TURNS,
        });
    }
    effects
}
