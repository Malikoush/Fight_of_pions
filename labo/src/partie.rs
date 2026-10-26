//! Le seul pont entre l'interface et le moteur : c'est ici, et seulement ici, qu'on
//! appelle `game.apply`. La partie tient aussi son enregistrement (configuration, graine,
//! actions acceptées), qui sert à copier, charger et annuler un coup.

use fight_of_pions::{Action, Event, Game, GameConfig, GameError, GameRecord, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

/// Ce que l'interface (et plus tard un bot) demande à la partie.
pub enum Demande {
    Agir(Action),
    NouvellePartie,
    AnnulerCoup,
    /// Jouer ce nombre de coups tirés au hasard parmi les actions légales.
    CoupsAuHasard(u32),
    /// Remplacer la partie par celle décrite dans ce texte JSON.
    Charger(String),
}

/// Ce que la partie annonce en retour.
pub enum Nouvelle {
    Moteur(Event),
    Refus(GameError),
    PartieNeuve,
    CoupAnnule,
    PartieChargee { coups: usize },
    ChargementRefuse(String),
}

pub struct Partie {
    game: Game,
    record: GameRecord,
    /// Le hasard des coups joués au hasard, initialisé avec la graine de la partie.
    hasard: Rng,
}

impl Partie {
    /// Une partie avec une graine tirée de l'horloge.
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        Self::avec_graine(config, graine_du_moment())
    }

    /// Une partie avec cette graine : sert aux tests, pour qu'ils soient reproductibles.
    pub fn avec_graine(config: GameConfig, graine: u64) -> Result<Self, GameError> {
        Ok(Self {
            game: Game::new(config, graine)?,
            record: GameRecord::new(config, graine),
            hasard: Rng::new(graine),
        })
    }

    /// Lecture seule : l'affichage ne modifie jamais la partie lui-même.
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// L'enregistrement de la partie en cours.
    pub fn record(&self) -> &GameRecord {
        &self.record
    }

    /// Une erreur du moteur devient une `Nouvelle`, jamais un plantage.
    pub fn traiter(&mut self, demande: Demande) -> Vec<Nouvelle> {
        match demande {
            Demande::Agir(action) => self.agir(action),
            Demande::NouvellePartie => self.recommencer(),
            Demande::AnnulerCoup => self.annuler_coup(),
            Demande::CoupsAuHasard(nombre) => self.coups_au_hasard(nombre),
            Demande::Charger(texte) => self.charger(&texte),
        }
    }

    fn agir(&mut self, action: Action) -> Vec<Nouvelle> {
        match self.game.apply(action) {
            Ok(events) => {
                self.record.actions.push(action);
                events.into_iter().map(Nouvelle::Moteur).collect()
            }
            Err(e) => vec![Nouvelle::Refus(e)],
        }
    }

    fn recommencer(&mut self) -> Vec<Nouvelle> {
        let config = self.record.config;
        match Partie::new(config) {
            Ok(neuve) => {
                *self = neuve;
                vec![Nouvelle::PartieNeuve]
            }
            Err(e) => vec![Nouvelle::Refus(e)],
        }
    }

    /// Rejoue la partie sans sa dernière action : le moteur n'a pas besoin de savoir
    /// « défaire », puisqu'il est déterministe.
    fn annuler_coup(&mut self) -> Vec<Nouvelle> {
        let mut precedent = self.record.clone();
        if precedent.actions.pop().is_none() {
            return Vec::new();
        }
        self.remplacer(precedent, Nouvelle::CoupAnnule)
    }

    fn coups_au_hasard(&mut self, nombre: u32) -> Vec<Nouvelle> {
        let mut nouvelles = Vec::new();
        for _ in 0..nombre {
            // Plus aucune action légale : la partie est finie.
            let Some(&action) = self.hasard.pick(&self.game.legal_actions()) else {
                break;
            };
            nouvelles.extend(self.agir(action));
        }
        nouvelles
    }

    fn charger(&mut self, texte: &str) -> Vec<Nouvelle> {
        match importer(texte) {
            Ok(record) => {
                let coups = record.actions.len();
                self.remplacer(record, Nouvelle::PartieChargee { coups })
            }
            Err(message) => vec![Nouvelle::ChargementRefuse(message)],
        }
    }

    /// Remplace la partie par la relecture de cet enregistrement, si elle réussit ; sinon
    /// rien ne change. Le hasard des « coups au hasard » repart de la graine : la suite de
    /// ces coups n'est pas reprise, mais l'enregistrement, lui, reste exact.
    fn remplacer(&mut self, record: GameRecord, succes: Nouvelle) -> Vec<Nouvelle> {
        match record.replay() {
            Ok(game) => {
                self.hasard = Rng::new(record.seed);
                self.game = game;
                self.record = record;
                vec![succes]
            }
            Err(e) => vec![Nouvelle::ChargementRefuse(e.to_string())],
        }
    }
}

/// La partie en JSON lisible, à coller ailleurs ou à joindre à un rapport de bug.
pub fn exporter(record: &GameRecord) -> Result<String, String> {
    serde_json::to_string_pretty(record).map_err(|e| format!("export impossible : {e}"))
}

