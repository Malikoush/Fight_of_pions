class_name HomeScreen
extends Control
## L'accueil, réduit pour l'instant à l'essentiel : lancer une partie contre le bot. Les
## salles entre amis, le draft et les autres pages viendront en E3f et E3h.

signal play_bot_requested

var _play: Button


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_L)
	column.add_child(Ui.centered_label(Labels.GAME_NAME, ThemeBuilder.DISPLAY_LABEL))
	column.add_child(Ui.centered_label(Labels.TAGLINE, ThemeBuilder.MUTED_LABEL))
	column.add_child(Ui.gap(Vector2(0, Look.GAP_XXL)))
	_play = Ui.button(Labels.PLAY_BOT, ThemeBuilder.PRIMARY_BUTTON)
	_play.pressed.connect(_on_play)
	column.add_child(_play)
	column.add_child(Ui.centered_label(Labels.PLAY_BOT_DETAIL, ThemeBuilder.MUTED_LABEL))
	add_child(Ui.page(Ui.limit_width(column, Look.CONTENT_MAX_WIDTH)))


## La demande n'a pas abouti (refus, coupure) : le bouton se réactive.
func reset() -> void:
	_play.disabled = false


func _on_play() -> void:
	_play.disabled = true
	play_bot_requested.emit()
