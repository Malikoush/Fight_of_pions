class_name Labels
extends RefCounted
## Tout le texte affiché par le client, en français, réuni ici. Le serveur envoie des
## identifiants (`One`, `EndTurn`…) ; ce fichier dit comment les montrer. Les messages
## d'erreur du serveur, déjà en français, sont affichés tels quels.

const GAME_NAME := "Fight of Pions"
const TAGLINE := "Morpion géant, cartes d'attaque, en 1 contre 1."

const CONNECTING := "Connexion au serveur…"
const RECONNECTING := "Reconnexion…"
const UNREACHABLE := "Serveur injoignable, nouvel essai…"
const RETRY := "RÉESSAYER"

const PLAY_BOT := "JOUER CONTRE LE BOT"
## Le temps des essais, le client joue contre le bot aléatoire, qui ne charge pas la machine.
const PLAY_BOT_DETAIL := "Bot aléatoire · draft, puis partie"
const FRIENDS_SECTION := "PARTIE ENTRE AMIS"
const CREATE_ROOM := "CRÉER UNE SALLE"
const CREATE_ROOM_DETAIL := "Un code à envoyer"
const JOIN := "REJOINDRE"
const JOIN_DETAIL := "Avec un code de 5 lettres"
const JOIN_TITLE := "Rejoindre une salle"
const JOIN_HELP := "Entre le code à 5 caractères que ton ami t'a envoyé."
const BACK := "RETOUR"
const FRIEND := "Ton ami"
## La salle d'attente d'une partie entre amis.
const LOBBY_TITLE := "Créer une salle"
const LOBBY_WAITING := "Envoie ce code à ton ami. Tu lanceras la partie quand il sera là."
const LOBBY_READY := "Ton ami est là : tu peux lancer la partie."
const PLAYER_READY := "PRÊT"
const PLAYER_WAITING := "EN ATTENTE"
const VERSUS := "contre"
const LAUNCH := "LANCER LA PARTIE"
const CLOSE_ROOM := "FERMER LA SALLE"

## Le draft.
const ROOM_CODE := "CODE DE LA SALLE"
const COPY := "COPIER"
const ROOM_WAITING := "Envoie ce code à ton ami : il rejoint la salle avec lui."
const LEAVE_ROOM := "QUITTER LA SALLE"
const MY_ARSENAL := "MON ARSENAL"
const DRAFT_CHOOSE := "CHOISIS UNE CARTE"
const DRAFT_WAIT := "%s choisit…"
const ALL_RARITIES := "TOUTES"
## Le nom de chaque phase du draft, dans la barre des étapes.
const DRAFT_PHASES := {"Ban": "BANS", "Pick": "PICKS", "Cache": "CACHE"}
## Ce qu'on fait à chaque étape : le titre, l'aide, le bouton (avec le nom de la carte).
const DRAFT_TITLES := {"Ban": "Bannis une carte", "Pick": "À toi de prendre", "Cache": "Choisis ta carte cachée"}
const DRAFT_HELP := {
	"Ban": "Les cartes tirées. Une carte bannie ne sera prise par personne.",
	"Pick": "Une carte tirée rejoint ton arsenal. Les picks sont publics.",
	"Cache": "Dans tout le catalogue. L'adversaire ne la verra qu'au moment où tu la lanceras.",
}
const DRAFT_ACTIONS := {"Ban": "BANNIR %s", "Pick": "PRENDRE %s", "Cache": "CACHER %s"}
## Ce que vient de faire l'adversaire, au draft.
const DRAFT_DONE := {"Ban": "a banni %s", "Pick": "a pris %s"}
const DRAFT_CACHED := "a mis une carte en cache"
## Les raretés, pour filtrer le catalogue pendant le choix de la cache.
const RARITY_FILTERS := {"Common": "COMMUNE", "Rare": "RARE", "Epic": "ÉPIQUE", "Legendary": "LÉGENDAIRE"}

const YOU := "Toi"
const BOT := "Bot"
const OPPONENT := "Adversaire"
const YOUR_TURN := "à toi de jouer"
const WAITS := "attend"
const PLAYS := "joue"
const THINKS := "réfléchit…"
## Sous le tour, pendant que l'adversaire décide.
const BOT_THINKING := "Le bot réfléchit…"
const OPPONENT_PLAYING := "L'adversaire joue…"

const WAITING := "EN ATTENTE"
## Le bandeau du début de ton tour.
const YOUR_TURN_BANNER := "À toi !"
const HOME := "ACCUEIL"

const FREE_PLACEMENT := "Pose gratuite disponible"
const PLACEMENT := "Pose disponible"
const PLACEMENTS := "%d poses disponibles"
const NO_PLACEMENT := "Plus de pose ce tour"
const GAME_OVER := "Partie finie"

## Les cartes et leur choix.
const SECRET_TAG := "CACHE"
const STEP_LAUNCHER := "LANCEUR"
const STEP_TARGETS := "CIBLES"
const CHOOSE_LAUNCHER := "Choisis le lanceur"
const CHOOSE_TARGET := "Choisis une cible"
const READY_TO_PLAY := "Prête à jouer"
const TARGETS_CHOSEN := "%d cible(s) choisie(s)"
const CANCEL := "ANNULER"
const PLAY_CARD := "JOUER LA CARTE"
const AIM_TOKENS := "VISER LES JETONS"
const HARE_DESTINATION := "Touche la case où va ton pion."

