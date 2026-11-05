class_name Look
extends RefCounted
## L'apparence du client : couleurs, tailles, marges, durées et proportions du plateau,
## définies une seule fois (comme `theme.rs` dans le labo). Aucun autre fichier n'écrit un
## nombre d'apparence : il lit ces constantes, ou le thème construit à partir d'elles
## (`ThemeBuilder`). Les nombres du jeu, eux, viennent de la vue du serveur.
##
## Les tailles d'interface sont en points logiques : la fenêtre de base fait 390 × 844
## (réglages du projet), la taille d'un téléphone en portrait.

# --- Couleurs, reprises des éléments fournis (design/elements) ---

## Le trait de contour de tous les éléments dessinés.
const INK := Color("1a120a")
## L'or : action principale, ce qui est à toi, ce qui est permis.
const GOLD := Color("f2b705")
const GOLD_SHADE := Color("d9a100")
const GOLD_DEEP := Color("8a6400")
## L'éclaircissement d'un bouton survolé.
const HOVER_LIGHTEN := 0.12
## Le vert sapin : fonds et panneaux, du plus profond au plus clair.
const FOREST_DEEP := Color("0a1c18")
const FOREST := Color("12302a")
const FOREST_PANEL := Color("1a4238")
const FOREST_RAISED := Color("1f5246")
## Le crème : texte clair, cases du plateau.
const CREAM := Color("f6e7c8")
## Le texte secondaire, sur fond sapin.
const MUTED := Color("9fb3ac")
## Un bouton désactivé.
const DISABLED_FILL := Color("3b5a51")
const DISABLED_TEXT := Color("8fa39c")

## Joueur 1 : le palet rond rouge, à bandes blanches.
const J1_TOP := Color("e8402a")
const J1_SIDE := Color("b52a18")
const J1_STRIPE := Color("f4f1ea")
const J1_CORE := GOLD
## Joueur 2 : le palet octogonal blanc, à motif bleu.
const J2_TOP := Color("f4f1ea")
const J2_SIDE := Color("cfc8b8")
const J2_MOTIF := Color("2457d6")

## Les cases du plateau (dessus, tranche gauche, tranche droite, arête).
const CASE_TOP := Color("f6e7c8")
const CASE_ALT_TOP := Color("ebd7ac")
const CASE_SIDE_A := Color("c9a96b")
const CASE_SIDE_B := Color("9c743a")
const CASE_EDGE := Color("8a6a33")
## Le mur.
const WALL_TOP := Color("1f5246")
const WALL_SIDE_A := Color("163b32")
const WALL_SIDE_B := Color("0f2b24")
const WALL_EDGE := Color("0a1c18")
## Une case de mur creusée par le Coup du taureau.
const DUG_TOP := Color("dcc596")
const DUG_SIDE_A := Color("b89562")
const DUG_SIDE_B := Color("9c7a45")
## La pierre d'une tombe.
const TOMB_TOP := Color("8e9aa0")
const TOMB_SIDE_A := Color("8e9aa0")
const TOMB_SIDE_B := Color("5e6b71")
## L'assombrissement de la tranche droite d'un objet d'une seule couleur.
const SIDE_DARKEN := 0.25

## La marque de la case survolée : un voile d'or et son bord.
const MARK_FILL := Color(GOLD, 0.22)
const MARK_BORDER := Color(GOLD, 0.95)

## Le bord d'un panneau au repos, et celui du joueur qui doit jouer.
const PANEL_BORDER := Color(GOLD, 0.28)
const PANEL_BORDER_ACTIVE := GOLD
## Le panneau du joueur qui attend passe en retrait.
const PANEL_IDLE_ALPHA := 0.72

