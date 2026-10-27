//! Ce que l'utilisateur veut faire, avant de savoir si c'est une demande à la partie ou
//! un geste d'interface : choisir une attaque se fait sur une copie, et seule la suite
//! complète est envoyée à la partie.

use crate::partie::Demande;
use bots::BotKind;
use fight_of_pions::{GameConfig, Position};

pub enum Intention {
    /// Une demande à transmettre telle quelle à la partie.
    Demande(Demande),
    /// Commencer à choisir l'attaque de cet emplacement de l'arsenal.
    CommencerAttaque(usize),
    /// Une case cliquée : pose, ou lanceur et cibles pendant une attaque.
    ChoisirCase(Position),
    /// Abandonner l'attaque en cours de choix (rien n'a encore été joué).
    AnnulerSelection,
    /// Revoir dans l'écran Partie une partie de l'arène : même configuration, même
    /// graine, mêmes bots (joueur 1, joueur 2).
    Revoir {
        config: GameConfig,
        graine: u64,
        bots: [BotKind; 2],
    },
}
