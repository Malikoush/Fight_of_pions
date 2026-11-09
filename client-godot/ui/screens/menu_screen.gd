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


## Un refus (serveur complet, code inconnu…) répond à un geste de l'accueil : il s'y affiche.
func on_refused(category: ServerErrors.Category, message: String) -> Reply:
	if not _pages.has(Tab.PLAY):
		return Reply.IGNORED
	(_pages[Tab.PLAY] as PlayTab).show_refusal(category, message)
	return Reply.HANDLED


## Le catalogue est arrivé : la page Cartes, déjà ouverte, est refaite avec lui.
func on_session_changed() -> void:
	if not _pages.has(Tab.CARDS) or session.catalog == _shown_catalog:
		return
	_pages[Tab.CARDS].queue_free()
	_pages.erase(Tab.CARDS)
	if _tab == Tab.CARDS:
		open_tab(Tab.CARDS)


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
			return RulesPage.new()
		Tab.SETTINGS:
			return SettingsPage.new()
	return _build_play()


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
