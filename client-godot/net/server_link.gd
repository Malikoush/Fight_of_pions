extends Node
## Le lien avec le serveur (autoload `ServerLink`) : une WebSocket, un message JSON par trame
## (docs/PROTOCOLE.md). Il ne connaît aucune règle : il envoie les demandes du client et
## annonce ce que le serveur répond. Il garde la place (code, joueur, clé) sur le disque et
## la reprend tout seul après une coupure, ou au lancement suivant.
## À chaque ouverture, il se connecte d'abord au compte du joueur (`login` avec le jeton
## gardé, ou `register` la première fois : le serveur l'exige pour créer ou rejoindre une
## salle), puis demande le catalogue des cartes (noms et textes à afficher).
## Il classe chaque refus du serveur par catégorie (`ServerErrors`).

## L'état de la connexion.
enum Status { OFFLINE, CONNECTING, ONLINE, RECONNECTING }

signal status_changed(status: Status)
## La partie a changé : la vue entière, et ce qui s'est passé depuis la précédente.
signal state_received(view: ViewData, events: Array)
## La réponse à un aperçu : la vue après ces coups, ou `null` si la carte est complète.
signal preview_received(view: ViewData)
## Le catalogue des cartes : les familles, leurs noms et leurs textes.
signal cards_received(cards: CardCatalog)
## L'adversaire humain s'est branché sur sa place, ou s'en est débranché.
signal opponent_presence(connected: bool)
## Une demande refusée : sa catégorie, et le message du serveur, en français, à afficher.
signal server_error(category: ServerErrors.Category, message: String)
## Le compte de cette connexion : son numéro et son pseudo (après la connexion au compte,
## ou un changement de pseudo).
signal account_received(id: int, pseudo: String)
## Ce que possède le compte : ses cartes et son trio de cache (`{"kind", "rarity"}`).
signal collection_received(cards: Array, cache_trio: Array)
## La partie vient de finir et le joueur a gagné cette carte (`{"kind", "rarity"}`).
signal reward_received(card: Dictionary)
## Où en est le lancement de la salle : dit en s'asseyant (`seated`), puis quand son créateur
## la lance (`launched`, vrai). Avant, personne ne joue.
signal room_launch(launched: bool)

## L'adresse du serveur lancé par `cargo serveur`, hors d'un navigateur. En IPv4 : le
## serveur n'écoute que là, et « localhost » peut désigner l'IPv6 sous Windows. Dans un
## navigateur, l'adresse vient de la page (`server_url`).
const DEFAULT_URL := "ws://127.0.0.1:3000/ws"
## La longueur du code d'une salle (protocole).
const ROOM_CODE_LENGTH := 5
const SEAT_PATH := "user://seat.cfg"
const SEAT_SECTION := "place"
const SEAT_FIELDS := ["code", "player", "key"]
## Le jeton du compte, gardé sur l'appareil : le serveur ne le donne qu'une fois, à la
## création du compte, et n'en garde que l'empreinte. Perdu, le compte l'est aussi.
const ACCOUNT_PATH := "user://account.cfg"
const ACCOUNT_SECTION := "compte"
const ACCOUNT_TOKEN := "jeton"
## Une vue complète peut approcher la mémoire de réception par défaut de Godot (64 Kio).
const INBOUND_BUFFER_BYTES := 1 << 20
## Les attentes entre deux essais de connexion, en secondes ; la dernière se répète.
const RETRY_DELAYS: Array[float] = [0.5, 1.0, 2.0, 4.0]
## Les demandes qui peuvent attendre l'ouverture de la connexion. Un coup préparé avant une
## coupure, lui, est perdu : la vue reçue au retour fait foi.
const QUEUED_REQUESTS := ["create", "join"]
## La réponse de chaque demande qui attend (`_pending`), par le `type` de la réponse.
const _REPLIES := {"seated": "join", "account": "set_pseudo", "collection": "set_cache_trio"}
## La longueur d'un message illisible recopiée dans le journal.
const LOGGED_CHARACTERS := 200

var status := Status.OFFLINE
var url := DEFAULT_URL
## Le nombre d'essais ratés d'affilée.
var failed_attempts := 0
## Le compte, une fois connecté : son numéro (0 tant qu'il n'est pas connu) et son pseudo.
var account_id := 0
var account_pseudo := ""

