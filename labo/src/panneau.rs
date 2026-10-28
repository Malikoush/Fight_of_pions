//! Le panneau de test : lit la partie par ses getters, envoie des demandes, et tient un
//! journal. Les lignes de jeu du journal viennent uniquement des `Nouvelle` reçues ; seules
//! les actions propres au labo (copier la partie) y ajoutent leur propre ligne.

use crate::intention::Intention;
use crate::partie::{self, Demande, Nouvelle};
use crate::reglage::ReglageDePartie;
use crate::theme;
use bots::BotKind;
use eframe::egui;
use fight_of_pions::{
    Action, Event, Game, GameConfig, GameEnd, GameRecord, Outcome, Player, Position,
};
use std::time::Duration;

pub struct Panneau {
    saisie: String,
    derniere_erreur: Option<String>,
    journal: Vec<String>,
    coups_au_hasard: u32,
    texte_a_charger: String,
    /// La configuration que lancera « Nouvelle partie ».
    prochaine: ReglageDePartie,
    /// Qui joue chaque joueur : un bot, ou un humain (`None`).
    controleurs: [Option<BotKind>; 2],
    /// Les bots jouent-ils seuls, avec cette attente entre deux coups ?
    lecture_auto: bool,
    attente_ms: u64,
}

impl Panneau {
    pub fn new() -> Panneau {
        Panneau {
            saisie: String::new(),
            derniere_erreur: None,
            journal: Vec::new(),
            coups_au_hasard: theme::COUPS_AU_HASARD_DEFAUT,
            texte_a_charger: String::new(),
            prochaine: ReglageDePartie::new("Prochaine partie"),
            controleurs: [None, None],
            lecture_auto: true,
            attente_ms: theme::ATTENTE_BOT_DEFAUT_MS,
        }
    }

    /// Le bot qui joue ce joueur, ou `None` pour un humain.
    pub fn controleur(&self, player: Player) -> Option<BotKind> {
        self.controleurs[player.index()]
    }

    /// Confie les deux joueurs à ces contrôleurs, et lance la lecture automatique.
    pub fn confier(&mut self, controleurs: [Option<BotKind>; 2]) {
        self.controleurs = controleurs;
        self.lecture_auto = true;
    }

    /// L'attente entre deux coups d'un bot, ou `None` si les bots ne jouent pas seuls.
    pub fn attente_des_bots(&self) -> Option<Duration> {
        self.lecture_auto
            .then(|| Duration::from_millis(self.attente_ms))
    }

    /// Arrête la lecture automatique ; les bots ne jouent plus qu'avec « Coup du bot ».
    pub fn arreter_la_lecture(&mut self) {
        self.lecture_auto = false;
    }

    /// La prochaine « Nouvelle partie » repartira de cette configuration.
    pub fn preparer_la_prochaine(&mut self, config: &GameConfig) {
        self.prochaine.reprendre(config);
    }

    /// Affiche un message d'erreur venu de l'interface, pas du moteur.
    pub fn signaler(&mut self, message: &str) {
        self.derniere_erreur = Some(message.to_string());
    }

    fn humains(&self) -> [bool; 2] {
        self.controleurs.map(|controleur| controleur.is_none())
    }

    /// Vrai si c'est à un humain de décider : les boutons de jeu ne servent qu'à lui.
    fn tour_humain(&self, game: &Game) -> bool {
        game.pending_decision()
            .is_some_and(|decision| self.controleur(decision.player()).is_none())
    }

