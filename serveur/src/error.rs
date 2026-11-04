use fight_of_pions::GameError;
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
    /// Le bot n'a pas pu jouer (un bug) : la partie attend.
    BotFailed(String),
    /// Le moteur refuse ce coup, ou cette configuration.
    Game(GameError),
    /// Le fichier des textes des cartes est mal écrit (un bug, que les tests empêchent).
    CardsUnavailable(String),
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
            ServerError::BotFailed(_) => "bot_failed",
            ServerError::Game(_) => "refused_move",
            ServerError::CardsUnavailable(_) => "cards_unavailable",
        }
    }
}

impl From<GameError> for ServerError {
    fn from(error: GameError) -> Self {
        ServerError::Game(error)
    }
}
