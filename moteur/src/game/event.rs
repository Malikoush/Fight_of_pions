use crate::{GameEnd, Pion, Player, Position};

/// Ce que le moteur annonce après une action acceptée.
///
/// Les clients mettent leur affichage à jour à partir de ces événements. Ils suffisent à
/// suivre tout l'état visible (plateau, réserves, actions, joueur courant, fin) sans
/// recalculer aucune règle ; un test le vérifie sur des centaines de parties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    /// Un pion a été posé.
    PionPlaced {
        /// Le pion posé, avec son identité et son propriétaire.
        pion: Pion,
        /// La case où il a été posé.
        at: Position,
    },
    /// Les compteurs d'actions d'un joueur ont changé (pose, fin de tour…). N'est jamais
    /// émis si rien n'a changé. Quand la partie se termine, les compteurs restent tels
    /// qu'ils étaient.
    ActionsChanged {
        /// Le joueur concerné.
        player: Player,
        /// Actions utilisables dans le tour en cours ; 0 si ce n'est pas son tour.
        actions_left: u32,
        /// Actions gardées pour son prochain tour.
        banked: u32,
    },
    /// Le tour est fini.
    TurnEnded {
        /// Le joueur qui joue maintenant.
        next: Player,
    },
    /// La partie est finie : victoire ou match nul, avec sa raison.
    GameEnded(GameEnd),
}
