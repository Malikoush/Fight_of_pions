class_name LobbyScreen
extends Screen
## La salle d'attente d'une partie entre amis (maquettes 1-5, p. 4), chez ses deux joueurs :
## le code de la salle en cases, les deux joueurs (toi prêt sur ta case ; ton ami en attente,
## sa place en pointillés, puis prêt), qui commence (tiré au sort par le serveur), et :
## - chez son créateur : « COPIER », « PARTAGER », et « LANCER LA PARTIE », permis quand son
##   ami est là ; « FERMER LA SALLE » ;
## - chez son ami : l'attente du lancement ; « QUITTER LA SALLE ».
## La partie (son draft) ne s'affiche qu'une fois lancée, chez les deux. La flèche revient à
## l'accueil.
##
## Le téléphone n'a pas encore de partage natif (R9) : « PARTAGER » copie un message
## d'invitation, prêt à coller.

signal launch_requested
signal home_requested

## Le numéro du dernier appui sur un bouton qui copie : seul son minuteur lui rend son nom.
const COPY_PRESS := &"copy_press"

## Le code de la salle : à régler avant l'entrée dans l'arbre.
var code := ""

@onready var _page: MarginContainer = %Page
@onready var _header: PageHeader = %Header
@onready var _code_title: Label = %CodeTitle
@onready var _code: CodeBoxes = %Code
@onready var _copy: Button = %Copy
@onready var _share: Button = %Share
@onready var _me: LobbySeat = %Me
@onready var _versus: Label = %Versus
@onready var _friend: LobbySeat = %Friend
@onready var _help: Label = %Help
@onready var _launch: Button = %Launch
@onready var _close: Button = %Close


func _ready() -> void:
	Ui.fit_page(_page, Look.CONTENT_MAX_WIDTH)
	var hosting := session.hosting()
	_header.set_title(Labels.LOBBY_TITLE if hosting else Labels.LOBBY_GUEST_TITLE)
	_header.back_pressed.connect(home_requested.emit)
	_code_title.text = Labels.ROOM_CODE
	_code.show_code(code)
	_copy.text = Labels.COPY
	_copy.icon = load(Ui.ICON_DIR + "copier.svg")
	_share.text = Labels.SHARE
	_versus.text = Labels.VERSUS
	_launch.text = Labels.LAUNCH
	_close.text = Labels.CLOSE_ROOM if hosting else Labels.LEAVE_ROOM
	# Seul le créateur partage le code et lance la partie.
	for button: Button in [_copy, _share, _launch]:
		button.visible = hosting
	for button: Button in [_copy, _share, _launch]:
		button.custom_minimum_size.y = Look.BUTTON_HEIGHT
	_copy.pressed.connect(_copy_to_clipboard.bind(_copy, code, Labels.COPY, Labels.COPIED))
	_share.pressed.connect(_copy_to_clipboard.bind(_share, Labels.INVITE % code, Labels.SHARE, Labels.INVITE_COPIED))
	_launch.pressed.connect(launch_requested.emit)
	_close.pressed.connect(home_requested.emit)
	on_session_changed()


## Ton ami est là, ou n'y est plus : sa place, l'aide et « LANCER LA PARTIE » suivent.
func on_session_changed() -> void:
	var present := session.opponent_present
	var mine := ServerLink.seat_player()
	_me.show_seat(mine, session.pseudo, true)
	_friend.show_seat(ViewData.other(mine), session.opponent_name, present)
	var status: String
	if session.hosting():
		status = Labels.LOBBY_READY if present else Labels.LOBBY_WAITING
	else:
		status = Labels.LOBBY_GUEST_READY if present else Labels.LOBBY_GUEST_ALONE
	var first := Labels.LOBBY_YOU_START if mine == ViewData.PLAYER_ONE else Labels.LOBBY_FRIEND_STARTS
	_help.text = "%s\n%s" % [status, first]
	_launch.disabled = not present


## Copie ce texte ; le bouton le dit un instant, puis reprend son nom (un nouvel appui
## relance l'instant).
func _copy_to_clipboard(button: Button, text: String, label: String, done: String) -> void:
	DisplayServer.clipboard_set(text)
	button.text = done
	var press := int(button.get_meta(COPY_PRESS, 0)) + 1
	button.set_meta(COPY_PRESS, press)
	get_tree().create_timer(Look.COPIED_FEEDBACK).timeout.connect(func() -> void:
		if is_instance_valid(button) and button.get_meta(COPY_PRESS) == press:
			button.text = label)
