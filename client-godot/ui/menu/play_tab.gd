class_name PlayTab
extends VBoxContainer
## L'onglet Jouer du menu, l'accueil (maquettes 1-5) :
## - en portrait : le logo et la pastille du joueur, le panneau du plateau avec « JOUER
##   CONTRE LE BOT », puis la partie entre amis en deux cartes : créer une salle, ou
##   rejoindre (l'écran du code) ;
## - en paysage : le panneau du plateau avec sa phrase d'accroche, et à sa droite celui de
##   la partie entre amis, où l'on crée une salle ou tape directement le code.
##
## La scène (`play_tab.tscn`) porte les deux mises en page : selon l'orientation, des
## éléments se montrent ou se cachent, et deux grilles passent d'une à deux colonnes. Un
## geste qui attend le serveur grise les boutons jusqu'à sa réponse.
##
## Au-dessus de « JOUER CONTRE LE BOT », des pastilles choisissent le bot (facile : le bot
## aléatoire ; difficile : le MCTS, qui fait gagner une carte), gardé sur l'appareil
## (`Settings.bot_opponent`) ; la racine le lit pour créer la salle.

signal play_bot_requested
signal create_room_requested
## Rejoindre, en portrait : le code se tape sur son propre écran.
signal join_screen_requested
## Rejoindre, en paysage : le code est tapé ici.
signal join_requested(code: String)
signal player_pressed

var _waiting := false
var _landscape := false
var _arranged := false
## La dernière demande faite au serveur, à refaire si le serveur était complet.
var _last_request := Callable()
var _create_card: RoomCard
var _join_card: RoomCard

@onready var _header: HBoxContainer = %Header
@onready var _logo: Label = %Logo
@onready var _pill: PlayerPill = %Pill
@onready var _error: Label = %Error
@onready var _body: GridContainer = %Body
@onready var _play_panel: PanelContainer = %PlayPanel
@onready var _preview: BoardPreview = %Preview
@onready var _play_row: GridContainer = %PlayRow
@onready var _pitch: VBoxContainer = %Pitch
@onready var _slogan: Label = %Slogan
@onready var _tagline: Label = %Tagline
@onready var _play_box: VBoxContainer = %PlayBox
@onready var _play_button: Button = %PlayButton
@onready var _play_caption: Label = %PlayCaption
@onready var _friends: VBoxContainer = %Friends
@onready var _friends_title: Label = %FriendsTitle
@onready var _choices: HBoxContainer = %Choices
@onready var _friends_panel: PanelContainer = %FriendsPanel
@onready var _panel_title: Label = %PanelTitle
@onready var _intro: Label = %Intro
@onready var _create_button: Button = %CreateButton
@onready var _or: Label = %Or
@onready var _join_label: Label = %JoinLabel
@onready var _code: CodeField = %Code
@onready var _hint: Label = %Hint
@onready var _join_error: Label = %JoinError
@onready var _join_button: Button = %JoinButton
@onready var _tip_icon: TextureRect = %TipIcon
@onready var _tip_text: Label = %TipText


func _ready() -> void:
	_fill_texts()
	_create_card = RoomCard.new("partager", Labels.CREATE_ROOM, Labels.CREATE_ROOM_DETAIL)
	_join_card = RoomCard.new("rejoindre", Labels.JOIN, Labels.JOIN_DETAIL)
	for card: RoomCard in [_create_card, _join_card]:
		card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		_choices.add_child(card)
	_create_button.icon = load(Ui.ICON_DIR + "partager.svg")
	_tip_icon.texture = load(Ui.ICON_DIR + "connexion.svg")
	_tip_icon.modulate = Look.GOLD
	_tip_icon.custom_minimum_size = Vector2.ONE * Look.ICON_SIZE
	_friends_panel.custom_minimum_size.x = Look.FRIENDS_PANEL_WIDTH
	for button: Button in [_play_button, _join_button]:
		button.custom_minimum_size.y = Look.BUTTON_HEIGHT

	_play_box.add_child(_bot_choice())
	_play_box.move_child(_play_box.get_child(-1), 0)
	_play_button.pressed.connect(_request.bind(play_bot_requested))
	_create_card.pressed.connect(_request.bind(create_room_requested))
	_create_button.pressed.connect(_request.bind(create_room_requested))
	_join_card.pressed.connect(join_screen_requested.emit)
	_join_button.pressed.connect(_join)
	_code.submitted.connect(func(_code_text: String) -> void: _join())
	_code.changed.connect(func(_code_text: String) -> void:
		_join_error.visible = false
		_refresh_buttons())
	_pill.pressed.connect(player_pressed.emit)
	resized.connect(_arrange)
	visibility_changed.connect(_focus_code)
	_arrange()
	_refresh_buttons()