var _socket: WebSocketPeer
var _wants_connection := false
var _retry_in := 0.0
var _outbox: Array[String] = []
var _seat := {}
## Le `type` de la demande qui attend sa réponse et dont un refus ne parle que d'elle
## (`join`, `set_pseudo`, `set_cache_trio` : `ServerErrors.category`), ou vide.
var _pending := ""
var _reopen_after_close := false
## Le jeton du compte, vide tant qu'aucun compte n'a été créé sur cet appareil.
var _token := ""


func _ready() -> void:
	url = server_url()
	_load_seat()
	_load_account()


## L'adresse du serveur. Dans un navigateur, la page a été servie par le serveur du jeu
## (qui sert aussi la WebSocket) : l'adresse s'en déduit, sans rien régler. Ailleurs (PC,
## éditeur), `DEFAULT_URL`.
static func server_url() -> String:
	if not OS.has_feature("web"):
		return DEFAULT_URL
	var page := func(property: String) -> String: return str(JavaScriptBridge.eval("window.location." + property))
	return url_for_page(page.call("protocol"), page.call("host"), page.call("pathname"))


## L'adresse de la WebSocket pour une page : `wss` pour une page en `https` (`ws` sinon), le
## même hôte (port compris), et `ws` dans le même dossier que la page.
static func url_for_page(protocol: String, host: String, path: String) -> String:
	var scheme := "wss" if protocol == "https:" else "ws"
	var folder := path.left(path.rfind("/") + 1)
	if not folder.begins_with("/"):
		folder = "/" + folder
	return "%s://%s%sws" % [scheme, host, folder]


## Ouvre la connexion, et la rouvre tant qu'elle n'est pas fermée volontairement.
func connect_to_server() -> void:
	_wants_connection = true
	if _socket == null:
		_open()


## Réessaie tout de suite, sans attendre la fin du délai en cours.
func retry_now() -> void:
	_retry_in = 0.0
	connect_to_server()


func disconnect_from_server() -> void:
	_wants_connection = false
	if _socket != null:
		_socket.close()


## Vrai si une place est gardée : elle sera reprise à l'ouverture de la connexion.
func has_saved_seat() -> bool:
	return not _seat.is_empty()


func forget_seat() -> void:
	_seat = {}
	DirAccess.remove_absolute(ProjectSettings.globalize_path(SEAT_PATH))


## Quitter sa salle : la place est oubliée et une connexion neuve s'ouvre, car le serveur
## n'accepte qu'une salle par connexion. Une demande faite d'ici là attend la nouvelle.
func leave_room() -> void:
	forget_seat()
	if _socket != null:
		_reopen_after_close = true
		_socket.close()
		_set_status(Status.CONNECTING)


## Créer une salle et s'y asseoir, à une place tirée au sort par le serveur. `opponent` :
## `human`, `mcts` ou `random`.
func create_room(opponent: String, variant: String = "") -> void:
	var message := {"type": "create", "opponent": opponent}
	if variant != "":
		message["variant"] = variant
	_send(message)


## Rejoindre la salle de ce code (5 caractères, la casse ne compte pas), à la place que son
## créateur a laissée.
func join_room(code: String) -> void:
	_ask({"type": "join", "code": code})


## Lancer la partie de la salle qu'on a créée, une fois son ami assis : les deux joueurs
## reçoivent `room_launch`.
func launch_room() -> void:
	_send({"type": "launch"})


## Vrai si un compte a été créé sur cet appareil : il sera repris à l'ouverture.
func has_account() -> bool:
	return _token != ""


## Oublie le compte de cet appareil : à la prochaine ouverture, un compte neuf est créé.
func forget_account() -> void:
	_token = ""
	account_id = 0
	account_pseudo = ""
	DirAccess.remove_absolute(ProjectSettings.globalize_path(ACCOUNT_PATH))


## Changer son pseudo (réponse : `account_received`, ou un refus `bad_pseudo`).
func set_pseudo(pseudo: String) -> void:
	_ask({"type": "set_pseudo", "pseudo": pseudo})


## Redemander ses cartes et son trio (`collection_received`).
func request_collection() -> void:
	_send({"type": "collection"})


