//! L'écran Arène : faire jouer des paires de bots sur une configuration (une variante,
//! retouchée ou non), lire leurs bilans, et revoir n'importe laquelle de ces parties dans
//! l'écran Partie.
//!
//! Cet écran ne calcule rien lui-même : il lance `bots::run_pairs` dans un fil à part,
//! pour ne pas figer la fenêtre, et affiche les bilans comme `cargo arene`.

use crate::intention::Intention;
use crate::reglage::ReglageDePartie;
use crate::theme;
use bots::{BotKind, Report, STANDARD_PAIRS, run_pairs};
use eframe::egui;
use fight_of_pions::{GameConfig, GameError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub struct EcranArene {
    /// La configuration sur laquelle les bots s'affrontent.
    reglage: ReglageDePartie,
    parties: u64,
    /// Les paires proposées (joueur 1, joueur 2), et si elles seront jouées.
    paires: Vec<([BotKind; 2], bool)>,
    /// La graine de la partie à revoir.
    graine: u64,
    calcul: Option<Calcul>,
    resultat: Option<Resultat>,
}

/// Ce qui est demandé à l'arène : la même chose sert au calcul en cours et au résultat.
#[derive(Clone)]
struct Simulation {
    /// D'où vient la configuration (variante, retouchée ou non), pour le titre des bilans.
    description: String,
    config: GameConfig,
    paires: Vec<[BotKind; 2]>,
    parties: u64,
}

struct Calcul {
    simulation: Simulation,
    depart: Instant,
    fil: JoinHandle<Result<Vec<Report>, GameError>>,
}

struct Resultat {
    simulation: Simulation,
    duree: Duration,
    bilans: Result<Vec<Report>, String>,
}

impl EcranArene {
    pub fn new() -> EcranArene {
        EcranArene {
            reglage: ReglageDePartie::new("Configuration simulée"),
            parties: theme::PARTIES_ARENE_DEFAUT,
            paires: STANDARD_PAIRS.iter().map(|&paire| (paire, true)).collect(),
            graine: 0,
            calcul: None,
            resultat: None,
        }
    }

    /// `en_cours` : la configuration de la partie de l'écran Partie, qu'on peut reprendre.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        en_cours: &GameConfig,
        intentions: &mut Vec<Intention>,
    ) {
        self.recuperer_le_calcul(ui.ctx());
        egui::Panel::left("reglages_arene")
            .default_size(theme::LARGEUR_PANNEAU)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("reglages_arene_defilants")
                    .auto_shrink(false)
                    .show(ui, |ui| self.reglages(ui, en_cours));
            });
        egui::CentralPanel::default().show_inside(ui, |ui| self.bilans(ui, intentions));
    }

    fn reglages(&mut self, ui: &mut egui::Ui, en_cours: &GameConfig) {
        ui.heading(theme::ECRAN_ARENE);
        self.reglage.show(ui, en_cours);
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut self.parties)
                    .range(theme::PARTIES_ARENE_MIN..=theme::PARTIES_ARENE_MAX),
            );
            ui.label("parties par paire");
        });
        ui.separator();
        ui.label("Paires (joueur 1 contre joueur 2)");
        for (paire, jouee) in &mut self.paires {
            ui.checkbox(jouee, nom_paire(*paire));
        }
        ui.separator();
        match &self.calcul {
            Some(calcul) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!(
                        "Calcul en cours… {:.0} s",
                        calcul.depart.elapsed().as_secs_f64()
                    ));
                });
            }
            None => {
                let une_paire_cochee = self.paires.iter().any(|(_, jouee)| *jouee);
                let config_valide = self.reglage.config().is_ok();
                if ui
                    .add_enabled(
                        une_paire_cochee && config_valide,
                        egui::Button::new("Lancer"),
                    )
                    .on_disabled_hover_text(
                        "Il faut au moins une paire cochée et une configuration cohérente.",
                    )
                    .clicked()
                {
                    self.lancer();
                }
            }
        }
    }

    /// Lance le calcul dans un fil à part, avec une copie de ce qui est demandé.
    fn lancer(&mut self) {
        let Ok(config) = self.reglage.config() else {
            return;
        };
        let simulation = Simulation {
            description: self.reglage.description(),
            config: config.clone(),
            paires: self
                .paires
                .iter()
                .filter(|(_, jouee)| *jouee)
                .map(|(paire, _)| *paire)
                .collect(),
            parties: self.parties,
        };
        let copie = simulation.clone();
        let fil = thread::spawn(move || run_pairs(&copie.config, &copie.paires, copie.parties));
        self.calcul = Some(Calcul {
            simulation,
            depart: Instant::now(),
            fil,
        });
    }

    /// Range le résultat quand le calcul est fini ; sinon, revient voir un peu plus tard.
    fn recuperer_le_calcul(&mut self, ctx: &egui::Context) {
        let fini = self
            .calcul
            .as_ref()
            .is_some_and(|calcul| calcul.fil.is_finished());
        if !fini {
            if self.calcul.is_some() {
                ctx.request_repaint_after(Duration::from_millis(theme::INTERVALLE_CALCUL_MS));
            }
            return;
        }
        let Some(calcul) = self.calcul.take() else {
            return;
        };
        let bilans = match calcul.fil.join() {
            Ok(bilans) => bilans.map_err(|erreur| format!("configuration invalide : {erreur}")),
            Err(cause) => {
                let detail = cause
                    .downcast_ref::<&str>()
                    .map(|texte| texte.to_string())
                    .or_else(|| cause.downcast_ref::<String>().cloned())
                    .unwrap_or_default();
                Err(format!(
                    "le calcul a planté (bug d'un bot ou du moteur) : {detail}"
                ))
            }
        };
        self.graine = self.graine.min(calcul.simulation.parties.saturating_sub(1));
        self.resultat = Some(Resultat {
            simulation: calcul.simulation,
            duree: calcul.depart.elapsed(),
            bilans,
        });
    }

    fn bilans(&mut self, ui: &mut egui::Ui, intentions: &mut Vec<Intention>) {
        let EcranArene {
            resultat, graine, ..
        } = self;
        let Some(resultat) = resultat else {
            ui.label("Règle la configuration, coche des paires de bots, puis lance l'arène.");
            return;
        };
        let simulation = &resultat.simulation;
        ui.heading(&simulation.description);
        ui.label(format!(
            "{} parties par paire, graines 0 à {}, en {:.1} s.",
            simulation.parties,
            simulation.parties.saturating_sub(1),
            resultat.duree.as_secs_f64()
        ));
        let bilans = match &resultat.bilans {
            Ok(bilans) => bilans,
            Err(message) => {
                ui.colored_label(theme::ERREUR, message);
                return;
            }
        };
        ui.horizontal(|ui| {
            ui.label("Revoir la partie de graine");
            ui.add(egui::DragValue::new(graine).range(0..=simulation.parties.saturating_sub(1)));
        });
        ui.separator();
        egui::ScrollArea::both().show(ui, |ui| {
            ui.monospace(Report::HEADER);
            for (paire, bilan) in simulation.paires.iter().zip(bilans) {
                ui.monospace(bilan.to_string());
                if ui
                    .button(format!("Revoir : {}", nom_paire(*paire)))
                    .clicked()
                {
                    intentions.push(Intention::Revoir {
                        config: simulation.config.clone(),
                        graine: *graine,
                        bots: *paire,
                    });
                }
            }
        });
    }
}

/// « glouton contre aléatoire ».
fn nom_paire([premier, second]: [BotKind; 2]) -> String {
    format!("{} contre {}", premier.name(), second.name())
}
