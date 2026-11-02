//! Attila (fiche n°04) : le contre des effets en cours, sur tout le plateau.
//!
//! - Commune : retire d'un pion tous les statuts placés par l'adversaire (immobilisé,
//!   protégé), quelle que soit la carte qui les a posés. Le pion peut être à toi (libérer
//!   un pion immobilisé) ou à l'adversaire (lui ôter sa protection).
//!
//! - Rare : annule 1 effet adverse entier.
//! - Épique : annule 1 ou 2 effets adverses entiers.
//!
//! Un **effet entier**, c'est tout ce qu'un jeu de carte a laissé en jeu : toutes les
//! immobilisations d'un Rugissement, les deux moitiés d'un pacte, les gardes d'une Reine…
//! Le joueur le désigne en visant un pion qui porte un effet adverse : tous les effets
//! adverses de ce pion sont annulés, chacun en entier, sur tous les pions qu'ils touchent.
//! L'Épique vise 1 ou 2 pions : le second, facultatif, porte un effet adverse que le
//! premier n'annule pas déjà ; sans un tel pion, l'attaque est jouée après le premier. La
//! couronne d'une reine reste (carnet, 2026-11-02).
//!
//! Les jetons de l'adversaire (Massacre divin), posés devant lui et non sur un pion, se
//! visent à part ([`crate::Action::ChooseEnemyTokens`]) et comptent pour un pion : toutes
//! ses piles sont annulées, chacune étant un effet entier.
//!
//! La Légendaire (Terre brûlée) viendra en B6d.

use std::ops::RangeInclusive;

use super::Effect;
use super::rules::{AttackContext, AttackRules};
use crate::game::cancellable_plays_on;
use crate::{Cell, PlayId, Position, Rarity};

pub(crate) struct Attila;

impl AttackRules for Attila {
    fn rarities(&self) -> &'static [Rarity] {
        &[Rarity::Common, Rarity::Rare, Rarity::Epic]
    }

    fn ignores_launcher(&self, _rarity: Rarity) -> bool {
        true
    }

    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize> {
        match rarity {
            Rarity::Epic => 1..=2,
            _ => 1..=1,
        }
    }

    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
        match context.rarity {
            Rarity::Common => carrying_enemy_statuses(context, chosen),
            _ => carrying_new_enemy_plays(context, chosen),
        }
    }

    fn enemy_tokens_effects(&self, context: &AttackContext) -> Vec<Effect> {
        if context.rarity == Rarity::Common || !context.enemy_has_tokens() {
            return Vec::new();
        }
        vec![Effect::CancelEnemyTokens]
    }

    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect> {
        targets
            .iter()
            .map(|&at| match context.rarity {
                Rarity::Common => Effect::RemoveEnemyStatuses { at },
                _ => Effect::CancelEnemyEffects { at },
            })
            .collect()
    }
}

/// Les jeux de carte adverses dont un effet annulable vise le pion de cette case.
fn enemy_plays_at(context: &AttackContext, pos: Position) -> Vec<PlayId> {
    context
        .board
        .get(pos)
        .and_then(Cell::pion)
        .map_or_else(Vec::new, |pion| {
            cancellable_plays_on(context.effects, pion.id(), context.player.opponent())
        })
}

/// Les pions, n'importe où, qui portent un effet adverse que les pions déjà choisis
/// n'annulent pas : viser un pion dont tous les effets partent déjà ne servirait à rien.
fn carrying_new_enemy_plays(context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
    let cancelled: Vec<PlayId> = chosen
        .iter()
        .flat_map(|&pos| enemy_plays_at(context, pos))
        .collect();
    context
        .board
        .positions()
        .filter(|pos| !chosen.contains(pos))
        .filter(|&pos| {
            enemy_plays_at(context, pos)
                .iter()
                .any(|play| !cancelled.contains(play))
        })
        .collect()
}

/// Les pions, n'importe où, qui portent un statut posé par l'adversaire.
fn carrying_enemy_statuses(context: &AttackContext, chosen: &[Position]) -> Vec<Position> {
    let enemy = context.player.opponent();
    context
        .board
        .positions()
        .filter(|pos| !chosen.contains(pos))
        .filter(|&pos| {
            context
                .board
                .get(pos)
                .and_then(Cell::pion)
                .is_some_and(|pion| {
                    context
                        .effects
                        .iter()
                        .any(|effect| effect.is_status_on_by(pion.id(), enemy))
                })
        })
        .collect()
}
