class_name CardsPage
extends MenuPage
## La collection des cartes (maquettes 12-17, p. 5), telle que le catalogue du serveur la
## décrit : le compte des familles et des réactions, des filtres (une rareté, ou les
## réactions), puis chaque famille avec ses cartes, de la Commune à la Légendaire.
##
## Toucher une carte ouvre sa fiche (`CardSheet`), d'où l'on met la carte dans son trio de
## cache, ou l'en retire (`Settings.cache_trio`, gardé sur l'appareil, mis en avant au
## draft). Les cartes du trio portent « TA CACHE ».

## Les filtres : toutes les cartes, une rareté, ou les réactions.
const ALL := ""
const REACTIONS := "reactions"

var _catalog: CardCatalog
var _filter := ALL
var _trio_count: Label
var _sections: VBoxContainer
var _sheet := CardSheet.new()
## La carte dont la fiche est ouverte.
var _shown: Dictionary = {}


func _init(catalog: CardCatalog) -> void:
	super(Labels.CARDS_TITLE)
	_catalog = catalog


func _fill() -> void:
	# La fiche passe par-dessus tout l'écran, hors de la mise en page.
	_sheet.top_level = true
	_sheet.action_pressed.connect(_toggle_shown)
	add_child(_sheet)
	if _catalog == null:
		body.add_child(_paragraph(Labels.CARDS_LOADING))
		return
	var families := _catalog.families()
	var reactions := families.filter(func(family: Variant) -> bool: return family is Dictionary and family.get("reaction", false))
	set_aside(Labels.CARDS_COUNT % [families.size(), reactions.size()])
	_trio_count = Ui.label("", ThemeBuilder.GOLD_CAPS_LABEL)
	body.add_child(_trio_count)
	body.add_child(_paragraph(Labels.TRIO_HELP, ThemeBuilder.CAPTION_LABEL))
	var choices := {ALL: Labels.ALL_RARITIES}
	choices.merge(Labels.RARITY_FILTERS)
	choices[REACTIONS] = Labels.REACTIONS_FILTER
	body.add_child(_filter_row(choices, _filter, func(value: String) -> void:
		_filter = value
		_show_families()))
	_sections = Ui.vbox(Look.GAP_L)
	body.add_child(_sections)
	_show_families()


## Chaque famille que garde le filtre : son nom, un trait, puis ses cartes.
func _show_families() -> void:
	Ui.clear(_sections)
	_trio_count.text = Labels.TRIO_COUNT % [Settings.cache_trio.size(), Settings.TRIO_SIZE]
	for family: Variant in _catalog.families():
		if not (family is Dictionary) or (_filter == REACTIONS and not family.get("reaction", false)):
			continue
		var cards := HFlowContainer.new()
		cards.add_theme_constant_override(&"h_separation", Look.CARD_GAP)
		cards.add_theme_constant_override(&"v_separation", Look.CARD_GAP)
		for rarity: Variant in family.get("rarities", []):
			var rarity_id: String = rarity.get("rarity", "") if rarity is Dictionary else ""
			if _filter != ALL and _filter != REACTIONS and rarity_id != _filter:
				continue
			cards.add_child(_card({"kind": family.get("kind", ""), "rarity": rarity_id}))
		if cards.get_child_count() == 0:
			cards.queue_free()
			continue
		var title := Ui.hbox(Look.GAP_M)
		title.add_child(Ui.label(family.get("name", ""), ThemeBuilder.NAME_LABEL))
		var line := HSeparator.new()
		line.theme_type_variation = ThemeBuilder.DIVIDER
		line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		line.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		title.add_child(line)
		_sections.add_child(title)
		_sections.add_child(cards)


func _card(attack: Dictionary) -> CardView:
	var card := CardView.new(Look.DRAFT_CARD_WIDTH)
	card.show_attack(attack, _catalog, 0)
	if Settings.in_trio(attack):
		card.set_tag(Labels.SHEET_YOUR_CACHE)
	card.pressed.connect(_open.bind(attack))
	card.details_requested.connect(_open.bind(attack))
	return card


## La fiche de cette carte, avec son bouton du trio. Le coût et la recharge viennent de la
## partie : ici, on ne les connaît pas.
func _open(attack: Dictionary) -> void:
	_shown = attack
	_sheet.show_card(attack, _catalog, {}, Settings.in_trio(attack))
	_refresh_action()


func _refresh_action() -> void:
	if Settings.in_trio(_shown):
		_sheet.set_action(Labels.TRIO_REMOVE, true)
	elif Settings.cache_trio.size() < Settings.TRIO_SIZE:
		_sheet.set_action(Labels.TRIO_ADD, true)
	else:
		_sheet.set_action(Labels.TRIO_FULL, false)


func _toggle_shown() -> void:
	Settings.toggle_trio(_shown)
	_sheet.show_card(_shown, _catalog, {}, Settings.in_trio(_shown))
	_refresh_action()
	_show_families()
