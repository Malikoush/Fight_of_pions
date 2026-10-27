//! Le panneau de test : lit la partie par ses getters, envoie des demandes, et tient un
//! journal. Les lignes de jeu du journal viennent uniquement des `Nouvelle` reçues ; seules
//! les actions propres au labo (copier la partie) y ajoutent leur propre ligne.

use crate::intention::Intention;
use crate::partie::{self, Demande, Nouvelle};
use crate::theme;
use eframe::egui;
use fight_of_pions::{Action, Attack, Event, Game, GameEnd, GameRecord, Outcome, Player, Position};

pub struct Panneau {
    saisie: String,
    derniere_erreur: Option<String>,
    journal: Vec<String>,
    coups_au_hasard: u32,
    texte_a_charger: String,
    /// Les attaques que chaque joueur aura à la prochaine nouvelle partie.
    attaques_de_test: [Vec<Attack>; 2],
}

impl Panneau {
    pub fn new() -> Panneau {
        Panneau {
            saisie: String::new(),
            derniere_erreur: None,
            journal: Vec::new(),
            coups_au_hasard: theme::COUPS_AU_HASARD_DEFAUT,
            texte_a_charger: String::new(),
            attaques_de_test: partie::attaques_initiales(),
        }
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
                ui.heading(theme::TITRE);
                ui.separator();
                etat(ui, game);
                ui.separator();
                self.commandes(ui, intentions);
                if let Some(erreur) = &self.derniere_erreur {
                    ui.colored_label(theme::ERREUR, erreur);
                }
                ui.separator();
                attaques(ui, game, selection_en_cours, intentions);
                ui.separator();
                self.outils(ui, game, record, intentions);
                ui.separator();
                ui.label("Journal");
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        for ligne in &self.journal {
                            ui.label(ligne);
                        }
                    });
            });
    }

    fn commandes(&mut self, ui: &mut egui::Ui, intentions: &mut Vec<Intention>) {
        ui.horizontal(|ui| {
            let champ = ui.add(
                egui::TextEdit::singleline(&mut self.saisie)
                    .hint_text(theme::INDICE_SAISIE)
                    .desired_width(theme::LARGEUR_SAISIE),
            );
            let entree = champ.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Poser").clicked() || entree {
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
        ui.horizontal(|ui| {
            if ui.button("Finir le tour").clicked() {
                intentions.push(Intention::Demande(Demande::Agir(Action::EndTurn)));
            }
            if ui.button("Nouvelle partie").clicked() {
                intentions.push(Intention::Demande(Demande::NouvellePartie {
                    attaques: self.attaques_de_test.clone(),
                }));
            }
        });
    }

    /// Les outils de test : annuler, jouer au hasard, copier et charger une partie,
    /// choisir les attaques de la prochaine partie.
    fn outils(
        &mut self,
        ui: &mut egui::Ui,
        game: &Game,
        record: &GameRecord,
        intentions: &mut Vec<Intention>,
    ) {
        ui.label("Outils");
        ui.horizontal(|ui| {
            let peut_annuler = !record.actions.is_empty();
            if ui
                .add_enabled(peut_annuler, egui::Button::new("Annuler le coup"))
                .clicked()
            {
                intentions.push(Intention::Demande(Demande::AnnulerCoup));
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
        self.arsenal_de_test(ui, game);
    }

    /// Les attaques de chaque joueur pour la prochaine « Nouvelle partie », parmi celles
    /// du catalogue.
    fn arsenal_de_test(&mut self, ui: &mut egui::Ui, game: &Game) {
        egui::CollapsingHeader::new("Arsenal de la prochaine partie").show(ui, |ui| {
            for player in [Player::One, Player::Two] {
                ui.label(theme::nom_joueur(player));
                let choisies = &mut self.attaques_de_test[player.index()];
                for entree in &game.config().attack_catalog {
                    let mut presente = choisies.contains(&entree.attack);
                    if ui
                        .checkbox(&mut presente, entree.attack.to_string())
                        .changed()
                    {
                        if presente {
                            choisies.push(entree.attack);
                        } else {
                            choisies.retain(|attaque| *attaque != entree.attack);
                        }
                    }
                }
            }
        });
    }
}

/// L'arsenal du joueur courant : chaque attaque, son coût, sa recharge, et un bouton
/// « Lancer » activé seulement si le moteur l'autorise.
fn attaques(
    ui: &mut egui::Ui,
    game: &Game,
    selection_en_cours: bool,
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
            let permise = !selection_en_cours && legales.contains(&Action::UseAttack(slot));
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
    ui.label(format!("Actions restantes : {}", game.actions_left()));
    for player in [Player::One, Player::Two] {
        ui.label(format!(
            "{} : {} pions en réserve, {} actions gardées",
            theme::nom_joueur(player),
            game.reserve_pions(player),
            game.banked_actions(player)
        ));
    }
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
