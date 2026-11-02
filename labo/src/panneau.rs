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
    Action, ActiveEffect, Event, Game, GameConfig, GameEnd, GameRecord, Outcome, Player, Position,
    TokenPile, TokenReward,
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

    /// `apercu` : la copie de la partie où se choisit une attaque, s'il y en a une.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        game: &Game,
        record: &GameRecord,
        apercu: Option<&Game>,
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
                        self.contenu(ui, game, record, apercu, intentions);
                    });
            });
    }

    fn contenu(
        &mut self,
        ui: &mut egui::Ui,
        game: &Game,
        record: &GameRecord,
        apercu: Option<&Game>,
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
        attaques(ui, game, apercu, tour_humain, intentions);
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
/// Pendant le choix d'une attaque (sur la copie `apercu`, ou dans la vraie partie si elle
/// a été chargée en plein choix), « Jouer l'attaque » est activé si le moteur permet de la
/// jouer avec les cibles déjà choisies.
fn attaques(
    ui: &mut egui::Ui,
    game: &Game,
    apercu: Option<&Game>,
    tour_humain: bool,
    intentions: &mut Vec<Intention>,
) {
    let selection_en_cours = apercu.is_some();
    let en_choix = apercu.unwrap_or(game);
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
    reine(ui, &legales, tour_humain && !selection_en_cours, intentions);
    if en_choix.attack_in_progress() {
        ui.label("Clique le pion lanceur, puis la ou les cibles.");
        // Une attaque « jusqu'à 2 » peut se jouer avant son nombre maximal de cibles.
        let legales_du_choix = en_choix.legal_actions();
        let fin_possible = tour_humain && legales_du_choix.contains(&Action::FinishAttack);
        // Les jetons adverses ne sont sur aucune case : un bouton les vise.
        let jetons_visables = tour_humain && legales_du_choix.contains(&Action::ChooseEnemyTokens);
        if jetons_visables
            && ui
                .button("Viser les jetons adverses")
                .on_hover_text("Annuler toutes les piles du Massacre divin de l'adversaire.")
                .clicked()
        {
            intentions.push(Intention::ViserJetons);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(fin_possible, egui::Button::new("Jouer l'attaque"))
                .on_hover_text("Jouer l'attaque avec les cibles déjà choisies.")
                .clicked()
            {
                intentions.push(Intention::FinirAttaque);
            }
            // Seule une sélection se défait : une attaque déjà envoyée au moteur va au bout.
            if selection_en_cours && ui.button("Annuler l'attaque").clicked() {
                intentions.push(Intention::AnnulerSelection);
            }
        });
    }
}

/// Les gestes de la reine que le moteur permet maintenant, un bouton chacun : son pas
/// gratuit vers une case, ou le tir d'un garde sur un pion. Cliquer une case libre sert
/// déjà à poser : un bouton évite toute confusion.
fn reine(ui: &mut egui::Ui, legales: &[Action], permis: bool, intentions: &mut Vec<Intention>) {
    let gestes: Vec<(Action, String)> = legales
        .iter()
        .filter_map(|&action| match action {
            Action::StepQueen(vers) => Some((action, format!("Reine → {vers}"))),
            Action::Shoot(cible) => Some((action, format!("Tir sur {cible}"))),
            _ => None,
        })
        .collect();
    if gestes.is_empty() {
        return;
    }
    ui.label("Reine");
    ui.horizontal_wrapped(|ui| {
        for (action, texte) in gestes {
            if ui.add_enabled(permis, egui::Button::new(texte)).clicked() {
                intentions.push(Intention::Demande(Demande::Agir(action)));
            }
        }
    });
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
    if !game.active_effects().is_empty() {
        ui.label(theme::LEGENDE_STATUTS);
    }
    for player in [Player::One, Player::Two] {
        let blesses = game
            .infirmary()
            .iter()
            .filter(|patient| patient.pion().owner() == player)
            .count();
        let morts = game
            .graveyard()
            .iter()
            .filter(|pion| pion.owner() == player)
            .count();
        let cimetiere = if morts > 0 {
            format!(
                ", {morts} au cimetière ({} pions en vie)",
                game.living_pions(player)
            )
        } else {
            String::new()
        };
        ui.label(format!(
            "{} : {} pions en réserve, {} à l'infirmerie{}, {} actions gardées",
            theme::nom_joueur(player),
            game.reserve_pions(player),
            blesses,
            cimetiere,
            game.banked_actions(player)
        ));
        for pile in game
            .token_piles()
            .iter()
            .filter(|pile| pile.owner() == player)
        {
            ui.label(format!("    {}", texte_pile(*pile)));
        }
    }
}

