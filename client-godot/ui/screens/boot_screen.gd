class_name BootScreen
extends Control
## L'écran de démarrage : le nom du jeu et l'état de la connexion au serveur, avec une barre
## qui court tant que la connexion se cherche.

signal retry_requested

var _status: Label
var _track: ColorRect
var _runner: ColorRect
var _retry: Button
var _loop: Tween


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_XL)
	column.add_child(Ui.centered_label(Labels.GAME_NAME, ThemeBuilder.DISPLAY_LABEL))

	_track = ColorRect.new()
	_track.color = Look.FOREST_RAISED
	_track.custom_minimum_size = Look.PROGRESS_SIZE
	_track.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_track.clip_contents = true
	_runner = ColorRect.new()
	_runner.color = Look.GOLD
	_runner.size = Vector2(Look.PROGRESS_SIZE.x / 2.0, Look.PROGRESS_SIZE.y)
	_track.add_child(_runner)
	column.add_child(_track)

	_status = Ui.centered_label(Labels.CONNECTING, ThemeBuilder.MUTED_LABEL)
	column.add_child(_status)

	_retry = Ui.button(Labels.RETRY, ThemeBuilder.SECONDARY_BUTTON)
	_retry.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_retry.visible = false
	_retry.pressed.connect(retry_requested.emit)
	column.add_child(_retry)
	add_child(Ui.page(Ui.limit_width(column, Look.CONTENT_MAX_WIDTH)))
	_run()


## Le texte d'état, et le bouton pour réessayer quand le serveur ne répond pas.
func set_status(text: String, can_retry: bool) -> void:
	_status.text = text
	_retry.visible = can_retry


func _run() -> void:
	_loop = create_tween().set_loops()
	_loop.tween_property(_runner, "position:x", Look.PROGRESS_SIZE.x, Look.PROGRESS_LOOP).from(-_runner.size.x)
