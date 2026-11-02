use std::ops::RangeInclusive;

use crate::game::{has_effect_of, is_child, pion_has};
use crate::{
    ActiveEffect, Board, Cell, EffectKind, Pion, PionId, Player, Position, Rarity, TokenPile,
};

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
    /// Les piles de jetons des deux joueurs (Massacre divin).
    pub token_piles: &'a [TokenPile],
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
    /// Faux si le joueur qui attaque est sous Terre brûlée : un coup qui créerait un effet
    /// continu n'est alors pas jouable (carnet, 2026-11-02). Toujours vrai pour « un jour ».
    pub may_create_effects: bool,
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

    /// Vrai si le pion de cette case est un enfant (Familia).
    pub fn is_child(&self, pos: Position) -> bool {
        self.board
            .get(pos)
            .and_then(Cell::pion)
            .is_some_and(|pion| is_child(self.effects, pion.id()))
    }

    /// Vrai si ce pion peut agir comme lanceur : à toi, et prêt au moment voulu.
    pub fn can_act(&self, pion: Pion) -> bool {
        can_act(pion, self.player, self.launch, self.effects, self.sick_from)
    }

    /// Vrai si l'adversaire du joueur qui attaque a au moins une pile de jetons.
    pub fn enemy_has_tokens(&self) -> bool {
        let enemy = self.player.opponent();
        self.token_piles.iter().any(|pile| pile.owner() == enemy)
    }
}

/// Les cibles déjà choisies d'une attaque : des cases, et peut-être les jetons de
/// l'adversaire ([`crate::Action::ChooseEnemyTokens`]), qui comptent pour une cible.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct Targets {
    /// Les cases choisies, dans l'ordre.
    pub cells: Vec<Position>,
    /// Vrai si les jetons de l'adversaire sont choisis.
    pub enemy_tokens: bool,
}

impl Targets {
    /// Le nombre de cibles choisies.
    pub fn len(&self) -> usize {
        self.cells.len() + usize::from(self.enemy_tokens)
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

    /// Les effets de l'attaque avec ces cases. Une attaque n'est jouable que si elle a au
    /// moins un effet : le moteur ne propose jamais un coup sans effet.
    fn effects(&self, context: &AttackContext, targets: &[Position]) -> Vec<Effect>;

    /// Les effets de viser les jetons de l'adversaire, qui comptent pour une cible : vide si
    /// l'attaque ne peut pas les viser maintenant. Par défaut, vide ; Attila Rare et Épique
    /// les annulent.
    fn enemy_tokens_effects(&self, _context: &AttackContext) -> Vec<Effect> {
        Vec::new()
    }

    /// Vrai si l'attaque peut s'arrêter après ces cibles, quand leur nombre le permet. Par
    /// défaut, toujours ; le Lièvre, par exemple, attend qu'un pion choisi ait bougé.
    fn can_stop(&self, _context: &AttackContext, _chosen: &Targets) -> bool {
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

/// Tous les effets de l'attaque avec ces cibles : ceux des cases, puis ceux des jetons de
/// l'adversaire s'ils sont choisis.
pub(crate) fn effects_of(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &Targets,
) -> Vec<Effect> {
    let mut effects = rules.effects(context, &chosen.cells);
    if chosen.enemy_tokens {
        effects.extend(rules.enemy_tokens_effects(context));
    }
    effects
}

/// Vrai si l'attaque peut être résolue avec ces cibles : leur nombre est permis, elle peut
/// s'arrêter là, elle fait quelque chose, et rien d'interdit par Terre brûlée.
pub(crate) fn is_complete(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &Targets,
) -> bool {
    if !rules.target_count(context.rarity).contains(&chosen.len())
        || !rules.can_stop(context, chosen)
    {
        return false;
    }
    let effects = effects_of(rules, context, chosen);
    !effects.is_empty()
        && (context.may_create_effects || !effects.iter().any(|effect| effect.creates_lasting()))
}

/// Vrai si les jetons de l'adversaire peuvent être la prochaine cible : pas encore choisis,
/// une cible de plus est permise, et l'attaque sait les viser maintenant.
pub(crate) fn can_choose_enemy_tokens(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &Targets,
) -> bool {
    !chosen.enemy_tokens
        && chosen.len() < *rules.target_count(context.rarity).end()
        && !rules.enemy_tokens_effects(context).is_empty()
}

/// Vrai si, à partir des cibles déjà choisies, on peut encore finir l'attaque. C'est ce qui
/// garantit qu'aucun choix proposé ne mène à une impasse : le moteur n'a pas d'« annuler ».
///
/// C'est une recherche en profondeur, sans mémoire. Elle reste courte parce que presque
/// toutes les familles deviennent complètes dès leur premier choix utile (un pas, un saut,
/// une cible) : la recherche s'arrête au premier choix qui réussit. Exception : Meilleurs
/// amis demande 2 choix, mais son second choix est direct (une case libre). Une famille plus profonde devra être
/// mesurée (`random_games`).
pub(crate) fn can_complete(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &mut Targets,
) -> bool {
    is_complete(rules, context, chosen) || can_choose_more(rules, context, chosen)
}

/// Vrai si l'on peut encore choisir une cible après lesquelles l'attaque peut finir.
pub(crate) fn can_choose_more(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &mut Targets,
) -> bool {
    if chosen.len() >= *rules.target_count(context.rarity).end() {
        return false;
    }
    let by_cell = rules
        .candidate_targets(context, &chosen.cells)
        .into_iter()
        .any(|target| {
            chosen.cells.push(target);
            let possible = can_complete(rules, context, chosen);
            chosen.cells.pop();
            possible
        });
    by_cell || can_complete_with_enemy_tokens(rules, context, chosen)
}

/// Vrai si les jetons de l'adversaire peuvent être la prochaine cible et que l'attaque peut
/// encore finir ensuite. `chosen` est rendu tel qu'il était.
pub(crate) fn can_complete_with_enemy_tokens(
    rules: &dyn AttackRules,
    context: &AttackContext,
    chosen: &mut Targets,
) -> bool {
    can_choose_enemy_tokens(rules, context, chosen) && {
        chosen.enemy_tokens = true;
        let possible = can_complete(rules, context, chosen);
        chosen.enemy_tokens = false;
        possible
    }
}
