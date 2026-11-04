class_name PlayerPanel
extends PanelContainer
## Ce qu'a un joueur, d'un coup d'œil : son palet, son nom et ce qu'il fait, ses pions en
## réserve sur ceux de départ, ses points du tour et sa banque, et ce qu'il peut encore
## poser. Bordé d'or quand c'est à lui de décider. Tout vient de la vue.

var _identity: HBoxContainer
var _chip_slot: Control
var _name: Label
var _status: Label
var _reserve: Label
var _reserve_total: Label
var _points: DiamondStrip
var _points_number: Label
var _bank: DiamondStrip
var _detail: Label
var _chip_player := ""


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_XS)
	add_child(column)

	_identity = Ui.hbox(Look.GAP_S)
	_chip_slot = Ui.hbox(0)
	_identity.add_child(_chip_slot)
	_name = Ui.label("", ThemeBuilder.NAME_LABEL)
	_identity.add_child(_name)
	_status = Ui.label("", ThemeBuilder.MUTED_LABEL)
	_identity.add_child(_status)
	column.add_child(_identity)

	var stats := Ui.hbox(Look.GAP_S)
	stats.add_child(Ui.icon("reserve", Look.CREAM))
	_reserve = Ui.label("", ThemeBuilder.NUMBER_LABEL)
	stats.add_child(_reserve)
	_reserve_total = Ui.label("", ThemeBuilder.MUTED_LABEL)
	stats.add_child(_reserve_total)
	stats.add_child(Ui.gap(Vector2(Look.GAP_M, 0)))
	_points = DiamondStrip.new()
	_points.diamond = Look.POINT_DIAMOND
	stats.add_child(_points)
	_points_number = Ui.label("", ThemeBuilder.NUMBER_LABEL)
	stats.add_child(_points_number)
	_bank = DiamondStrip.new()
	stats.add_child(_bank)
	column.add_child(stats)

	_detail = Ui.label("", ThemeBuilder.STRONG_LABEL)
	column.add_child(_detail)


## Montre ce joueur, tel que la vue le décrit.
func show_side(view: ViewData, player: String, display_name: String) -> void:
	if player != _chip_player:
		_chip_player = player
		for old in _chip_slot.get_children():
			old.queue_free()
		_chip_slot.add_child(Ui.chip(player))
	var side := view.side(player)
	var config := view.config()
	var deciding := view.decision_player() == player
	var mine := player == view.viewer()
	_name.text = display_name
	if mine:
		_status.text = Labels.YOUR_TURN if deciding else ""
	else:
		_status.text = Labels.PLAYS if deciding else Labels.WAITS
	_reserve.text = str(side.get("reserve", 0))
	_reserve_total.text = "/%d" % config.get("pions_per_player", 0)
	var points: int = side.get("actions_left", 0)
	_points.filled = 1 if points > 0 else 0
	_points_number.text = str(points)
	_bank.count = config.get("max_banked_actions", 0)
	_bank.filled = side.get("banked_actions", 0)
	var my_turn := view.current_player() == player and not view.is_over()
	_detail.text = Labels.placements(side.get("placements_left", 0), config.get("place_cost", 0), my_turn and mine)
	_detail.visible = _detail.text != ""
	theme_type_variation = ThemeBuilder.PANEL_ACTIVE if deciding else ThemeBuilder.PANEL


## Montre ou cache le palet, le nom et l'état (cachés pour sa propre barre, en portrait).
func set_identity_visible(shown: bool) -> void:
	_identity.visible = shown

