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
/// Une carte de la cache adverse, pas encore jouée.
pub const CARTE_SECRETE: &str = "carte secrète (cache)";
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
/// Une case de mur, que seul le Coup du taureau creuse.
pub const CASE_MUR: Color32 = Color32::from_rgb(141, 110, 99);
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
/// Croix sur un pion immobilisé : sa couleur, sa demi-largeur en fraction du côté d'une
/// case, et son épaisseur en points.
pub const MARQUE_IMMOBILISE: Color32 = Color32::from_rgb(25, 118, 210);
pub const DEMI_CROIX_IMMOBILISE: f32 = 0.22;
pub const EPAISSEUR_CROIX_IMMOBILISE: f32 = 3.0;
/// Anneau autour d'un pion protégé, entre le pion et l'anneau de choix.
pub const ANNEAU_PROTEGE: Color32 = Color32::from_rgb(56, 142, 60);
pub const RAYON_ANNEAU_PROTEGE: f32 = 0.42;
/// Point en haut à droite d'un pion lié par un pacte (Meilleurs amis) : sa couleur, son
/// rayon et son décalage depuis le centre, en fraction du côté d'une case.
pub const MARQUE_PACTE: Color32 = Color32::from_rgb(229, 57, 53);
pub const RAYON_MARQUE_PACTE: f32 = 0.09;
pub const DECALAGE_MARQUE_PACTE: f32 = 0.3;
/// La tombe d'un pion tué (le Médecin) : une pierre dans le bas de la case. Sa couleur,
/// sa taille et sa marge depuis le bas, en fraction du côté d'une case, et son arrondi.
pub const COULEUR_TOMBE: Color32 = Color32::from_rgb(117, 117, 117);
pub const LARGEUR_TOMBE: f32 = 0.3;
pub const HAUTEUR_TOMBE: f32 = 0.22;
pub const MARGE_TOMBE: f32 = 0.06;
pub const ARRONDI_TOMBE: f32 = 3.0;
/// La couronne de la reine : un point doré en haut à gauche du pion (rayon et décalage en
/// fraction du côté d'une case), avec le nombre de ses gardes dessus.
pub const COULEUR_COURONNE: Color32 = Color32::from_rgb(255, 193, 7);
pub const RAYON_COURONNE: f32 = 0.13;
pub const DECALAGE_COURONNE: f32 = 0.28;
pub const TAILLE_GARDES: f32 = 0.2;
pub const TEXTE_GARDES: Color32 = Color32::from_rgb(60, 30, 50);
/// La légende des effets continus, sous l'état de la partie : elle nomme les couleurs
/// ci-dessus.
pub const LEGENDE_STATUTS: &str = "Croix bleue : pion immobilisé, il ne peut pas attaquer. \
     Anneau vert : pion protégé, il ne peut pas être tué. \
     Point rouge : pion lié par un pacte, son ami le vengera. \
     Pierre grise : tombe d'un pion à l'infirmerie, le Médecin peut le ressusciter. \
     Point doré : la reine, avec le nombre de ses gardes.";

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
