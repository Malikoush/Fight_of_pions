use std::ops::RangeInclusive;

use crate::game::{has_effect_of, pion_has};
use crate::{ActiveEffect, Board, EffectKind, Pion, PionId, Player, Position, Rarity};

use super::Effect;

/// Quand un pion doit pouvoir lancer une attaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Launch {
    /// Maintenant : le mal du débarquement et l'immobilisation comptent.
    Now,
    /// Un jour : un pion posé ce tour ou immobilisé pourra lancer plus tard.
    Someday,
}

/// Vrai si ce pion est arrivé ce tour alors que le mal du débarquement compte
/// (`sick_from` : l'identité du premier pion arrivé ce tour, `None` sans mal du
/// débarquement). Les identités suivent l'ordre des poses.
pub(crate) fn is_new(pion: Pion, sick_from: Option<PionId>) -> bool {
    sick_from.is_some_and(|first_new| pion.id() >= first_new)
}

/// La seule définition d'un lanceur : un pion de ce joueur qui peut agir `when` (maintenant :
/// ni arrivé ce tour avec le mal du débarquement, ni immobilisé). Partagée par la partie
/// (le lanceur) et par les familles qui font agir plusieurs de tes pions : chacun est un
/// lanceur.
pub(crate) fn can_act(
    pion: Pion,
    player: Player,
    when: Launch,
    effects: &[ActiveEffect],
    sick_from: Option<PionId>,
) -> bool {
    let held_back = when == Launch::Now
        && (is_new(pion, sick_from) || has_effect_of(effects, pion.id(), EffectKind::Immobilized));
    pion.owner() == player && !held_back
}

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
    /// Quand les pions doivent pouvoir lancer (voir [`AttackContext::can_act`]).
    pub launch: Launch,
    /// L'identité du premier pion arrivé ce tour, si le mal du débarquement compte.
    pub sick_from: Option<PionId>,
    /// Les pions que le joueur qui attaque a en réserve (les cartes qui posent en ont
    /// besoin).
    pub reserve: u32,
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

    /// Vrai si ce pion peut agir comme lanceur : à toi, et prêt au moment voulu.
    pub fn can_act(&self, pion: Pion) -> bool {
        can_act(pion, self.player, self.launch, self.effects, self.sick_from)
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

    /// Vrai si l'attaque peut s'arrêter après ces cibles, quand leur nombre le permet. Par
    /// défaut, toujours ; le Lièvre, par exemple, attend qu'un pion choisi ait bougé.
    fn can_stop(&self, _context: &AttackContext, _chosen: &[Position]) -> bool {
        true
    }

    /// Vrai si, dans cette rareté, ce que l'attaque peut viser ne dépend pas de la case du
    /// lanceur (ses cibles sont n'importe où) : savoir si elle a une cible demande alors un
    /// seul lanceur prêt, au lieu de tous les essayer. Par défaut, faux. Un test vérifie, sur
    /// des parties au hasard, que chaque famille qui le dit a raison.
    fn ignores_launcher(&self, _rarity: Rarity) -> bool {
        false
    }
}

/// Vrai si l'attaque peut être résolue avec ces cibles : leur nombre est permis, elle peut
/// s'arrêter là, et elle fait quelque chose.
pub(crate) fn is_complete(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &[Position],
) -> bool {
    rules.target_count(context.rarity).contains(&chosen.len())
        && rules.can_stop(context, chosen)
        && !rules.effects(context, chosen).is_empty()
}

/// Vrai si, à partir des cibles déjà choisies, on peut encore finir l'attaque. C'est ce qui
/// garantit qu'aucun choix proposé ne mène à une impasse : le moteur n'a pas d'« annuler ».
///
/// C'est une recherche en profondeur, sans mémoire. Elle reste courte parce que toutes les
/// familles actuelles deviennent complètes dès leur premier choix utile (un pas, un saut,
/// une cible) : la recherche s'arrête au premier choix qui réussit. Une famille où ce
/// n'est plus vrai devra être mesurée (`random_games`).
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
