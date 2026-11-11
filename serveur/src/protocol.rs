//! Le protocole : les messages JSON échangés sur la WebSocket, un message par trame
//! texte. Le client n'a besoin d'aucune règle : le serveur lui envoie sa vue de la partie
//! ([`PlayerView`]), avec la décision attendue et ses coups permis ; il renvoie l'un de
//! ces coups. Voir `docs/PROTOCOLE.md` pour des exemples.

use crate::ServerError;
use crate::cards::CardFamily;
use fight_of_pions::{Action, Attack, Event, Player, PlayerView};
use serde::{Deserialize, Serialize};

/// Un message du client. Créer ou rejoindre une salle demande d'être connecté à son compte
/// (`register` la première fois, puis `login`) : la cache se choisit parmi ses cartes. Un
/// compte n'est assis que dans une salle à la fois ; s'y connecter le rassoit dans sa salle
/// en cours, s'il en a une ([`ServerMessage::Account`], `resumed`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    /// Créer une salle et s'y asseoir, à une place tirée au sort (J1 commence le draft et
    /// la partie). Contre un bot, la partie est lancée tout de suite ; contre un humain,
    /// elle attend que le créateur la lance ([`ClientMessage::Launch`]).
    Create {
        /// Contre qui jouer.
        opponent: Opponent,
        /// Le nom de la variante ; seule celle du jeu en ligne (`bots::ONLINE_VARIANT`) est
        /// acceptée, et c'est celle par défaut.
        #[serde(default)]
        variant: Option<String>,
    },
    /// Rejoindre la salle de ce code, à la place laissée libre par son créateur.
    Join {
        /// Le code donné à celui qui a créé la salle.
        code: String,
    },
    /// Reprendre sa place après une déconnexion, avec sa clé, depuis le compte assis à
    /// cette place. Rarement utile : se connecter à son compte rassoit déjà dans sa salle.
    Rejoin {
        /// Le code de la salle.
        code: String,
        /// Sa place.
        player: Player,
        /// La clé reçue en s'asseyant ([`ServerMessage::Seated`]).
        key: String,
    },
    /// Quitter sa salle. Une partie lancée et pas finie est perdue par forfait ; avant le
    /// lancement, la place de l'invité redevient libre, et la salle de son créateur est
    /// fermée (l'invité reçoit l'erreur `room_closed`). Réponse : [`ServerMessage::Left`],
    /// même sans salle. On peut ensuite créer ou rejoindre une autre salle.
    Leave,
    /// Lancer la partie de la salle qu'on a créée, une fois l'adversaire assis : les deux
    /// joueurs reçoivent [`ServerMessage::Launched`]. Avant, personne ne peut jouer.
    Launch,
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
    /// Créer un compte invité et s'y connecter. La réponse ([`ServerMessage::Account`])
    /// donne son jeton, une seule fois : le client le garde pour revenir avec `login`.
    Register {
        /// Le pseudo ; sans pseudo, « Pion » et quatre chiffres.
        #[serde(default)]
        pseudo: Option<String>,
    },
    /// Se connecter à son compte. Réponses : [`ServerMessage::Account`] (sans jeton), puis
    /// [`ServerMessage::Collection`].
    Login {
        /// Le jeton reçu à la création du compte.
        token: String,
    },
    /// Changer son pseudo. Réponse : [`ServerMessage::Account`].
    SetPseudo {
        /// Le nouveau pseudo.
        pseudo: String,
    },
    /// Redemander sa collection ([`ServerMessage::Collection`]).
    Collection,
    /// Ranger son trio de cache : jusqu'à 3 cartes différentes de sa collection ; vide, la
    /// serveur tire 3 cartes de sa collection au hasard à chaque partie. Réponse : [`ServerMessage::Collection`].
    /// Le trio compte pour les salles créées ou rejointes ensuite.
    SetCacheTrio {
        /// Les cartes, dans l'ordre voulu.
        cards: Vec<Attack>,
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
    /// Le client est assis dans une salle. Il garde `key` pour reprendre sa place.
    Seated {
        /// Le code de la salle, à donner à l'adversaire.
        code: String,
        /// Sa place.
        player: Player,
        /// Sa clé de place, secrète.
        key: String,
        /// Vrai si la partie est lancée ; sinon, elle attend que le créateur la lance
        /// ([`ServerMessage::Launched`]).
        launched: bool,
    },
    /// Le créateur de la salle a lancé la partie : on peut jouer. Envoyé aux deux joueurs.
    Launched,
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
    /// L'adversaire humain vient de se brancher sur sa place, ou de s'en débrancher (fermé,
    /// coupé, parti). Envoyé aussi à celui qui s'assoit, juste après sa vue, dès qu'un
    /// humain occupe l'autre place, là ou non. Jamais pour un bot, toujours là.
    Opponent {
        /// Vrai s'il est branché.
        connected: bool,
        /// Son pseudo.
        pseudo: String,
    },
    /// La réponse à [`ClientMessage::Leave`] : le client n'est plus dans aucune salle.
    Left,
    /// Le compte de cette connexion, après `register`, `login` ou `set_pseudo`.
    Account {
        /// Son numéro, stable.
        id: i64,
        /// Son pseudo.
        pseudo: String,
        /// Son jeton secret, seulement en réponse à `register` : le client doit le garder,
        /// le serveur ne le redonnera jamais.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token: Option<String>,
        /// Vrai si la connexion au compte vient de le rasseoir dans sa salle en cours :
        /// [`ServerMessage::Seated`] suit. Faux sinon, et après `register` ou `set_pseudo`.
        #[serde(default)]
        resumed: bool,
    },
    /// Ce que possède le compte : après `register`, `login`, `collection` ou
    /// `set_cache_trio`.
    Collection {
        /// Ses cartes, dans l'ordre des familles puis des raretés.
        cards: Vec<Attack>,
        /// Son trio de cache, de 0 à 3 cartes.
        cache_trio: Vec<Attack>,
    },
    /// La partie vient de finir et le gagnant a gagné cette carte : elle est dans sa
    /// collection. Seulement au gagnant d'un autre compte ou du MCTS (pas du bot aléatoire),
    /// et s'il lui manque encore une carte du catalogue de la partie.
    Reward {
        /// La carte gagnée.
        card: Attack,
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