## Ce qui dure, dans le panneau d'un joueur.
const FACT_SCORCHED := "Terre brûlée · jusqu'au tour %d"
const FACT_INFIRMARY := "Infirmerie %d"
const FACT_GRAVEYARD := "Cimetière %d"
const FACT_GUARDS := "Gardes %d"

## Le tour adverse : l'annonce d'une carte adverse, sa cache révélée, la fenêtre de réaction.
const ATTACK_TAG := "ATTAQUE"
const REACTION_TAG := "RÉACTION"
const ANNOUNCE_TITLE := "%s lance %s"
const ANNOUNCE_FROM := "depuis son pion en %s"
const REVEAL_TAG := "CACHE RÉVÉLÉE"
const REVEAL_TITLE := "%s lance sa carte cachée"
const REVEAL_DETAIL := "%s · coût %d"
const TAP_TO_CONTINUE := "Toucher pour continuer"
const REACT_QUESTION := "Réagir ?"
const REACT_DEFAULT_TITLE := "L'adversaire vient de jouer"
const PLAY_REACTION := "JOUER %s"
const NOT_PLAYABLE_NOW := "pas jouable maintenant"

## Le journal des coups.
const JOURNAL_TITLE := "JOURNAL DES COUPS"
const JOURNAL_ORDER := "les plus récents en haut"
const JOURNAL_EMPTY := "Rien pour l'instant : les coups s'inscriront ici."
const JOURNAL_TURN := "T%d"
## Les lettres des colonnes, de gauche à droite (le moteur s'arrête aussi à Z).
const COLUMN_LETTERS := "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
const CLOSE := "FERMER"
const JOURNAL_PLACE := "pose en %s"
const JOURNAL_REVEAL := "révèle sa carte cachée : %s"
const JOURNAL_ATTACK := "lance %s depuis %s"
const JOURNAL_SHOT := "tire avec un garde de sa reine sur %s"
const JOURNAL_KILLED := "pion tué en %s"
const JOURNAL_MOVED := "pion de %s à %s"
const JOURNAL_SWAPPED := "échange de %s et %s"
const JOURNAL_DUG := "mur creusé en %s"
const JOURNAL_GRAVEYARD := "un pion part au cimetière"
const JOURNAL_HEALED := "un pion sort de l'infirmerie"
const JOURNAL_SCORCHED := "sous Terre brûlée jusqu'au tour %d"
const JOURNAL_TOKENS := "reçoit une pile : %s"
const JOURNAL_CHILD := "enfant en %s"
## Les effets continus racontés quand ils commencent, par sorte (`%s` : la case du pion).
const JOURNAL_EFFECTS := {
	"Immobilized": "pion immobilisé en %s",
	"Protected": "pion protégé en %s",
	"Pact": "pacte en %s",
	"Crown": "reine couronnée en %s",
	"Guard": "un garde pour la reine en %s",
	"Return": "retour prévu pour le pion en %s",
	"StepTokens": "jetons de pas en %s",
}

## Le libellé de chaque coup proposé dans le menu d'une case.
const CELL_ACTIONS := {
	"Place": "POSER ICI",
	"StepQueen": "PAS DE LA REINE",
	"Shoot": "TIR DE LA REINE",
	"HareStep": "PAS DU LIÈVRE",
}

## Le libellé des coups sans case, sur le bouton d'action.
const ACTION_LABELS := {
	"EndTurn": "FINIR LE TOUR",
	"Pass": "PASSER",
}


## « TOUR 7 · À TOI » ou « TOUR 8 · ADVERSAIRE ».
static func turn_pill(turn: int, mine: bool) -> String:
	return "TOUR %d · %s" % [turn, "À TOI" if mine else "ADVERSAIRE"]


## Ce qui reste à poser ce tour, ou une chaîne vide quand ce n'est pas à dire.
static func placements(left: int, cost: int, my_turn: bool) -> String:
	if not my_turn:
		return ""
	if left == 0:
		return NO_PLACEMENT
	if left > 1:
		return PLACEMENTS % left
	return FREE_PLACEMENT if cost == 0 else PLACEMENT


## Une pile de jetons du Massacre divin : « 3 jetons · +1 point » ou « 2 jetons · cimetière ».
## `reward` est la récompense telle que la vue l'écrit (`{"Points": 1}` ou `"Graveyard"`).
static func token_pile(tokens: int, reward: Variant) -> String:
	var count := _plural(tokens, "jeton")
	var points: Variant = Wire.content(reward)
	if points is int:
		return "%s · +%s" % [count, _plural(points, "point")]
	return "%s · cimetière" % count


## « 1 jeton », « 3 jetons ».
static func _plural(count: int, word: String) -> String:
	return "%d %s%s" % [count, word, "s" if count > 1 else ""]


## Le nom d'une case, comme le moteur l'écrit (`Position`) : la lettre de sa colonne, puis
## le numéro de sa rangée, comptés sur toute la grille, mur compris (« C5 »).
static func cell_name(cell: Vector2i) -> String:
	if cell.x < 0 or cell.x >= COLUMN_LETTERS.length() or cell.y < 0:
		return "?"
	return "%s%d" % [COLUMN_LETTERS[cell.x], cell.y + 1]
