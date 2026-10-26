//! Le panneau de test : lit la partie par ses getters, envoie des demandes, et tient un
//! journal. Les lignes de jeu du journal viennent uniquement des `Nouvelle` reçues ; seules
//! les actions propres au labo (copier la partie) y ajoutent leur propre ligne.

use crate::partie::{self, Demande, Nouvelle};
use crate::theme;
use eframe::egui;
use fight_of_pions::{Action, Event, Game, GameEnd, GameRecord, Outcome, Player, Position};

pub struct Panneau {
    saisie: String,
    derniere_erreur: Option<String>,
    journal: Vec<String>,
    coups_au_hasard: u32,
    texte_a_charger: String,
}

impl Panneau {
    pub fn new() -> Panneau {
        Panneau {
            saisie: String::new(),
            derniere_erreur: None,
            journal: Vec::new(),
            coups_au_hasard: theme::COUPS_AU_HASARD_DEFAUT,
            texte_a_charger: String::new(),
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
        demandes: &mut Vec<Demande>,
    ) {
        egui::Panel::left("panneau")
            .default_size(theme::LARGEUR_PANNEAU)
            .show_inside(ui, |ui| {
                ui.heading(theme::TITRE);
                ui.separator();
                etat(ui, game);
                ui.separator();
                self.commandes(ui, demandes);
                if let Some(erreur) = &self.derniere_erreur {
                    ui.colored_label(theme::ERREUR, erreur);
                }
                ui.separator();
                self.outils(ui, record, demandes);
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

    fn commandes(&mut self, ui: &mut egui::Ui, demandes: &mut Vec<Demande>) {
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
                        demandes.push(Demande::Agir(Action::Place(pos)));
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
                demandes.push(Demande::Agir(Action::EndTurn));
            }
            if ui.button("Nouvelle partie").clicked() {
                demandes.push(Demande::NouvellePartie);
            }
        });
    }

    /// Les outils de test : annuler, jouer au hasard, copier et charger une partie.
    fn outils(&mut self, ui: &mut egui::Ui, record: &GameRecord, demandes: &mut Vec<Demande>) {
        ui.label("Outils");
        ui.horizontal(|ui| {
            let peut_annuler = !record.actions.is_empty();
            if ui
                .add_enabled(peut_annuler, egui::Button::new("Annuler le coup"))
                .clicked()
            {
                demandes.push(Demande::AnnulerCoup);
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
                demandes.push(Demande::CoupsAuHasard(self.coups_au_hasard));
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
            demandes.push(Demande::Charger(self.texte_a_charger.clone()));
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
/// (les compteurs d'actions sont déjà affichés en haut du panneau).
fn decrire(event: &Event) -> Option<String> {
    match event {
        Event::PionPlaced { pion, at } => Some(format!(
            "{} pose le pion {} en {}",
            theme::nom_joueur(pion.owner()),
            pion.id(),
            at
        )),
        Event::ActionsChanged { .. } => None,
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
