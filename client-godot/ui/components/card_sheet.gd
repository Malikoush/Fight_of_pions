class_name CardSheet
extends Control
## La fiche d'une carte (maquettes 12-17, p. 1), en feuille du bas par-dessus l'écran : la
## carte en grand, son nom, sa rareté (et « TA CACHE » pour sa propre carte cachée), son
## coût en losanges, sa recharge (quand on les connaît : ils viennent de la partie), et ce
## qu'elle fait, tel que le catalogue du serveur le dit. Un bouton peut s'y ajouter (mettre
## la carte dans son trio de cache). Le voile, la croix ou Échap la ferment.

## Le bouton de la fiche (`set_action`) est touché.
signal action_pressed

var _card: CardView
var _name: Label
var _rarity: Label
var _secret: PanelContainer
var _cost: DiamondStrip
var _cooldown: Label
var _effect: Label
var _effect_box: PanelContainer
var _stats: HBoxContainer
var _action: Button


func _init() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	visible = false
	var dim := ColorRect.new()
	dim.color = Look.DIALOG_DIM
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(dim)

	var page := MarginContainer.new()
	page.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	page.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(page)
	Ui.fit_page(page, Look.CONTENT_MAX_WIDTH)
	var stack := Ui.vbox(0)
	stack.add_child(Ui.spacer())
	page.add_child(stack)
	var sheet := PanelContainer.new()
	sheet.theme_type_variation = ThemeBuilder.SHEET
	stack.add_child(sheet)
	var column := Ui.vbox(Look.GAP_M)
	sheet.add_child(column)

	var close := Ui.icon_button("fermer", Labels.CLOSE)
	close.size_flags_horizontal = Control.SIZE_SHRINK_END
	close.pressed.connect(hide)
	column.add_child(close)
	_card = CardView.new(Look.SHEET_CARD_WIDTH)
	_card.focus_mode = Control.FOCUS_NONE
	_card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_card.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	column.add_child(_card)
	_name = Ui.centered_label("", ThemeBuilder.TITLE_LABEL)
	column.add_child(_name)
	var rarity_row := Ui.hbox(Look.GAP_S)
	rarity_row.alignment = BoxContainer.ALIGNMENT_CENTER
	_rarity = Ui.label("", ThemeBuilder.GOLD_CAPS_LABEL)
	rarity_row.add_child(_rarity)
	_secret = PanelContainer.new()
	_secret.theme_type_variation = ThemeBuilder.TAG
	_secret.add_child(Ui.label(Labels.SHEET_YOUR_CACHE, ThemeBuilder.TAG_LABEL))
	rarity_row.add_child(_secret)
	column.add_child(rarity_row)

	_stats = Ui.hbox(Look.GAP_M)
	_cost = DiamondStrip.new()
	_cost.diamond = Look.SHEET_COST_DIAMOND
	_cost.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_stats.add_child(_stat_box(Labels.SHEET_COST, _cost))
	var cooldown := Ui.hbox(Look.GAP_XS)
	cooldown.alignment = BoxContainer.ALIGNMENT_CENTER
	cooldown.add_child(Ui.icon(Ui.HOURGLASS, Look.GOLD))
	_cooldown = Ui.label("", ThemeBuilder.PILL_LABEL)
	cooldown.add_child(_cooldown)
	_stats.add_child(_stat_box(Labels.SHEET_COOLDOWN, cooldown))
	column.add_child(_stats)

	_effect_box = PanelContainer.new()
	_effect_box.theme_type_variation = ThemeBuilder.WAITING_PANEL
	_effect = Ui.label("", ThemeBuilder.STRONG_LABEL)
	_effect.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var effect_margin := MarginContainer.new()
	effect_margin.theme_type_variation = ThemeBuilder.INSET
	effect_margin.add_child(_effect)
	_effect_box.add_child(effect_margin)
	column.add_child(_effect_box)
	_action = Ui.button("", ThemeBuilder.PRIMARY_BUTTON)
	_action.pressed.connect(action_pressed.emit)
	column.add_child(_action)


## Ouvre la fiche de cette carte (`{"kind", "rarity"}`) : ses réglages (`{"cost",
## "cooldown"}`, lus dans la configuration de la partie), et si c'est sa propre carte
## cachée.
func show_card(attack: Dictionary, catalog: CardCatalog, stats: Dictionary, secret: bool) -> void:
	var kind: String = attack.get("kind", "")
	var rarity: String = attack.get("rarity", "")
	var cost: int = stats.get("cost", 0)
	_card.show_attack(attack, catalog, cost)
	_name.text = catalog.family_name(kind) if catalog != null else kind
	_rarity.text = Labels.RARITY_FILTERS.get(rarity, rarity)
	_secret.visible = secret
	_cost.count = cost
	_cost.filled = cost
	var cooldown: int = stats.get("cooldown", 0)
	_cooldown.text = Labels.turns(cooldown) if cooldown > 0 else Labels.SHEET_NO_COOLDOWN
	_effect.text = catalog.effect(kind, rarity) if catalog != null else ""
	_effect_box.visible = _effect.text != ""
	_stats.visible = not stats.is_empty()
	_action.visible = false
	# Mise hors de la mise en page d'une page (`top_level`), elle couvre l'écran elle-même.
	if top_level:
		global_position = Vector2.ZERO
		size = get_viewport_rect().size
	visible = true


## Le bouton de la fiche, avec ce texte (grisé s'il n'est pas permis).
func set_action(text: String, enabled: bool) -> void:
	_action.text = text
	_action.disabled = not enabled
	_action.visible = true


func _gui_input(event: InputEvent) -> void:
	# Un toucher sur le voile, hors de la feuille, la ferme.
	var button := event as InputEventMouseButton
	if button != null and button.pressed and button.button_index == MOUSE_BUTTON_LEFT:
		hide()
		accept_event()


func _unhandled_input(event: InputEvent) -> void:
	if visible and event.is_action_pressed(&"ui_cancel"):
		hide()
		get_viewport().set_input_as_handled()


## Une case de la fiche : son nom en capitales, et ce qu'elle montre dessous.
func _stat_box(title: String, content: Control) -> PanelContainer:
	var box := PanelContainer.new()
	box.theme_type_variation = ThemeBuilder.TIP_PANEL
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var column := Ui.vbox(Look.GAP_S)
	column.add_child(Ui.centered_label(title, ThemeBuilder.SECTION_LABEL))
	column.add_child(content)
	box.add_child(column)
	return box