## Le dernier coup adverse : un cadre en pointillés (marques/dernier.svg), sans voile.
const LAST_MARK := Color(INK, 0.85)
## Une case touchée où rien n'est permis : un voile terne, bref.
const REFUSAL_FILL := Color(MUTED, 0.28)
const REFUSAL_BORDER := Color(MUTED, 0.7)
## Une case proposée pendant le choix d'une carte (marques/permise.svg) : un voile d'or léger
## et un petit losange au centre ; une case déjà choisie (marques/choisie.svg), plus dorée.
const CANDIDATE_FILL := Color(GOLD, 0.16)
const CANDIDATE_DOT := GOLD
const CANDIDATE_DOT_SIZE := 0.18
## L'anneau d'une case proposée, juste autour d'un palet (rayon et demi-épaisseur, en part
## de la case).
const CANDIDATE_RING := 0.44
const CANDIDATE_RING_WIDTH := 0.035
const CHOSEN_FILL := Color(GOLD, 0.34)
const CHOSEN_BORDER := GOLD
## Pendant un choix, un toucher s'aimante à la case proposée la plus proche, jusqu'à cette
## distance (en cases).
const MAGNET_RADIUS := 0.9
## Les anneaux des statuts, sous un pion (effets/anneau-*.svg) : son milieu (`band`), son
## bord (`edge`), et ses traits (`dash`, `dashes` sur le tour ; 0 pour un anneau plein).
## Immobilisé : une chaîne grise ; protégé : un anneau d'or ; pacte : des tirets magenta.
const PACT := Color("a33a7e")
const STATUS_RINGS := {
	"Immobilized": {"band": Color("56636a"), "edge": Color("2b3438"), "dash": Color("b4bfc3"), "dashes": 20.0},
	"Protected": {"band": GOLD, "edge": INK, "dash": GOLD, "dashes": 0.0},
	"Pact": {"band": Color(PACT, 0.0), "edge": Color(PACT, 0.0), "dash": PACT, "dashes": 16.0},
}
## Les éclats des événements : l'or d'une attaque lancée, le rouge d'un pion tué, la
## poussière d'un mur creusé.
const ATTACK_PULSE := GOLD
const KILL_PULSE := Color("ff5a3c")
const DIG_PULSE := Color("c9a96b")
## La menace adverse (maquettes) : l'annonce d'une attaque, la fenêtre de réaction ; et
## Terre brûlée, la même braise sur le panneau du joueur visé.
const THREAT := Color("f0866a")
const SCORCHED := THREAT
## Le palet fantôme (survol, coup envoyé en attente de réponse) : sa transparence.
const GHOST_ALPHA := 0.55

## Le cadre en pointillés du bouton d'action quand on attend l'adversaire (maquettes).
const WAITING_BORDER := Color(MUTED, 0.6)
const DASH_LENGTH := 7.0
const DASH_GAP := 5.0
## Le nombre de traits d'un coin arrondi dessiné.
const CORNER_STEPS := 6

# --- Typographie (tailles en points) ---

const FONT_DISPLAY := 52
const FONT_HEADING := 22
const FONT_BODY := 17
const FONT_BUTTON := 21
const FONT_NUMBER := 32
## Les faits d'un panneau de joueur (infirmerie, jetons…).
const FONT_FACT := 14
## Le graissage de la police gothique des titres (police variable).
const DISPLAY_WEIGHT := 800
## L'espacement des lettres des libellés en capitales (boutons, pastilles).
const CAPS_SPACING := 1
## L'ombre portée des titres.
const TITLE_SHADOW := Vector2(0, 3)

# --- Espacements et formes ---

const GAP_XS := 4
const GAP_S := 8
const GAP_M := 12
const GAP_L := 16
const GAP_XL := 24
const GAP_XXL := 40
## La marge de la page, de chaque côté de l'écran.
const PAGE_MARGIN := 16
const RADIUS_PANEL := 20
const RADIUS_CONTROL := 14
## Assez grand pour qu'une pastille soit arrondie à ses deux bouts.
const RADIUS_PILL := 40
const BORDER := 2
const BORDER_STRONG := 3
## Un toucher qui bouge de moins que ça reste un toucher, pas un glissé.
const TAP_SLOP := 12.0

## Les boutons d'action hexagonaux : hauteur, largeur des pointes, tranche du bas.
const BUTTON_HEIGHT := 60
const BUTTON_POINT := 18.0
const BUTTON_SHADE := 5.0
const BUTTON_PADDING := 28

