use fight_of_pions::{Attack, GameError};
use std::fmt;

/// Pourquoi le serveur refuse une demande. Rien n'a changé ; le message est envoyé au
/// client ([`crate::ServerMessage::Error`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    /// Le message n'est pas du JSON du protocole.
    BadMessage(String),
    /// Aucune salle n'a ce code.
    UnknownRoom(String),
    /// Les deux places de la salle sont prises.
    RoomFull,
    /// Cette clé ne correspond pas à cette place.
    BadKey,
    /// Il faut d'abord créer ou rejoindre une salle.
    NotSeated,
    /// Le client est déjà assis dans une salle.
    AlreadySeated,
    /// Aucune variante n'a ce nom.
    UnknownVariant(String),
    /// Ce n'est pas à ce joueur de décider.
    NotYourTurn,
    /// Une reconnexion a repris cette place : cette connexion ne joue plus.
    SeatTaken,
    /// Un aperçu continue après le coup qui finit les choix de la carte.
    PreviewPastTheCard,
    /// Le serveur a déjà son nombre maximal de salles.
    ServerFull,
    /// La partie n'est pas encore lancée : personne ne joue.
    NotLaunched,
    /// Seul le créateur de la salle la lance.
    NotHost,
    /// La partie ne se lance qu'une fois l'adversaire assis.
    OpponentMissing,
    /// Le bot n'a pas pu jouer (un bug) : la partie attend.
    BotFailed(String),
    /// Le moteur refuse ce coup, ou cette configuration.
    Game(GameError),
    /// Le fichier des textes des cartes est mal écrit (un bug, que les tests empêchent).
    CardsUnavailable(String),
    /// Il faut d'abord se connecter à son compte.
    NotLoggedIn,
    /// Cette connexion est déjà celle d'un compte.
    AlreadyLoggedIn,
    /// Aucun compte n'a ce jeton.
    UnknownAccount,
    /// Ce pseudo n'est pas permis, pour cette raison.
    BadPseudo(String),
    /// Le compte ne possède pas cette carte.
    NotOwned(Attack),
    /// Ce trio de cache n'est pas permis, pour cette raison.
    BadCacheTrio(String),
    /// Aucune carte du compte ne peut être mise en cache dans cette variante.
    NothingToCache,
    /// La base n'a pas pu lire ou écrire (le détail est dans la sortie du serveur).
    Storage,
    /// Le fichier du paquet de départ est mal écrit (un bug, que les tests empêchent).
    StarterUnavailable(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::BadMessage(why) => write!(f, "message illisible : {why}"),
            ServerError::UnknownRoom(code) => write!(f, "aucune salle n'a le code {code}"),
            ServerError::RoomFull => write!(f, "cette salle est complète"),
            ServerError::BadKey => write!(f, "cette clé ne correspond pas à cette place"),
            ServerError::NotSeated => write!(f, "il faut d'abord créer ou rejoindre une salle"),
            ServerError::AlreadySeated => write!(f, "tu es déjà dans une salle"),
            ServerError::UnknownVariant(name) => {
                write!(f, "aucune variante ne s'appelle « {name} »")
            }
            ServerError::NotYourTurn => write!(f, "ce n'est pas à toi de jouer"),
            ServerError::SeatTaken => write!(f, "ta place a été reprise par une autre connexion"),
            ServerError::PreviewPastTheCard => {
                write!(
                    f,
                    "un aperçu s'arrête au coup qui finit les choix de la carte"
                )
            }
            ServerError::ServerFull => write!(f, "le serveur est complet, réessaie plus tard"),
            ServerError::NotLaunched => write!(f, "la partie n'est pas encore lancée"),
            ServerError::NotHost => write!(f, "seul le créateur de la salle la lance"),
            ServerError::OpponentMissing => {
                write!(f, "la partie se lance quand ton adversaire est là")
            }
            ServerError::BotFailed(why) => {
                write!(
                    f,
                    "le bot n'a pas pu jouer ({why}) : la partie est en attente"
                )
            }
            ServerError::Game(error) => write!(f, "coup refusé : {error}"),
            ServerError::CardsUnavailable(why) => {
                write!(f, "les textes des cartes sont illisibles ({why})")
            }
            ServerError::NotLoggedIn => write!(f, "il faut d'abord se connecter à son compte"),
            ServerError::AlreadyLoggedIn => write!(f, "tu es déjà connecté à un compte"),
            ServerError::UnknownAccount => write!(f, "ce compte n'existe pas"),
            ServerError::BadPseudo(why) => write!(f, "pseudo refusé : {why}"),
            ServerError::NotOwned(card) => write!(f, "tu ne possèdes pas la carte {card}"),
            ServerError::BadCacheTrio(why) => write!(f, "trio de cache refusé : {why}"),
            ServerError::NothingToCache => write!(
                f,
                "aucune de tes cartes ne peut être mise en cache dans cette variante"
            ),
            ServerError::Storage => write!(
                f,
                "le serveur n'a pas pu enregistrer ou relire tes données, réessaie plus tard"
            ),
            ServerError::StarterUnavailable(why) => {
                write!(f, "le paquet de départ est illisible ({why})")
            }
        }
    }
}

impl ServerError {
    /// La sorte d'erreur, stable et sans accent, pour qu'un client réagisse sans lire le
    /// message (revenir au menu sur `unknown_room`, se remettre à jour sur
    /// `not_your_turn`…).
    pub fn kind(&self) -> &'static str {
        match self {
            ServerError::BadMessage(_) => "bad_message",
            ServerError::UnknownRoom(_) => "unknown_room",
            ServerError::RoomFull => "room_full",
            ServerError::BadKey => "bad_key",
            ServerError::NotSeated => "not_seated",
            ServerError::AlreadySeated => "already_seated",
            ServerError::UnknownVariant(_) => "unknown_variant",
            ServerError::NotYourTurn => "not_your_turn",
            ServerError::SeatTaken => "seat_taken",
            ServerError::PreviewPastTheCard => "preview_past_the_card",
            ServerError::ServerFull => "server_full",
            ServerError::NotLaunched => "not_launched",
            ServerError::NotHost => "not_host",
            ServerError::OpponentMissing => "opponent_missing",
            ServerError::BotFailed(_) => "bot_failed",
            ServerError::Game(_) => "refused_move",
            ServerError::CardsUnavailable(_) => "cards_unavailable",
            ServerError::NotLoggedIn => "not_logged_in",
            ServerError::AlreadyLoggedIn => "already_logged_in",
            ServerError::UnknownAccount => "unknown_account",
            ServerError::BadPseudo(_) => "bad_pseudo",
            ServerError::NotOwned(_) => "not_owned",
            ServerError::BadCacheTrio(_) => "bad_cache_trio",
            ServerError::NothingToCache => "nothing_to_cache",
            ServerError::Storage => "storage",
            ServerError::StarterUnavailable(_) => "starter_unavailable",
        }
    }
}

impl From<GameError> for ServerError {
    fn from(error: GameError) -> Self {
        ServerError::Game(error)
    }
}
