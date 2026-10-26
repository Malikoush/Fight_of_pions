//! Le damier : dessine la partie et transforme un clic en demande.
//! Les pions sont lus dans `game.board()` à chaque image : un pion n'apparaît
//! que si le moteur l'a accepté, et le plateau ne peut pas contredire le panneau.

use crate::partie::Demande;
use crate::theme;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use fight_of_pions::{Action, Game, Position};

/// Où se trouve le damier à l'écran pour cette image.
struct Geometrie {
    /// Coin haut-gauche de la case A1.
    origine: Pos2,
    /// Côté d'une case, en points.
    cote: f32,
    /// Nombre de cases par côté.
    taille: usize,
}

impl Geometrie {
    fn new(zone: Rect, taille: usize) -> Self {
        let disponible = (zone.width().min(zone.height()) - 2.0 * theme::MARGE_PLATEAU).max(0.0);
        let cote = disponible / taille as f32;
        let origine = zone.center() - Vec2::splat(cote * taille as f32 / 2.0);
        Self { origine, cote, taille }
    }

    /// La seule conversion Position → écran.
    fn case_vers_rect(&self, pos: Position) -> Rect {
        let coin = self.origine + Vec2::new(pos.col as f32, pos.row as f32) * self.cote;
        Rect::from_min_size(coin, Vec2::splat(self.cote))
    }

    /// La seule conversion écran → Position. `None` hors du damier.
    fn point_vers_case(&self, point: Pos2) -> Option<Position> {
        let relatif = (point - self.origine) / self.cote;
        if relatif.x < 0.0 || relatif.y < 0.0 {
            return None;
        }
        let pos = Position {
            row: relatif.y as usize,
            col: relatif.x as usize,
        };
        (pos.row < self.taille && pos.col < self.taille).then_some(pos)
    }
}

pub fn show(ui: &mut egui::Ui, game: &Game, demandes: &mut Vec<Demande>) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let geo = Geometrie::new(response.rect, game.board().size());
        let survol = response.hover_pos().and_then(|p| geo.point_vers_case(p));

        for row in 0..geo.taille {
            for col in 0..geo.taille {
                let pos = Position { row, col };
                let rect = geo.case_vers_rect(pos);
                let fond = if survol == Some(pos) {
                    theme::CASE_SURVOL
                } else if (row + col) % 2 == 0 {
                    theme::CASE_CLAIRE
                } else {
                    theme::CASE_FONCEE
                };
                painter.rect_filled(rect, 0.0, fond);

                if let Some(player) = game.board().owner(pos) {
                    painter.circle(
                        rect.center(),
                        geo.cote * theme::RAYON_PION,
                        theme::couleur_joueur(player),
                        Stroke::new(theme::EPAISSEUR_CONTOUR_PION, theme::CONTOUR_PION),
                    );
                }
            }
        }

        etiquettes(&painter, &geo, ui.visuals().text_color());

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos().and_then(|p| geo.point_vers_case(p))
        {
            demandes.push(Demande::Agir(Action::Place(pos)));
        }
    });
}

/// Lettres au-dessus, numéros à gauche. Le texte vient du `Display` de `Position`
/// (« C3 » pour i = 2), pour ne pas dupliquer la façon de nommer les cases.
fn etiquettes(painter: &egui::Painter, geo: &Geometrie, couleur: Color32) {
    let police = FontId::proportional(theme::TAILLE_ETIQUETTE);
    let decalage = theme::MARGE_PLATEAU / 2.0;
    for i in 0..geo.taille {
        let nom = Position { row: i, col: i }.to_string();
        let (lettre, numero) = nom.split_at(1);

        let haut = geo.case_vers_rect(Position { row: 0, col: i }).center_top();
        painter.text(haut - Vec2::new(0.0, decalage), Align2::CENTER_CENTER, lettre, police.clone(), couleur);

        let gauche = geo.case_vers_rect(Position { row: i, col: 0 }).left_center();
        painter.text(gauche - Vec2::new(decalage, 0.0), Align2::CENTER_CENTER, numero, police.clone(), couleur);
    }
}
