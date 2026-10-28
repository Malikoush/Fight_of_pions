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
/// La limite proposée quand on coche « Poses par tour : limitées ».
pub const POSES_PAR_TOUR_PROPOSEES: u32 = 1;
/// Hauteur maximale du journal, en points : au-delà, il défile.
pub const HAUTEUR_JOURNAL: f32 = 220.0;

/// Les deux écrans du labo.
pub const ECRAN_PARTIE: &str = "Partie";
pub const ECRAN_ARENE: &str = "Arène";
/// Ce qui contrôle un joueur quand ce n'est pas un bot.
pub const HUMAIN: &str = "humain";
/// Attente entre deux coups d'un bot en lecture automatique, en millisecondes : bornes
/// et valeur proposée.
pub const ATTENTE_BOT_MIN_MS: u64 = 0;
pub const ATTENTE_BOT_DEFAUT_MS: u64 = 400;
pub const ATTENTE_BOT_MAX_MS: u64 = 2000;
/// Parties par paire de bots dans l'écran Arène : bornes et valeur proposée. Moteur et
/// bots sont optimisés même en debug : 50 parties par paire prennent ~15 s par
/// configuration, et donnent des fourchettes de ±13 points au plus.
pub const PARTIES_ARENE_MIN: u64 = 1;
pub const PARTIES_ARENE_DEFAUT: u64 = 50;
pub const PARTIES_ARENE_MAX: u64 = 500;
/// Toutes les combien de millisecondes l'écran Arène regarde si le calcul est fini.
pub const INTERVALLE_CALCUL_MS: u64 = 200;

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
/// Anneau sur les cases qu'on peut choisir pendant une attaque (lanceur ou cible).
pub const ANNEAU_CHOIX: Color32 = Color32::from_rgb(211, 47, 47);
/// Anneau sur le pion lanceur déjà choisi.
pub const ANNEAU_LANCEUR: Color32 = Color32::from_rgb(255, 160, 0);
/// Point au centre d'un pion posé ce tour, quand le mal du débarquement l'empêche
/// d'attaquer, et son rayon en fraction du côté d'une case.
pub const MARQUE_NOUVEAU: Color32 = Color32::from_rgb(0, 150, 136);
pub const RAYON_MARQUE_NOUVEAU: f32 = 0.1;
/// Sa légende, sous l'état de la partie : elle nomme la couleur ci-dessus.
pub const LEGENDE_MARQUE_NOUVEAU: &str =
    "Point turquoise : pion posé ce tour, il ne peut pas encore attaquer.";
/// Rayon de ces anneaux, en fraction du côté d'une case, et leur épaisseur en points.
pub const RAYON_ANNEAU: f32 = 0.46;
pub const EPAISSEUR_ANNEAU: f32 = 3.0;

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
