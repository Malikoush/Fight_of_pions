//! Le réglage d'une configuration de partie, partagé par les deux écrans : la prochaine
//! partie (écran Partie) et la configuration simulée (écran Arène). On part d'une
//! variante ou de la partie en cours, puis on retouche chaque réglage. Le labo ne vérifie
//! rien lui-même : c'est `GameConfig::validate` qui dit si la configuration se lance.

use crate::partie;
use crate::theme;
use bots::{Variant, variants};
use eframe::egui;
use fight_of_pions::{Attack, GameConfig, GameError, Player};

/// Le nom du départ quand on reprend la configuration de la partie en cours.
const PARTIE_EN_COURS: &str = "partie en cours";

pub struct ReglageDePartie {
    /// Ce que règle ce réglage, en tête de sa section ; sert aussi à distinguer ses
    /// widgets de ceux de l'autre écran.
    intitule: &'static str,
    variantes: Vec<Variant>,
    /// La dernière variante choisie dans la liste.
    variante: usize,
    /// La configuration dont on est parti, et son nom : une variante ou la partie en cours.
    depart: GameConfig,
    nom_du_depart: &'static str,
    config: GameConfig,
}

impl ReglageDePartie {
    pub fn new(intitule: &'static str) -> ReglageDePartie {
        let mut reglage = ReglageDePartie {
            intitule,
            variantes: variants(),
            variante: 0,
            depart: partie::config_initiale(),
            nom_du_depart: PARTIE_EN_COURS,
            config: partie::config_initiale(),
        };
        reglage.partir_de(0);
        reglage
    }

    /// La configuration à lancer, ou ce que le moteur lui reproche.
    pub fn config(&self) -> Result<&GameConfig, GameError> {
        self.config.validate().map(|()| &self.config)
    }

    /// Repart de cette variante : les retouches sont perdues.
    fn partir_de(&mut self, indice: usize) {
        if let Some(variante) = self.variantes.get(indice) {
            self.variante = indice;
            self.depart = variante.config.clone();
            self.nom_du_depart = variante.name;
            self.config = variante.config.clone();
        }
    }

    /// Repart de la configuration de la partie en cours : après un chargement, ou pour
    /// relancer une partie revue de l'arène avec une autre graine.
    pub fn reprendre(&mut self, en_cours: &GameConfig) {
        self.depart = en_cours.clone();
        self.nom_du_depart = PARTIE_EN_COURS;
        self.config = en_cours.clone();
    }

    /// Vrai si la configuration n'est plus celle dont on est parti.
    fn retouchee(&self) -> bool {
        self.config != self.depart
    }

    /// D'où vient la configuration, et si elle a été retouchée.
    pub fn description(&self) -> String {
        let retouche = if self.retouchee() { ", retouchée" } else { "" };
        format!("{}{retouche}", self.nom_du_depart)
    }

    pub fn show(&mut self, ui: &mut egui::Ui, en_cours: &GameConfig) {
        egui::CollapsingHeader::new(format!("{} : {}", self.intitule, self.description()))
            .id_salt(self.intitule)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Partir de");
                    if choisir_variante(ui, self.intitule, &self.variantes, &mut self.variante) {
                        self.partir_de(self.variante);
                    }
                });
                if ui.button("Reprendre la partie en cours").clicked() {
                    self.reprendre(en_cours);
                }
                self.reglages(ui);
                self.catalogue(ui);
                self.arsenaux(ui);
                match self.config() {
                    Ok(_) => ui.label("Configuration valide"),
                    Err(erreur) => ui.colored_label(theme::ERREUR, erreur.to_string()),
                };
            });
    }

    fn reglages(&mut self, ui: &mut egui::Ui) {
        let config = &mut self.config;
        egui::Grid::new((self.intitule, "reglages")).show(ui, |ui| {
            ligne(ui, "Taille du plateau", &mut config.board_size);
            ligne(ui, "Pions à aligner", &mut config.align_to_win);
            ligne(ui, "Pions par joueur", &mut config.pions_per_player);
            ligne(
                ui,
                "Actions du premier tour",
                &mut config.first_turn_actions,
            );
            ligne(ui, "Actions par tour", &mut config.actions_per_turn);
            ligne(
                ui,
                "Actions gardées au plus",
                &mut config.max_banked_actions,
            );
            ligne(
                ui,
                "Banque de départ J1",
                &mut config.starting_banked_actions[0],
            );
            ligne(
                ui,
                "Banque de départ J2",
                &mut config.starting_banked_actions[1],
            );
            ligne(ui, "Coût d'une pose", &mut config.place_cost);
            poses_par_tour(ui, &mut config.max_placements_per_turn);
            ui.label("Mal du débarquement");
            ui.checkbox(&mut config.summoning_sickness, "");
            ui.end_row();
            ligne(ui, "Limite de tours", &mut config.max_turns);
        });
    }

    /// Le coût et la recharge de chaque attaque du catalogue.
    fn catalogue(&mut self, ui: &mut egui::Ui) {
        ui.label("Catalogue");
        egui::Grid::new((self.intitule, "catalogue")).show(ui, |ui| {
            ui.label("");
            ui.label("coût");
            ui.label("recharge");
            ui.end_row();
            for entree in &mut self.config.attack_catalog {
                ui.label(entree.attack.to_string());
                ui.add(egui::DragValue::new(&mut entree.stats.cost));
                ui.add(egui::DragValue::new(&mut entree.stats.cooldown));
                ui.end_row();
            }
        });
    }

    /// Les attaques de chaque joueur au début de la partie, parmi celles du catalogue.
    fn arsenaux(&mut self, ui: &mut egui::Ui) {
        let catalogue = self.config.catalog_attacks();
        for joueur in [Player::One, Player::Two] {
            ui.label(format!("Arsenal du {}", theme::nom_joueur(joueur)));
            let arsenal = &mut self.config.starting_attacks[joueur.index()];
            for &attaque in &catalogue {
                let mut presente = arsenal.contains(&attaque);
                if ui.checkbox(&mut presente, attaque.to_string()).changed() {
                    cocher(arsenal, &catalogue, attaque, presente);
                }
            }
        }
    }
}

