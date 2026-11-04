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
## Le temps des essais (E3a à E3e), le client joue contre le bot aléatoire.
const PLAY_BOT_DETAIL := "Bot aléatoire, le temps des essais"

const YOU := "Toi"
const BOT := "Bot"
const OPPONENT := "Adversaire"
const YOUR_TURN := "à toi de jouer"
const WAITS := "attend"
const PLAYS := "joue"

const WAITING := "EN ATTENTE"
## Le bandeau du début de ton tour.
const YOUR_TURN_BANNER := "À toi !"
const HOME := "ACCUEIL"

const FREE_PLACEMENT := "Pose gratuite disponible"
const PLACEMENT := "Pose disponible"
const PLACEMENTS := "%d poses disponibles"
const NO_PLACEMENT := "Plus de pose ce tour"
const GAME_OVER := "Partie finie"

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
