//! Tout ce qui se règle à l'œil : couleurs, tailles, textes.
//! Aucun autre fichier du labo ne contient de nombre en dur.

use eframe::egui::Color32;
use fight_of_pions::Player;

pub const TITRE: &str = "Labo Fight of Pions";
pub const TAILLE_FENETRE: [f32; 2] = [1100.0, 760.0];
pub const LARGEUR_PANNEAU: f32 = 300.0;
pub const LARGEUR_SAISIE: f32 = 60.0;
pub const INDICE_SAISIE: &str = "C5";
/// Bornes et valeur proposée pour le nombre de « coups au hasard ».
pub const COUPS_AU_HASARD_MIN: u32 = 1;
pub const COUPS_AU_HASARD_DEFAUT: u32 = 10;
pub const COUPS_AU_HASARD_MAX: u32 = 500;
/// Hauteur, en lignes, de la zone où l'on colle une partie à charger.
pub const LIGNES_ZONE_CHARGEMENT: usize = 4;
pub const INDICE_CHARGEMENT: &str = "Coller ici une partie copiée";

pub const CASE_CLAIRE: Color32 = Color32::from_rgb(252, 228, 236);
pub const CASE_FONCEE: Color32 = Color32::from_rgb(244, 143, 177);
pub const CASE_SURVOL: Color32 = Color32::from_rgb(255, 241, 118);
pub const ERREUR: Color32 = Color32::from_rgb(211, 47, 47);

/// Place laissée autour du damier pour les lettres et les numéros.
pub const MARGE_PLATEAU: f32 = 28.0;
pub const TAILLE_ETIQUETTE: f32 = 14.0;
/// Arrondi des coins d'une case, en points (0 = coins carrés).
pub const ARRONDI_CASE: f32 = 0.0;
/// Rayon d'un pion, en fraction du côté d'une case.
pub const RAYON_PION: f32 = 0.38;
pub const EPAISSEUR_CONTOUR_PION: f32 = 1.5;
pub const CONTOUR_PION: Color32 = Color32::from_rgb(60, 30, 50);
/// Petit point sur les cases où le joueur courant peut poser (lu dans `legal_actions`).
pub const MARQUE_JOUABLE: Color32 = Color32::from_rgb(194, 24, 91);
/// Rayon de ce point, en fraction du côté d'une case.
pub const RAYON_MARQUE_JOUABLE: f32 = 0.07;

pub fn couleur_joueur(player: Player) -> Color32 {
    match player {
        Player::One => Color32::from_rgb(106, 27, 154),
        Player::Two => Color32::from_rgb(255, 255, 255),
    }
}

pub fn nom_joueur(player: Player) -> &'static str {
    match player {
        Player::One => "Joueur 1 (violet)",
        Player::Two => "Joueur 2 (blanc)",
    }
}