## Les icônes et le palet des panneaux de joueur.
const ICON_SIZE := 20
const CHIP_SIZE := Vector2(40, 30)
## La toile commune des éléments du plateau (design/elements/LISEZMOI.txt), et la partie
## où se trouve le palet d'un pion.
const ELEMENT_CANVAS := Vector2(128, 144)
const CHIP_REGION := Rect2(20, 58, 90, 62)
## Le losange des points d'action, et les petits losanges de la banque.
const POINT_DIAMOND := 22.0
const BANK_PIP := 12.0
const DIAMOND_OUTLINE := 2.0

## Les cartes : la toile des cadres fournis (design/elements/cartes), et où s'y trouvent le
## ruban du nom, le losange du coût et la fenêtre du dessin.
const CARD_CANVAS := Vector2(220, 306)
const CARD_RIBBON := Rect2(10, 214, 200, 30)
const CARD_COST_CENTER := Vector2(38, 68)
const CARD_COST_SIZE := 44.0
const CARD_WINDOW := Rect2(35, 51, 150, 155)
## La largeur d'une carte de l'arsenal, et l'écart entre deux cartes.
const CARD_WIDTH := 80.0
const CARD_GAP := 8
## Une carte qu'on ne peut pas jouer maintenant est assombrie ; une carte choisie se lève.
const CARD_UNPLAYABLE := Color(0.55, 0.58, 0.56)
const CARD_LIFT := 10.0
const CARD_LIFT_TIME := 0.12
## Le voile d'une carte en recharge, sous le sablier.
const CARD_COOLDOWN_SHADE := Color(FOREST_DEEP, 0.78)
const FONT_CARD_NAME := 12
const FONT_CARD_COST := 17
const FONT_CARD_TAG := 11

## Les badges des pions, par-dessus le plateau : leur taille suit celle d'une case à l'écran
## (en part de sa largeur), entre deux bornes en points ; un losange chiffré est un peu plus
## grand, et son nombre en occupe cette part. Le calque des badges passe sous l'interface.
const BADGE_CELL_SHARE := 0.55
const BADGE_MIN := 18.0
const BADGE_MAX := 30.0
const NUMBER_BADGE_SHARE := 1.15
const BADGE_NUMBER_SHARE := 0.5
const BADGE_GAP := 2
const BADGE_CANVAS_LAYER := 0
## Les badges se posent au bord droit du dessus d'un pion (en part de son rayon).
const BADGE_ANCHOR_SIDE := 0.35
## Le tour adverse :
## - l'annonce d'une carte adverse, en haut de l'écran : la largeur de sa carte, son temps à
##   l'écran, son entrée et sa sortie ;
const ANNOUNCE_CARD_WIDTH := 52.0
const ANNOUNCE_HOLD := 2.4
const ANNOUNCE_FADE := 0.25
## - la cache révélée : le voile, la carte en grand, les rayons derrière elle (nombre, couleur,
##   longueur en part de l'écran, vitesse de rotation en tours par seconde), son entrée ;
const REVEAL_DIM := Color(FOREST_DEEP, 0.93)
const REVEAL_CARD_WIDTH := 180.0
const REVEAL_RAYS := 16
const REVEAL_RAY_COLOR := Color(GOLD, 0.13)
const REVEAL_RAY_LENGTH := 0.75
const REVEAL_RAY_SPEED := 0.02
const REVEAL_IN := 0.3
## - la fenêtre de réaction : la largeur de ses cartes, et les lignes de leur légende.
const REACTION_CARD_WIDTH := 92.0
const CAPTION_LINES := 3
## Le bouton qui joue la réaction, plus large que « PASSER » : il porte le nom de la carte.
const PLAY_BUTTON_STRETCH := 2.0
## Le journal des coups : les lignes affichées (les plus anciennes s'en vont), la largeur de
## sa colonne en paysage, celle de la colonne des tours, et le palet de chaque ligne.
const JOURNAL_SHOWN := 80
const JOURNAL_WIDTH := 300
const JOURNAL_TURN_WIDTH := 32
const JOURNAL_CHIP := Vector2(24, 18)
## Un bouton carré à icône, et le voile d'une page qui couvre l'écran.
const ICON_BUTTON_SIZE := 44
const OVERLAY_FILL := Color(FOREST_DEEP, 0.97)
## Les faits d'un panneau de joueur (infirmerie, jetons…) : la taille de leur icône.
const FACT_ICON_SIZE := 16

