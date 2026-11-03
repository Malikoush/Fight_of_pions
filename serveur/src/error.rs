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
    /// Ce jeton ne correspond pas à cette place.
    BadToken,
    /// Il faut d'abord créer ou rejoindre une salle.
    NotSeated,
    /// Le client est déjà assis dans une salle.
    AlreadySeated,
    /// Aucune variante n'a ce nom.
    UnknownVariant(String),
    /// Ce n'est pas à ce joueur de décider.
    NotYourTurn,
    /// Le moteur refuse ce coup, ou cette configuration.
    Game(GameError),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::BadMessage(why) => write!(f, "message illisible : {why}"),
            ServerError::UnknownRoom(code) => write!(f, "aucune salle n'a le code {code}"),
            ServerError::RoomFull => write!(f, "cette salle est complète"),
            ServerError::BadToken => write!(f, "ce jeton ne correspond pas à cette place"),
            ServerError::NotSeated => write!(f, "il faut d'abord créer ou rejoindre une salle"),
            ServerError::AlreadySeated => write!(f, "tu es déjà dans une salle"),
            ServerError::UnknownVariant(name) => {
                write!(f, "aucune variante ne s'appelle « {name} »")
            }
            ServerError::NotYourTurn => write!(f, "ce n'est pas à toi de jouer"),
            ServerError::Game(error) => write!(f, "coup refusé : {error}"),
        }
    }
}

impl From<GameError> for ServerError {
    fn from(error: GameError) -> Self {
        ServerError::Game(error)
    }
}
