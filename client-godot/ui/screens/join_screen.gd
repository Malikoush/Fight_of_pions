class_name JoinScreen
extends Control
## Rejoindre la salle d'un ami (maquettes) : le code à 5 caractères, en capitales, et
## « REJOINDRE », allumé quand le code est complet. Un refus du serveur (salle inconnue,
## complète) s'affiche dessous ; le code reste tapé, pour corriger une lettre.

signal join_requested(code: String)
signal back_requested

## La longueur d'un code de salle (protocole).
const CODE_LENGTH := 5

var _code: LineEdit
var _join: Button
var _error: Label


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_L)
	column.alignment = BoxContainer.ALIGNMENT_CENTER
	column.add_child(Ui.centered_label(Labels.JOIN_TITLE, ThemeBuilder.HEADING_LABEL))
	column.add_child(Ui.centered_label(Labels.JOIN_HELP, ThemeBuilder.MUTED_LABEL))
	_code = LineEdit.new()
	_code.theme_type_variation = ThemeBuilder.CODE_INPUT
	_code.max_length = CODE_LENGTH
	_code.alignment = HORIZONTAL_ALIGNMENT_CENTER
	_code.custom_minimum_size.x = Look.CODE_INPUT_WIDTH
	_code.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_code.text_changed.connect(_on_code_changed)
	_code.text_submitted.connect(func(_text: String) -> void: _on_join())
	column.add_child(_code)
	_error = Ui.centered_label("", ThemeBuilder.THREAT_LABEL)
	_error.visible = false
	column.add_child(_error)
	_join = Ui.button(Labels.JOIN, ThemeBuilder.PRIMARY_BUTTON)
	_join.disabled = true
	_join.pressed.connect(_on_join)
	column.add_child(_join)
	var back := Ui.button(Labels.BACK, ThemeBuilder.SECONDARY_BUTTON)
	back.pressed.connect(back_requested.emit)
	column.add_child(back)
	add_child(Ui.page(Ui.limit_width(column, Look.CONTENT_MAX_WIDTH)))
	_code.grab_focus()


## Le serveur a refusé ce code : son message, et le bouton de nouveau permis.
func show_error(message: String) -> void:
	_error.text = message
	_error.visible = true
	_join.disabled = _code.text.length() != CODE_LENGTH


func _on_code_changed(text: String) -> void:
	var caret := _code.caret_column
	_code.text = text.to_upper()
	_code.caret_column = caret
	_error.visible = false
	_join.disabled = _code.text.length() != CODE_LENGTH


func _on_join() -> void:
	if _join.disabled:
		return
	_join.disabled = true
	join_requested.emit(_code.text)
