use crate::{EffectEnd, EffectKind, PionId, Position, TokenReward};

/// Une opération de base produite par une attaque. Les familles décrivent leurs effets avec
/// ces primitives ; seul le résolveur de la partie les applique, émet les événements et
/// déclenche les conséquences (infirmerie, fin des effets d'un pion tué, vengeance…). De
/// nouvelles primitives s'ajouteront avec les familles qui en ont besoin (catalogue dans
/// l'architecture).
///
/// Les effets s'appliquent dans l'ordre : chaque case désigne le pion qui s'y trouve au
/// moment où l'effet s'applique (après les déplacements et les kills qui le précèdent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Effect {
    /// Tuer le pion de cette case. Les familles ne le produisent jamais sur un pion protégé
    /// (`AttackContext::can_kill`). Un pion protégé atteint sans être visé (survol, charge)
    /// devra survivre : le résolveur le vérifiera avec les familles qui le permettent (B4).
    Kill {
        /// La case du pion tué.
        at: Position,
    },
    /// Donner un effet continu au pion de cette case, pour `turns` tours (au moins 1).
    Apply {
        /// La case du pion visé.
        at: Position,
        /// L'effet donné (immobilisé, protégé).
        kind: EffectKind,
        /// Sa durée, en tours de l'adversaire du joueur qui attaque.
        turns: u32,
    },
    /// Donner au pion de cette case un effet continu qui finit autrement que par une durée :
    /// un retour programmé (Interchange Commune), des jetons de pas (Lièvre Épique).
    Attach {
        /// La case du pion visé.
        at: Position,
        /// L'effet donné.
        kind: EffectKind,
        /// Comment il finit.
        end: EffectEnd,
    },
    /// Retirer du pion de cette case tous les statuts (immobilisé, protégé) posés par
    /// l'adversaire du joueur qui attaque.
    RemoveEnemyStatuses {
        /// La case du pion visé.
        at: Position,
    },
    /// Annuler en entier les effets de l'adversaire du joueur qui attaque qui visent le pion
    /// de cette case : chaque jeu de carte adverse qui l'a touché perd tous ses effets
    /// annulables, sur tous les pions (Attila Rare et Épique). La couronne reste.
    CancelEnemyEffects {
        /// La case du pion visé.
        at: Position,
    },
    /// Terre brûlée : jusqu'à la fin de son prochain tour, l'adversaire du joueur qui
    /// attaque ne peut jouer aucun coup qui créerait un effet continu. Ce n'en est pas un.
    ScorchEarth,
    /// Annuler toutes les piles de jetons de l'adversaire du joueur qui attaque (Attila Rare
    /// et Épique) : chacune est un effet entier.
    CancelEnemyTokens,
    /// Poser devant le joueur qui attaque une pile de jetons (Massacre divin) : chacun de
    /// ses prochains kills en use un.
    AddTokens {
        /// Ce que rapporte chaque jeton.
        reward: TokenReward,
        /// Le nombre de jetons, au moins 1.
        tokens: u32,
    },
    /// Déplacer le pion de `from` vers la case libre `to`. C'est la famille qui calcule le
    /// trajet (pas, saut…) ; seule l'arrivée compte pour la partie.
    Move {
        /// La case du pion, au moment où l'effet s'applique.
        from: Position,
        /// La case d'arrivée.
        to: Position,
    },
    /// Poser un pion de la réserve du joueur qui attaque sur cette case libre. Comme une
    /// pose : il reçoit une nouvelle identité et compte comme posé ce tour, mais n'utilise
    /// pas la pose normale du tour et ne coûte rien de plus.
    Place {
        /// La case où il arrive.
        at: Position,
    },
    /// Ressusciter ce pion du joueur qui attaque : il sort de l'infirmerie (sa tombe
    /// s'efface) et revient sur la case libre `at`, comme un pion posé ce tour. Il est
    /// désigné par son identité : si une autre tombe a remplacé la sienne (l'occupant tué
    /// par le Médecin Épique), il revient quand même.
    Resurrect {
        /// Le pion à ressusciter.
        pion: PionId,
        /// La case où il revient.
        at: Position,
    },
    /// Couronner le pion de cette case (il devient une reine), avec ce nombre de gardes.
    Crown {
        /// La case de la nouvelle reine.
        at: Position,
        /// Ses gardes.
        guards: u32,
    },
    /// Faire du pion de `at` un enfant (Familia) : il est protégé tant que les pions de
    /// `framed_by` l'encadrent de part et d'autre, en ligne droite.
    MakeChild {
        /// La case de l'enfant.
        at: Position,
        /// Les cases des deux membres de sa famille qui l'encadrent.
        framed_by: [Position; 2],
    },
    /// Lier par un pacte (Meilleurs amis) les pions de ces deux cases.
    Bind {
        /// La case du premier ami.
        a: Position,
        /// La case du second ami.
        b: Position,
    },
    /// Échanger les pions de deux cases, d'un coup.
    Swap {
        /// La case du premier pion.
        a: Position,
        /// La case du second pion.
        b: Position,
    },
}

impl Effect {
    /// Vrai si cet effet crée un effet continu ou une pile de jetons : ce que Terre brûlée
    /// interdit à l'adversaire. Sans joker : chaque nouvelle primitive doit trancher.
    pub(crate) fn creates_lasting(self) -> bool {
        match self {
            Effect::Apply { .. }
            | Effect::Attach { .. }
            | Effect::Bind { .. }
            | Effect::Crown { .. }
            | Effect::MakeChild { .. }
            | Effect::AddTokens { .. } => true,
            Effect::Kill { .. }
            | Effect::RemoveEnemyStatuses { .. }
            | Effect::CancelEnemyEffects { .. }
            | Effect::CancelEnemyTokens
            | Effect::ScorchEarth
            | Effect::Move { .. }
            | Effect::Place { .. }
            | Effect::Resurrect { .. }
            | Effect::Swap { .. } => false,
        }
    }

    /// La case du seul pion que vise cet effet, s'il n'en vise qu'un : si une attaque y a
    /// été annulée par un garde, elle ne lui fait plus rien d'autre. `None` pour les effets
    /// qui touchent plusieurs cases (déplacer, échanger, lier) ou aucune (les jetons).
    pub(crate) fn single_target(self) -> Option<Position> {
        match self {
            Effect::Kill { at }
            | Effect::Apply { at, .. }
            | Effect::Attach { at, .. }
            | Effect::RemoveEnemyStatuses { at }
            | Effect::CancelEnemyEffects { at }
            | Effect::Place { at }
            | Effect::Resurrect { at, .. }
            | Effect::Crown { at, .. }
            | Effect::MakeChild { at, .. } => Some(at),
            Effect::Move { .. }
            | Effect::Swap { .. }
            | Effect::Bind { .. }
            | Effect::CancelEnemyTokens
            | Effect::ScorchEarth
            | Effect::AddTokens { .. } => None,
        }
    }
}
