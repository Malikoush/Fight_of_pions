use std::fmt;

/// Une famille d'attaques (Ours, Lièvre, Beauté fatale…). Ses règles vivent dans un
/// fichier à son nom, dans `attacks/`. Les numéros sont ceux des fiches attaques du
/// 2026-10-29 (`docs/FICHES_ATTAQUES.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttackKind {
    /// Le charme qui fige les pions adverses, puis les séduit (fiche n°01).
    BeauteFatale,
    /// Des sauts de 2 cases qui frappent le pion survolé (fiche n°02).
    SautDuLoup,
    /// La reine et sa garde (fiche n°03).
    Reine,
    /// Le contre des effets en cours (fiche n°04).
    Attila,
    /// La résurrection des pions tombés, sur leur tombe (fiche n°05).
    Medecin,
    /// L'échange de cases avec un pion adverse (fiche n°06).
    Interchange,
    /// Le corps à corps, sur les 8 cases collées au lanceur (fiche n°09).
    Ours,
    /// Le mouvement pur, pas à pas (fiche n°10).
    Lievre,
    /// Deux pions liés par un pacte : si l'un est tué, l'autre le venge (fiche n°12).
    MeilleursAmis,
}

impl fmt::Display for AttackKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AttackKind::BeauteFatale => write!(f, "Beauté fatale"),
            AttackKind::SautDuLoup => write!(f, "Saut du loup"),
            AttackKind::Reine => write!(f, "Reine"),
            AttackKind::Attila => write!(f, "Attila"),
            AttackKind::Medecin => write!(f, "Médecin"),
            AttackKind::Interchange => write!(f, "Interchange"),
            AttackKind::Ours => write!(f, "Ours"),
            AttackKind::Lievre => write!(f, "Lièvre"),
            AttackKind::MeilleursAmis => write!(f, "Meilleurs amis"),
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
