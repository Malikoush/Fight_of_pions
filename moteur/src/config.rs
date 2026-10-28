use crate::error::GameError;
use crate::{Attack, AttackKind, AttackStats, CatalogEntry, Position, Rarity};
use std::fmt;

/// Tous les réglages d'une partie, vérifiés par [`GameConfig::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct GameConfig {
    /// Nombre de cases par côté du plateau, entre [`GameConfig::MIN_BOARD_SIZE`]
    /// et [`GameConfig::MAX_BOARD_SIZE`].
    pub board_size: usize,
    /// Longueur de la ligne qui fait gagner, entre [`GameConfig::MIN_ALIGN_TO_WIN`]
    /// et `board_size`.
    pub align_to_win: usize,
    /// Pions en réserve pour chaque joueur au début, au moins 1.
    pub pions_per_player: u32,
    /// Actions du tout premier tour de la partie, au moins 1.
    pub first_turn_actions: u32,
    /// Actions des tours suivants, au moins 1.
    pub actions_per_turn: u32,
    /// Actions non utilisées qu'on peut garder pour le tour suivant.
    pub max_banked_actions: u32,
    /// Actions que chaque joueur a en banque au début de la partie (indice :
    /// `Player::index`), au plus `max_banked_actions`. La Pièce, qui compense le premier
    /// joueur : `[0, 1]`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub starting_banked_actions: [u32; 2],
    /// Actions que coûte la pose d'un pion, atteignable en économisant : au plus
    /// `actions_per_turn + max_banked_actions`. Au moins 1, sauf si les poses sont limitées
    /// par tour : 0 rend alors la pose gratuite.
    pub place_cost: u32,
    /// Le plus de pions qu'un joueur peut poser dans un tour, au moins 1 ; les poses non
    /// faites ne s'accumulent pas. `None` : seules les actions limitent les poses.
    #[cfg_attr(feature = "serde", serde(default))]
    pub max_placements_per_turn: Option<u32>,
    /// Le mal du débarquement : un pion posé ce tour ne peut pas lancer d'attaque.
    #[cfg_attr(feature = "serde", serde(default))]
    pub summoning_sickness: bool,
    /// La partie s'arrête (match nul) à la fin de ce tour, au moins 1. Filet de sécurité
    /// pour les bots : une vraie partie ne doit pas l'atteindre.
    pub max_turns: u32,
    /// Le catalogue des attaques : le coût et la recharge de chaque variante jouable.
    pub attack_catalog: Vec<CatalogEntry>,
    /// Les attaques de chaque joueur au début de la partie (indice : `Player::index`).
    /// En attendant le draft (B10), c'est ici qu'on les choisit ; plus tard, ce sera la
    /// cache secrète de chaque joueur.
    pub starting_attacks: [Vec<Attack>; 2],
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board_size: 10,
            align_to_win: 6,
            pions_per_player: 15,
            first_turn_actions: 1,
            actions_per_turn: 2,
            max_banked_actions: 2,
            starting_banked_actions: [0, 0],
            place_cost: 1,
            max_placements_per_turn: None,
            summoning_sickness: false,
            max_turns: 200,
            attack_catalog: default_attack_catalog(),
            starting_attacks: [Vec::new(), Vec::new()],
        }
    }
}

/// Les variantes que le moteur sait jouer, avec les coûts indicatifs du GDD v0.2
/// (Rare 2 actions, Épique 3) et une recharge d'un tour (« une fois par tour »).
fn default_attack_catalog() -> Vec<CatalogEntry> {
    let entry = |kind, rarity, cost| CatalogEntry {
        attack: Attack { kind, rarity },
        stats: AttackStats { cost, cooldown: 1 },
    };
    vec![
        entry(AttackKind::Ours, Rarity::Rare, 2),
        entry(AttackKind::Ours, Rarity::Epic, 3),
    ]
}

