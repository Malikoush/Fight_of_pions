mod app;
mod panneau;
mod partie;
mod plateau;
mod theme;

use eframe::egui;
use fight_of_pions::GameConfig;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(theme::TAILLE_FENETRE),
        ..Default::default()
    };
    eframe::run_native(theme::TITRE, options, Box::new(|_| {
        let labo = app::Labo::new(GameConfig::default()).map_err(|e| e.to_string())?;
        Ok(Box::new(labo))
    }))
}
