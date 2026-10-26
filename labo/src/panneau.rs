//! Le panneau de test : lit la partie par ses getters, envoie des demandes,
//! et tient un journal construit uniquement à partir des `Nouvelle` reçues.

use crate::partie::{Demande, Nouvelle};
use crate::theme;
use eframe::egui;
use fight_of_pions::{Action, Event, Game, Player, Position};

#[derive(Default)]
pub struct Panneau {
    saisie: String,
    derniere_erreur: Option<String>,
    journal: Vec<String>,
}

impl Panneau {
    pub fn recevoir(&mut self, nouvelle: &Nouvelle) {
        match nouvelle {
            Nouvelle::Moteur(event) => {
                self.derniere_erreur = None;
                self.journal.push(decrire(event));
            }
            Nouvelle::Refus(e) => {
                self.derniere_erreur = Some(e.to_string());
                self.journal.push(format!("Refusé : {}", e));
            }
            Nouvelle::PartieNeuve => {
                self.derniere_erreur = None;
                self.journal.push("— Nouvelle partie —".to_string());
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, game: &Game, demandes: &mut Vec<Demande>) {
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
}

fn etat(ui: &mut egui::Ui, game: &Game) {
    ui.label(format!("Tour {}", game.turn()));
    match game.winner() {
        Some(winner) => {
            ui.colored_label(
                theme::couleur_joueur(winner),
                format!("{} a gagné !", theme::nom_joueur(winner)),
            );
        }
        None => {
            ui.label(format!("Au tour de : {}", theme::nom_joueur(game.current_player())));
        }
    }
    ui.label(format!("Actions restantes : {}", game.actions_left()));
    for player in [Player::One, Player::Two] {
        ui.label(format!(
            "Réserve {} : {}",
            theme::nom_joueur(player),
            game.reserve_pions(player)
        ));
    }
}

fn decrire(event: &Event) -> String {
    match event {
        Event::PawnPlaced { player, at } => {
            format!("{} pose en {}", theme::nom_joueur(*player), at)
        }
        Event::TurnEnded { next } => format!("Au tour de {}", theme::nom_joueur(*next)),
        Event::GameWon { winner } => format!("{} gagne la partie !", theme::nom_joueur(*winner)),
    }
}
