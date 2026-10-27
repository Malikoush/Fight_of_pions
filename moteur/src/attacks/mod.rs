//! Les attaques : leurs données (familles, raretés, caractéristiques) et leurs règles,
//! une famille par fichier. Une famille ne modifie jamais la partie : elle décrit des
//! effets primitifs ([`Effect`]) que le résolveur de la partie applique.

mod card;
mod effect;
mod ours;
mod rules;

pub use card::{Attack, AttackKind, AttackStats, CatalogEntry, Rarity};
pub(crate) use effect::Effect;
pub(crate) use rules::{AttackContext, AttackRules, can_complete};

impl AttackKind {
    /// Les règles de cette famille.
    pub(crate) fn rules(self) -> &'static dyn AttackRules {
        match self {
            AttackKind::Ours => &ours::Ours,
        }
    }
}

impl Attack {
    /// Vrai si le moteur sait jouer cette variante.
    pub fn is_implemented(self) -> bool {
        self.kind.rules().rarities().contains(&self.rarity)
    }
}
