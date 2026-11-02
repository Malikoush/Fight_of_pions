//! Les attaques : leurs données (familles, raretés, caractéristiques) et leurs règles,
//! une famille par fichier. Une famille ne modifie jamais la partie : elle décrit des
//! effets primitifs ([`Effect`]) que le résolveur de la partie applique.

mod attila;
mod beaute_fatale;
mod card;
mod effect;
mod familia;
mod interchange;
mod lievre;
mod medecin;
mod meilleurs_amis;
mod ours;
mod reach;
mod reine;
mod rules;
mod saut_du_loup;

pub use card::{Attack, AttackKind, AttackStats, CatalogEntry, Rarity};
pub(crate) use effect::Effect;
pub(crate) use reach::{around, both_sides, first_in_lines};
pub(crate) use rules::{
    AttackContext, AttackRules, Launch, can_act, can_choose_more, can_complete, is_complete, is_new,
};

impl AttackKind {
    /// Les règles de cette famille.
    pub(crate) fn rules(self) -> &'static dyn AttackRules {
        match self {
            AttackKind::BeauteFatale => &beaute_fatale::BeauteFatale,
            AttackKind::SautDuLoup => &saut_du_loup::SautDuLoup,
            AttackKind::Reine => &reine::Reine,
            AttackKind::Attila => &attila::Attila,
            AttackKind::Medecin => &medecin::Medecin,
            AttackKind::Interchange => &interchange::Interchange,
            AttackKind::Ours => &ours::Ours,
            AttackKind::Lievre => &lievre::Lievre,
            AttackKind::MeilleursAmis => &meilleurs_amis::MeilleursAmis,
            AttackKind::Familia => &familia::Familia,
        }
    }
}

impl Attack {
    /// Vrai si le moteur sait jouer cette variante.
    pub fn is_implemented(self) -> bool {
        self.kind.rules().rarities().contains(&self.rarity)
    }
}
