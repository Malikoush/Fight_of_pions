use std::fmt;

/// Une famille d'attaques (Ours, Lièvre, Beauté fatale…). Ses règles vivent dans un
/// fichier à son nom, dans `attacks/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttackKind {
    /// Attaque de corps à corps sur les 8 cases autour du lanceur (GDD n°09).
    Ours,
}

impl fmt::Display for AttackKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AttackKind::Ours => write!(f, "Ours"),
        }
    }
}

/// La variante d'une attaque, et sa place dans le draft. Elle n'impose pas le coût :
/// celui-ci vient du catalogue de la configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Rarity {
    /// Commune.
    Common,
    /// Rare.
    Rare,
    /// Épique.
    Epic,
    /// Légendaire.
    Legendary,
}

impl fmt::Display for Rarity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rarity::Common => write!(f, "Commune"),
            Rarity::Rare => write!(f, "Rare"),
            Rarity::Epic => write!(f, "Épique"),
            Rarity::Legendary => write!(f, "Légendaire"),
        }
    }
}

/// Une attaque précise : une famille dans une rareté (« Ours Rare »). C'est une carte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Attack {
    /// La famille.
    pub kind: AttackKind,
    /// La variante.
    pub rarity: Rarity,
}

/// « Ours (Rare) ».
impl fmt::Display for Attack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.kind, self.rarity)
    }
}

/// Les caractéristiques chiffrées d'une attaque, réglables dans la configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct AttackStats {
    /// Actions dépensées pour la lancer, au moins 1.
    pub cost: u32,
    /// Tours du lanceur avant de pouvoir la relancer, au moins 1 (1 = une fois par tour).
    pub cooldown: u32,
}

/// Une ligne du catalogue des attaques : une attaque et ses caractéristiques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct CatalogEntry {
    /// L'attaque.
    pub attack: Attack,
    /// Son coût et sa recharge.
    pub stats: AttackStats,
}
