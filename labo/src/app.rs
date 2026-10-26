//! Assemble les morceaux du labo et fait circuler les messages à chaque image :
//! l'affichage produit des `Demande`, la partie les traite, le panneau reçoit les `Nouvelle`.

use crate::panneau::Panneau;
use crate::partie::Partie;
use crate::plateau;
use eframe::egui;
use fight_of_pions::{GameConfig, GameError};

pub struct Labo {
    partie: Partie,
    panneau: Panneau,
}

impl Labo {
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        Ok(Self {
            partie: Partie::new(config)?,
            panneau: Panneau::default(),
        })
    }
}

impl eframe::App for Labo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut demandes = Vec::new();
        // Le panneau latéral doit être montré avant le plateau, qui prend la place restante.
        self.panneau.show(ui, self.partie.game(), &mut demandes);
        plateau::show(ui, self.partie.game(), &mut demandes);

        if demandes.is_empty() {
            return;
        }
        for demande in demandes {
            for nouvelle in self.partie.traiter(demande) {
                self.panneau.recevoir(&nouvelle);
            }
        }
        // L'image vient d'être dessinée avec l'ancien état : on en redemande une.
        ui.ctx().request_repaint();
    }
}
