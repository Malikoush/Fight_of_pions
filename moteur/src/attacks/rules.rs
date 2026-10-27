use crate::{Board, Player, Position, Rarity};

use super::Effect;

/// Tout ce qu'une famille sait d'une attaque en cours de choix.
pub(crate) struct AttackContext<'a> {
    pub board: &'a Board,
    /// Le joueur qui attaque.
    pub player: Player,
    /// La case du pion lanceur.
    pub launcher: Position,
    pub rarity: Rarity,
}

/// Les règles d'une famille d'attaques. Sans état : tout vient de la partie, et la partie
/// ne garde que des données simples. Une famille ne modifie jamais rien elle-même : elle
/// décrit des effets primitifs, que le résolveur de la partie applique.
pub(crate) trait AttackRules {
    /// Les raretés que le moteur sait jouer pour cette famille.
    fn rarities(&self) -> &'static [Rarity];

    /// Le nombre de cibles à choisir.
    fn target_count(&self, rarity: Rarity) -> usize;

    /// Les cases qu'on peut choisir comme prochaine cible, sachant celles déjà choisies.
    fn candidate_targets(&self, context: &AttackContext, chosen: &[Position]) -> Vec<Position>;

    /// Les effets de l'attaque, une fois toutes les cibles choisies.
    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect>;
}

/// Vrai si, à partir des cibles déjà choisies, on peut encore finir l'attaque. C'est ce qui
/// garantit qu'aucun choix proposé ne mène à une impasse : le moteur n'a pas d'« annuler ».
pub(crate) fn can_complete(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &mut Vec<Position>,
) -> bool {
    if chosen.len() >= rules.target_count(context.rarity) {
        return true;
    }
    for target in rules.candidate_targets(context, chosen) {
        chosen.push(target);
        let possible = can_complete(rules, context, chosen);
        chosen.pop();
        if possible {
            return true;
        }
    }
    false
}