## Le pseudo a changé : la pastille le reprend.
func refresh_player(pseudo: String) -> void:
	_pill.show_pseudo(pseudo)


## Une coupure a pu perdre la demande : les boutons se réactivent.
func on_connection_lost() -> void:
	_set_waiting(false)


## Un refus du serveur : son message, sous le code s'il le concerne (en paysage, les cases
## bordées de braise), en haut sinon ; les boutons de nouveau permis.
func show_refusal(category: ServerErrors.Category, message: String) -> void:
	_set_waiting(false)
	var about_code := category == ServerErrors.Category.JOIN_REFUSED and _landscape
	var label := _join_error if about_code else _error
	label.text = Labels.sentence(message)
	label.visible = true
	if about_code:
		_code.show_refused()


func _fill_texts() -> void:
	_logo.text = Labels.GAME_NAME
	_slogan.text = Labels.SLOGAN
	_tagline.text = Labels.TAGLINE
	_play_button.text = Labels.PLAY_BOT
	_play_caption.text = Labels.BOT_DETAILS[Settings.bot_opponent]
	_friends_title.text = Labels.FRIENDS_SECTION
	_panel_title.text = Labels.FRIENDS_TITLE
	_intro.text = Labels.FRIENDS_INTRO
	_create_button.text = Labels.CREATE_ROOM
	_or.text = Labels.OR
	_join_label.text = Labels.JOIN_WITH_CODE
	_hint.text = Labels.CODE_HINT
	_join_button.text = Labels.JOIN
	_tip_text.text = Labels.RESUME_TIP


## Les pastilles du bot à affronter, centrées ; le choix est gardé, la phrase sous le
## bouton le suit.
func _bot_choice() -> HBoxContainer:
	var row := Ui.hbox(Look.GAP_S)
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	var choices := {}
	for bot: String in Settings.BOT_OPPONENTS:
		choices[bot] = Labels.BOT_LEVELS[bot]
	Ui.add_filter_pills(row, choices, Settings.bot_opponent, func(bot: String) -> void:
		Settings.bot_opponent = bot
		Settings.save()
		_play_caption.text = Labels.BOT_DETAILS[bot])
	return row


## Refait la dernière demande (le serveur était complet).
func retry() -> void:
	if _last_request.is_valid():
		_last_request.call()


## Un geste qui attend le serveur : les boutons se grisent jusqu'à sa réponse.
func _request(request: Signal) -> void:
	_last_request = _request.bind(request)
	_error.visible = false
	_set_waiting(true)
	request.emit()


func _join() -> void:
	if _waiting or not _code.is_complete():
		return
	_last_request = _join
	_join_error.visible = false
	_set_waiting(true)
	join_requested.emit(_code.code())


func _set_waiting(waiting: bool) -> void:
	_waiting = waiting
	_refresh_buttons()


func _refresh_buttons() -> void:
	for button: Button in [_play_button, _create_card, _join_card, _create_button]:
		button.disabled = _waiting
	_join_button.disabled = _waiting or not _code.is_complete()


## Range l'onglet pour l'orientation de la fenêtre.
func _arrange() -> void:
	var landscape := Ui.is_landscape(self)
	if _arranged and landscape == _landscape:
		return
	_arranged = true
	_landscape = landscape
	# En paysage, le logo et la pastille sont dans la barre du haut.
	_header.visible = not landscape
	_body.columns = 2 if landscape else 1
	# En portrait, le plateau est une bande, et la place qui reste va en bas (maquettes) ;
	# en paysage, il prend toute la hauteur.
	_play_panel.size_flags_vertical = Control.SIZE_EXPAND_FILL if landscape else Control.SIZE_FILL
	_preview.aspect = 0.0 if landscape else Look.PREVIEW_ASPECT
	_friends.visible = not landscape
	_friends_panel.visible = landscape
	_pitch.visible = landscape
	_play_row.columns = 2 if landscape else 1
	_play_box.size_flags_horizontal = Control.SIZE_SHRINK_END if landscape else Control.SIZE_EXPAND_FILL
	_focus_code()


## Sur un PC, en paysage, le code se tape sans toucher les cases ; sur un téléphone, le
## clavier ne s'ouvre qu'au toucher.
func _focus_code() -> void:
	if _landscape and is_visible_in_tree() and not DisplayServer.has_feature(DisplayServer.FEATURE_VIRTUAL_KEYBOARD):
		_code.focus()
