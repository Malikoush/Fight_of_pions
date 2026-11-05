class_name PlayerPanel
extends PanelContainer
## Ce qu'a un joueur, d'un coup d'œil : son palet, son nom et ce qu'il fait, ses pions en
## réserve sur ceux de départ, ses points du tour et sa banque, ce qu'il peut encore poser,
## et ce qui dure (Terre brûlée, infirmerie, cimetière, jetons, gardes de sa reine). Bordé
## d'or quand c'est à lui de décider, de braise en pointillés quand il attend sous Terre
## brûlée. Tout vient de la vue.

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
var _facts: HFlowContainer
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

	_facts = HFlowContainer.new()
	_facts.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_facts.add_theme_constant_override(&"h_separation", Look.GAP_M)
	_facts.add_theme_constant_override(&"v_separation", Look.GAP_XS)
	column.add_child(_facts)


## Montre ce joueur, tel que la vue le décrit. `deciding_text` dit ce qu'il fait quand c'est
## à l'adversaire de décider (« joue », ou « réfléchit… » pour un bot).
func show_side(view: ViewData, player: String, display_name: String, deciding_text: String = Labels.PLAYS) -> void:
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
		_status.text = deciding_text if deciding else Labels.WAITS
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
	var scorched := view.scorched_until(player)
	_show_facts(view, player, scorched)
	if deciding:
		theme_type_variation = ThemeBuilder.PANEL_ACTIVE
	else:
		theme_type_variation = ThemeBuilder.PANEL_SCORCHED if scorched >= 0 else ThemeBuilder.PANEL
	modulate.a = 1.0 if deciding else Look.PANEL_IDLE_ALPHA


## Ce qui dure pour ce joueur, une ligne courte par chose, seulement ce qui existe.
func _show_facts(view: ViewData, player: String, scorched: int) -> void:
	for old in _facts.get_children():
		_facts.remove_child(old)
		old.queue_free()
	if scorched >= 0:
		_add_fact("terre-brulee", Labels.FACT_SCORCHED % scorched, true)
	var guards := _queen_guards(view, player)
	if guards > 0:
		_add_fact("gardes", Labels.FACT_GUARDS % guards)
	for pile in view.token_piles(player):
		_add_fact("jetons", Labels.token_pile(pile.get("tokens", 0), pile.get("reward")))
	var patients := view.infirmary(player).size()
	if patients > 0:
		_add_fact("infirmerie", Labels.FACT_INFIRMARY % patients)
	var dead := view.graveyard(player).size()
	if dead > 0:
		_add_fact("cimetiere", Labels.FACT_GRAVEYARD % dead)
	_facts.visible = _facts.get_child_count() > 0


func _add_fact(icon: String, text: String, scorched: bool = false) -> void:
	var fact := Ui.hbox(Look.GAP_XS)
	fact.add_child(Ui.icon(icon, Look.SCORCHED if scorched else Look.GOLD, Look.FACT_ICON_SIZE))
	fact.add_child(Ui.label(text, ThemeBuilder.SCORCHED_LABEL if scorched else ThemeBuilder.FACT_LABEL))
	_facts.add_child(fact)


## Les gardes de la reine de ce joueur, s'il en a une.
static func _queen_guards(view: ViewData, player: String) -> int:
	var queen: Variant = view.side(player).get("queen")
	var pion: Variant = view.pion_at(ViewData.to_cell(queen)) if queen is Dictionary else null
	if not (pion is Dictionary):
		return 0
	return PionStatus.of(view.effects_on(pion.get("id", -1))).guards


## Montre ou cache le palet, le nom et l'état (cachés pour sa propre barre, en portrait).
func set_identity_visible(shown: bool) -> void:
	_identity.visible = shown

