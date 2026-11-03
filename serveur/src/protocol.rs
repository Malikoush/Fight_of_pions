//! Le protocole : les messages JSON échangés sur la WebSocket, un message par trame
//! texte. Le client n'a besoin d'aucune règle : le serveur lui envoie sa vue de la partie
//! ([`PlayerView`]), avec la décision attendue et ses coups permis ; il renvoie l'un de
//! ces coups. Voir `docs/PROTOCOLE.md` pour des exemples.

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
        /// Le nom d'une variante (`bots::variants`) ; par défaut, les règles actuelles
        /// avec draft.
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
        /// Le jeton reçu en s'asseyant ([`ServerMessage::Seated`]).
        token: String,
    },
    /// Jouer ce coup : un de ceux de `legal_actions` dans sa dernière vue.
    Play {
        /// Le coup.
        action: Action,
    },
    /// Essayer ces coups à la suite sur une copie de la partie, sans rien jouer : la
    /// réponse ([`ServerMessage::Preview`]) donne les choix suivants. C'est ainsi qu'un
    /// client sans règle choisit une carte en plusieurs étapes et peut l'annuler avant de
    /// l'envoyer, coup par coup, avec `play`.
    Preview {
        /// Les coups, depuis la partie actuelle.
        actions: Vec<Action>,
    },
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
    /// Le client est assis dans une salle. Il garde `token` pour reprendre sa place.
    Seated {
        /// Le code de la salle, à donner à l'adversaire.
        code: String,
        /// Sa place.
        player: Player,
        /// Son jeton, secret.
        token: String,
    },
    /// La partie a changé : sa vue entière, et ce qui s'est passé depuis la précédente
    /// (pour animer et tenir un journal), filtré pour lui.
    State {
        /// Ce qu'il voit maintenant.
        view: Box<PlayerView>,
        /// Les événements depuis sa vue précédente, dans l'ordre.
        events: Vec<Event>,
    },
    /// La vue après les coups d'un aperçu, rien n'étant joué.
    Preview {
        /// Ce qu'il verrait.
        view: Box<PlayerView>,
    },
    /// Une demande refusée ; rien n'a changé.
    Error {
        /// Pourquoi, en français, à afficher.
        message: String,
    },
}