/// Ce qui rend une configuration incohérente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigError {
    /// La taille du plateau sort des bornes permises.
    BoardSizeOutOfRange,
    /// La ligne gagnante est trop courte.
    AlignmentTooShort,
    /// La ligne gagnante est plus longue que le plateau.
    AlignmentLongerThanBoard,
    /// Un joueur commencerait sans pion.
    NoPions,
    /// Un tour ne donnerait aucune action.
    NoActionsPerTurn,
    /// Poser un pion ne coûterait rien sans limite par tour : toute la réserve partirait
    /// en un tour.
    FreePlacement,
    /// La limite de poses par tour vaudrait 0 : personne ne pourrait jamais poser.
    NoPlacementPerTurn,
    /// Poser un pion coûterait plus que le maximum d'actions d'un tour, même en économisant.
    PlacementUnreachable,
    /// Un joueur commencerait avec plus d'actions en banque qu'on ne peut en garder.
    StartingBankTooLarge,
    /// La limite de tours vaudrait 0 : la partie finirait avant de commencer.
    NoTurnLimit,
    /// Cette attaque du catalogue n'est pas (encore) jouable par le moteur.
    AttackNotImplemented(Attack),
    /// Cette attaque apparaît deux fois dans le catalogue.
    AttackListedTwice(Attack),
    /// Cette attaque ne coûterait rien.
    FreeAttack(Attack),
    /// Cette attaque coûterait plus que le maximum d'actions d'un tour, même en économisant.
    AttackUnreachable(Attack),
    /// Cette attaque n'aurait pas de recharge : elle pourrait servir plusieurs fois par tour.
    NoCooldown(Attack),
    /// Un joueur commencerait avec une attaque absente du catalogue.
    AttackNotInCatalog(Attack),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::BoardSizeOutOfRange => write!(
                f,
                "la taille du plateau doit être comprise entre {} et {}",
                GameConfig::MIN_BOARD_SIZE,
                GameConfig::MAX_BOARD_SIZE
            ),
            ConfigError::AlignmentTooShort => write!(
                f,
                "il faut aligner au moins {} pions pour gagner",
                GameConfig::MIN_ALIGN_TO_WIN
            ),
            ConfigError::AlignmentLongerThanBoard => {
                write!(f, "l'alignement est plus long que le plateau")
            }
            ConfigError::NoPions => write!(f, "chaque joueur doit avoir au moins un pion"),
            ConfigError::NoActionsPerTurn => {
                write!(f, "chaque tour doit donner au moins une action")
            }
            ConfigError::FreePlacement => write!(
                f,
                "poser un pion doit coûter au moins une action, ou être limité par tour"
            ),
            ConfigError::NoPlacementPerTurn => {
                write!(f, "la limite de poses par tour doit valoir au moins 1")
            }
            ConfigError::PlacementUnreachable => write!(
                f,
                "poser un pion coûte plus que le maximum d'actions d'un tour, même en économisant"
            ),
            ConfigError::StartingBankTooLarge => write!(
                f,
                "la banque de départ dépasse le maximum d'actions gardées"
            ),
            ConfigError::NoTurnLimit => write!(f, "la limite de tours doit valoir au moins 1"),
            ConfigError::AttackNotImplemented(attack) => {
                write!(f, "l'attaque {} n'est pas encore jouable", attack)
            }
            ConfigError::AttackListedTwice(attack) => {
                write!(f, "l'attaque {} est deux fois dans le catalogue", attack)
            }
            ConfigError::FreeAttack(attack) => {
                write!(f, "l'attaque {} doit coûter au moins une action", attack)
            }
            ConfigError::AttackUnreachable(attack) => write!(
                f,
                "l'attaque {} coûte plus que le maximum d'actions d'un tour, même en économisant",
                attack
            ),
            ConfigError::NoCooldown(attack) => {
                write!(
                    f,
                    "l'attaque {} doit avoir une recharge d'au moins 1 tour",
                    attack
                )
            }
            ConfigError::AttackNotInCatalog(attack) => {
                write!(f, "l'attaque {} n'est pas dans le catalogue", attack)
            }
        }
    }
}

impl GameConfig {
    /// Le plus petit plateau accepté.
    pub const MIN_BOARD_SIZE: usize = 2;
    /// Le plus grand plateau accepté : une colonne par lettre, de A à Z.
    pub const MAX_BOARD_SIZE: usize = Position::NAMED_COLUMNS;
    /// La plus courte ligne gagnante acceptée.
    pub const MIN_ALIGN_TO_WIN: usize = 2;

    /// Toutes les attaques du catalogue, dans son ordre.
    pub fn catalog_attacks(&self) -> Vec<Attack> {
        self.attack_catalog
            .iter()
            .map(|entry| entry.attack)
            .collect()
    }

    /// Cette configuration, où chaque joueur commence avec toutes les attaques du
    /// catalogue : pratique pour essayer les attaques en attendant le draft.
    pub fn with_full_arsenals(self) -> GameConfig {
        let all = self.catalog_attacks();
        GameConfig {
            starting_attacks: [all.clone(), all],
            ..self
        }
    }

    /// Le coût et la recharge d'une attaque, si elle est dans le catalogue.
    pub fn attack_stats(&self, attack: Attack) -> Option<AttackStats> {
        self.attack_catalog
            .iter()
            .find(|entry| entry.attack == attack)
            .map(|entry| entry.stats)
    }

    /// Vérifie que les réglages sont cohérents entre eux.
    pub fn validate(&self) -> Result<(), GameError> {
        self.find_problem()
            .map_or(Ok(()), |problem| Err(GameError::InvalidConfig(problem)))
    }

