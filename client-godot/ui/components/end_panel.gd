class_name EndPanel
extends VBoxContainer
## La fin de la partie (maquettes 12-17, p. 2 à 4), à la place de l'arsenal et de la barre :
## le résultat dans un bandeau (d'or pour une victoire, sombre pour une défaite ou un match
## nul), sa raison, contre qui et à quel tour, puis « REJOUER » et « ACCUEIL ». Tout vient de
## la fin que donne la vue (`end`).

signal replay_requested
signal home_requested

const WON := "Won"

var _banner: PanelContainer
var _result: Label
var _reason: Label
var _caption: Label


func _init() -> void:
	theme_type_variation = ThemeBuilder.COLUMN_M
	_banner = PanelContainer.new()
	_banner.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_result = Ui.centered_label("", ThemeBuilder.BANNER_LABEL)
	# Sur une ligne : le bandeau prend la largeur du mot.
	_result.autowrap_mode = TextServer.AUTOWRAP_OFF
	_banner.add_child(_result)
	add_child(_banner)
	_reason = Ui.centered_label("", ThemeBuilder.STRONG_LABEL)
	add_child(_reason)
	_caption = Ui.centered_label("", ThemeBuilder.CAPTION_LABEL)
	add_child(_caption)
	var replay := Ui.button(Labels.REPLAY, ThemeBuilder.PRIMARY_BUTTON)
	replay.pressed.connect(replay_requested.emit)
	add_child(replay)
	var home := Ui.button(Labels.HOME, ThemeBuilder.SECONDARY_BUTTON)
	home.pressed.connect(home_requested.emit)
	add_child(home)


## Montre la fin de cette vue, contre cet adversaire (son nom affiché).
func show_end(view: ViewData, opponent_name: String) -> void:
	var end: Variant = view.end()
	var outcome: Variant = end.get("outcome") if end is Dictionary else null
	var won: bool = Wire.tag(outcome) == WON and Wire.content(outcome) == view.viewer()
	if Wire.tag(outcome) == WON:
		_result.text = Labels.RESULT_WIN if won else Labels.RESULT_LOSS
	else:
		_result.text = Labels.RESULT_DRAW
	# Le bandeau d'or ne va qu'à la victoire.
	_banner.theme_type_variation = ThemeBuilder.BANNER if won else ThemeBuilder.BANNER_DARK
	_result.theme_type_variation = ThemeBuilder.BANNER_LABEL if won else ThemeBuilder.BANNER_LIGHT_LABEL
	var reason: String = Labels.END_REASONS.get(Wire.tag(end.get("reason") if end is Dictionary else null), "")
	_reason.text = reason % view.config().get("align_to_win", 0) if "%d" in reason else reason
	_caption.text = Labels.END_CAPTION % [opponent_name, view.turn()]
