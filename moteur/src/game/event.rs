use crate::{ActiveEffect, Attack, GameEnd, Patient, Pion, PionId, Player, Position};

/// Ce que le moteur annonce après une action acceptée.
///
/// Les clients mettent leur affichage à jour à partir de ces événements. Ils suffisent à
/// suivre tout l'état visible (plateau, réserves, infirmerie, effets continus, actions,
/// poses restantes, recharges, joueur courant, fin) sans recalculer aucune règle ; un test
/// le vérifie sur des centaines de parties. Les choix intermédiaires d'une attaque
/// (lanceur, cibles) n'en produisent aucun : l'attaque est annoncée quand elle est résolue.
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
    /// Un pion a été tué et retiré du plateau. Suivent la fin de ses effets continus
    /// ([`Event::EffectEnded`]), puis son départ à l'infirmerie
    /// ([`Event::InfirmaryEntered`], puis sa tombe : [`Event::TombRemoved`] pour celle
    /// qu'elle remplace, et [`Event::TombAdded`]) ou dans la réserve
    /// ([`Event::ReserveChanged`]).
    PionKilled {
        /// Le pion tué.
        pion: Pion,
        /// La case où il était.
        at: Position,
        /// Le pion qui l'a tué, s'il y en a un.
        by: Option<PionId>,
    },
    /// Un pion a changé de case (saut, pas…). Il garde son identité, ses effets et son
    /// ancienneté.
    PionMoved {
        /// Le pion déplacé.
        pion: Pion,
        /// La case qu'il quitte.
        from: Position,
        /// La case où il arrive, libre jusque-là.
        to: Position,
    },
    /// Deux pions ont échangé leurs cases, d'un coup : `first` était en `first_at` et
    /// arrive en `second_at`, et inversement.
    PionsSwapped {
        /// Le premier pion.
        first: Pion,
        /// Sa case avant l'échange.
        first_at: Position,
        /// Le second pion.
        second: Pion,
        /// Sa case avant l'échange.
        second_at: Position,
    },
    /// La réserve de pions d'un joueur a changé (pose, pion tué sans infirmerie, sortie de
    /// l'infirmerie).
    ReserveChanged {
        /// Le joueur concerné.
        player: Player,
        /// Pions qu'il peut maintenant poser.
        pions: u32,
    },
    /// Un effet continu commence (un pion immobilisé, protégé…).
    EffectStarted(ActiveEffect),
    /// Un effet continu s'arrête : sa durée est écoulée, une carte l'a retiré, ou son pion
    /// a été tué.
    EffectEnded(ActiveEffect),
    /// Un pion tué entre à l'infirmerie, jusqu'à la fin du tour [`Patient::last_turn`].
    /// Suit sa tombe ([`Event::TombRemoved`] pour celle qu'elle remplace, puis
    /// [`Event::TombAdded`]).
    InfirmaryEntered(Patient),
    /// Un pion sort de l'infirmerie. Suivent [`Event::TombRemoved`], s'il avait encore une
    /// tombe, puis [`Event::ReserveChanged`] (il revient dans la réserve de son
    /// propriétaire) ou [`Event::PionPlaced`] (ressuscité par le Médecin, avec une nouvelle
    /// identité).
    InfirmaryLeft(Patient),
    /// Un pion tué laisse sa tombe sur la case où il est mort. Une tombe ne bloque pas la
    /// case ; une case porte au plus une tombe.
    TombAdded {
        /// Le pion tué.
        pion: Pion,
        /// La case de la tombe.
        at: Position,
    },
    /// Une tombe s'efface : son pion sort de l'infirmerie, ou une nouvelle tombe la
    /// remplace.
    TombRemoved {
        /// Le pion de la tombe.
        pion: Pion,
        /// La case de la tombe.
        at: Position,
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
