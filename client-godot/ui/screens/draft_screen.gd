class_name DraftScreen
extends Control
## Le draft, avant la partie (maquettes) : la barre des étapes, l'adversaire (ce qu'il vient
## de faire, ses cartes, sa cache en dos), ce qu'il faut choisir, la grille des cartes, mon
## arsenal et ses places encore vides, et le bouton qui joue le choix (« BANNIR OURS »).
##
## Rien n'est déduit : les étapes viennent du format de la partie (`config.draft`), les
## cartes qu'on peut toucher des coups permis (`Draft`), et ce que fait l'adversaire de ses
## événements. Pendant le choix de la cache (tout le catalogue), des filtres par rareté.
##
## La flèche, en haut à gauche, quitte la partie (contre le bot comme entre amis).

## Le joueur joue ce coup du draft, recopié des coups permis.
signal play_requested(action: Variant)
## Le joueur quitte la partie.
signal home_requested

const DRAFT := "Draft"
## Les étapes qui ajoutent une carte à l'arsenal (carnet, « Draft ») : un pick, une cache ;
## un ban, non. Sert seulement à dessiner les places encore vides.
const STEPS_WITH_CARD: Array[String] = ["Pick", "Cache"]
const ALL := ""

## Le nom affiché de l'adversaire, et le catalogue des cartes.
var opponent_name := Labels.OPPONENT
var catalog: CardCatalog

var _view: ViewData
var _waiting := false
var _selected: Variant = null
var _filter := ALL

var _steps: HBoxContainer
var _opponent_chip: Control
var _opponent_status: Label
var _opponent_cards: HBoxContainer
var _title: Label
var _help: Label
var _filters: HFlowContainer
var _grid: GridContainer
var _my_cards: HBoxContainer
var _play: Button
var _quit_dialog: ConfirmDialog
## La largeur de l'écran, limitée sur un PC par des marges calculées sur la place de la page.
var _width_limit: MarginContainer


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_M)
	var header := Ui.hbox(Look.GAP_M)
	var back := Ui.icon_button("retour", Labels.BACK)
	back.pressed.connect(func() -> void: _quit_dialog.ask())
	header.add_child(back)
	_steps = Ui.hbox(Look.GAP_L)
	_steps.alignment = BoxContainer.ALIGNMENT_CENTER
	_steps.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(_steps)
	column.add_child(header)
	column.add_child(_build_opponent())
	_title = Ui.label("", ThemeBuilder.HEADING_LABEL)
	column.add_child(_title)
	_help = Ui.label("", ThemeBuilder.MUTED_LABEL)
	_help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_help)
	_filters = HFlowContainer.new()
	_filters.add_theme_constant_override(&"h_separation", Look.GAP_S)
	_filters.add_theme_constant_override(&"v_separation", Look.GAP_S)
	column.add_child(_filters)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	# Sans barre, mais pas « désactivé » : désactivé, la zone prendrait la largeur de la
	# grille, et la grille ses colonnes de la zone, sans jamais pouvoir rétrécir.
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	_grid = GridContainer.new()
	_grid.add_theme_constant_override(&"h_separation", Look.CARD_GAP)
	_grid.add_theme_constant_override(&"v_separation", Look.CARD_GAP)
	_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_grid)
	column.add_child(scroll)
	column.add_child(_build_mine())
	_play = Ui.button(Labels.DRAFT_CHOOSE, ThemeBuilder.PRIMARY_BUTTON)
	_play.pressed.connect(_on_play)
	column.add_child(_play)
	_width_limit = MarginContainer.new()
	_width_limit.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_width_limit.add_child(column)
	add_child(Ui.page(_width_limit))
	_quit_dialog = ConfirmDialog.new(Labels.QUIT_TITLE, Labels.QUIT_TEXT, Labels.STAY, Labels.QUIT)
	_quit_dialog.confirmed.connect(home_requested.emit)
	add_child(_quit_dialog)
	resized.connect(_fit_columns)
	_fit_columns()


## Montre cette vue du draft, et ce que l'adversaire vient d'y faire.
func show_view(view: ViewData, events: Array) -> void:
	_waiting = false
	_redraw(view, events)


