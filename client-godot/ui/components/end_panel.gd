class_name EndPanel
extends PanelContainer
## La fin de la partie, à la place de l'arsenal et de la barre : le résultat (« Victoire ! »,
## « Défaite », « Match nul »), sa raison, et « REJOUER » ou « ACCUEIL ». Tout vient de la
## fin que donne la vue (`end`).

signal replay_requested
signal home_requested

const WON := "Won"

var _result: Label
var _reason: Label


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL_ACTIVE
	var column := Ui.vbox(Look.GAP_M)
	_result = Ui.centered_label("", ThemeBuilder.DISPLAY_LABEL)
	column.add_child(_result)
	_reason = Ui.centered_label("", ThemeBuilder.MUTED_LABEL)
	column.add_child(_reason)
	var buttons := Ui.hbox(Look.GAP_M)
	var home := Ui.button(Labels.HOME, ThemeBuilder.SECONDARY_BUTTON)
	home.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	home.pressed.connect(home_requested.emit)
	buttons.add_child(home)
	var replay := Ui.button(Labels.REPLAY, ThemeBuilder.PRIMARY_BUTTON)
	replay.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	replay.pressed.connect(replay_requested.emit)
	buttons.add_child(replay)
	column.add_child(buttons)
	add_child(column)


## Montre la fin de cette vue.
func show_end(view: ViewData) -> void:
	var end: Variant = view.end()
	var outcome: Variant = end.get("outcome") if end is Dictionary else null
	if Wire.tag(outcome) == WON:
		_result.text = Labels.RESULT_WIN if Wire.content(outcome) == view.viewer() else Labels.RESULT_LOSS
	else:
		_result.text = Labels.RESULT_DRAW
	var reason: String = Labels.END_REASONS.get(Wire.tag(end.get("reason") if end is Dictionary else null), "")
	_reason.text = reason % view.config().get("align_to_win", 0) if "%d" in reason else reason
