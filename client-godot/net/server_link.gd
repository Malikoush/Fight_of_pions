extends Node
## Le lien avec le serveur (autoload `ServerLink`) : une WebSocket, un message JSON par trame
## (docs/PROTOCOLE.md). Il ne connaît aucune règle : il envoie les demandes du client et
## annonce ce que le serveur répond. Il garde la place (code, joueur, clé) sur le disque et
## la reprend tout seul après une coupure, ou au lancement suivant.
## À chaque ouverture, il demande le catalogue des cartes (noms et textes à afficher).
##
## Rejoindre une salle par son code arrivera avec l'étape qui s'en sert (E3f).

## L'état de la connexion.
enum Status { OFFLINE, CONNECTING, ONLINE, RECONNECTING }

signal status_changed(status: Status)
## La partie a changé : la vue entière, et ce qui s'est passé depuis la précédente.
signal state_received(view: ViewData, events: Array)
## La réponse à un aperçu : la vue après ces coups, ou `null` si la carte est complète.
signal preview_received(view: ViewData)
## Le catalogue des cartes : les familles, leurs noms et leurs textes.
signal cards_received(cards: CardCatalog)
## Une demande refusée (`kind` est stable, `message` est en français).
signal server_error(kind: String, message: String)

## L'adresse du serveur lancé par `cargo serveur` ; réglable dans Réglages plus tard (E3h).
## En IPv4 : le serveur n'écoute que là, et « localhost » peut désigner l'IPv6 sous Windows.
const DEFAULT_URL := "ws://127.0.0.1:3000/ws"
const SESSION_PATH := "user://session.cfg"
const SESSION_SECTION := "place"
const SESSION_FIELDS := ["code", "player", "key"]
## Une vue complète peut approcher la mémoire de réception par défaut de Godot (64 Kio).
const INBOUND_BUFFER_BYTES := 1 << 20
## Les attentes entre deux essais de connexion, en secondes ; la dernière se répète.
const RETRY_DELAYS: Array[float] = [0.5, 1.0, 2.0, 4.0]
## Les erreurs qui disent que la place gardée n'existe plus : on l'oublie.
const LOST_SEAT_ERRORS := ["unknown_room", "bad_key", "room_full"]
## Une autre connexion a repris la place : on arrête de se reconnecter.
const SEAT_TAKEN := "seat_taken"
## Les demandes qui peuvent attendre l'ouverture de la connexion. Un coup préparé avant une
## coupure, lui, est perdu : la vue reçue au retour fait foi.
const QUEUED_REQUESTS := ["create"]
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
var _session := {}
var _rejoining := false
var _reopen_after_close := false


func _ready() -> void:
	_load_session()


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
func has_session() -> bool:
	return not _session.is_empty()


func forget_session() -> void:
	_session = {}
	DirAccess.remove_absolute(ProjectSettings.globalize_path(SESSION_PATH))


## Quitter sa salle : la place est oubliée et une connexion neuve s'ouvre, car le serveur
## n'accepte qu'une salle par connexion. Une demande faite d'ici là attend la nouvelle.
func leave_room() -> void:
	forget_session()
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
	if has_session():
		_rejoining = true
		_write({"type": "rejoin", "code": _session["code"], "player": _session["player"], "key": _session["key"]})
	var waiting := _outbox
	_outbox = []
	for text in waiting:
		_socket.send_text(text)


func _on_closed() -> void:
	_socket = null
	_rejoining = false
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
			_rejoining = false
			_save_session(message)
		"state":
			state_received.emit(ViewData.new(message["view"]), message.get("events", []))
		"preview":
			var view: Variant = message["view"]
			preview_received.emit(ViewData.new(view) if view is Dictionary else null)
		"cards":
			cards_received.emit(CardCatalog.new(message["cards"]))
		"error":
			_on_error(message["kind"], message["message"])


static func _is_well_formed(message: Dictionary) -> bool:
	match message.get("type"):
		"seated":
			return SESSION_FIELDS.all(func(field: String) -> bool: return message.get(field) is String)
		"state":
			return message.get("view") is Dictionary and message.get("events", []) is Array
		"error":
			return message.get("kind") is String and message.get("message") is String
		"preview":
			return message.has("view") and (message["view"] == null or message["view"] is Dictionary)
		"cards":
			return message.get("cards") is Array
	return false


func _on_error(kind: String, text: String) -> void:
	if _rejoining and kind in LOST_SEAT_ERRORS:
		_rejoining = false
		forget_session()
	if kind == SEAT_TAKEN:
		disconnect_from_server()
	server_error.emit(kind, text)


func _set_status(new_status: Status) -> void:
	if new_status != status:
		status = new_status
		status_changed.emit(status)


func _save_session(message: Dictionary) -> void:
	_session = {}
	var file := ConfigFile.new()
	for field: String in SESSION_FIELDS:
		_session[field] = message[field]
		file.set_value(SESSION_SECTION, field, message[field])
	file.save(SESSION_PATH)


func _load_session() -> void:
	var file := ConfigFile.new()
	if file.load(SESSION_PATH) != OK:
		return
	var loaded := {}
	for field: String in SESSION_FIELDS:
		loaded[field] = file.get_value(SESSION_SECTION, field, "")
	if loaded.values().all(func(value: Variant) -> bool: return value is String and value != ""):
		_session = loaded