/// La liste des variantes, où l'on choisit celle d'indice `choisie` ; `id` distingue
/// celle de chaque écran. Vrai si l'on vient de cliquer une variante, même celle déjà
/// choisie.
fn choisir_variante(
    ui: &mut egui::Ui,
    id: &str,
    variantes: &[Variant],
    choisie: &mut usize,
) -> bool {
    let nom = variantes.get(*choisie).map_or("", |variante| variante.name);
    let mut cliquee = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(nom)
        .show_ui(ui, |ui| {
            for (indice, variante) in variantes.iter().enumerate() {
                cliquee |= ui
                    .selectable_value(choisie, indice, variante.name)
                    .clicked();
            }
        });
    cliquee
}

/// Ajoute ou retire cette attaque de l'arsenal, dans l'ordre du catalogue : décocher puis
/// recocher rend le même arsenal, aux mêmes emplacements.
fn cocher(arsenal: &mut Vec<Attack>, catalogue: &[Attack], attaque: Attack, presente: bool) {
    *arsenal = catalogue
        .iter()
        .copied()
        .filter(|&autre| {
            if autre == attaque {
                presente
            } else {
                arsenal.contains(&autre)
            }
        })
        .collect();
}

/// Les poses par tour : sans limite (seules les actions comptent), ou au plus tant.
fn poses_par_tour(ui: &mut egui::Ui, limite: &mut Option<u32>) {
    ui.label("Poses par tour");
    ui.horizontal(|ui| {
        let mut limitees = limite.is_some();
        if ui.checkbox(&mut limitees, "limitées").changed() {
            *limite = limitees.then_some(theme::POSES_PAR_TOUR_PROPOSEES);
        }
        if let Some(poses) = limite {
            ui.add(egui::DragValue::new(poses));
        }
    });
    ui.end_row();
}

/// Un réglage numérique : son nom, et sa valeur qu'on fait glisser ou qu'on tape.
fn ligne<Nombre: egui::emath::Numeric>(ui: &mut egui::Ui, nom: &str, valeur: &mut Nombre) {
    ui.label(nom);
    ui.add(egui::DragValue::new(valeur));
    ui.end_row();
}

#[cfg(test)]
mod tests {
    use super::*;
    use fight_of_pions::ConfigError;

    #[test]
    fn on_part_de_la_regle_actuelle_sans_retouche() {
        let reglage = ReglageDePartie::new("essai");
        assert_eq!(reglage.config(), Ok(&partie::config_initiale()));
        assert!(!reglage.retouchee());
        assert_eq!(reglage.nom_du_depart, reglage.variantes[0].name);
    }

    #[test]
    fn une_configuration_incoherente_ne_se_lance_pas() {
        let mut reglage = ReglageDePartie::new("essai");
        reglage.config.align_to_win = reglage.config.board_size + 1;
        assert!(reglage.retouchee());
        assert_eq!(
            reglage.config(),
            Err(GameError::InvalidConfig(
                ConfigError::AlignmentLongerThanBoard
            ))
        );
    }

    #[test]
    fn repartir_d_une_variante_efface_les_retouches() {
        let mut reglage = ReglageDePartie::new("essai");
        reglage.config.max_turns += 1;
        reglage.partir_de(1);
        assert!(!reglage.retouchee());
        assert_eq!(reglage.config(), Ok(&reglage.variantes[1].config));
    }

    #[test]
    fn reprendre_la_partie_en_cours_part_de_sa_configuration() {
        let mut reglage = ReglageDePartie::new("essai");
        let en_cours = GameConfig {
            board_size: 7,
            align_to_win: 4,
            ..partie::config_initiale()
        };
        reglage.reprendre(&en_cours);
        assert!(!reglage.retouchee());
        assert_eq!(reglage.nom_du_depart, PARTIE_EN_COURS);
        assert_eq!(reglage.config(), Ok(&en_cours));
    }

    #[test]
    fn decocher_puis_recocher_rend_le_meme_arsenal() {
        let mut reglage = ReglageDePartie::new("essai");
        let catalogue = reglage.config.catalog_attacks();
        let arsenal = &mut reglage.config.starting_attacks[0];
        let premiere = catalogue[0];
        cocher(arsenal, &catalogue, premiere, false);
        assert!(!arsenal.contains(&premiere));
        cocher(arsenal, &catalogue, premiere, true);
        assert_eq!(*arsenal, catalogue);
        assert!(!reglage.retouchee());
    }
}
