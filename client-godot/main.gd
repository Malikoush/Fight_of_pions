extends Node
## La racine du client : le plateau 3D, la couche d'interface par-dessus, et l'écran
## affiché. Elle enchaîne les écrans selon ce que répond le serveur (`ServerLink`) :
## démarrage → accueil (→ rejoindre une salle, ou salle d'attente) → draft → partie. Elle
## ne connaît aucune règle : une vue dont la décision est un choix de draft s'affiche sur
## l'écran du draft. Celui qui crée une salle attend son ami dans la salle d'attente, et la
## partie ne s'affiche qu'une fois qu'il l'a lancée.

## Le temps des essais, le bot est le bot aléatoire, qui ne charge pas la machine. La
## variante est celle du jeu en ligne (draft compris), choisie par le serveur.
const BOT_OPPONENT := "random"
const FRIEND_OPPONENT := "human"
## Les refus qui concernent un code tapé : la page « Rejoindre » les affiche.
const JOIN_ERRORS := ["unknown_room", "room_full"]
## Le bot du serveur ne peut plus jouer.
const BOT_FAILED := "bot_failed"
## Après autant d'essais ratés, l'écran de démarrage dit que le serveur ne répond pas.
const FAILURES_BEFORE_WARNING := 3
## Les refus qui veulent dire « ta vue est en retard » : on la redemande.
const STALE_VIEW_ERRORS := ["not_your_turn", "refused_move"]

var _board: Board3D
var _ui: Control
var _screen: Control
var _opponent_name := Labels.OPPONENT
var _opponent_is_bot := false
## Vrai quand on a créé une salle entre amis et qu'on ne l'a pas encore lancée : les vues
## reçues attendent (`_held`), la salle d'attente s'affiche.
var _hosting := false
var _held: Array = []
## Vrai quand l'adversaire humain est branché sur sa place (message du serveur).
var _opponent_present := false
## Vrai depuis que le bot du serveur a cessé de jouer, jusqu'à la vue suivante.
var _bot_failed := false
## Le catalogue des cartes (noms et textes), reçu à chaque connexion.
var _catalog: CardCatalog


func _ready() -> void:
	RenderingServer.set_default_clear_color(Look.FOREST)
	_board = Board3D.new()
	_board.name = "Board"
	_board.visible = false
	add_child(_board)
	var layer := CanvasLayer.new()
	layer.name = "Interface"
	add_child(layer)
	_ui = Control.new()
	_ui.name = "Screens"
	_ui.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_ui.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_ui.theme = ThemeBuilder.shared()
	layer.add_child(_ui)

	ServerLink.status_changed.connect(_on_status_changed)
	ServerLink.state_received.connect(_on_state_received)
	ServerLink.server_error.connect(_on_server_error)
	ServerLink.preview_received.connect(_on_preview_received)
	ServerLink.cards_received.connect(_on_cards_received)
	ServerLink.opponent_presence.connect(_on_opponent_presence)
	_show_boot()
	ServerLink.connect_to_server()


func _show_boot() -> void:
	var boot := BootScreen.new()
	boot.retry_requested.connect(ServerLink.retry_now)
	_show(boot)


func _show_home() -> void:
	var home := HomeScreen.new()
	home.play_bot_requested.connect(_on_play_bot)
	home.create_room_requested.connect(_on_create_room)
	home.join_requested.connect(_show_join)
	_show(home)


func _show_join() -> void:
	var join := JoinScreen.new()
	join.join_requested.connect(_on_join)
	join.back_requested.connect(_show_home)
	_show(join)


func _show_lobby() -> void:
	var lobby := LobbyScreen.new(ServerLink.room_code())
	lobby.launch_requested.connect(_on_launch)
	lobby.home_requested.connect(_on_home_requested)
	_show(lobby)
	lobby.set_opponent_present(_opponent_present)


func _show_draft() -> DraftScreen:
	var draft := DraftScreen.new()
	draft.opponent_name = _opponent_name
	draft.catalog = _catalog
	draft.play_requested.connect(ServerLink.play)
	draft.home_requested.connect(_on_home_requested)
	_show(draft)
	return draft


func _show_game() -> GameScreen:
	var game := GameScreen.new(_board)
	game.opponent_name = _opponent_name
	game.opponent_is_bot = _opponent_is_bot
	_hosting = false
	game.catalog = _catalog
	game.play_requested.connect(ServerLink.play)
	game.home_requested.connect(_on_home_requested)
	game.replay_requested.connect(_on_replay)
	_show(game)
	return game


## Remplace l'écran affiché, en fondu.
func _show(screen: Control) -> void:
	if _screen != null:
		var old := _screen
		var fade := old.create_tween()
		fade.tween_property(old, "modulate:a", 0.0, Look.SCREEN_FADE)
		fade.tween_callback(old.queue_free)
	_screen = screen
	_board.visible = screen is GameScreen
	screen.modulate.a = 0.0
	_ui.add_child(screen)
	screen.create_tween().tween_property(screen, "modulate:a", 1.0, Look.SCREEN_FADE)


func _on_status_changed(status: ServerLink.Status) -> void:
	var online := status == ServerLink.Status.ONLINE
	if _screen is BootScreen:
		if online:
			# Avec une place gardée, la vue arrive après la reprise : on l'attend.
			if not ServerLink.has_session():
				_show_home()
		else:
			var lost := ServerLink.failed_attempts >= FAILURES_BEFORE_WARNING
			(_screen as BootScreen).set_status(Labels.UNREACHABLE if lost else Labels.CONNECTING, lost)
	elif _screen is GameScreen:
		if not online:
			(_screen as GameScreen).on_connection_lost()
		_refresh_notice()
	elif _screen is HomeScreen and status == ServerLink.Status.RECONNECTING:
		# Une coupure a pu perdre la demande de partie : on peut la refaire.
		(_screen as HomeScreen).reset()


