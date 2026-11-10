class_name AlertDialog
extends Control
## Une fenêtre par-dessus l'écran, sur un voile (maquettes 12-17, p. 14 à 17) : une plaque
## crème bordée d'or, la pastille ronde de son icône (d'or, ou de braise pour un problème),
## son titre, sa phrase, une barre de chargement si l'on attend, et deux boutons : « non »
## (bordé d'encre) et « oui » (d'or). Le voile arrête les touchers ; Échap la ferme sans
## répondre (il ne quitte jamais une partie).
##
## Elle sert aux questions (« Quitter la partie ? ») comme aux coupures (connexion perdue,
## bot muet, serveur complet, partie ouverte ailleurs).

## « Oui » : le bouton d'or.
signal confirmed
## « Non » : le bouton bordé d'encre.
signal declined

## Vrai : Échap ne ferme pas la fenêtre (une page qui n'est qu'elle).
var keep_on_cancel := false

var _progress: ProgressRunner


## « Quitter la partie ? », au draft comme pendant la partie.
static func quit_game() -> AlertDialog:
	return AlertDialog.new(Labels.QUIT_TITLE, Labels.QUIT_TEXT, Labels.STAY, Labels.QUIT, "retour", true)


## `icon` : le nom d'une icône des éléments fournis ; `threat` : elle est de braise.
func _init(title: String, text: String, no_label: String, yes_label: String, icon: String, threat: bool) -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	visible = false
	var dim := ColorRect.new()
	dim.color = Look.DIALOG_DIM
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(dim)
	var panel := PanelContainer.new()
	panel.theme_type_variation = ThemeBuilder.ALERT_PANEL
	var column := Ui.vbox(Look.GAP_M)
	var badge := PanelContainer.new()
	badge.theme_type_variation = ThemeBuilder.ALERT_BADGE
	badge.custom_minimum_size = Vector2.ONE * Look.ALERT_BADGE_SIZE
	badge.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	var picture := Ui.icon(icon, Look.THREAT if threat else Look.GOLD, Look.ALERT_ICON_SIZE)
	picture.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	picture.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	badge.add_child(picture)
	column.add_child(badge)
	column.add_child(Ui.centered_label(title, ThemeBuilder.ALERT_TITLE_LABEL))
	var phrase := Ui.centered_label(text, ThemeBuilder.ALERT_TEXT_LABEL)
	phrase.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(phrase)
	_progress = ProgressRunner.new()
	_progress.visible = false
	column.add_child(_progress)
	var buttons := Ui.hbox(Look.GAP_M)
	var no := Ui.button(no_label, ThemeBuilder.ALERT_SECONDARY)
	no.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	no.pressed.connect(_answer.bind(declined))
	buttons.add_child(no)
	var yes := Ui.button(yes_label, ThemeBuilder.PRIMARY_BUTTON)
	yes.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	yes.pressed.connect(_answer.bind(confirmed))
	buttons.add_child(yes)
	column.add_child(buttons)
	panel.add_child(column)
	add_child(Ui.page(Ui.limit_width(panel, Look.CONTENT_MAX_WIDTH)))


## Montre la fenêtre ; `waiting` : avec la barre de chargement (une reconnexion).
func ask(waiting := false) -> void:
	_progress.visible = waiting
	visible = true


func _answer(answer: Signal) -> void:
	if not keep_on_cancel:
		hide()
	answer.emit()


func _unhandled_input(event: InputEvent) -> void:
	if visible and not keep_on_cancel and event.is_action_pressed(&"ui_cancel"):
		hide()
		get_viewport().set_input_as_handled()