    fn find_problem(&self) -> Option<ConfigError> {
        if !(Self::MIN_BOARD_SIZE..=Self::MAX_BOARD_SIZE).contains(&self.board_size) {
            return Some(ConfigError::BoardSizeOutOfRange);
        }
        if self.align_to_win < Self::MIN_ALIGN_TO_WIN {
            return Some(ConfigError::AlignmentTooShort);
        }
        if self.align_to_win > self.board_size {
            return Some(ConfigError::AlignmentLongerThanBoard);
        }
        if self.pions_per_player == 0 {
            return Some(ConfigError::NoPions);
        }
        if self.first_turn_actions == 0 || self.actions_per_turn == 0 {
            return Some(ConfigError::NoActionsPerTurn);
        }
        if self.place_cost == 0 && self.max_placements_per_turn.is_none() {
            return Some(ConfigError::FreePlacement);
        }
        if self.max_placements_per_turn == Some(0) {
            return Some(ConfigError::NoPlacementPerTurn);
        }
        if self.place_cost > self.max_actions_in_a_turn() {
            return Some(ConfigError::PlacementUnreachable);
        }
        if self
            .starting_banked_actions
            .iter()
            .any(|&bank| bank > self.max_banked_actions)
        {
            return Some(ConfigError::StartingBankTooLarge);
        }
        if self.max_turns == 0 {
            return Some(ConfigError::NoTurnLimit);
        }
        self.find_attack_problem()
    }

    fn find_attack_problem(&self) -> Option<ConfigError> {
        for (index, entry) in self.attack_catalog.iter().enumerate() {
            let attack = entry.attack;
            if !attack.is_implemented() {
                return Some(ConfigError::AttackNotImplemented(attack));
            }
            if self.attack_catalog[..index]
                .iter()
                .any(|earlier| earlier.attack == attack)
            {
                return Some(ConfigError::AttackListedTwice(attack));
            }
            if entry.stats.cost == 0 {
                return Some(ConfigError::FreeAttack(attack));
            }
            if entry.stats.cost > self.max_actions_in_a_turn() {
                return Some(ConfigError::AttackUnreachable(attack));
            }
            if entry.stats.cooldown == 0 {
                return Some(ConfigError::NoCooldown(attack));
            }
        }
        self.starting_attacks
            .iter()
            .flatten()
            .find(|attack| self.attack_stats(**attack).is_none())
            .map(|attack| ConfigError::AttackNotInCatalog(*attack))
    }

