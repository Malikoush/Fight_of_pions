class_name HomeScreen
extends Control
## L'accueil (maquettes) : le titre, « JOUER CONTRE LE BOT », puis la partie entre amis :
## créer une salle (un code à envoyer) ou en rejoindre une. Les pages Cartes, Règles et
## Réglages viendront en E3h.

signal play_bot_requested
signal create_room_requested
signal join_requested

var _buttons: Array[Button] = []


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_L)
	column.alignment = BoxContainer.ALIGNMENT_CENTER
	column.add_child(Ui.centered_label(Labels.GAME_NAME, ThemeBuilder.DISPLAY_LABEL))
	column.add_child(Ui.centered_label(Labels.TAGLINE, ThemeBuilder.MUTED_LABEL))
	column.add_child(Ui.gap(Vector2(0, Look.GAP_XL)))
	column.add_child(_button(Labels.PLAY_BOT, ThemeBuilder.PRIMARY_BUTTON, play_bot_requested))
	column.add_child(Ui.centered_label(Labels.PLAY_BOT_DETAIL, ThemeBuilder.MUTED_LABEL))
	column.add_child(Ui.gap(Vector2(0, Look.GAP_XL)))
	column.add_child(Ui.label(Labels.FRIENDS_SECTION, ThemeBuilder.CAPS_LABEL))
	var friends := Ui.hbox(Look.GAP_M)
	friends.add_child(_choice(Labels.CREATE_ROOM, Labels.CREATE_ROOM_DETAIL, create_room_requested))
	friends.add_child(_choice(Labels.JOIN, Labels.JOIN_DETAIL, join_requested))
	column.add_child(friends)
	add_child(Ui.page(Ui.limit_width(column, Look.CONTENT_MAX_WIDTH)))


## La demande n'a pas abouti (refus, coupure) : les boutons se réactivent.
func reset() -> void:
	for button in _buttons:
		button.disabled = false


## Un choix de la partie entre amis : le bouton, et ce qu'il fait, dessous.
func _choice(text: String, detail: String, chosen: Signal) -> VBoxContainer:
	var column := Ui.vbox(Look.GAP_XS)
	column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	column.add_child(_button(text, ThemeBuilder.SECONDARY_BUTTON, chosen))
	column.add_child(Ui.centered_label(detail, ThemeBuilder.CAPTION_LABEL))
	return column


## Un bouton qui se grise une fois touché, le temps que le serveur réponde.
func _button(text: String, variation: StringName, chosen: Signal) -> Button:
	var button := Ui.button(text, variation)
	button.pressed.connect(func() -> void:
		for other in _buttons:
			other.disabled = true
		chosen.emit())
	_buttons.append(button)
	return button