fn nom_controleur(controleur: Option<BotKind>) -> &'static str {
    controleur.map_or(theme::HUMAIN, BotKind::name)
}

/// La ligne de journal d'un événement, ou `None` pour ceux qui le rendraient illisible
/// (compteurs d'actions, poses restantes, réserves et recharges sont déjà affichés dans
/// le panneau).
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
        Event::QueenShot { queen, target } => Some(format!(
            "Un garde de la reine en {} tire sur {}",
            queen, target
        )),
        Event::PionKilled { pion, at, .. } => Some(format!(
            "Le pion {} de {} est tué en {}",
            pion.id(),
            theme::nom_joueur(pion.owner()),
            at
        )),
        Event::PionMoved { pion, from, to } => Some(format!(
            "Le pion {} de {} va de {} en {}",
            pion.id(),
            theme::nom_joueur(pion.owner()),
            from,
            to
        )),
        Event::PionsSwapped {
            first,
            first_at,
            second,
            second_at,
        } => Some(format!(
            "Les pions {} ({}) et {} ({}) échangent leurs cases",
            first.id(),
            first_at,
            second.id(),
            second_at
        )),
        Event::EffectStarted(effet) => Some(format!(
            "Le pion {} est {}{} ({})",
            effet.pion(),
            effet.kind(),
            jusqu_a(*effet),
            effet.source()
        )),
        Event::EffectEnded(effet) => Some(format!(
            "Le pion {} n'est plus {}",
            effet.pion(),
            effet.kind()
        )),
        Event::InfirmaryEntered(patient) => Some(format!(
            "Le pion {} de {} part à l'infirmerie jusqu'à la fin du tour {}",
            patient.pion().id(),
            theme::nom_joueur(patient.pion().owner()),
            patient.last_turn()
        )),
        Event::TombAdded { pion, at } => Some(format!(
            "Le pion {} de {} laisse sa tombe en {}",
            pion.id(),
            theme::nom_joueur(pion.owner()),
            at
        )),
        Event::TombRemoved { pion, at } => {
            Some(format!("La tombe du pion {} s'efface en {}", pion.id(), at))
        }
        Event::InfirmaryLeft(patient) => Some(format!(
            "Le pion {} de {} sort de l'infirmerie",
            patient.pion().id(),
            theme::nom_joueur(patient.pion().owner())
        )),
        Event::GraveyardEntered(pion) => Some(format!(
            "Le pion {} de {} part au cimetière, pour toujours",
            pion.id(),
            theme::nom_joueur(pion.owner())
        )),
        Event::TokenPileStarted(pile) => Some(format!(
            "{} pose {}",
            theme::nom_joueur(pile.owner()),
            texte_pile(*pile)
        )),
        Event::TokenUsed(pile) => Some(format!("Un jeton sert : {}", texte_pile(*pile))),
        Event::TokenPileEnded(pile) => Some(format!(
            "La pile de {} ({}) disparaît",
            theme::nom_joueur(pile.owner()),
            pile.source()
        )),
        Event::ActionsChanged { .. }
        | Event::PlacementsChanged { .. }
        | Event::ReserveChanged { .. }
        | Event::CooldownChanged { .. } => None,
        Event::TurnEnded { next } => Some(format!("Au tour de {}", theme::nom_joueur(*next))),
        Event::GameEnded(fin) => Some(texte_fin(*fin)),
    }
}

/// « Massacre divin (Rare) : 3 jetons, 1 point par kill ».
fn texte_pile(pile: TokenPile) -> String {
    let jetons = match pile.tokens() {
        1 => "1 jeton".to_string(),
        n => format!("{n} jetons"),
    };
    let gain = match pile.reward() {
        TokenReward::Points(1) => "1 point par kill".to_string(),
        TokenReward::Points(n) => format!("{n} points par kill"),
        TokenReward::Graveyard => "le pion tué part au cimetière".to_string(),
    };
    format!("{} : {jetons}, {gain}", pile.source())
}

/// « jusqu'à la fin du tour 6 », « tant que sa famille l'encadre » (un enfant de la
/// Familia), ou rien pour un autre effet sans durée (un pacte).
fn jusqu_a(effet: ActiveEffect) -> String {
    if effet.framed_by().is_some() {
        return " tant que sa famille l'encadre".to_string();
    }
    effet.last_turn().map_or(String::new(), |tour| {
        format!(" jusqu'à la fin du tour {tour}")
    })
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
