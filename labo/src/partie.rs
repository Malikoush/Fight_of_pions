//! Le seul pont entre l'interface et le moteur : c'est ici, et seulement ici, qu'on
//! appelle `game.apply`, que le coup vienne d'un clic ou d'un bot. La partie tient aussi
//! son enregistrement (configuration, graine, actions acceptées), qui sert à copier,
//! charger et annuler un coup.

use bots::{Bot, BotKind, RandomBot, current_rules, start_game};
use fight_of_pions::{Action, Decision, Event, Game, GameConfig, GameError, GameRecord, Rng};
use std::iter;
use std::time::{SystemTime, UNIX_EPOCH};

/// Ce que l'interface demande à la partie.
pub enum Demande {
    Agir(Action),
    /// Jouer cette suite d'actions d'un bloc (une attaque entière, choisie dans
    /// l'interface) : si l'une est refusée, rien n'est joué.
    Jouer(Vec<Action>),
    /// Recommencer avec cette configuration, et cette graine ou, à défaut, une graine
    /// tirée de l'horloge. Avec la graine d'une partie de l'arène et les mêmes bots, on
    /// revoit cette partie à l'identique.
    NouvellePartie {
        config: GameConfig,
        graine: Option<u64>,
    },
    /// Annuler le dernier coup. Face à un bot, on remonte jusqu'au dernier coup d'un
    /// humain : sinon le bot rejouerait aussitôt ce qu'on vient d'annuler.
    AnnulerCoup {
        /// Quels joueurs sont humains (indice : `Player::index`).
        humains: [bool; 2],
    },
    /// Faire jouer un coup à ce bot pour le joueur qui doit décider.
    CoupDuBot(BotKind),
    /// Jouer ce nombre de coups du bot aléatoire.
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
    /// Le hasard des bots (coups au hasard compris), initialisé avec la graine de la
    /// partie, comme dans l'arène.
    hasard: Rng,
    /// L'état du hasard avant chaque action de l'enregistrement (autant que d'actions) :
    /// annuler le remet tel qu'il était, pour que les bots rejouent la même partie.
    hasard_avant: Vec<Rng>,
}