/// Lit un enregistrement JSON produit par [`exporter`].
pub fn importer(texte: &str) -> Result<GameRecord, String> {
    serde_json::from_str(texte.trim()).map_err(|e| format!("texte illisible : {e}"))
}

/// Une graine différente à chaque partie, tirée de l'horloge. Le moteur, lui, n'utilise
/// jamais l'horloge : c'est au client de choisir la graine, et elle est gardée dans la
/// partie pour pouvoir la rejouer.
fn graine_du_moment() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duree| duree.as_secs() ^ u64::from(duree.subsec_nanos()))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fight_of_pions::Position;

    /// Une graine fixe : un test qui échoue rejoue exactement la même partie.
    const GRAINE: u64 = 42;

    fn partie() -> Partie {
        Partie::avec_graine(GameConfig::default(), GRAINE).unwrap()
    }

    fn place(row: usize, col: usize) -> Action {
        Action::Place(Position { row, col })
    }

    fn poser(partie: &mut Partie, row: usize, col: usize) {
        partie.traiter(Demande::Agir(place(row, col)));
    }

    /// Charge ce texte et vérifie qu'il est refusé sans rien changer : ni la partie,
    /// ni son enregistrement.
    fn assert_chargement_refuse_sans_effet(texte: String) {
        let mut partie = partie();
        poser(&mut partie, 4, 4);
        let (game_avant, record_avant) = (partie.game().clone(), partie.record().clone());

        let nouvelles = partie.traiter(Demande::Charger(texte));
        assert!(matches!(nouvelles[..], [Nouvelle::ChargementRefuse(_)]));
        assert_eq!(partie.game(), &game_avant);
        assert_eq!(partie.record(), &record_avant);
    }

    #[test]
    fn export_puis_import_redonne_le_meme_enregistrement() {
        let mut partie = partie();
        poser(&mut partie, 4, 4);
        poser(&mut partie, 0, 0);
        let texte = exporter(partie.record()).unwrap();
        assert_eq!(importer(&texte).as_ref(), Ok(partie.record()));
    }

    #[test]
    fn charger_reconstruit_la_partie() {
        let mut source = partie();
        source.traiter(Demande::CoupsAuHasard(7));
        let texte = exporter(source.record()).unwrap();

        // Une partie d'une autre graine : le chargement doit aussi reprendre la graine.
        let mut cible = Partie::avec_graine(GameConfig::default(), GRAINE + 1).unwrap();
        cible.traiter(Demande::Charger(texte));
        assert_eq!(cible.game(), source.game());
        assert_eq!(cible.record(), source.record());
    }

    #[test]
    fn un_texte_illisible_ne_change_rien() {
        assert_chargement_refuse_sans_effet("pas du JSON".to_string());
    }

    #[test]
    fn une_partie_avec_une_action_refusee_ne_change_rien() {
        let mut record = GameRecord::new(GameConfig::default(), GRAINE);
        // Le joueur 2 rejoue sur la case que le joueur 1 vient de prendre.
        record.actions = vec![place(0, 0), place(0, 0)];
        assert_chargement_refuse_sans_effet(exporter(&record).unwrap());
    }

    #[test]
    fn une_configuration_invalide_ne_change_rien() {
        let config = GameConfig {
            board_size: 0,
            ..GameConfig::default()
        };
        let record = GameRecord::new(config, GRAINE);
        assert_chargement_refuse_sans_effet(exporter(&record).unwrap());
    }

    #[test]
    fn un_champ_inconnu_est_refuse_au_lieu_d_etre_ignore() {
        let record = GameRecord::new(GameConfig::default(), GRAINE);
        let texte = exporter(&record)
            .unwrap()
            .replacen('{', "{\"bonus\": 1,", 1);
        assert_chargement_refuse_sans_effet(texte);
    }

    #[test]
    fn annuler_revient_au_coup_precedent() {
        let mut partie = partie();
        poser(&mut partie, 4, 4);
        let avant = partie.game().clone();
        poser(&mut partie, 5, 5);
        partie.traiter(Demande::AnnulerCoup);
        assert_eq!(partie.game(), &avant);
        assert_eq!(partie.record().actions.len(), 1);
    }

    #[test]
    fn annuler_sans_coup_ne_fait_rien() {
        let mut partie = partie();
        assert!(partie.traiter(Demande::AnnulerCoup).is_empty());
    }

    #[test]
    fn un_coup_refuse_n_est_pas_enregistre() {
        let mut partie = partie();
        poser(&mut partie, 4, 4);
        poser(&mut partie, 4, 4);
        assert_eq!(partie.record().actions.len(), 1);
    }

    #[test]
    fn les_coups_au_hasard_s_arretent_a_la_fin_de_la_partie() {
        let mut partie = partie();
        partie.traiter(Demande::CoupsAuHasard(10_000));
        assert!(partie.game().end().is_some());
        assert_eq!(partie.record().replay().as_ref(), Ok(partie.game()));
    }
}
