//! Le damier : dessine la partie et signale la case cliquée.
//! Les pions sont lus dans `game.board()` à chaque image : un pion n'apparaît
//! que si le moteur l'a accepté, et le plateau ne peut pas contredire le panneau.
//! Les cases à marquer viennent de `legal_actions` : le plateau ne connaît aucune règle.

use crate::intention::Intention;
use crate::theme;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use fight_of_pions::{Action, Decision, EffectKind, Game, Player, Position};

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
        Self {
            origine,
            cote,
            taille,
        }
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

/// Dessine `game` : la vraie partie, ou sa copie pendant le choix d'une attaque.
pub fn show(ui: &mut egui::Ui, game: &Game, intentions: &mut Vec<Intention>) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let geo = Geometrie::new(response.rect, game.board().size());
        let survol = response.hover_pos().and_then(|p| geo.point_vers_case(p));
        let legales = game.legal_actions();
        let posables = |pos| legales.contains(&Action::Place(pos));
        let choisissables = |pos| legales.contains(&Action::Choose(pos));
        let lanceur = match game.pending_decision() {
            Some(Decision::ChooseTarget { launcher, .. }) => Some(launcher),
            _ => None,
        };

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
                painter.rect_filled(rect, theme::ARRONDI_CASE, fond);
                if game.board().tomb(pos).is_some() {
                    tombe(&painter, &geo, rect);
                }
                if posables(pos) {
                    painter.circle_filled(
                        rect.center(),
                        geo.cote * theme::RAYON_MARQUE_JOUABLE,
                        theme::MARQUE_JOUABLE,
                    );
                }

                if let Some(player) = game.board().owner(pos) {
                    painter.circle(
                        rect.center(),
                        geo.cote * theme::RAYON_PION,
                        theme::couleur_joueur(player),
                        Stroke::new(theme::EPAISSEUR_CONTOUR_PION, theme::CONTOUR_PION),
                    );
                    if game.is_summoning_sick(pos) {
                        painter.circle_filled(
                            rect.center(),
                            geo.cote * theme::RAYON_MARQUE_NOUVEAU,
                            theme::MARQUE_NOUVEAU,
                        );
                    }
                    statuts(&painter, &geo, game, pos);
                }

                let anneau = if lanceur == Some(pos) {
                    Some(theme::ANNEAU_LANCEUR)
                } else if choisissables(pos) {
                    Some(theme::ANNEAU_CHOIX)
                } else {
                    None
                };
                if let Some(couleur) = anneau {
                    painter.circle_stroke(
                        rect.center(),
                        geo.cote * theme::RAYON_ANNEAU,
                        Stroke::new(theme::EPAISSEUR_ANNEAU, couleur),
                    );
                }
            }
        }

        etiquettes(&painter, &geo, ui.visuals().text_color());

        if response.clicked()
            && let Some(pos) = response
                .interact_pointer_pos()
                .and_then(|p| geo.point_vers_case(p))
        {
            intentions.push(Intention::ChoisirCase(pos));
        }
    });
}

/// Une pierre tombale dans le bas de la case : la tombe d'un pion tué, que le Médecin peut
/// ressusciter. Elle est dessinée sous le pion, qui peut occuper la case.
fn tombe(painter: &egui::Painter, geo: &Geometrie, rect: Rect) {
    let taille = Vec2::new(
        geo.cote * theme::LARGEUR_TOMBE,
        geo.cote * theme::HAUTEUR_TOMBE,
    );
    let bas = rect.center_bottom() - Vec2::new(0.0, geo.cote * theme::MARGE_TOMBE);
    let pierre = Rect::from_min_size(bas - Vec2::new(taille.x / 2.0, taille.y), taille);
    painter.rect_filled(pierre, theme::ARRONDI_TOMBE, theme::COULEUR_TOMBE);
}

/// Les statuts du pion de cette case, lus dans le moteur : une croix s'il est immobilisé,
/// un anneau s'il est protégé.
fn statuts(painter: &egui::Painter, geo: &Geometrie, game: &Game, pos: Position) {
    let rect = geo.case_vers_rect(pos);
    let centre = rect.center();
    if game.has_effect(pos, EffectKind::Immobilized) {
        let demi = geo.cote * theme::DEMI_CROIX_IMMOBILISE;
        let trait_croix = Stroke::new(theme::EPAISSEUR_CROIX_IMMOBILISE, theme::MARQUE_IMMOBILISE);
        painter.line_segment(
            [centre - Vec2::splat(demi), centre + Vec2::splat(demi)],
            trait_croix,
        );
        painter.line_segment(
            [
                centre + Vec2::new(-demi, demi),
                centre + Vec2::new(demi, -demi),
            ],
            trait_croix,
        );
    }
    if game.has_effect(pos, EffectKind::Protected) {
        painter.circle_stroke(
            centre,
            geo.cote * theme::RAYON_ANNEAU_PROTEGE,
            Stroke::new(theme::EPAISSEUR_ANNEAU, theme::ANNEAU_PROTEGE),
        );
    }
    let reine = [Player::One, Player::Two]
        .into_iter()
        .any(|joueur| game.queen_of(joueur) == Some(pos));
    if reine {
        let coin =
            centre + Vec2::splat(geo.cote * theme::DECALAGE_COURONNE) * Vec2::new(-1.0, -1.0);
        painter.circle_filled(
            coin,
            geo.cote * theme::RAYON_COURONNE,
            theme::COULEUR_COURONNE,
        );
        let gardes = game
            .board()
            .get(pos)
            .and_then(|case| case.pion())
            .map_or(0, |pion| game.guards_of(pion.id()));
        if gardes > 0 {
            painter.text(
                coin,
                Align2::CENTER_CENTER,
                gardes.to_string(),
                FontId::proportional(geo.cote * theme::TAILLE_GARDES),
                theme::TEXTE_GARDES,
            );
        }
    }
    if game.pact_friend(pos).is_some() {
        painter.circle_filled(
            centre + Vec2::splat(geo.cote * theme::DECALAGE_MARQUE_PACTE) * Vec2::new(1.0, -1.0),
            geo.cote * theme::RAYON_MARQUE_PACTE,
            theme::MARQUE_PACTE,
        );
    }
}

/// Lettres au-dessus, numéros à gauche. La lettre vient du moteur
/// (`Position::column_letter`), pour ne pas dupliquer la façon de nommer les cases.
fn etiquettes(painter: &egui::Painter, geo: &Geometrie, couleur: Color32) {
    let police = FontId::proportional(theme::TAILLE_ETIQUETTE);
    let decalage = theme::MARGE_PLATEAU / 2.0;
    for i in 0..geo.taille {
        let lettre = Position::column_letter(i).unwrap_or('?').to_string();
        let numero = (i + 1).to_string();

        let haut = geo.case_vers_rect(Position { row: 0, col: i }).center_top();
        painter.text(
            haut - Vec2::new(0.0, decalage),
            Align2::CENTER_CENTER,
            lettre,
            police.clone(),
            couleur,
        );

        let gauche = geo
            .case_vers_rect(Position { row: i, col: 0 })
            .left_center();
        painter.text(
            gauche - Vec2::new(decalage, 0.0),
            Align2::CENTER_CENTER,
            numero,
            police.clone(),
            couleur,
        );
    }
}
