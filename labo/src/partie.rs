//! Le seul pont entre l'interface et le moteur :
//! c'est ici, et seulement ici, qu'on appelle `game.apply`.

use fight_of_pions::{Action, Event, Game, GameConfig, GameError};

/// Ce que l'interface (et plus tard un bot) demande à la partie.
pub enum Demande {
    Agir(Action),
    NouvellePartie,
}

/// Ce que la partie annonce en retour.
pub enum Nouvelle {
    Moteur(Event),
    Refus(GameError),
    PartieNeuve,
}

pub struct Partie {
    game: Game,
    config: GameConfig,
}

impl Partie {
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        Ok(Self {
            game: Game::new(config)?,
            config,
        })
    }

    /// Lecture seule : l'affichage ne modifie jamais la partie lui-même.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// Une erreur du moteur devient une `Nouvelle::Refus`, jamais un plantage.
    pub fn traiter(&mut self, demande: Demande) -> Vec<Nouvelle> {
        match demande {
            Demande::Agir(action) => match self.game.apply(action) {
                Ok(events) => events.into_iter().map(Nouvelle::Moteur).collect(),
                Err(e) => vec![Nouvelle::Refus(e)],
            },
            Demande::NouvellePartie => match Game::new(self.config) {
                Ok(game) => {
                    self.game = game;
                    vec![Nouvelle::PartieNeuve]
                }
                Err(e) => vec![Nouvelle::Refus(e)],
            },
        }
    }
}
