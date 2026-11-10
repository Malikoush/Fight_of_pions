extends Node
## La racine du client (`main.tscn`) : le plateau 3D, la couche d'interface par-dessus (le
## fond, puis les écrans), la session en cours, et l'enchaînement des écrans selon ce que
## répond le serveur (`ServerLink`) : démarrage → menu (→ rejoindre, ou salle d'attente) →
## draft → partie.
##
## Elle ne décide que du passage d'un écran à l'autre ; tout le reste est transmis à l'écran
## affiché par les méthodes de `Screen`. Elle ne connaît aucune règle : une vue dont la
## décision est un choix de draft s'affiche sur l'écran du draft, les autres sur celui de la
## partie.

## Le bot est celui choisi à l'accueil (`Settings.bot_opponent`). La variante est celle du
## jeu en ligne (draft compris), choisie par le serveur.
const FRIEND_OPPONENT := "human"
## Les écrans construits en scène.
const BOOT_SCENE := preload("res://ui/screens/boot_screen.tscn")
const MENU_SCENE := preload("res://ui/screens/menu_screen.tscn")
const JOIN_SCENE := preload("res://ui/screens/join_screen.tscn")
const LOBBY_SCENE := preload("res://ui/screens/lobby_screen.tscn")
const DRAFT_SCENE := preload("res://ui/screens/draft_screen.tscn")

var _session := Session.new(Session.SAVE_PATH)
var _router: ScreenRouter

@onready var _board: Board3D = $Board
@onready var _backdrop: Backdrop = $Interface/Backdrop
@onready var _screens: Control = $Interface/Screens


func _ready() -> void:
	Settings.load_saved()
	RenderingServer.set_default_clear_color(Look.FOREST)
	add_child(Sounds.new())
	_screens.theme = ThemeBuilder.shared()
	# Une place gardée sera reprise : sa session aussi (contre qui, salle à lancer).
	if ServerLink.has_saved_seat():
		_session.restore()
	_router = ScreenRouter.new(_screens, _board, _backdrop, _session)

	ServerLink.status_changed.connect(_on_status_changed)
	ServerLink.state_received.connect(_on_state_received)
	ServerLink.preview_received.connect(func(view: ViewData) -> void: _router.current.on_preview(view))
	ServerLink.cards_received.connect(_on_cards_received)
	ServerLink.opponent_presence.connect(_on_opponent_presence)
	ServerLink.account_received.connect(_on_account_received)
	ServerLink.collection_received.connect(_on_collection_received)
	ServerLink.reward_received.connect(_on_reward_received)
	ServerLink.room_launch.connect(_on_room_launch)
	ServerLink.server_error.connect(_on_server_error)
	_open_boot()
	ServerLink.connect_to_server()


# --- Les écrans ---

func _open_boot() -> void:
	var boot: BootScreen = BOOT_SCENE.instantiate()
	boot.retry_requested.connect(ServerLink.retry_now)
	_router.show(boot)


## Le menu, sur son accueil : jouer, et les pages Cartes, Règles, Réglages.
func _open_menu() -> void:
	var menu: MenuScreen = MENU_SCENE.instantiate()
	menu.play_bot_requested.connect(_on_play_bot)
	menu.create_room_requested.connect(_on_create_room)
	menu.join_screen_requested.connect(_open_join)
	menu.join_requested.connect(_on_join)
	menu.pseudo_submitted.connect(ServerLink.set_pseudo)
	menu.cache_trio_submitted.connect(ServerLink.set_cache_trio)
	_router.show(menu)


func _open_join() -> void:
	var join: JoinScreen = JOIN_SCENE.instantiate()
	join.join_requested.connect(_on_join)
	join.back_requested.connect(_open_menu)
	_router.show(join)


func _open_lobby() -> void:
	var lobby: LobbyScreen = LOBBY_SCENE.instantiate()
	lobby.code = ServerLink.room_code()
	lobby.launch_requested.connect(_on_launch)
	lobby.home_requested.connect(_on_leave)
	_router.show(lobby)


func _open_draft() -> DraftScreen:
	var draft: DraftScreen = DRAFT_SCENE.instantiate()
	draft.play_requested.connect(ServerLink.play)
	draft.home_requested.connect(_on_leave)
	_router.show(draft)
	return draft


func _open_game() -> GameScreen:
	var game := GameScreen.new(_board)
	game.play_requested.connect(ServerLink.play)
	game.home_requested.connect(_on_leave)
	game.replay_requested.connect(_on_replay)
	game.reconnect_requested.connect(ServerLink.retry_now)
	_router.show(game)
	return game