impl Partie {
    /// Une partie avec une graine tirée de l'horloge.
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        Self::avec_graine(config, graine_du_moment())
    }

    /// Une partie avec cette graine, commencée comme dans l'arène : avec les mêmes bots,
    /// on revoit exactement une partie de l'arène. Sert aussi aux tests reproductibles.
    pub fn avec_graine(config: GameConfig, graine: u64) -> Result<Self, GameError> {
        let (game, hasard) = start_game(&config, graine)?;
        Ok(Self {
            game,
            record: GameRecord::new(config, graine),
            hasard,
            hasard_avant: Vec::new(),
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

    /// La partie telle qu'elle serait après ces actions, sans rien jouer pour de vrai :
    /// sert à choisir une attaque dans l'interface, sur une copie. Annuler ce choix est
    /// donc gratuit, alors que le moteur, lui, n'a pas d'« annuler ».
    pub fn apercu(&self, actions: &[Action]) -> Result<Game, GameError> {
        let mut copie = self.game.clone();
        for &action in actions {
            copie.apply(action)?;
        }
        Ok(copie)
    }

    /// Une erreur du moteur devient une `Nouvelle`, jamais un plantage.
    pub fn traiter(&mut self, demande: Demande) -> Vec<Nouvelle> {
        match demande {
            // Un humain ne consomme pas de hasard : il est le même avant chaque action.
            Demande::Agir(action) => self.jouer(vec![action], &self.hasard.clone()),
            Demande::Jouer(actions) => self.jouer(actions, &self.hasard.clone()),
            Demande::NouvellePartie { config, graine } => self.recommencer(config, graine),
            Demande::AnnulerCoup { humains } => self.annuler_coup(humains),
            Demande::CoupDuBot(sorte) => self.coup_du_bot(sorte.bot()),
            Demande::CoupsAuHasard(nombre) => self.coups_au_hasard(nombre),
            Demande::Charger(texte) => self.charger(&texte),
        }
    }

    /// Joue ces actions d'un bloc, sur une copie d'abord : si l'une est refusée, la
    /// partie et son enregistrement ne changent pas. `hasard_avant` est l'état du hasard
    /// avant ces actions.
    fn jouer(&mut self, actions: Vec<Action>, hasard_avant: &Rng) -> Vec<Nouvelle> {
        let mut copie = self.game.clone();
        let mut nouvelles = Vec::new();
        for &action in &actions {
            match copie.apply(action) {
                Ok(events) => nouvelles.extend(events.into_iter().map(Nouvelle::Moteur)),
                Err(e) => return vec![Nouvelle::Refus(e)],
            }
        }
        self.game = copie;
        self.hasard_avant
            .extend(iter::repeat_n(hasard_avant.clone(), actions.len()));
        self.record.actions.extend(actions);
        nouvelles
    }

    fn recommencer(&mut self, config: GameConfig, graine: Option<u64>) -> Vec<Nouvelle> {
        match Partie::avec_graine(config, graine.unwrap_or_else(graine_du_moment)) {
            Ok(neuve) => {
                *self = neuve;
                vec![Nouvelle::PartieNeuve]
            }
            Err(e) => vec![Nouvelle::Refus(e)],
        }
    }

    /// Rejoue la partie sans son dernier coup : le moteur n'a pas besoin de savoir
    /// « défaire », puisqu'il est déterministe. Une attaque compte pour un seul coup :
    /// on retire ses choix jusqu'à revenir avant elle. Face à un bot, on continue jusqu'à
    /// revenir au tour d'un humain ; s'il n'y en a aucun avant (le bot a commencé et
    /// l'humain n'a pas encore joué), rien n'est annulé. Le hasard reprend son état
    /// d'alors : les bots rejouent les mêmes coups.
    fn annuler_coup(&mut self, humains: [bool; 2]) -> Vec<Nouvelle> {
        let face_a_un_bot = humains.contains(&true) && humains.contains(&false);
        let mut precedent = self.record.clone();
        let mut arret_trouve = false;
        while precedent.actions.pop().is_some() {
            let Ok(game) = precedent.replay() else {
                break;
            };
            let tour_humain = game
                .pending_decision()
                .is_some_and(|decision| humains[decision.player().index()]);
            if !en_plein_choix(&game) && (tour_humain || !face_a_un_bot) {
                arret_trouve = true;
                break;
            }
        }
        if !arret_trouve {
            return Vec::new();
        }
        let gardees = precedent.actions.len();
        let hasard = self
            .hasard_avant
            .get(gardees)
            .cloned()
            .unwrap_or_else(|| Rng::new(precedent.seed));
        let mut hasard_avant = self.hasard_avant.clone();
        hasard_avant.truncate(gardees);
        self.remplacer(precedent, hasard_avant, hasard, Nouvelle::CoupAnnule)
    }

    /// Un coup du bot pour le joueur qui doit décider : une action, et toute la suite si
    /// c'est une attaque, qui va donc au bout dans le même coup. S'arrête au premier refus,
    /// qui serait un bug du bot, affiché comme tout refus.
    fn coup_du_bot(&mut self, bot: &dyn Bot) -> Vec<Nouvelle> {
        let mut nouvelles = Vec::new();
        loop {
            let avant = self.hasard.clone();
            // `None` : la partie est finie.
            let Some(action) = bot.choose(&self.game, &mut self.hasard) else {
                break;
            };
            let jouees = self.jouer(vec![action], &avant);
            let refus = jouees.iter().any(|n| matches!(n, Nouvelle::Refus(_)));
            nouvelles.extend(jouees);
            if refus {
                // Le coup n'est pas joué : son hasard non plus.
                self.hasard = avant;
                break;
            }
            if !en_plein_choix(&self.game) {
                break;
            }
        }
        nouvelles
    }

    fn coups_au_hasard(&mut self, nombre: u32) -> Vec<Nouvelle> {
        let mut nouvelles = Vec::new();
        for _ in 0..nombre {
            if self.game.end().is_some() {
                break;
            }
            nouvelles.extend(self.coup_du_bot(&RandomBot));
        }
        nouvelles
    }

    /// Une partie chargée ne dit rien du hasard de ses bots : il repart de la graine. La
    /// suite de leurs coups n'est donc pas reprise, mais l'enregistrement reste exact.
    fn charger(&mut self, texte: &str) -> Vec<Nouvelle> {
        match importer(texte) {
            Ok(record) => {
                let coups = record.actions.len();
                let graine = Rng::new(record.seed);
                let hasard_avant = vec![graine.clone(); coups];
                self.remplacer(
                    record,
                    hasard_avant,
                    graine,
                    Nouvelle::PartieChargee { coups },
                )
            }
            Err(message) => vec![Nouvelle::ChargementRefuse(message)],
        }
    }

    /// Remplace la partie par la relecture de cet enregistrement, avec cet état du hasard,
    /// si elle réussit ; sinon rien ne change.
    fn remplacer(
        &mut self,
        record: GameRecord,
        hasard_avant: Vec<Rng>,
        hasard: Rng,
        succes: Nouvelle,
    ) -> Vec<Nouvelle> {
        match record.replay() {
            Ok(game) => {
                self.game = game;
                self.record = record;
                self.hasard = hasard;
                self.hasard_avant = hasard_avant;
                vec![succes]
            }
            Err(e) => vec![Nouvelle::ChargementRefuse(e.to_string())],
        }
    }
}

/// La configuration avec laquelle le labo démarre : la règle actuelle, première des
/// variantes proposées.
pub fn config_initiale() -> GameConfig {
    current_rules()
}

/// Vrai si la partie attend le lanceur ou une cible d'une attaque commencée.
pub fn en_plein_choix(game: &Game) -> bool {
    matches!(
        game.pending_decision(),
        Some(Decision::ChooseLauncher { .. } | Decision::ChooseTarget { .. })
    )
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
    use fight_of_pions::{Player, Position};

    /// Une graine fixe : un test qui échoue rejoue exactement la même partie.
    const GRAINE: u64 = 42;
    /// Deux humains : annuler retire un seul coup.
    const DEUX_HUMAINS: [bool; 2] = [true, true];

    fn partie() -> Partie {
        Partie::avec_graine(config_initiale(), GRAINE).unwrap()
    }

    /// Fait jouer ces bots (joueur 1, joueur 2) jusqu'à la fin de la partie.
    fn jouer_jusqu_au_bout(partie: &mut Partie, bots: [BotKind; 2]) {
        while let Some(decision) = partie.game().pending_decision() {
            partie.traiter(Demande::CoupDuBot(bots[decision.player().index()]));
        }
    }

    /// La partie que l'arène joue avec cette configuration, cette graine et ces bots.
    fn partie_de_l_arene(config: &GameConfig, graine: u64, bots: [BotKind; 2]) -> GameRecord {
        bots::record_game(config, graine, [bots[0].bot(), bots[1].bot()]).unwrap()
    }

    #[test]
    fn une_partie_de_l_arene_se_revoit_a_l_identique() {
        // L'aléatoire consomme du hasard à chaque choix, le glouton seulement en se
        // développant ; la deuxième configuration n'est pas celle par défaut.
        let configs = [config_initiale(), bots::variants()[2].config.clone()];
        let paires = [
            [BotKind::Banker, BotKind::Greedy],
            [BotKind::Random, BotKind::Greedy],
        ];
        for config in &configs {
            for bots in paires {
                for graine in 0..2 {
                    let mut partie = Partie::avec_graine(config.clone(), graine).unwrap();
                    jouer_jusqu_au_bout(&mut partie, bots);
                    let attendue = partie_de_l_arene(config, graine, bots);
                    assert_eq!(partie.record(), &attendue, "graine {graine}");
                }
            }
        }
    }

    #[test]
    fn annuler_pendant_qu_on_revoit_une_partie_ne_la_change_pas() {
        let bots = [BotKind::Random, BotKind::Greedy];
        for graine in 0..5 {
            let mut partie = Partie::avec_graine(config_initiale(), graine).unwrap();
            for _ in 0..5 {
                if let Some(decision) = partie.game().pending_decision() {
                    partie.traiter(Demande::CoupDuBot(bots[decision.player().index()]));
                }
            }
            partie.traiter(Demande::AnnulerCoup {
                humains: [false, false],
            });
            jouer_jusqu_au_bout(&mut partie, bots);
            let attendue = partie_de_l_arene(&config_initiale(), graine, bots);
            assert_eq!(partie.record(), &attendue, "graine {graine}");
        }
    }

    #[test]
    fn entre_deux_bots_annuler_retire_un_seul_coup() {
        let mut partie = partie();
        for _ in 0..3 {
            partie.traiter(Demande::CoupDuBot(BotKind::Greedy)); // des poses, une par coup
        }
        partie.traiter(Demande::AnnulerCoup {
            humains: [false, false],
        });
        assert_eq!(partie.record().actions.len(), 2);
    }

    #[test]
    fn face_a_un_bot_qui_commence_annuler_avant_tout_coup_humain_ne_fait_rien() {
        let mut partie = partie();
        partie.traiter(Demande::CoupDuBot(BotKind::Greedy)); // le joueur 1, un bot
        let avant = partie.record().clone();
        let nouvelles = partie.traiter(Demande::AnnulerCoup {
            humains: [false, true],
        });
        assert!(nouvelles.is_empty());
        assert_eq!(partie.record(), &avant);
    }

    #[test]
    fn un_coup_du_bot_ne_laisse_jamais_une_attaque_a_moitie() {
        let mut partie = partie();
        while partie.game().end().is_none() && partie.record().actions.len() < 200 {
            partie.traiter(Demande::CoupDuBot(BotKind::Banker));
            assert!(!en_plein_choix(partie.game()));
        }
    }

    #[test]
    fn face_a_un_bot_annuler_revient_au_tour_de_l_humain() {
        let humains = [true, false];
        let mut partie = partie();
        poser(&mut partie, 4, 4); // le joueur 1, humain : son premier tour n'a qu'une action
        while partie.game().current_player() == Player::Two {
            partie.traiter(Demande::CoupDuBot(BotKind::Greedy));
        }
        let apres_le_tour_du_bot = partie.game().clone();
        poser(&mut partie, 0, 0);

        // Le dernier coup est humain : un seul coup est annulé.
        partie.traiter(Demande::AnnulerCoup { humains });
        assert_eq!(partie.game(), &apres_le_tour_du_bot);
        // Ensuite, tout le tour du bot part avec la pose humaine qui l'a précédé.
        partie.traiter(Demande::AnnulerCoup { humains });
        assert!(partie.record().actions.is_empty());
    }

    fn place(row: usize, col: usize) -> Action {
        Action::Place(Position { row, col })
    }

    fn choisir(row: usize, col: usize) -> Action {
        Action::Choose(Position { row, col })
    }

    fn poser(partie: &mut Partie, row: usize, col: usize) {
        partie.traiter(Demande::Agir(place(row, col)));
    }

    /// Le joueur 1 pose en E5, le joueur 2 en F5 et F6 : le joueur 1 peut attaquer.
    fn avant_une_attaque() -> Partie {
        let mut partie = partie();
        poser(&mut partie, 4, 4);
        poser(&mut partie, 4, 5);
        poser(&mut partie, 5, 5);
        partie
    }

    /// L'Ours Rare (emplacement 0) depuis E5 sur F5.
    fn attaque_complete() -> Vec<Action> {
        vec![Action::UseAttack(0), choisir(4, 4), choisir(4, 5)]
    }

    #[test]
    fn une_attaque_choisie_sur_une_copie_se_joue_d_un_bloc() {
        let mut partie = avant_une_attaque();
        let apercu = partie.apercu(&attaque_complete()[..2]).unwrap();
        assert!(en_plein_choix(&apercu));
        assert!(!en_plein_choix(partie.game())); // la vraie partie n'a pas bougé

        partie.traiter(Demande::Jouer(attaque_complete()));
        assert_eq!(
            partie.game().board().owner(Position { row: 4, col: 5 }),
            None
        );
        assert_eq!(partie.record().replay().as_ref(), Ok(partie.game()));
    }

    #[test]
    fn une_suite_refusee_ne_change_rien() {
        let mut partie = avant_une_attaque();
        let (game_avant, record_avant) = (partie.game().clone(), partie.record().clone());
        // La cible J10 n'est pas à côté du lanceur : toute la suite est refusée.
        let nouvelles = partie.traiter(Demande::Jouer(vec![
            Action::UseAttack(0),
            choisir(4, 4),
            choisir(9, 9),
        ]));
        assert!(matches!(nouvelles[..], [Nouvelle::Refus(_)]));
        assert_eq!(partie.game(), &game_avant);
        assert_eq!(partie.record(), &record_avant);
    }

    #[test]
    fn annuler_retire_toute_l_attaque() {
        let mut partie = avant_une_attaque();
        let avant = partie.game().clone();
        partie.traiter(Demande::Jouer(attaque_complete()));
        partie.traiter(Demande::AnnulerCoup {
            humains: DEUX_HUMAINS,
        });
        assert_eq!(partie.game(), &avant);
        assert!(!en_plein_choix(partie.game()));
    }

    #[test]
    fn les_coups_au_hasard_ne_laissent_jamais_une_attaque_a_moitie() {
        for graine in 0..30 {
            let mut partie = Partie::avec_graine(config_initiale(), graine).unwrap();
            for _ in 0..20 {
                partie.traiter(Demande::CoupsAuHasard(1));
                assert!(!en_plein_choix(partie.game()), "graine {graine}");
            }
        }
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
        partie.traiter(Demande::AnnulerCoup {
            humains: DEUX_HUMAINS,
        });
        assert_eq!(partie.game(), &avant);
        assert_eq!(partie.record().actions.len(), 1);
    }

    #[test]
    fn annuler_sans_coup_ne_fait_rien() {
        let mut partie = partie();
        assert!(
            partie
                .traiter(Demande::AnnulerCoup {
                    humains: DEUX_HUMAINS
                })
                .is_empty()
        );
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