## La barre de chargement de l'écran de démarrage.
const PROGRESS_SIZE := Vector2(210, 6)

## La largeur maximale d'une colonne de contenu (accueil, menus) sur un grand écran.
const CONTENT_MAX_WIDTH := 400

## En paysage (PC), la colonne de droite.
const SIDE_COLUMN_WIDTH := 340

# --- Mouvement (secondes) ---

const SCREEN_FADE := 0.2
const PION_MOVE := 0.22
## La pose : le palet tombe, s'écrase un peu à l'arrivée, puis reprend sa forme.
const PION_DROP_TIME := 0.24
const PION_SQUASH := 0.72
const PION_SQUASH_TIME := 0.22
## Ce qui se passe (E3d2), joué dans l'ordre des événements (`BoardTimeline`) :
## - l'éclat d'une case (attaque lancée, pion tué, mur creusé) : un anneau qui s'élargit en
##   s'effaçant ; l'attaque laisse la suite commencer avant la fin de son éclat ;
const PULSE_TIME := 0.45
const ATTACK_BEAT := 0.3
const DIG_BEAT := 0.3
## - un pion tué gonfle (cette part du temps), puis se ratatine en s'élevant de cette
##   hauteur ;
const KILL_TIME := 0.32
const KILL_POP := 1.2
const KILL_POP_SHARE := 0.3
const KILL_RISE := 0.35
## - une tombe sort du sol ;
const TOMB_RISE_TIME := 0.2
## - un saut (Saut du loup, Interchange, pas de la reine…) : sa durée, sa hauteur par case
##   franchie et sa hauteur maximale ; dans un échange, le second saute moins haut ;
const JUMP_TIME := 0.38
const JUMP_ARC_PER_CELL := 0.22
const JUMP_ARC_MAX := 0.9
const SWAP_LOW_ARC := 0.45
## - le tir d'un garde : la balle vole en cloche.
const SHOT_TIME := 0.3
const SHOT_ARC := 0.7
## Ce qui disparaît ou apparaît part d'une échelle presque nulle (une échelle nulle n'a pas
## d'inverse).
const TINY_SCALE := 0.01
## Un pion qui change de taille (devient enfant, ou ne l'est plus).
const STATUS_CHANGE_TIME := 0.25
## Le recadrage de la caméra quand le plateau change (une case creusée).
const REFRAME_TIME := 0.45
## Le voile d'une case refusée s'efface en si peu de temps.
const REFUSAL_TIME := 0.3
## Un pion touché alors que rien n'est permis tremble : l'écart (unités du monde), le nombre
## d'allers-retours et leur durée totale.
const WIGGLE_DISTANCE := 0.06
const WIGGLE_SWINGS := 3
const WIGGLE_TIME := 0.24
## Le bandeau « À TOI » : son entrée, son temps à l'écran, sa sortie, et sa taille de départ.
const TURN_BANNER_IN := 0.18
const TURN_BANNER_HOLD := 0.7
const TURN_BANNER_OUT := 0.25
const TURN_BANNER_START_SCALE := 0.8
## La vibration du téléphone au début de son tour, en millisecondes.
const TURN_VIBRATION_MS := 35
## Un tour de la barre de chargement indéterminée.
const PROGRESS_LOOP := 1.4

# --- Plateau 3D (unités du monde : une case fait 1 de côté) ---