    pub fn recevoir(&mut self, nouvelle: &Nouvelle) {
        match nouvelle {
            Nouvelle::Moteur(event) => {
                self.derniere_erreur = None;
                if let Some(ligne) = decrire(event) {
                    self.journal.push(ligne);
                }
            }
            Nouvelle::Refus(e) => {
                self.derniere_erreur = Some(e.to_string());
                self.journal.push(format!("Refusé : {}", e));
            }
            Nouvelle::PartieNeuve => {
                self.derniere_erreur = None;
                self.journal.push("— Nouvelle partie —".to_string());
            }
            Nouvelle::CoupAnnule => {
                self.derniere_erreur = None;
                self.journal.push("— Coup annulé —".to_string());
            }
            Nouvelle::PartieChargee { coups } => {
                self.derniere_erreur = None;
                self.texte_a_charger.clear();
                self.journal
                    .push(format!("— Partie chargée ({coups} actions rejouées) —"));
            }
            Nouvelle::ChargementRefuse(message) => {
                self.derniere_erreur = Some(message.clone());
                self.journal.push(format!("Chargement refusé : {message}"));
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        game: &Game,
        record: &GameRecord,
        selection_en_cours: bool,
        intentions: &mut Vec<Intention>,
    ) {
        egui::Panel::left("panneau")
            .default_size(theme::LARGEUR_PANNEAU)
            .show_inside(ui, |ui| {
                // Tout le panneau défile : rien ne sort de l'écran, même configuration
                // dépliée.
                egui::ScrollArea::vertical()
                    .id_salt("panneau_defilant")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        self.contenu(ui, game, record, selection_en_cours, intentions);
                    });
            });
    }

