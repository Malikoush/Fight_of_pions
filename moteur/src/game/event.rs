use crate::{Attack, GameEnd, Pion, PionId, Player, Position};

/// Ce que le moteur annonce après une action acceptée.
///
/// Les clients mettent leur affichage à jour à partir de ces événements. Ils suffisent à
/// suivre tout l'état visible (plateau, réserves, actions, poses restantes, recharges,
/// joueur courant, fin) sans recalculer aucune règle ; un test le vérifie sur des centaines de parties. Les
/// choix intermédiaires d'une attaque (lanceur, cibles) n'en produisent aucun : l'attaque
/// est annoncée quand elle est résolue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    /// Un pion a été posé.
    PionPlaced {
        /// Le pion posé, avec son identité et son propriétaire.
        pion: Pion,
        /// La case où il a été posé.
        at: Position,
    },
    /// Une attaque est lancée ; ses effets (pions tués…) suivent.
    AttackUsed {
        /// Le joueur qui attaque.
        player: Player,
        /// L'attaque lancée.
        attack: Attack,
        /// La case du pion lanceur.
        launcher: Position,
    },
    /// Un pion a été tué et retiré du plateau.
    PionKilled {
        /// Le pion tué.
        pion: Pion,
        /// La case où il était.
        at: Position,
        /// Le pion qui l'a tué, s'il y en a un.
        by: Option<PionId>,
    },
    /// La réserve de pions d'un joueur a changé (pose, pion tué…).
    ReserveChanged {
        /// Le joueur concerné.
        player: Player,
        /// Pions qu'il peut maintenant poser.
        pions: u32,
    },
    /// Les compteurs d'actions d'un joueur ont changé (pose, attaque, fin de tour…).
    /// N'est jamais émis si rien n'a changé. Quand la partie se termine, les compteurs
    /// restent tels qu'ils étaient.
    ActionsChanged {
        /// Le joueur concerné.
        player: Player,
        /// Actions utilisables dans le tour en cours ; 0 si ce n'est pas son tour.
        actions_left: u32,
        /// Actions gardées pour son prochain tour.
        banked: u32,
    },
    /// Les poses restantes d'un joueur ont changé ([`crate::Game::placements_left`]) : pose,
    /// attaque qui dépense des actions ou libère une case, tour qui commence ou finit, fin
    /// de partie (plus aucune pose). N'est jamais émis si rien n'a changé ; toujours avant
    /// [`Event::GameEnded`].
    PlacementsChanged {
        /// Le joueur concerné.
        player: Player,
        /// Pions qu'il peut encore poser ce tour ; 0 si ce n'est pas son tour.
        placements_left: u32,
    },
    /// La recharge d'une attaque d'un arsenal a changé (attaque lancée, tour qui commence).
    CooldownChanged {
        /// Le joueur qui possède l'attaque.
        player: Player,
        /// L'emplacement de l'attaque dans son arsenal.
        slot: usize,
        /// Tours avant de pouvoir la relancer ; 0 si elle est prête.
        turns_left: u32,
    },
    /// Le tour est fini.
    TurnEnded {
        /// Le joueur qui joue maintenant.
        next: Player,
    },
    /// La partie est finie : victoire ou match nul, avec sa raison.
    GameEnded(GameEnd),
}