## Ranger son trio de cache : de 0 à 3 cartes de sa collection (`{"kind", "rarity"}`) ;
## vide, le serveur en tire 3 au hasard à chaque partie. Réponse : `collection_received`.
func set_cache_trio(cards: Array) -> void:
	_ask({"type": "set_cache_trio", "cards": cards})


## Le code de la salle où l'on est assis, ou vide.
func room_code() -> String:
	return _seat.get("code", "")


## Le joueur de la place où l'on est assis ; sans place, le joueur 1 (celui du créateur
## d'une salle, le seul qui la regarde avant d'y être assis).
func seat_player() -> String:
	return _seat.get("player", ViewData.PLAYER_ONE)


## Jouer un coup de `legal_actions`, recopié tel quel.
func play(action: Variant) -> void:
	_send({"type": "play", "action": action})


## Redemander sa vue actuelle.
func sync() -> void:
	_send({"type": "sync"})


## Essayer ces coups sur une copie, sans rien jouer : la réponse donne les choix suivants
## d'une carte (`preview_received`).
func preview(actions: Array) -> void:
	_send({"type": "preview", "actions": actions})


func _process(delta: float) -> void:
	if _socket == null:
		if _wants_connection:
			_retry_in -= delta
			if _retry_in <= 0.0:
				_open()
		return
	_socket.poll()
	match _socket.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			if status != Status.ONLINE and not _reopen_after_close:
				_on_opened()
			while _socket != null and _socket.get_available_packet_count() > 0:
				_receive(_socket.get_packet().get_string_from_utf8())
		WebSocketPeer.STATE_CLOSED:
			_on_closed()


func _open() -> void:
	_socket = WebSocketPeer.new()
	_socket.inbound_buffer_size = INBOUND_BUFFER_BYTES
	if _socket.connect_to_url(url) != OK:
		_socket = null
		_schedule_retry()
		return
	_set_status(Status.CONNECTING if failed_attempts == 0 else Status.RECONNECTING)


func _on_opened() -> void:
	failed_attempts = 0
	_set_status(Status.ONLINE)
	# Le compte d'abord : le serveur traite les messages dans l'ordre, et les demandes en
	# attente (créer, rejoindre) en ont besoin.
	_write(_account_request())
	_write({"type": "cards"})
	if has_saved_seat():
		_write({"type": "rejoin", "code": _seat["code"], "player": _seat["player"], "key": _seat["key"]})
	var waiting := _outbox
	_outbox = []
	for text in waiting:
		_socket.send_text(text)


func _on_closed() -> void:
	_socket = null
	# Une demande perdue avec la connexion n'attend plus de réponse.
	_pending = ""
	if _reopen_after_close:
		_reopen_after_close = false
		failed_attempts = 0
		_open()
	elif _wants_connection:
		_schedule_retry()
	else:
		_set_status(Status.OFFLINE)


func _schedule_retry() -> void:
	_retry_in = RETRY_DELAYS[mini(failed_attempts, RETRY_DELAYS.size() - 1)]
	failed_attempts += 1
	_set_status(Status.RECONNECTING)


## Envoie ce message, ou le met en attente de la connexion s'il peut attendre ; rend faux
## s'il est perdu.
func _send(message: Dictionary) -> bool:
	if status == Status.ONLINE and _socket != null and not _reopen_after_close:
		_write(message)
	elif message.get("type", "") in QUEUED_REQUESTS:
		_outbox.append(Wire.encode(message))
	else:
		return false
	return true


## Envoie une demande dont un refus ne parle que d'elle, jusqu'à sa réponse.
func _ask(message: Dictionary) -> void:
	if _send(message):
		_pending = message["type"]


func _write(message: Dictionary) -> void:
	_socket.send_text(Wire.encode(message))


