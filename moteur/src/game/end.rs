use crate::Player;
use std::fmt;

/// Le résultat d'une partie terminée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Ce joueur a gagné.
    Won(Player),
    /// Personne n'a gagné.
    Draw,
}

/// Pourquoi une partie s'est terminée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndReason {
    /// Un joueur a une ligne gagnante : faite en posant (sans la règle d'or), ou encore là
    /// au début de son tour, ou au moment où la partie s'arrête.
    Alignment,
    /// Plus personne ne pourra jamais agir autrement qu'en finissant son tour.
    Frozen,
    /// La limite de tours [`crate::GameConfig::max_turns`] est atteinte.
    TurnLimit,
}

/// « limite de tours atteinte » : la raison, à afficher après le résultat.
impl fmt::Display for EndReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EndReason::Alignment => write!(f, "alignement"),
            EndReason::Frozen => write!(f, "partie figée, plus personne ne peut jouer"),
            EndReason::TurnLimit => write!(f, "limite de tours atteinte"),
        }
    }
}

/// La fin d'une partie : son résultat et sa raison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameEnd {
    /// Qui a gagné, ou match nul.
    pub outcome: Outcome,
    /// Ce qui a arrêté la partie.
    pub reason: EndReason,
}

impl GameEnd {
    /// Le vainqueur, s'il y en a un.
    pub fn winner(self) -> Option<Player> {
        match self.outcome {
            Outcome::Won(player) => Some(player),
            Outcome::Draw => None,
        }
    }
}