## Redessine l'écran pour cette vue, sans toucher à l'attente d'une réponse (un filtre
## changé ne doit pas permettre de rejouer).
func _redraw(view: ViewData, events: Array) -> void:
	_view = view
	_selected = null
	var step: Variant = view.draft_step()
	var kind := Wire.tag(step)
	var mine: bool = Wire.content(step) == view.viewer()
	_show_steps(view, kind)
	_show_opponent(view, events)
	if kind == "Cache" and mine and _filters.get_child_count() == 0:
		_build_filters()
	_filters.visible = kind == "Cache" and mine
	_title.text = Labels.DRAFT_TITLES.get(kind, "") if mine else Labels.DRAFT_WAIT % opponent_name
	_help.text = Labels.DRAFT_HELP.get(kind, "")
	_show_cards(view, kind, mine)
	_show_mine(view)
	_refresh_button(kind)


## Le serveur a refusé le choix : on peut rechoisir.
func on_refused() -> void:
	_waiting = false
	if _view != null:
		_refresh_button(Wire.tag(_view.draft_step()))


# --- Les parties de l'écran ---

## La barre des étapes : une par phase du format (bans, picks, cache), celle en cours en or.
func _show_steps(view: ViewData, current: String) -> void:
	for old in _steps.get_children():
		old.queue_free()
	var phases: Array[String] = []
	for step: Variant in view.draft_steps():
		var kind := Wire.tag(step)
		if phases.is_empty() or phases[-1] != kind:
			phases.append(kind)
	for index in phases.size():
		var text := "%d %s" % [index + 1, Labels.DRAFT_PHASES.get(phases[index], phases[index])]
		_steps.add_child(Ui.label(text, ThemeBuilder.GOLD_CAPS_LABEL if phases[index] == current else ThemeBuilder.CAPS_LABEL))


func _build_opponent() -> PanelContainer:
	var panel := PanelContainer.new()
	panel.theme_type_variation = ThemeBuilder.PANEL
	var row := Ui.hbox(Look.GAP_M)
	_opponent_chip = Ui.hbox(0)
	row.add_child(_opponent_chip)
	var text := Ui.vbox(0)
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_opponent_status = Ui.label("", ThemeBuilder.MUTED_LABEL)
	# Passe à la ligne plutôt que d'élargir l'écran (« Bot · a pris Coup du taureau Rare »).
	_opponent_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_opponent_status.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text.add_child(_opponent_status)
	row.add_child(text)
	_opponent_cards = Ui.hbox(Look.GAP_XS)
	row.add_child(_opponent_cards)
	panel.add_child(row)
	return panel


## L'adversaire : son palet, son nom et ce qu'il vient de faire (ou qu'il choisit), ses cartes.
func _show_opponent(view: ViewData, events: Array) -> void:
	var opponent := view.opponent()
	if _opponent_chip.get_child_count() == 0:
		_opponent_chip.add_child(Ui.chip(opponent))
	var status := ""
	for event: Variant in events:
		var data := GameEvents.payload(event)
		match GameEvents.kind(event):
			"Drafted":
				var step: Variant = data.get("step")
				if Wire.content(step) == opponent:
					status = Labels.DRAFT_DONE.get(Wire.tag(step), "") % CardCatalog.card_name(catalog, data.get("attack"))
			"CardCached":
				if data.get("player") == opponent:
					status = Labels.DRAFT_CACHED
	if status == "" and Wire.content(view.draft_step()) == opponent:
		status = Labels.THINKS
	if status != "":
		_opponent_status.text = "%s · %s" % [opponent_name, status]
	elif _opponent_status.text == "":
		_opponent_status.text = opponent_name
	_show_arsenal(_opponent_cards, view, opponent)


func _build_filters() -> void:
	var group := ButtonGroup.new()
	var choices := {ALL: Labels.ALL_RARITIES}
	choices.merge(Labels.RARITY_FILTERS)
	for rarity: String in choices:
		var chip := Ui.button(choices[rarity], ThemeBuilder.SECONDARY_BUTTON)
		chip.custom_minimum_size.y = Look.FILTER_HEIGHT
		chip.toggle_mode = true
		chip.button_group = group
		chip.button_pressed = rarity == _filter
		chip.pressed.connect(func() -> void:
			_filter = rarity
			_redraw(_view, []))
		_filters.add_child(chip)