    fn contenu(
        &mut self,
        ui: &mut egui::Ui,
        game: &Game,
        record: &GameRecord,
        selection_en_cours: bool,
        intentions: &mut Vec<Intention>,
    ) {
        ui.heading(theme::TITRE);
        ui.separator();
        etat(ui, game);
        ui.separator();
        let tour_humain = self.tour_humain(game);
        self.commandes(ui, tour_humain, intentions);
        if let Some(erreur) = &self.derniere_erreur {
            ui.colored_label(theme::ERREUR, erreur);
        }
        self.prochaine.show(ui, game.config());
        ui.separator();
        self.joueurs(ui, game, intentions);
        ui.separator();
        attaques(ui, game, selection_en_cours, tour_humain, intentions);
        ui.separator();
        self.outils(ui, record, intentions);
        ui.separator();
        ui.label("Journal");
        egui::ScrollArea::vertical()
            .id_salt("journal")
            .stick_to_bottom(true)
            .max_height(theme::HAUTEUR_JOURNAL)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for ligne in &self.journal {
                    ui.label(ligne);
                }
            });
    }

    /// Poser et finir le tour ne servent qu'au tour d'un humain ; une nouvelle partie se
    /// lance à tout moment.
    fn commandes(&mut self, ui: &mut egui::Ui, tour_humain: bool, intentions: &mut Vec<Intention>) {
        ui.horizontal(|ui| {
            let champ = ui.add(
                egui::TextEdit::singleline(&mut self.saisie)
                    .hint_text(theme::INDICE_SAISIE)
                    .desired_width(theme::LARGEUR_SAISIE),
            );
            let entree = champ.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let poser = ui
                .add_enabled(tour_humain, egui::Button::new("Poser"))
                .clicked();
            if poser || (entree && tour_humain) {
                match self.saisie.trim().parse::<Position>() {
                    Ok(pos) => {
                        intentions.push(Intention::Demande(Demande::Agir(Action::Place(pos))));
                        self.saisie.clear();
                    }
                    Err(message) => self.derniere_erreur = Some(message),
                }
                // On garde le curseur dans le champ pour enchaîner les coups au clavier.
                champ.request_focus();
            }
        });
        if ui
            .add_enabled(tour_humain, egui::Button::new("Finir le tour"))
            .clicked()
        {
            intentions.push(Intention::Demande(Demande::Agir(Action::EndTurn)));
        }
        // La configuration vient de « Prochaine partie » ; incohérente, elle ne se lance pas.
        let config = self.prochaine.config();
        if ui
            .add_enabled(config.is_ok(), egui::Button::new("Nouvelle partie"))
            .on_disabled_hover_text("La configuration de la prochaine partie est incohérente.")
            .clicked()
            && let Ok(config) = config
        {
            intentions.push(Intention::Demande(Demande::NouvellePartie {
                config: config.clone(),
                graine: None,
            }));
        }
    }

    /// Qui joue chaque joueur, et comment les bots jouent : seuls, avec une attente
    /// réglable, ou coup par coup.
    fn joueurs(&mut self, ui: &mut egui::Ui, game: &Game, intentions: &mut Vec<Intention>) {
        ui.label("Joueurs");
        for player in [Player::One, Player::Two] {
            let controleur = &mut self.controleurs[player.index()];
            egui::ComboBox::from_label(theme::nom_joueur(player))
                .selected_text(nom_controleur(*controleur))
                .show_ui(ui, |ui| {
                    ui.selectable_value(controleur, None, theme::HUMAIN);
                    for sorte in BotKind::ALL {
                        ui.selectable_value(controleur, Some(sorte), sorte.name());
                    }
                });
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.lecture_auto, "Lecture automatique");
            ui.add(
                egui::Slider::new(
                    &mut self.attente_ms,
                    theme::ATTENTE_BOT_MIN_MS..=theme::ATTENTE_BOT_MAX_MS,
                )
                .suffix(" ms"),
            );
        });
        let bot = game
            .pending_decision()
            .and_then(|decision| self.controleur(decision.player()));
        if ui
            .add_enabled(bot.is_some(), egui::Button::new("Coup du bot"))
            .clicked()
            && let Some(sorte) = bot
        {
            intentions.push(Intention::Demande(Demande::CoupDuBot(sorte)));
        }
    }

    /// Les outils de test : annuler, jouer au hasard, copier et charger une partie.
    fn outils(&mut self, ui: &mut egui::Ui, record: &GameRecord, intentions: &mut Vec<Intention>) {
        ui.label("Outils");
        ui.horizontal(|ui| {
            let peut_annuler = !record.actions.is_empty();
            if ui
                .add_enabled(peut_annuler, egui::Button::new("Annuler le coup"))
                .clicked()
            {
                intentions.push(Intention::Demande(Demande::AnnulerCoup {
                    humains: self.humains(),
                }));
            }
            if ui.button("Copier la partie").clicked() {
                match partie::exporter(record) {
                    Ok(texte) => {
                        ui.ctx().copy_text(texte);
                        self.journal
                            .push("Partie copiée dans le presse-papiers".to_string());
                    }
                    Err(message) => self.derniere_erreur = Some(message),
                }
            }
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut self.coups_au_hasard)
                    .range(theme::COUPS_AU_HASARD_MIN..=theme::COUPS_AU_HASARD_MAX),
            );
            if ui.button("coups au hasard").clicked() {
                intentions.push(Intention::Demande(Demande::CoupsAuHasard(
                    self.coups_au_hasard,
                )));
            }
        });
        ui.add(
            egui::TextEdit::multiline(&mut self.texte_a_charger)
                .hint_text(theme::INDICE_CHARGEMENT)
                .desired_rows(theme::LIGNES_ZONE_CHARGEMENT),
        );
        if ui
            .add_enabled(
                !self.texte_a_charger.trim().is_empty(),
                egui::Button::new("Charger la partie"),
            )
            .clicked()
        {
            intentions.push(Intention::Demande(Demande::Charger(
                self.texte_a_charger.clone(),
            )));
        }
    }
}

