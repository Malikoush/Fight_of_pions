class_name LobbyScreen
extends Control
## La salle d'attente d'une partie entre amis (maquettes) : le code de la salle à envoyer,
## « COPIER », les deux joueurs (toi prêt, ton ami en attente puis prêt), et « LANCER LA
## PARTIE », permis quand ton ami est là. La partie (son draft) ne s'affiche qu'une fois
## lancée. « FERMER LA SALLE » (ou la flèche) revient à l'accueil.

signal launch_requested
signal home_requested

var _friend_status: Label
var _help: Label
var _launch: Button
var _code := ""


func _init(code: String) -> void:
	_code = code


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_L)
	var header := Ui.hbox(Look.GAP_M)
	var back := Ui.icon_button("retour", Labels.BACK)
	back.pressed.connect(home_requested.emit)
	header.add_child(back)
	header.add_child(Ui.label(Labels.LOBBY_TITLE, ThemeBuilder.HEADING_LABEL))
	column.add_child(header)

	var code_panel := PanelContainer.new()
	code_panel.theme_type_variation = ThemeBuilder.PANEL
	var code_column := Ui.vbox(Look.GAP_S)
	code_column.add_child(Ui.centered_label(Labels.ROOM_CODE, ThemeBuilder.CAPS_LABEL))
	code_column.add_child(Ui.centered_label(_code, ThemeBuilder.CODE_LABEL))
	var copy := Ui.button(Labels.COPY, ThemeBuilder.SECONDARY_BUTTON)
	copy.pressed.connect(func() -> void: DisplayServer.clipboard_set(_code))
	code_column.add_child(copy)
	code_panel.add_child(code_column)
	column.add_child(code_panel)

	var players := Ui.hbox(Look.GAP_L)
	players.alignment = BoxContainer.ALIGNMENT_CENTER
	players.add_child(_player(ViewData.PLAYER_ONE, Labels.YOU, Labels.PLAYER_READY, ThemeBuilder.GOLD_CAPS_LABEL))
	players.add_child(Ui.label(Labels.VERSUS, ThemeBuilder.HEADING_LABEL))
	var friend := _player(ViewData.PLAYER_TWO, Labels.FRIEND, Labels.PLAYER_WAITING, ThemeBuilder.CAPS_LABEL)
	_friend_status = friend.get_child(2)
	players.add_child(friend)
	column.add_child(players)

	_help = Ui.centered_label(Labels.LOBBY_WAITING, ThemeBuilder.MUTED_LABEL)
	column.add_child(_help)
	_launch = Ui.button(Labels.LAUNCH, ThemeBuilder.PRIMARY_BUTTON)
	_launch.disabled = true
	_launch.pressed.connect(launch_requested.emit)
	column.add_child(_launch)
	var close := Ui.button(Labels.CLOSE_ROOM, ThemeBuilder.SECONDARY_BUTTON)
	close.pressed.connect(home_requested.emit)
	column.add_child(close)
	add_child(Ui.page(Ui.limit_width(column, Look.CONTENT_MAX_WIDTH)))


## Ton ami est là, ou n'y est plus : « LANCER LA PARTIE » suit.
func set_opponent_present(present: bool) -> void:
	_friend_status.text = Labels.PLAYER_READY if present else Labels.PLAYER_WAITING
	_friend_status.theme_type_variation = ThemeBuilder.GOLD_CAPS_LABEL if present else ThemeBuilder.CAPS_LABEL
	_help.text = Labels.LOBBY_READY if present else Labels.LOBBY_WAITING
	_launch.disabled = not present


## Un joueur de la salle : son palet, son nom, et s'il est prêt.
func _player(player: String, display_name: String, status: String, variation: StringName) -> VBoxContainer:
	var column := Ui.vbox(Look.GAP_XS)
	var chip := Ui.chip(player)
	chip.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	column.add_child(chip)
	column.add_child(Ui.centered_label(display_name, ThemeBuilder.STRONG_LABEL))
	column.add_child(Ui.centered_label(status, variation))
	return column