## Une autre connexion a repris la place : la reprendre ici (la place gardée), ou revenir
## à l'accueil en l'oubliant.
func _open_seat_taken() -> void:
	var page := MessageScreen.new(Labels.SEAT_TAKEN_TITLE, Labels.SEAT_TAKEN_TEXT, Labels.RESUME_HERE, Labels.HOME, "partager")
	page.primary_pressed.connect(ServerLink.connect_to_server)
	page.secondary_pressed.connect(func() -> void:
		ServerLink.forget_seat()
		ServerLink.connect_to_server()
		_session.reset()
		_open_menu())
	_router.show(page)


# --- Les nouvelles du serveur ---

func _on_status_changed(status: ServerLink.Status) -> void:
	# Au démarrage, sans place gardée, on attend le menu ; avec une place gardée, la vue
	# arrive après la reprise, et ouvrira son écran.
	if _router.current is BootScreen and status == ServerLink.Status.ONLINE and not ServerLink.has_saved_seat():
		_open_menu()
		return
	_router.current.on_connection(status)


func _on_state_received(view: ViewData, events: Array) -> void:
	if _session.waiting_launch():
		# La salle n'est pas lancée : la vue attend, la salle d'attente s'affiche.
		_session.hold(view, events)
		if not (_router.current is LobbyScreen):
			_open_lobby()
		return
	_session.note_bot_failed(false)
	_screen_for(view).on_view(view, events)
	if view.is_over():
		# La salle fermera : il n'y aura plus de place à reprendre.
		ServerLink.forget_seat()


## L'écran qui montre cette vue : celui du draft pour un choix de draft, celui de la partie
## sinon ; ouvert s'il n'est pas déjà affiché.
func _screen_for(view: ViewData) -> Screen:
	if view.decision_kind() == DraftScreen.DRAFT:
		var draft := _router.current as DraftScreen
		return draft if draft != null else _open_draft()
	var game := _router.current as GameScreen
	return game if game != null else _open_game()


func _on_cards_received(cards: CardCatalog) -> void:
	_session.note_catalog(cards)
	_router.current.on_session_changed()


func _on_opponent_presence(connected: bool) -> void:
	_session.note_opponent_present(connected)
	_router.current.on_session_changed()


func _on_account_received(_id: int, pseudo: String) -> void:
	_session.note_account(pseudo)
	_router.current.on_session_changed()


func _on_collection_received(cards: Array, cache_trio: Array) -> void:
	_session.note_collection(CardCollection.new(cards, cache_trio))
	_router.current.on_session_changed()


## Une carte gagnée : l'écran la montre, et la collection est redemandée au serveur, qui
## seul la tient à jour.
func _on_reward_received(card: Dictionary) -> void:
	_router.current.on_reward(card)
	ServerLink.request_collection()


## Un refus du serveur : ce qui change d'écran est décidé ici, le reste par l'écran affiché.
func _on_server_error(category: ServerErrors.Category, message: String) -> void:
	print("Refus du serveur : %s" % message)
	match category:
		ServerErrors.Category.SEAT_TAKEN:
			_open_seat_taken()
			return
		ServerErrors.Category.SEAT_LOST:
			# La place gardée n'existe plus (ServerLink l'a oubliée).
			if not (_router.current is MenuScreen):
				_session.reset()
				_open_menu()
				return
		ServerErrors.Category.BOT_DOWN:
			# La partie attend ; le prochain message relance le bot (protocole).
			_session.note_bot_failed(true)
			_router.current.on_session_changed()
			get_tree().create_timer(Look.BOT_RETRY_DELAY).timeout.connect(ServerLink.sync)
			return
		ServerErrors.Category.TRIO_REFUSED:
			# La collection du client est sans doute en retard : la redemander.
			ServerLink.request_collection()
	if _router.current.on_refused(category, message) == Screen.Reply.RESYNC:
		ServerLink.sync()


# --- Les demandes des écrans ---

func _on_play_bot() -> void:
	_session.start_bot()
	ServerLink.create_room(Settings.bot_opponent)


func _on_create_room() -> void:
	_session.start_host()
	ServerLink.create_room(FRIEND_OPPONENT)


func _on_join(code: String) -> void:
	_session.start_guest()
	ServerLink.join_room(code)


## Le créateur de la salle la lance : le serveur l'annoncera aux deux joueurs.
func _on_launch() -> void:
	ServerLink.launch_room()


## Le serveur dit où en est le lancement de la salle : une fois lancée, la vue gardée
## s'affiche (le draft commence), chez le créateur comme chez son ami.
func _on_room_launch(launched: bool) -> void:
	var held := _session.note_launched(launched)
	if held.size() == 2:
		_on_state_received(held[0], held[1])


## Quitter la partie (ou la salle) : la place est libérée, retour à l'accueil.
func _on_leave() -> void:
	ServerLink.leave_room()
	_session.reset()
	_open_menu()


## Une autre partie contre le même adversaire : le bot tout de suite ; entre amis, retour à
## l'accueil pour recréer ou rejoindre une salle.
func _on_replay() -> void:
	var against_bot := _session.against_bot()
	_on_leave()
	if against_bot:
		_on_play_bot()
