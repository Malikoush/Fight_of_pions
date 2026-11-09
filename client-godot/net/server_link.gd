extends Node
## Le lien avec le serveur (autoload `ServerLink`) : une WebSocket, un message JSON par trame
## (docs/PROTOCOLE.md). Il ne connaît aucune règle : il envoie les demandes du client et
## annonce ce que le serveur répond. Il garde la place (code, joueur, clé) sur le disque et
## la reprend tout seul après une coupure, ou au lancement suivant.
## À chaque ouverture, il demande le catalogue des cartes (noms et textes à afficher).
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

## L'adresse du serveur lancé par `cargo serveur`. En IPv4 : le serveur n'écoute que là,
## et « localhost » peut désigner l'IPv6 sous Windows.
const DEFAULT_URL := "ws://127.0.0.1:3000/ws"
## La longueur du code d'une salle (protocole).
const ROOM_CODE_LENGTH := 5
const SEAT_PATH := "user://seat.cfg"
const SEAT_SECTION := "place"
const SEAT_FIELDS := ["code", "player", "key"]
## Une vue complète peut approcher la mémoire de réception par défaut de Godot (64 Kio).
const INBOUND_BUFFER_BYTES := 1 << 20
## Les attentes entre deux essais de connexion, en secondes ; la dernière se répète.
const RETRY_DELAYS: Array[float] = [0.5, 1.0, 2.0, 4.0]
## Les demandes qui peuvent attendre l'ouverture de la connexion. Un coup préparé avant une
## coupure, lui, est perdu : la vue reçue au retour fait foi.
const QUEUED_REQUESTS := ["create", "join"]
## La longueur d'un message illisible recopiée dans le journal.
const LOGGED_CHARACTERS := 200

var status := Status.OFFLINE
var url := DEFAULT_URL
## Le nombre d'essais ratés d'affilée.
var failed_attempts := 0

var _socket: WebSocketPeer
var _wants_connection := false
var _retry_in := 0.0
var _outbox: Array[String] = []
var _seat := {}
## Vrai entre une demande `join` et sa réponse : un refus parle alors du code tapé.
var _joining := false
var _reopen_after_close := false


func _ready() -> void:
	_load_seat()


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


## Créer une salle et s'y asseoir en joueur 1. `opponent` : `human`, `mcts` ou `random`.
func create_room(opponent: String, variant: String = "") -> void:
	var message := {"type": "create", "opponent": opponent}
	if variant != "":
		message["variant"] = variant
	_send(message)


## Rejoindre la salle de ce code (5 caractères, la casse ne compte pas) en joueur 2.
func join_room(code: String) -> void:
	_joining = true
	_send({"type": "join", "code": code})


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
	_write({"type": "cards"})
	if has_saved_seat():
		_write({"type": "rejoin", "code": _seat["code"], "player": _seat["player"], "key": _seat["key"]})
	var waiting := _outbox
	_outbox = []
	for text in waiting:
		_socket.send_text(text)


func _on_closed() -> void:
	_socket = null
	# Une demande `join` perdue avec la connexion n'attend plus de réponse.
	_joining = false
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


func _send(message: Dictionary) -> void:
	if status == Status.ONLINE and _socket != null and not _reopen_after_close:
		_write(message)
	elif message.get("type", "") in QUEUED_REQUESTS:
		_outbox.append(Wire.encode(message))


func _write(message: Dictionary) -> void:
	_socket.send_text(Wire.encode(message))


## Lit un message du serveur. Un message mal formé (pas du JSON, ou un champ attendu
## absent ou du mauvais type) est signalé dans le journal et ignoré.
func _receive(text: String) -> void:
	var message: Variant = Wire.decode(text)
	if not (message is Dictionary) or not _is_well_formed(message):
		push_warning("Message mal formé du serveur : %s" % text.left(LOGGED_CHARACTERS))
		return
	match message["type"]:
		"seated":
			_joining = false
			_save_seat(message)
		"state":
			state_received.emit(ViewData.new(message["view"]), message.get("events", []))
		"preview":
			var view: Variant = message["view"]
			preview_received.emit(ViewData.new(view) if view is Dictionary else null)
		"cards":
			cards_received.emit(CardCatalog.new(message["cards"]))
		"opponent":
			opponent_presence.emit(message["connected"])
		"error":
			_on_error(message["kind"], message["message"])


static func _is_well_formed(message: Dictionary) -> bool:
	match message.get("type"):
		"seated":
			return SEAT_FIELDS.all(func(field: String) -> bool: return message.get(field) is String)
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
	return false


## Un refus : la place gardée perdue est oubliée, une place reprise ailleurs arrête les
## reconnexions ; puis il est annoncé par sa catégorie.
func _on_error(kind: String, text: String) -> void:
	var category := ServerErrors.category(kind, _joining)
	_joining = false
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
