//! Assemble les morceaux du labo et fait circuler les messages à chaque image :
//! l'affichage produit des `Intention`, l'application en fait des `Demande` à la partie
//! ou des gestes de sélection, et le panneau reçoit les `Nouvelle`.
//!
//! Choisir une attaque se fait sur une copie de la partie (`Partie::apercu`) : seule la
//! suite complète (attaque, lanceur, cibles) est jouée, d'un bloc. Annuler est gratuit.

use crate::intention::Intention;
use crate::panneau::Panneau;
use crate::partie::{Demande, Nouvelle, Partie, en_plein_choix};
use crate::plateau;
use eframe::egui;
use fight_of_pions::{Action, GameConfig, GameError, Position};

pub struct Labo {
    partie: Partie,
    panneau: Panneau,
    /// L'attaque en cours de choix dans l'interface, pas encore jouée.
    selection: Option<Vec<Action>>,
}

impl Labo {
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        Ok(Self {
            partie: Partie::new(config)?,
            panneau: Panneau::new(),
            selection: None,
        })
    }

    /// Transmet une demande à la partie et ses nouvelles au panneau.
    fn demander(&mut self, demande: Demande) {
        self.selection = None;
        for nouvelle in self.partie.traiter(demande) {
            self.panneau.recevoir(&nouvelle);
        }
    }

    /// Une case cliquée : pose hors attaque, ou prochain choix de l'attaque en cours.
    fn choisir_case(&mut self, pos: Position) {
        let Some(mut suite) = self.selection.take() else {
            // Sans sélection : une pose, ou un choix si la vraie partie en attend un
            // (partie chargée au milieu d'une attaque, par exemple).
            let action = if en_plein_choix(self.partie.game()) {
                Action::Choose(pos)
            } else {
                Action::Place(pos)
            };
            self.demander(Demande::Agir(action));
            return;
        };
        suite.push(Action::Choose(pos));
        match self.partie.apercu(&suite) {
            // Le choix est accepté et l'attaque attend encore un choix : on continue.
            Ok(apercu) if en_plein_choix(&apercu) => self.selection = Some(suite),
            // Le dernier choix est fait : l'attaque entière est jouée pour de vrai.
            Ok(_) => self.demander(Demande::Jouer(suite)),
            // Choix refusé : on l'explique, et la sélection reste où elle était.
            Err(e) => {
                suite.pop();
                self.selection = Some(suite);
                self.panneau.recevoir(&Nouvelle::Refus(e));
            }
        }
    }
}

impl eframe::App for Labo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Pendant le choix d'une attaque, on montre la copie : lanceur et cibles marqués.
        let apercu = self
            .selection
            .as_ref()
            .and_then(|suite| self.partie.apercu(suite).ok());
        let affichee = apercu.as_ref().unwrap_or(self.partie.game());

        let mut intentions = Vec::new();
        // Le panneau latéral doit être montré avant le plateau, qui prend la place restante.
        self.panneau.show(
            ui,
            self.partie.game(),
            self.partie.record(),
            self.selection.is_some(),
            &mut intentions,
        );
        plateau::show(ui, affichee, &mut intentions);

        if intentions.is_empty() {
            return;
        }
        for intention in intentions {
            match intention {
                Intention::Demande(demande) => self.demander(demande),
                Intention::CommencerAttaque(slot) => {
                    self.selection = Some(vec![Action::UseAttack(slot)]);
                }
                Intention::ChoisirCase(pos) => self.choisir_case(pos),
                Intention::AnnulerSelection => self.selection = None,
            }
        }
        // L'image vient d'être dessinée avec l'ancien état : on en redemande une.
        ui.ctx().request_repaint();
    }
}
