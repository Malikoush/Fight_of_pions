class_name ConfirmDialog
extends Control
## Une question par-dessus l'écran, sur un voile (« Quitter la partie ? ») : son titre, une
## phrase, et deux boutons. Le voile arrête les touchers ; Échap répond non.

signal confirmed

var _title: Label
var _text: Label
var _yes: Button


func _init(title: String, text: String, no_label: String, yes_label: String) -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	visible = false
	var dim := ColorRect.new()
	dim.color = Look.DIALOG_DIM
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(dim)
	var panel := PanelContainer.new()
	panel.theme_type_variation = ThemeBuilder.PANEL_THREAT
	var column := Ui.vbox(Look.GAP_M)
	_title = Ui.centered_label(title, ThemeBuilder.HEADING_LABEL)
	column.add_child(_title)
	_text = Ui.centered_label(text, ThemeBuilder.MUTED_LABEL)
	column.add_child(_text)
	var buttons := Ui.hbox(Look.GAP_M)
	var no := Ui.button(no_label, ThemeBuilder.SECONDARY_BUTTON)
	no.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	no.pressed.connect(hide)
	buttons.add_child(no)
	_yes = Ui.button(yes_label, ThemeBuilder.PRIMARY_BUTTON)
	_yes.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_yes.pressed.connect(func() -> void:
		hide()
		confirmed.emit())
	buttons.add_child(_yes)
	column.add_child(buttons)
	panel.add_child(column)
	add_child(Ui.page(Ui.limit_width(panel, Look.CONTENT_MAX_WIDTH)))


func ask() -> void:
	visible = true


func _unhandled_input(event: InputEvent) -> void:
	if visible and event.is_action_pressed(&"ui_cancel"):
		hide()
		get_viewport().set_input_as_handled()