    /// Le plus d'actions qu'un joueur peut avoir en un tour ordinaire, en ayant économisé.
    fn max_actions_in_a_turn(&self) -> u32 {
        self.actions_per_turn
            .saturating_add(self.max_banked_actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem(config: GameConfig) -> Option<ConfigError> {
        match config.validate() {
            Ok(()) => None,
            Err(GameError::InvalidConfig(problem)) => Some(problem),
            Err(other) => panic!("erreur inattendue : {other:?}"),
        }
    }

    #[test]
    fn default_config_is_valid() {
        assert_eq!(problem(GameConfig::default()), None);
    }

    #[test]
    fn board_size_must_be_between_2_and_26() {
        let with_size = |board_size| GameConfig {
            board_size,
            align_to_win: 2,
            ..GameConfig::default()
        };
        for size in [0, 1, 27, usize::MAX] {
            assert_eq!(
                problem(with_size(size)),
                Some(ConfigError::BoardSizeOutOfRange)
            );
        }
        assert_eq!(problem(with_size(2)), None);
        assert_eq!(problem(with_size(26)), None);
    }

    #[test]
    fn alignment_must_be_between_2_and_board_size() {
        let with_align = |align_to_win| GameConfig {
            align_to_win,
            ..GameConfig::default()
        };
        assert_eq!(problem(with_align(0)), Some(ConfigError::AlignmentTooShort));
        assert_eq!(problem(with_align(1)), Some(ConfigError::AlignmentTooShort));
        assert_eq!(
            problem(with_align(11)),
            Some(ConfigError::AlignmentLongerThanBoard)
        );
        assert_eq!(problem(with_align(2)), None);
        assert_eq!(problem(with_align(10)), None);
    }

    #[test]
    fn each_player_needs_at_least_one_pion() {
        let config = GameConfig {
            pions_per_player: 0,
            ..GameConfig::default()
        };
        assert_eq!(problem(config), Some(ConfigError::NoPions));
    }

    #[test]
    fn each_turn_needs_at_least_one_action() {
        let first = GameConfig {
            first_turn_actions: 0,
            ..GameConfig::default()
        };
        let later = GameConfig {
            actions_per_turn: 0,
            ..GameConfig::default()
        };
        assert_eq!(problem(first), Some(ConfigError::NoActionsPerTurn));
        assert_eq!(problem(later), Some(ConfigError::NoActionsPerTurn));
    }

    #[test]
    fn placing_is_free_only_when_limited_per_turn() {
        let free = GameConfig {
            place_cost: 0,
            ..GameConfig::default()
        };
        assert_eq!(problem(free.clone()), Some(ConfigError::FreePlacement));
        let limited = |max| GameConfig {
            max_placements_per_turn: Some(max),
            ..free.clone()
        };
        assert_eq!(problem(limited(1)), None);
        assert_eq!(problem(limited(0)), Some(ConfigError::NoPlacementPerTurn));
    }

    #[test]
    fn the_starting_bank_fits_in_the_bank() {
        let with_bank = |starting_banked_actions| GameConfig {
            starting_banked_actions,
            ..GameConfig::default()
        };
        // On garde au plus 2 actions par défaut.
        assert_eq!(problem(with_bank([0, 2])), None);
        assert_eq!(
            problem(with_bank([3, 0])),
            Some(ConfigError::StartingBankTooLarge)
        );
    }

    #[test]
    fn placing_must_be_reachable_by_saving_actions() {
        let with_cost = |place_cost| GameConfig {
            place_cost,
            ..GameConfig::default()
        };
        // 2 actions par tour + 2 gardées = 4 au plus.
        assert_eq!(problem(with_cost(4)), None);
        assert_eq!(
            problem(with_cost(5)),
            Some(ConfigError::PlacementUnreachable)
        );
    }

    #[test]
    fn impossible_victory_is_accepted_for_tests() {
        let config = GameConfig {
            pions_per_player: 2,
            ..GameConfig::default()
        };
        assert_eq!(problem(config), None);
    }

    #[test]
    fn turn_limit_must_be_at_least_one() {
        let config = GameConfig {
            max_turns: 0,
            ..GameConfig::default()
        };
        assert_eq!(problem(config), Some(ConfigError::NoTurnLimit));
    }

    fn ours(rarity: Rarity) -> Attack {
        Attack {
            kind: AttackKind::Ours,
            rarity,
        }
    }

    fn with_catalog(attack: Attack, cost: u32, cooldown: u32) -> GameConfig {
        GameConfig {
            attack_catalog: vec![CatalogEntry {
                attack,
                stats: AttackStats { cost, cooldown },
            }],
            ..GameConfig::default()
        }
    }

    #[test]
    fn default_catalog_holds_the_playable_ours() {
        let config = GameConfig::default();
        assert_eq!(
            config.attack_stats(ours(Rarity::Rare)),
            Some(AttackStats {
                cost: 2,
                cooldown: 1
            })
        );
        assert!(config.attack_stats(ours(Rarity::Common)).is_none());
    }

    #[test]
    fn catalog_only_holds_attacks_the_engine_can_play() {
        let attack = ours(Rarity::Common);
        assert_eq!(
            problem(with_catalog(attack, 1, 1)),
            Some(ConfigError::AttackNotImplemented(attack))
        );
    }

    #[test]
    fn an_attack_cannot_be_listed_twice() {
        let mut config = GameConfig::default();
        config.attack_catalog.push(config.attack_catalog[0]);
        assert_eq!(
            problem(config),
            Some(ConfigError::AttackListedTwice(ours(Rarity::Rare)))
        );
    }

    #[test]
    fn an_attack_costs_at_least_one_reachable_action() {
        let attack = ours(Rarity::Rare);
        assert_eq!(
            problem(with_catalog(attack, 0, 1)),
            Some(ConfigError::FreeAttack(attack))
        );
        // 2 actions par tour + 2 gardées = 4 au plus.
        assert_eq!(problem(with_catalog(attack, 4, 1)), None);
        assert_eq!(
            problem(with_catalog(attack, 5, 1)),
            Some(ConfigError::AttackUnreachable(attack))
        );
    }

    #[test]
    fn an_attack_recharges_for_at_least_one_turn() {
        let attack = ours(Rarity::Rare);
        assert_eq!(
            problem(with_catalog(attack, 2, 0)),
            Some(ConfigError::NoCooldown(attack))
        );
    }

    #[test]
    fn starting_attacks_must_be_in_the_catalog() {
        let config = GameConfig {
            attack_catalog: Vec::new(),
            starting_attacks: [vec![ours(Rarity::Rare)], Vec::new()],
            ..GameConfig::default()
        };
        assert_eq!(
            problem(config),
            Some(ConfigError::AttackNotInCatalog(ours(Rarity::Rare)))
        );
    }

    #[test]
    fn error_message_uses_the_real_bounds() {
        assert_eq!(
            ConfigError::BoardSizeOutOfRange.to_string(),
            "la taille du plateau doit être comprise entre 2 et 26"
        );
    }
}