## Lit un message du serveur. Un message mal formé (pas du JSON, ou un champ attendu
## absent ou du mauvais type) est signalé dans le journal et ignoré.
func _receive(text: String) -> void:
	var message: Variant = Wire.decode(text)
	if not (message is Dictionary) or not _is_well_formed(message):
		push_warning("Message mal formé du serveur : %s" % text.left(LOGGED_CHARACTERS))
		return
	if _REPLIES.get(message["type"]) == _pending:
		_pending = ""
	match message["type"]:
		"seated":
			_save_seat(message)
			room_launch.emit(message.get("launched", false) == true)
		"launched":
			room_launch.emit(true)
		"state":
			state_received.emit(ViewData.new(message["view"]), message.get("events", []))
		"preview":
			var view: Variant = message["view"]
			preview_received.emit(ViewData.new(view) if view is Dictionary else null)
		"cards":
			cards_received.emit(CardCatalog.new(message["cards"]))
		"opponent":
			opponent_presence.emit(message["connected"])
		"account":
			_on_account(message)
		"collection":
			collection_received.emit(message["cards"], message["cache_trio"])
		"reward":
			reward_received.emit(message["card"])
		"error":
			_on_error(message["kind"], message["message"])


static func _is_well_formed(message: Dictionary) -> bool:
	match message.get("type"):
		"seated":
			return (
				SEAT_FIELDS.all(func(field: String) -> bool: return message.get(field) is String)
				and (not message.has("launched") or message["launched"] is bool)
			)
		"launched":
			return true
		"state":
			return message.get("view") is Dictionary and message.get("events", []) is Array
		"error":
			return message.get("kind") is String and message.get("message") is String
		"preview":
			return message.has("view") and (message["view"] == null or message["view"] is Dictionary)
		"cards":
			return message.get("cards") is Array
		"opponent":
			return message.get("connected") is bool
		"account":
			return (
				message.get("id") is int
				and message.get("pseudo") is String
				and (not message.has("token") or message["token"] is String)
			)
		"collection":
			return message.get("cards") is Array and message.get("cache_trio") is Array
		"reward":
			return message.get("card") is Dictionary
	return false


## Un refus : la place gardée perdue est oubliée, une place reprise ailleurs arrête les
## reconnexions ; puis il est annoncé par sa catégorie. Un compte inconnu du serveur (sa
## base a été vidée) est oublié et remplacé par un compte neuf, sans rien annoncer.
func _on_error(kind: String, text: String) -> void:
	if kind == ServerErrors.UNKNOWN_ACCOUNT:
		forget_account()
		if status == Status.ONLINE and _socket != null:
			_write(_account_request())
		return
	var category := ServerErrors.category(kind, _pending)
	_pending = ""
	match category:
		ServerErrors.Category.SEAT_LOST:
			forget_seat()
		ServerErrors.Category.SEAT_TAKEN:
			disconnect_from_server()
	server_error.emit(category, text)


func _set_status(new_status: Status) -> void:
	if new_status != status:
		status = new_status
		status_changed.emit(status)


## La demande qui connecte au compte : `login` avec le jeton gardé, ou `register`.
func _account_request() -> Dictionary:
	if has_account():
		return {"type": "login", "token": _token}
	return {"type": "register"}


## Le compte de la connexion : son jeton est gardé s'il vient d'être créé.
func _on_account(message: Dictionary) -> void:
	account_id = message["id"]
	account_pseudo = message["pseudo"]
	var token: String = message.get("token", "")
	if token != "":
		_token = token
		var file := ConfigFile.new()
		file.set_value(ACCOUNT_SECTION, ACCOUNT_TOKEN, token)
		file.save(ACCOUNT_PATH)
	account_received.emit(account_id, account_pseudo)


func _load_account() -> void:
	var file := ConfigFile.new()
	if file.load(ACCOUNT_PATH) != OK:
		return
	var token: Variant = file.get_value(ACCOUNT_SECTION, ACCOUNT_TOKEN, "")
	if token is String:
		_token = token


func _save_seat(message: Dictionary) -> void:
	_seat = {}
	var file := ConfigFile.new()
	for field: String in SEAT_FIELDS:
		_seat[field] = message[field]
		file.set_value(SEAT_SECTION, field, message[field])
	file.save(SEAT_PATH)


func _load_seat() -> void:
	var file := ConfigFile.new()
	if file.load(SEAT_PATH) != OK:
		return
	var loaded := {}
	for field: String in SEAT_FIELDS:
		loaded[field] = file.get_value(SEAT_SECTION, field, "")
	if loaded.values().all(func(value: Variant) -> bool: return value is String and value != ""):
		_seat = loaded
