class_name MenuScreen
extends Screen
## Le menu (maquettes 1-5 et 12-17) : la barre de navigation et l'onglet ouvert : Jouer
## (l'accueil), Cartes, Règles, Réglages. Changer d'onglet ne change pas d'écran : la barre
## reste, seul le contenu change, sans fondu, comme dans une application. Chaque onglet est
## construit à sa première ouverture, puis gardé (un code à moitié tapé reste là).
##
## En portrait, la barre est en bas ; en paysage, en haut (`NavBar`).

signal play_bot_requested
signal create_room_requested
signal join_screen_requested
signal join_requested(code: String)

enum Tab { PLAY, CARDS, RULES, SETTINGS }

## L'icône et le nom de chaque onglet, dans l'ordre de `Tab`.
const TABS := [
	["jouer", Labels.NAV_PLAY],
	["cartes", Labels.NAV_CARDS],
	["regles", Labels.NAV_RULES],
	["reglages", Labels.NAV_SETTINGS],
]
const PLAY_TAB := preload("res://ui/menu/play_tab.tscn")

var _tab := Tab.PLAY
## Le serveur est complet (maquettes) : « RÉGLAGES », ou « RÉESSAYER » la demande.
var _server_full := AlertDialog.new(Labels.SERVER_FULL_TITLE, Labels.SERVER_FULL_TEXT, Labels.NAV_SETTINGS, Labels.RETRY, "connexion", true)
var _pages: Dictionary[Tab, Control] = {}
## Le catalogue que montre la page Cartes, pour la refaire quand il arrive.
var _shown_catalog: CardCatalog

@onready var _layout: VBoxContainer = %Layout
@onready var _content: MarginContainer = %Content
@onready var _nav: NavBar = %Nav


func _ready() -> void:
	_nav.setup(TABS)
	_nav.tab_selected.connect(func(index: int) -> void: open_tab(index as Tab))
	_nav.player_pressed.connect(open_tab.bind(Tab.SETTINGS))
	_server_full.declined.connect(open_tab.bind(Tab.SETTINGS))
	_server_full.confirmed.connect(func() -> void:
		open_tab(Tab.PLAY)
		(_pages[Tab.PLAY] as PlayTab).retry())
	add_child(_server_full)
	resized.connect(_arrange)
	_arrange()
	open_tab(_tab)


## Ouvre cet onglet (et allume le sien dans la barre).
func open_tab(tab: Tab) -> void:
	_tab = tab
	_nav.select(tab)
	if not _pages.has(tab):
		_pages[tab] = _build(tab)
		_content.add_child(_pages[tab])
	for shown: Tab in _pages:
		_pages[shown].visible = shown == tab


## Une coupure a pu perdre la demande de partie : l'accueil réactive ses boutons.
func on_connection(status: ServerLink.Status) -> void:
	if status == ServerLink.Status.RECONNECTING and _pages.has(Tab.PLAY):
		(_pages[Tab.PLAY] as PlayTab).on_connection_lost()


## Un refus répond à un geste de l'accueil : le serveur complet, en fenêtre ; les autres
## (code inconnu…), sur l'accueil.
func on_refused(category: ServerErrors.Category, message: String) -> Reply:
	if not _pages.has(Tab.PLAY):
		return Reply.IGNORED
	var play := _pages[Tab.PLAY] as PlayTab
	if category == ServerErrors.Category.SERVER_FULL:
		play.on_connection_lost()
		_server_full.ask()
	else:
		play.show_refusal(category, message)
	return Reply.HANDLED


## Le catalogue est arrivé : les pages qui montrent des cartes (Cartes, Règles), déjà
## construites, sont refaites avec lui.
func on_session_changed() -> void:
	if session.catalog == _shown_catalog:
		return
	_shown_catalog = session.catalog
	for tab: Tab in [Tab.CARDS, Tab.RULES]:
		if _pages.has(tab):
			_pages[tab].queue_free()
			_pages.erase(tab)
	if _tab in [Tab.CARDS, Tab.RULES]:
		open_tab(_tab)


## Échap (ou le retour d'Android) ramène à l'accueil.
func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed(&"ui_cancel") and _tab != Tab.PLAY:
		open_tab(Tab.PLAY)
		get_viewport().set_input_as_handled()


func _build(tab: Tab) -> Control:
	match tab:
		Tab.CARDS:
			_shown_catalog = session.catalog
			return CardsPage.new(_shown_catalog)
		Tab.RULES:
			_shown_catalog = session.catalog
			var rules := RulesPage.new(_shown_catalog)
			rules.play_requested.connect(open_tab.bind(Tab.PLAY))
			return rules
		Tab.SETTINGS:
			var settings := SettingsPage.new()
			settings.player_name_changed.connect(_refresh_player)
			return settings
	return _build_play()


## Le pseudo a changé : les pastilles du joueur (barre du haut, accueil) le reprennent.
func _refresh_player() -> void:
	_nav.refresh_player()
	if _pages.has(Tab.PLAY):
		(_pages[Tab.PLAY] as PlayTab).refresh_player()


## L'accueil : ses gestes deviennent ceux du menu, que la racine écoute.
func _build_play() -> PlayTab:
	var play: PlayTab = PLAY_TAB.instantiate()
	play.play_bot_requested.connect(play_bot_requested.emit)
	play.create_room_requested.connect(create_room_requested.emit)
	play.join_screen_requested.connect(join_screen_requested.emit)
	play.join_requested.connect(join_requested.emit)
	play.player_pressed.connect(open_tab.bind(Tab.SETTINGS))
	return play


## En portrait, la barre est en bas et couvre le bord inaccessible du bas ; en paysage, en
## haut, et couvre celui du haut. Le contenu a les marges d'une page de l'autre côté.
func _arrange() -> void:
	var landscape := Ui.is_landscape(self)
	_nav.set_wide(landscape)
	_layout.move_child(_nav, 0 if landscape else -1)
	var insets := Ui.safe_insets()
	_nav.set_insets(insets.position.y if landscape else 0.0, 0.0 if landscape else insets.size.y)
	Ui.set_page_margins(_content, not landscape, landscape, Look.PAGE_MARGIN_WIDE if landscape else Look.PAGE_MARGIN)
