use crate::config::ConfigError;
use crate::{Attack, Position};
use std::fmt;

/// Pourquoi le moteur refuse une configuration ou une action.
///
/// Une action refusée ne modifie jamais la partie. Le message affiché commence par une
/// minuscule, pour qu'un client puisse écrire « Coup refusé : {erreur} ».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameError {
    /// La configuration est incohérente.
    InvalidConfig(ConfigError),
    /// La position est hors du plateau.
    OutOfBounds(Position),
    /// La case n'est pas libre.
    CellOccupied(Position),
    /// Le joueur n'a plus de pion en réserve.
    EmptyReserve,
    /// Le joueur a déjà posé autant de pions que permis ce tour.
    NoPlacementLeft,
    /// Il ne reste pas assez d'actions ce tour.
    NotEnoughActions {
        /// Coût de l'action demandée.
        needed: u32,
        /// Actions qu'il reste ce tour.
        available: u32,
    },
    /// La partie est terminée.
    GameOver,
    /// Cette action ne répond pas à la décision attendue (par exemple poser un pion
    /// pendant le choix d'une cible).
    UnexpectedAction,
    /// Le joueur n'a pas d'attaque à cet emplacement de son arsenal.
    NoSuchAttack(usize),
    /// Cette attaque est en recharge.
    AttackRecharging(Attack),
    /// Cette attaque n'a aucune cible possible en ce moment.
    NoTarget(Attack),
    /// Cette attaque aurait des cibles, mais seulement depuis des pions qui ne peuvent pas
    /// attaquer maintenant : posés ce tour (mal du débarquement) ou immobilisés.
    NoReadyLauncher(Attack),
    /// Ce choix de lanceur ou de cible n'est pas permis.
    InvalidChoice(Position),
    /// L'attaque en cours ne peut pas encore être jouée : il faut choisir d'autres cibles.
    TargetsMissing(Attack),
    /// La reine ne peut pas faire son pas vers cette case : pas de reine, déjà fait ce tour,
    /// posée ce tour, immobilisée, ou case ni libre ni collée à elle.
    QueenCannotStep(Position),
    /// Cette attaque créerait un effet continu, et le joueur est sous Terre brûlée (Attila
    /// Légendaire de l'adversaire) : elle n'est pas jouable avant la fin de son tour.
    ScorchedEarth(Attack),
    /// Le pion de `from` ne peut pas faire un pas du Lièvre vers `to` : pas de jeton de pas,
    /// déjà dépensé et ses pas faits ce tour, immobilisé, case ni libre ni collée, ou retour
    /// sur sa case de départ.
    HareCannotStep {
        /// La case du pion.
        from: Position,
        /// La case visée.
        to: Position,
    },
    /// La reine ne peut pas tirer sur cette case : pas de reine ni de garde, déjà tiré ce
    /// tour, posée ce tour, immobilisée, ou pas un pion adverse non protégé au bout d'une
    /// ligne droite depuis elle.
    QueenCannotShoot(Position),
    /// Seule une réaction se joue pendant le tour adverse : cette carte est une attaque.
    NotAReaction(Attack),
    /// Rien ne déclenche cette réaction maintenant : elle ne se joue que pendant le tour
    /// adverse, juste après son déclencheur.
    NotTriggered(Attack),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::InvalidConfig(problem) => write!(f, "configuration invalide : {}", problem),
            GameError::OutOfBounds(pos) => write!(f, "la case {} est hors du plateau", pos),
            GameError::CellOccupied(pos) => write!(f, "la case {} est déjà occupée", pos),
            GameError::EmptyReserve => write!(f, "la réserve de pions est vide"),
            GameError::NoPlacementLeft => write!(f, "plus aucune pose permise ce tour"),
            GameError::NotEnoughActions { needed, available } => write!(
                f,
                "pas assez d'actions : il en faut {}, il en reste {}",
                needed, available
            ),
            GameError::GameOver => write!(f, "la partie est terminée"),
            GameError::UnexpectedAction => {
                write!(f, "ce n'est pas le moment de cette action")
            }
            GameError::NoSuchAttack(slot) => {
                write!(
                    f,
                    "aucune attaque à l'emplacement {}",
                    slot.saturating_add(1)
                )
            }
            GameError::AttackRecharging(attack) => {
                write!(f, "l'attaque {} est en recharge", attack)
            }
            GameError::NoTarget(attack) => {
                write!(f, "l'attaque {} n'a aucune cible possible", attack)
            }
            GameError::NoReadyLauncher(attack) => write!(
                f,
                "l'attaque {} n'a aucun lanceur prêt : un pion posé ce tour ou immobilisé ne peut pas attaquer",
                attack
            ),
            GameError::InvalidChoice(pos) => write!(f, "la case {} ne peut pas être choisie", pos),
            GameError::QueenCannotStep(pos) => {
                write!(f, "la reine ne peut pas avancer en {}", pos)
            }
            GameError::ScorchedEarth(attack) => write!(
                f,
                "{} créerait un effet continu : interdit sous Terre brûlée",
                attack
            ),
            GameError::HareCannotStep { from, to } => {
                write!(f, "le pion en {} ne peut pas avancer en {}", from, to)
            }
            GameError::QueenCannotShoot(pos) => {
                write!(f, "aucun garde de la reine ne peut tirer sur {}", pos)
            }
            GameError::NotAReaction(attack) => write!(
                f,
                "{} n'est pas une réaction : pendant le tour adverse, seule une réaction se joue",
                attack
            ),
            GameError::NotTriggered(attack) => write!(
                f,
                "{} est une réaction : elle se joue pendant le tour adverse, quand elle est déclenchée",
                attack
            ),
            GameError::TargetsMissing(attack) => {
                write!(f, "l'attaque {} attend encore une cible", attack)
            }
        }
    }
}