## La grille : les cartes qu'on peut choisir (coups permis), sinon les cartes tirées.
func _show_cards(view: ViewData, kind: String, mine: bool) -> void:
	for old in _grid.get_children():
		_grid.remove_child(old)
		old.queue_free()
	var cards: Array = []
	if mine:
		for action: Variant in view.legal_actions():
			var attack: Variant = Wire.content(action) if Wire.tag(action) == DRAFT else null
			if attack is Dictionary and (kind != "Cache" or _filter == ALL or attack.get("rarity") == _filter):
				cards.append(attack)
	else:
		cards = view.draft_pool()
	for attack: Variant in cards:
		var card := CardView.new(Look.DRAFT_CARD_WIDTH)
		card.show_attack(attack, catalog, view.stats_of(attack).get("cost", 0))
		card.disabled = not mine
		card.pressed.connect(_select.bind(attack, card))
		_grid.add_child(card)


func _build_mine() -> PanelContainer:
	var panel := PanelContainer.new()
	panel.theme_type_variation = ThemeBuilder.PANEL
	var row := Ui.hbox(Look.GAP_M)
	row.add_child(Ui.label(Labels.MY_ARSENAL, ThemeBuilder.GOLD_CAPS_LABEL))
	row.add_child(Ui.spacer())
	_my_cards = Ui.hbox(Look.GAP_XS)
	row.add_child(_my_cards)
	panel.add_child(row)
	return panel


func _show_mine(view: ViewData) -> void:
	_show_arsenal(_my_cards, view, view.viewer())


## Les cartes d'un joueur, en petit (la cache adverse en dos), puis une place vide en
## pointillés pour chaque carte qu'il lui reste à choisir, d'après le format du draft.
func _show_arsenal(row: HBoxContainer, view: ViewData, player: String) -> void:
	for old in row.get_children():
		old.queue_free()
	var arsenal := view.arsenal(player)
	for index in arsenal.size():
		var card := CardView.new(Look.DRAFT_MINI_CARD)
		card.focus_mode = Control.FOCUS_NONE
		card.mouse_filter = Control.MOUSE_FILTER_IGNORE
		card.show_slot(view, player, index, catalog, true)
		row.add_child(card)
	var expected := view.starting_attacks(player).size()
	for step: Variant in view.draft_steps():
		if Wire.content(step) == player and Wire.tag(step) in STEPS_WITH_CARD:
			expected += 1
	for index in maxi(expected - arsenal.size(), 0):
		var place := PanelContainer.new()
		place.theme_type_variation = ThemeBuilder.WAITING_PANEL
		place.custom_minimum_size = CardView.footprint(Look.DRAFT_MINI_CARD)
		place.mouse_filter = Control.MOUSE_FILTER_IGNORE
		row.add_child(place)


# --- Le choix ---

func _select(attack: Dictionary, card: CardView) -> void:
	if _waiting:
		return
	_selected = attack
	for other: Node in _grid.get_children():
		(other as CardView).set_selected(other == card)
	_refresh_button(Wire.tag(_view.draft_step()))


func _refresh_button(kind: String) -> void:
	var mine: bool = _view != null and Wire.content(_view.draft_step()) == _view.viewer()
	_play.visible = mine
	if _selected is Dictionary:
		var card_name: String = catalog.family_name(_selected.get("kind", "")) if catalog != null else str(_selected.get("kind"))
		_play.text = Labels.DRAFT_ACTIONS.get(kind, "%s") % card_name.to_upper()
	else:
		_play.text = Labels.DRAFT_CHOOSE
	_play.disabled = _waiting or not (_selected is Dictionary)


func _on_play() -> void:
	if _waiting or not (_selected is Dictionary):
		return
	for action: Variant in _view.legal_actions():
		if Wire.tag(action) == DRAFT and Wire.content(action) == _selected:
			_waiting = true
			_refresh_button(Wire.tag(_view.draft_step()))
			play_requested.emit(action)
			return


## La largeur de l'écran, d'après la place de la page (jamais d'après le contenu : rien
## ne s'élargit en boucle), et autant de colonnes de cartes qu'elle en laisse tenir.
func _fit_columns() -> void:
	var insets := Ui.safe_insets()
	var page := size.x - 2 * Look.PAGE_MARGIN - insets.position.x - insets.size.x
	var width := minf(page, Look.DRAFT_MAX_WIDTH)
	var side := int(maxf(0.0, (page - width) / 2.0))
	_width_limit.add_theme_constant_override(&"margin_left", side)
	_width_limit.add_theme_constant_override(&"margin_right", side)
	var cell := CardView.footprint(Look.DRAFT_CARD_WIDTH).x + Look.CARD_GAP
	_grid.columns = maxi(1, int((width + Look.CARD_GAP) / cell))