/// L'arsenal du joueur courant : chaque attaque, son coût, sa recharge, et un bouton
/// « Lancer » activé seulement si le moteur l'autorise et que c'est à un humain de jouer.
fn attaques(
    ui: &mut egui::Ui,
    game: &Game,
    selection_en_cours: bool,
    tour_humain: bool,
    intentions: &mut Vec<Intention>,
) {
    ui.label("Attaques");
    let player = game.current_player();
    if game.arsenal(player).is_empty() {
        ui.label("(aucune)");
    }
    let legales = game.legal_actions();
    for (slot, emplacement) in game.arsenal(player).iter().enumerate() {
        let attaque = emplacement.attack();
        let cout = game
            .config()
            .attack_stats(attaque)
            .map_or(String::new(), |stats| format!(", coût {}", stats.cost));
        let etat = match emplacement.cooldown_left() {
            0 => String::new(),
            1 => ", recharge 1 tour".to_string(),
            tours => format!(", recharge {tours} tours"),
        };
        ui.horizontal(|ui| {
            let permise =
                tour_humain && !selection_en_cours && legales.contains(&Action::UseAttack(slot));
            if ui
                .add_enabled(permise, egui::Button::new("Lancer"))
                .clicked()
            {
                intentions.push(Intention::CommencerAttaque(slot));
            }
            ui.label(format!("{attaque}{cout}{etat}"));
        });
    }
    if selection_en_cours {
        ui.label("Clique le pion lanceur, puis la ou les cibles.");
        if ui.button("Annuler l'attaque").clicked() {
            intentions.push(Intention::AnnulerSelection);
        }
    }
}

fn etat(ui: &mut egui::Ui, game: &Game) {
    ui.label(format!("Tour {} — graine {}", game.turn(), game.seed()));
    match game.end() {
        Some(fin) => {
            let couleur = match fin.outcome {
                Outcome::Won(winner) => theme::couleur_joueur(winner),
                Outcome::Draw => ui.visuals().text_color(),
            };
            ui.colored_label(couleur, texte_fin(fin));
        }
        None => {
            ui.label(format!(
                "Au tour de : {}",
                theme::nom_joueur(game.current_player())
            ));
        }
    }
    ui.label(format!(
        "Actions restantes : {}, poses restantes : {}",
        game.actions_left(),
        game.placements_left()
    ));
    if game.config().summoning_sickness {
        ui.label(theme::LEGENDE_MARQUE_NOUVEAU);
    }
    for player in [Player::One, Player::Two] {
        ui.label(format!(
            "{} : {} pions en réserve, {} actions gardées",
            theme::nom_joueur(player),
            game.reserve_pions(player),
            game.banked_actions(player)
        ));
    }
}

fn nom_controleur(controleur: Option<BotKind>) -> &'static str {
    controleur.map_or(theme::HUMAIN, BotKind::name)
}

/// La ligne de journal d'un événement, ou `None` pour ceux qui le rendraient illisible
/// (compteurs d'actions, réserves et recharges sont déjà affichés dans le panneau).
fn decrire(event: &Event) -> Option<String> {
    match event {
        Event::PionPlaced { pion, at } => Some(format!(
            "{} pose le pion {} en {}",
            theme::nom_joueur(pion.owner()),
            pion.id(),
            at
        )),
        Event::AttackUsed {
            player,
            attack,
            launcher,
        } => Some(format!(
            "{} lance {} depuis {}",
            theme::nom_joueur(*player),
            attack,
            launcher
        )),
        Event::PionKilled { pion, at, .. } => Some(format!(
            "Le pion {} de {} est tué en {}",
            pion.id(),
            theme::nom_joueur(pion.owner()),
            at
        )),
        Event::ActionsChanged { .. }
        | Event::ReserveChanged { .. }
        | Event::CooldownChanged { .. } => None,
        Event::TurnEnded { next } => Some(format!("Au tour de {}", theme::nom_joueur(*next))),
        Event::GameEnded(fin) => Some(texte_fin(*fin)),
    }
}

/// « Joueur 1 (violet) gagne la partie (alignement) » ou « Match nul (…) ».
fn texte_fin(fin: GameEnd) -> String {
    match fin.outcome {
        Outcome::Won(winner) => format!(
            "{} gagne la partie ({})",
            theme::nom_joueur(winner),
            fin.reason
        ),
        Outcome::Draw => format!("Match nul ({})", fin.reason),
    }
}