func _on_state_received(view: ViewData, events: Array) -> void:
	if _hosting:
		# La salle n'est pas lancée : la vue attend, la salle d'attente s'affiche.
		_held = [view, events]
		if not (_screen is LobbyScreen):
			_show_lobby()
		return
	if view.decision_kind() == DraftScreen.DRAFT:
		var draft: DraftScreen = _screen if _screen is DraftScreen else _show_draft()
		draft.show_view(view, events)
		return
	var game: GameScreen = _screen if _screen is GameScreen else _show_game()
	game.show_view(view, events)
	_bot_failed = false
	_refresh_notice()
	if view.is_over():
		# La salle fermera : il n'y aura plus de place à reprendre.
		ServerLink.forget_session()


## La réponse à un aperçu : elle sert au choix de la carte en cours, sur l'écran de partie.
func _on_preview_received(view: ViewData) -> void:
	if _screen is GameScreen:
		(_screen as GameScreen).on_preview(view)


## Le catalogue des cartes, reçu à chaque connexion : gardé pour les écrans à venir.
func _on_cards_received(cards: CardCatalog) -> void:
	_catalog = cards
	if _screen is GameScreen:
		(_screen as GameScreen).catalog = cards
	elif _screen is DraftScreen:
		(_screen as DraftScreen).catalog = cards


## Un refus du serveur. L'affichage de son message viendra avec les fenêtres d'erreur (E3g) ;
## d'ici là, il est noté dans le journal (un refus est une réponse normale du serveur, pas
## un défaut du client) et l'écran redevient jouable. Quand la vue peut être en retard (un
## refus qui le dit, ou une carte refusée en cours d'envoi), on la redemande.
func _on_server_error(kind: String, message: String) -> void:
	print("Refus du serveur (%s) : %s" % [kind, message])
	if kind == ServerLink.SEAT_TAKEN:
		_show_seat_taken()
	elif kind == BOT_FAILED:
		# Le prochain message relance le bot (protocole) : on le redemande après un temps.
		_bot_failed = true
		_refresh_notice()
		get_tree().create_timer(Look.BOT_RETRY_DELAY).timeout.connect(ServerLink.sync)
	elif _screen is JoinScreen and kind in JOIN_ERRORS:
		(_screen as JoinScreen).show_error(message)
	elif kind in ServerLink.LOST_SEAT_ERRORS and not (_screen is HomeScreen):
		_show_home()
	elif _screen is GameScreen:
		var was_sending := (_screen as GameScreen).on_refused()
		if was_sending or kind in STALE_VIEW_ERRORS:
			ServerLink.sync()
	elif _screen is DraftScreen:
		(_screen as DraftScreen).on_refused()
		ServerLink.sync()
	elif _screen is HomeScreen:
		(_screen as HomeScreen).reset()
		(_screen as HomeScreen).show_error(message)


## L'adversaire est arrivé, ou parti : la salle d'attente le montre, la partie aussi.
func _on_opponent_presence(connected: bool) -> void:
	_opponent_present = connected
	if _screen is LobbyScreen:
		(_screen as LobbyScreen).set_opponent_present(connected)
	_refresh_notice()


## L'avertissement de la partie, sous le tour : la connexion d'abord, puis le bot qui ne
## répond plus, puis l'ami parti ; rien si tout va bien.
func _refresh_notice() -> void:
	if not (_screen is GameScreen):
		return
	var text := ""
	if ServerLink.status != ServerLink.Status.ONLINE:
		text = Labels.RECONNECTING
	elif _bot_failed:
		text = Labels.BOT_FAILED
	elif not _opponent_is_bot and not _opponent_present:
		text = Labels.FRIEND_LEFT
	(_screen as GameScreen).set_notice(text, text != "")


## Une autre connexion a repris la place : reprendre ici (la place gardée), ou revenir à
## l'accueil en l'oubliant.
func _show_seat_taken() -> void:
	var page := MessageScreen.new(Labels.SEAT_TAKEN_TITLE, Labels.SEAT_TAKEN_TEXT, Labels.RESUME_HERE, Labels.HOME)
	page.primary_pressed.connect(ServerLink.connect_to_server)
	page.secondary_pressed.connect(func() -> void:
		ServerLink.forget_session()
		ServerLink.connect_to_server()
		_on_home_requested())
	_show(page)


## Une autre partie, contre le même adversaire : le bot tout de suite ; entre amis, retour à
## l'accueil pour recréer ou rejoindre une salle.
func _on_replay() -> void:
	var against_bot := _opponent_is_bot
	_on_home_requested()
	if against_bot:
		_on_play_bot()


## Le créateur de la salle la lance : la vue gardée s'affiche (le draft commence).
func _on_launch() -> void:
	_hosting = false
	if _held.size() == 2:
		_on_state_received(_held[0], _held[1])
	_held = []


func _on_play_bot() -> void:
	_opponent_name = Labels.BOT
	_opponent_is_bot = true
	_hosting = false
	ServerLink.create_room(BOT_OPPONENT)


func _on_create_room() -> void:
	_opponent_name = Labels.FRIEND
	_opponent_present = false
	_opponent_is_bot = false
	_hosting = true
	ServerLink.create_room(FRIEND_OPPONENT)


func _on_join(code: String) -> void:
	_opponent_name = Labels.FRIEND
	_opponent_present = false
	_opponent_is_bot = false
	_hosting = false
	ServerLink.join_room(code)


func _on_home_requested() -> void:
	ServerLink.leave_room()
	_opponent_name = Labels.OPPONENT
	_opponent_is_bot = false
	_hosting = false
	_held = []
	_opponent_present = false
	_show_home()