## L'épaisseur d'une case, et celle du mur, plus haut que le plateau.
const CASE_HEIGHT := 0.11
const WALL_HEIGHT := 0.245
## Le trait sur l'arête des cases.
const TILE_EDGE := 0.012
## Ce qui est posé à plat (ombre, marque) flotte de si peu au-dessus de sa case, pour ne
## pas se confondre avec elle.
const FLAT_LIFT := 0.005
## Les pions : rayon et épaisseur ; le palet octogonal est un peu plus large pour peser
## autant que le rond.
const PION_RADIUS := 0.38
const PION_HEIGHT := 0.15
const PION_OCTAGON_SCALE := 1.05
## Le prisme de J2 tourne d'un seizième de tour pour présenter une face à la caméra.
const PION_OCTAGON_TURN := TAU / 16.0
## La hauteur d'où tombe un palet posé, et celle où flotte le palet d'un coup envoyé.
const PION_DROP_HEIGHT := 0.9
const PION_PENDING_LIFT := 0.3
## Le contour noir des pions, en unités du monde.
const PION_OUTLINE := 0.018
## Le trait du bord du dessus (en part du rayon), et celui du haut et du bas de la tranche.
const PION_RIM := 0.045
const PION_SIDE_RIM := 0.012
## Les bandes de la tranche et les pointillés du dessus du palet de J1.
const PION_STRIPES := 12.0
const PION_DASHES := 14.0
## La fausse lumière qui nuance la tranche : sa direction, et l'ombre et l'éclat qu'elle
## donne ; le dessous, jamais éclairé.
const PION_LIGHT := Vector3(0.35, 0.55, 1.0)
const PION_SHADE_DARK := 0.72
const PION_SHADE_LIGHT := 1.05
const PION_UNDERSIDE := 0.6
## L'ombre douce sous un pion : sa taille, sa force, et où commence son fondu (part du
## rayon).
const PION_SHADOW_SIZE := 1.0
const PION_SHADOW_ALPHA := 0.32
const PION_SHADOW_SOFTNESS := 0.45
## L'anneau d'un statut sous un pion : son rayon, sa demi-épaisseur (en part de la case), et
## la part de son milieu (le reste est son bord).
const STATUS_RING := 0.45
const STATUS_RING_WIDTH := 0.045
const STATUS_RING_CORE := 0.55
## L'éclat d'un événement : l'anneau (rayon et demi-épaisseur, en part de la case) et la
## taille où il commence et finit ; la balle du tir de la reine.
const PULSE_RING := 0.4
const PULSE_RING_WIDTH := 0.05
const PULSE_START_SCALE := 0.6
const PULSE_END_SCALE := 1.6
const SHOT_RADIUS := 0.1
## Un enfant (Familia) est un pion plus petit (design/elements/LISEZMOI.txt).
const CHILD_SCALE := 0.68
## La couronne d'une reine : son rayon, la hauteur de ses creux et de ses pointes, son
## nombre de pointes, son trait, et la pierre sur son devant.
const CROWN_RADIUS := 0.25
const CROWN_BASE := 0.12
const CROWN_PEAK := 0.27
const CROWN_POINTS := 5
const CROWN_INK := 0.016
const CROWN_JEWEL_RADIUS := 0.04
const CROWN_JEWEL := Color("e84a5f")
## La tombe : une stèle posée au fond de sa case.
const TOMB_SIZE := Vector3(0.3, 0.34, 0.11)
const TOMB_OFFSET := Vector2(-0.22, -0.22)
const TOMB_BAND := 0.07
const TOMB_BAND_OVERHANG := 1.04
## La marque de case survolée, et le cadre en pointillés du dernier coup.
const MARK_SIZE := 0.94
const MARK_BORDER_WIDTH := 0.07
const LAST_MARK_SIZE := 0.84
const LAST_MARK_DASHES := 16.0
## La caméra filme le plateau droit, face à ses rangées (essai du 2026-11-04, demande de
## Martin) : les cases sont deux fois plus grandes qu'en losange, et les lignes se lisent
## comme sur un damier. Sa plongée est choisie parmi celles-ci, la plus lisible.
const CAMERA_PITCHES_DEGREES: Array[float] = [-45.0, -55.0]
const CAMERA_YAW_DEGREES := 0.0
const CAMERA_DISTANCE := 30.0
## Ce que la caméra cadre au-delà des cases jouables, en cases, et la marge qu'elle ajoute.
const FRAME_PADDING := 0.5
const FRAME_MARGIN := 1.0
