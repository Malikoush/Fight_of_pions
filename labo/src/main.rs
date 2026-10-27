mod app;
mod arene;
mod intention;
mod panneau;
mod partie;
mod plateau;
mod reglage;
mod theme;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(theme::TAILLE_FENETRE),
        ..Default::default()
    };
    eframe::run_native(
        theme::TITRE,
        options,
        Box::new(|_| {
            let labo = app::Labo::new(partie::config_initiale()).map_err(|e| e.to_string())?;
            Ok(Box::new(labo))
        }),
    )
}
