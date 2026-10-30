use std::ops::RangeInclusive;

use crate::game::pion_has;
use crate::{ActiveEffect, Board, EffectKind, Player, Position, Rarity};

use super::Effect;

/// Tout ce qu'une famille sait d'une attaque en cours de choix.
pub(crate) struct AttackContext<'a> {
    pub board: &'a Board,
    /// Les effets continus en jeu, pour lire les statuts des pions.
    pub effects: &'a [ActiveEffect],
    /// Le joueur qui attaque.
    pub player: Player,
    /// La case du pion lanceur.
    pub launcher: Position,
    pub rarity: Rarity,
}

impl AttackContext<'_> {
    /// Vrai si cette case porte un pion de l'adversaire du joueur qui attaque.
    pub fn is_enemy(&self, pos: Position) -> bool {
        self.board.owner(pos) == Some(self.player.opponent())
    }

    /// Vrai si le pion de cette case a un effet de cette sorte.
    pub fn has(&self, pos: Position, kind: EffectKind) -> bool {
        pion_has(self.board, self.effects, pos, kind)
    }

    /// Vrai si l'on peut choisir ce pion comme cible d'un kill : adverse et non protégé
    /// (carnet, 2026-10-30 : un kill ne vise jamais un pion qu'il ne tuerait pas).
    pub fn can_kill(&self, pos: Position) -> bool {
        self.is_enemy(pos) && !self.has(pos, EffectKind::Protected)
    }
}

/// Les règles d'une famille d'attaques. Sans état : tout vient de la partie, et la partie
/// ne garde que des données simples. Une famille ne modifie jamais rien elle-même : elle
/// décrit des effets primitifs, que le résolveur de la partie applique.
pub(crate) trait AttackRules {
    /// Les raretés que le moteur sait jouer pour cette famille.
    fn rarities(&self) -> &'static [Rarity];

    /// Combien de cibles choisir après le lanceur : de `min` à `max` (« jusqu'à 2 » :
    /// `1..=2`). Une attaque sans cible (`0..=0`) se résout dès le choix du lanceur.
    fn target_count(&self, rarity: Rarity) -> RangeInclusive<usize>;

    /// Les cases qu'on peut choisir comme prochaine cible, sachant celles déjà choisies.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position>;

    /// Les effets de l'attaque avec ces cibles. Une attaque n'est jouable que si elle en a
    /// au moins un : le moteur ne propose jamais un coup sans effet.
    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect>;
}

/// Vrai si l'attaque peut être résolue avec ces cibles : leur nombre est permis, et elle
/// fait quelque chose.
pub(crate) fn is_complete(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &[Position],
) -> bool {
    rules.target_count(context.rarity).contains(&chosen.len())
        && !rules.effects(context, chosen).is_empty()
}

/// Vrai si, à partir des cibles déjà choisies, on peut encore finir l'attaque. C'est ce qui
/// garantit qu'aucun choix proposé ne mène à une impasse : le moteur n'a pas d'« annuler ».
pub(crate) fn can_complete(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &mut Vec<Position>,
) -> bool {
    is_complete(rules, context, chosen) || can_choose_more(rules, context, chosen)
}

/// Vrai si l'on peut encore choisir une cible après lesquelles l'attaque peut finir.
pub(crate) fn can_choose_more(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &mut Vec<Position>,
) -> bool {
    if chosen.len() >= *rules.target_count(context.rarity).end() {
        return false;
    }
    rules
        .candidate_targets(context, chosen)
        .into_iter()
        .any(|target| {
            chosen.push(target);
            let possible = can_complete(rules, context, chosen);
            chosen.pop();
            possible
        })
}
