class_name Session
extends RefCounted
## Ce que le client sait de la partie en cours, au-delà de la vue : contre qui il joue (un
## bot ou un ami, son nom, s'il est branché), s'il attend de lancer la salle qu'il a créée,
## si le bot du serveur ne joue plus, et le catalogue des cartes.
##
## Une seule session vit pendant tout le jeu. Ses champs se lisent partout mais ne
## s'écrivent que par ses méthodes : `start_*` et `reset` (une nouvelle partie), `hold` et
## `launch` (la salle à lancer), `note_*` (ce que le serveur annonce). Les écrans la lisent ;
## seule la racine l'écrit.
##
## Avec un fichier (`SAVE_PATH`), elle y garde comment la partie a commencé et si la salle
## attend son lancement : après une relance du jeu, la place gardée est reprise, et
## `restore` retrouve contre qui l'on joue.

## Comment la partie a commencé.
enum Mode { NONE, BOT, HOST, GUEST }

## Le fichier de la session du jeu, à côté de la place gardée (`ServerLink.SEAT_PATH`).
const SAVE_PATH := "user://session.cfg"
const SAVE_SECTION := "session"
## Ce qui ne va pas, à dire pendant la partie, du plus grave au moins grave.
enum Warning { NONE, OFFLINE, BOT_DOWN, FRIEND_LEFT }

var mode: Mode:
	get:
		return _mode
	set(_value):
		push_error("Session.mode s'écrit par start_* ou reset")
var opponent_name: String:
	get:
		return _opponent_name
	set(_value):
		push_error("Session.opponent_name s'écrit par start_* ou reset")
## Vrai quand l'adversaire humain est branché sur sa place (message `opponent`).
var opponent_present: bool:
	get:
		return _opponent_present
	set(_value):
		push_error("Session.opponent_present s'écrit par note_opponent_present")
## Vrai depuis que le bot du serveur ne joue plus, jusqu'à la vue suivante.
var bot_failed: bool:
	get:
		return _bot_failed
	set(_value):
		push_error("Session.bot_failed s'écrit par note_bot_failed")
## Le catalogue des cartes (noms et textes), reçu à chaque connexion ; `null` avant.
var catalog: CardCatalog:
	get:
		return _catalog
	set(_value):
		push_error("Session.catalog s'écrit par note_catalog")

var _mode := Mode.NONE
var _opponent_name := Labels.OPPONENT
var _opponent_present := false
var _bot_failed := false
var _catalog: CardCatalog
## Vrai tant que le créateur d'une salle ne l'a pas lancée : les vues attendent.
var _waiting_launch := false
## La dernière vue reçue pendant cette attente, et ses événements.
var _held: Array = []
## Le fichier où la session se garde ; vide : nulle part (les tests).
var _save_path := ""


func _init(save_path := "") -> void:
	_save_path = save_path


# --- Une nouvelle partie ---

## Une partie contre le bot du serveur.
func start_bot() -> void:
	_begin(Mode.BOT, false)


## Une salle créée pour un ami : elle attend son lancement.
func start_host() -> void:
	_begin(Mode.HOST, true)


## La salle d'un ami, rejointe avec son code.
func start_guest() -> void:
	_begin(Mode.GUEST, false)


## Plus de partie (retour à l'accueil). Le catalogue reste : il vaut pour toutes.
func reset() -> void:
	_begin(Mode.NONE, false)


## Reprend la session gardée dans son fichier : contre qui l'on joue, et si la salle attend
## son lancement. À n'appeler qu'avec une place gardée : c'est la partie de cette place.
func restore() -> void:
	var file := ConfigFile.new()
	if _save_path == "" or file.load(_save_path) != OK:
		return
	var mode_name: Variant = file.get_value(SAVE_SECTION, "mode", "")
	if not (mode_name is String and Mode.has(mode_name)):
		return
	_begin(Mode[mode_name] as Mode, file.get_value(SAVE_SECTION, "waiting_launch", false) == true)


## Une nouvelle partie, de ce mode : tout ce qu'on savait de la précédente est oublié.
func _begin(new_mode: Mode, waiting: bool) -> void:
	_mode = new_mode
	_opponent_name = _name_against(new_mode)
	_opponent_present = false
	_bot_failed = false
	_waiting_launch = waiting
	_held = []
	_save()


static func _name_against(against: Mode) -> String:
	match against:
		Mode.BOT:
			return Labels.BOT
		Mode.HOST, Mode.GUEST:
			return Labels.FRIEND
	return Labels.OPPONENT


func _save() -> void:
	if _save_path == "":
		return
	var file := ConfigFile.new()
	file.set_value(SAVE_SECTION, "mode", Mode.find_key(_mode))
	file.set_value(SAVE_SECTION, "waiting_launch", _waiting_launch)
	file.save(_save_path)


# --- La salle à lancer ---

## Vrai tant que la salle créée attend que son créateur la lance.
func waiting_launch() -> bool:
	return _waiting_launch


## Garde cette vue jusqu'au lancement de la salle (seule la dernière compte).
func hold(view: ViewData, events: Array) -> void:
	_held = [view, events]


## Lance la salle : rend la vue gardée (`[vue, événements]`, vide s'il n'y en a pas).
func launch() -> Array:
	_waiting_launch = false
	_save()
	var held := _held
	_held = []
	return held


# --- Ce que le serveur annonce ---

func note_catalog(cards: CardCatalog) -> void:
	_catalog = cards


func note_opponent_present(present: bool) -> void:
	_opponent_present = present


func note_bot_failed(failed: bool) -> void:
	_bot_failed = failed


# --- Les lectures ---

func against_bot() -> bool:
	return _mode == Mode.BOT


## Ce qui ne va pas pendant la partie, le plus grave d'abord : la connexion, puis le bot
## qui ne joue plus, puis l'ami débranché.
func warning(online: bool) -> Warning:
	if not online:
		return Warning.OFFLINE
	if _bot_failed:
		return Warning.BOT_DOWN
	if _mode in [Mode.HOST, Mode.GUEST] and not _opponent_present:
		return Warning.FRIEND_LEFT
	return Warning.NONE
