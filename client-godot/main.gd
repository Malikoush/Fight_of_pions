extends Node
## La racine du client : le plateau 3D, la couche d'interface par-dessus, et l'écran
## affiché. Elle enchaîne les écrans selon ce que répond le serveur (`ServerLink`) :
## démarrage → accueil → partie. Elle ne connaît aucune règle.

## Le temps des essais (E3a à E3e), le client joue la variante sans draft contre le bot
## aléatoire, qui ne charge pas la machine. Le draft et la variante en ligne arrivent en E3f.
const DEV_OPPONENT := "random"
const DEV_VARIANT := "règles actuelles"
## Après autant d'essais ratés, l'écran de démarrage dit que le serveur ne répond pas.
const FAILURES_BEFORE_WARNING := 3
## Les refus qui veulent dire « ta vue est en retard » : on la redemande.
const STALE_VIEW_ERRORS := ["not_your_turn", "refused_move"]

var _board: Board3D
var _ui: Control
var _screen: Control
var _opponent_name := Labels.OPPONENT
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
	_ui.theme = ThemeBuilder.build()
	layer.add_child(_ui)

	ServerLink.status_changed.connect(_on_status_changed)
	ServerLink.state_received.connect(_on_state_received)
	ServerLink.server_error.connect(_on_server_error)
	ServerLink.preview_received.connect(_on_preview_received)
	ServerLink.cards_received.connect(_on_cards_received)
	_show_boot()
	ServerLink.connect_to_server()


func _show_boot() -> void:
	var boot := BootScreen.new()
	boot.retry_requested.connect(ServerLink.retry_now)
	_show(boot)


func _show_home() -> void:
	var home := HomeScreen.new()
	home.play_bot_requested.connect(_on_play_bot)
	_show(home)


func _show_game() -> GameScreen:
	var game := GameScreen.new(_board)
	game.opponent_name = _opponent_name
	game.catalog = _catalog
	game.play_requested.connect(ServerLink.play)
	game.home_requested.connect(_on_home_requested)
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
		var game := _screen as GameScreen
		game.set_notice("" if online else Labels.RECONNECTING)
		if not online:
			game.on_connection_lost()
	elif _screen is HomeScreen and status == ServerLink.Status.RECONNECTING:
		# Une coupure a pu perdre la demande de partie : on peut la refaire.
		(_screen as HomeScreen).reset()


func _on_state_received(view: ViewData, events: Array) -> void:
	var game: GameScreen = _screen if _screen is GameScreen else _show_game()
	game.show_view(view, events)
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


## Un refus du serveur. L'affichage de son message viendra avec les fenêtres d'erreur (E3g) ;
## d'ici là, il est noté dans le journal (un refus est une réponse normale du serveur, pas
## un défaut du client) et l'écran redevient jouable. Quand la vue peut être en retard (un
## refus qui le dit, ou une carte refusée en cours d'envoi), on la redemande.
func _on_server_error(kind: String, message: String) -> void:
	print("Refus du serveur (%s) : %s" % [kind, message])
	if kind in ServerLink.LOST_SEAT_ERRORS and not (_screen is HomeScreen):
		_show_home()
	elif _screen is GameScreen:
		var was_sending := (_screen as GameScreen).on_refused()
		if was_sending or kind in STALE_VIEW_ERRORS:
			ServerLink.sync()
	elif _screen is HomeScreen:
		(_screen as HomeScreen).reset()


func _on_play_bot() -> void:
	_opponent_name = Labels.BOT
	ServerLink.create_room(DEV_OPPONENT, DEV_VARIANT)


func _on_home_requested() -> void:
	ServerLink.leave_room()
	_opponent_name = Labels.OPPONENT
	_show_home()
