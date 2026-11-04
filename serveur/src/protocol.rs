//! Le protocole : les messages JSON échangés sur la WebSocket, un message par trame
//! texte. Le client n'a besoin d'aucune règle : le serveur lui envoie sa vue de la partie
//! ([`PlayerView`]), avec la décision attendue et ses coups permis ; il renvoie l'un de
//! ces coups. Voir `docs/PROTOCOLE.md` pour des exemples.

use crate::ServerError;
use crate::cards::CardFamily;
use fight_of_pions::{Action, Event, Player, PlayerView};
use serde::{Deserialize, Serialize};

/// Un message du client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    /// Créer une salle et s'y asseoir comme joueur 1.
    Create {
        /// Contre qui jouer.
        opponent: Opponent,
        /// Le nom d'une variante (`bots::variants`) ; par défaut, celle du jeu en ligne
        /// (`bots::ONLINE_VARIANT`).
        #[serde(default)]
        variant: Option<String>,
    },
    /// Rejoindre la salle de ce code, comme joueur 2.
    Join {
        /// Le code donné à celui qui a créé la salle.
        code: String,
    },
    /// Reprendre sa place après une déconnexion.
    Rejoin {
        /// Le code de la salle.
        code: String,
        /// Sa place.
        player: Player,
        /// La clé reçue en s'asseyant ([`ServerMessage::Seated`]).
        key: String,
    },
    /// Redemander sa vue actuelle ([`ServerMessage::State`], sans événement) : pour se
    /// remettre à jour si une vue a été perdue.
    Sync,
    /// Jouer ce coup : un de ceux de `legal_actions` dans sa dernière vue.
    Play {
        /// Le coup.
        action: Action,
    },
    /// Essayer ces coups à la suite sur une copie de la partie, sans rien jouer : la
    /// réponse ([`ServerMessage::Preview`]) donne les choix suivants. C'est ainsi qu'un
    /// client sans règle choisit une carte en plusieurs étapes et peut l'annuler avant de
    /// l'envoyer, coup par coup, avec `play`. L'aperçu s'arrête au coup qui finit les
    /// choix de la carte : la suite dépendrait de ce que l'adversaire cache.
    Preview {
        /// Les coups, depuis la partie actuelle.
        actions: Vec<Action>,
    },
    /// Demander le catalogue des cartes ([`ServerMessage::Cards`]) : leurs noms et leurs
    /// textes, à afficher. Possible à tout moment, assis ou non.
    Cards,
}

/// L'adversaire choisi en créant une salle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Opponent {
    /// Un autre humain, qui rejoint avec le code.
    Human,
    /// Le bot MCTS, sur le serveur.
    Mcts,
    /// Le bot qui joue au hasard (essais, tests).
    Random,
}

/// Un message du serveur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Le client est assis dans une salle. Il garde `key` pour reprendre sa place.
    Seated {
        /// Le code de la salle, à donner à l'adversaire.
        code: String,
        /// Sa place.
        player: Player,
        /// Sa clé de place, secrète.
        key: String,
    },
    /// La partie a changé : sa vue entière, et ce qui s'est passé depuis la précédente
    /// (pour animer et tenir un journal), filtré pour lui.
    State {
        /// Ce qu'il voit maintenant.
        view: Box<PlayerView>,
        /// Les événements depuis sa vue précédente, dans l'ordre.
        events: Vec<Event>,
    },
    /// La réponse à un aperçu, rien n'étant joué.
    Preview {
        /// Ce qu'il verrait après ces coups, pour faire le choix suivant de la carte ;
        /// `None` (`null`) si le dernier coup finit ses choix : ces coups sont prêts à être
        /// joués avec `play`.
        view: Option<Box<PlayerView>>,
    },
    /// Le catalogue des cartes, en réponse à [`ClientMessage::Cards`] : toutes les familles,
    /// dans l'ordre des fiches attaques. Le coût et la recharge de chaque carte sont dans la
    /// configuration de la partie (`config.attack_catalog` de la vue).
    Cards {
        /// Les familles, avec leurs raretés jouables.
        cards: Vec<CardFamily>,
    },
    /// Une demande refusée ; rien n'a changé. Arrive aussi sans demande si le bot du
    /// serveur ne peut plus jouer.
    Error {
        /// La sorte d'erreur, pour le client ([`ServerError::kind`]).
        kind: String,
        /// Pourquoi, en français, à afficher.
        message: String,
    },
}

impl ServerMessage {
    /// Le message qui annonce cette erreur.
    pub fn error(error: &ServerError) -> ServerMessage {
        ServerMessage::Error {
            kind: error.kind().to_string(),
            message: error.to_string(),
        }
    }
}
