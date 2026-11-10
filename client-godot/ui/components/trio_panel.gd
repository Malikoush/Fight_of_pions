class_name TrioPanel
extends PanelContainer
## L'encart du trio de cache : « MA CACHE » et où il en est (« 1 / 3 »), puis ses cartes
## exposées côte à côte, une place en pointillés (« LIBRE ») pour chaque carte qui manque,
## et une phrase d'aide. Toucher une de ses cartes demande sa fiche (`card_pressed`).
##
## Il ne sait rien du serveur : il montre la collection qu'on lui donne (`show_trio`).

## Le joueur touche cette carte du trio (`{"kind", "rarity"}`).
signal card_pressed(attack: Dictionary)

var _count: Label
var _slots: HBoxContainer


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL
	var column := Ui.vbox(Look.GAP_M)
	var header := Ui.hbox(Look.GAP_M)
	var title := Ui.label(Labels.TRIO_TITLE, ThemeBuilder.TAG_LABEL)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(title)
	_count = Ui.label("", ThemeBuilder.GOLD_CAPS_LABEL)
	header.add_child(_count)
	column.add_child(header)
	_slots = Ui.hbox(Look.CARD_GAP)
	_slots.alignment = BoxContainer.ALIGNMENT_CENTER
	column.add_child(_slots)
	var help := Ui.label(Labels.TRIO_HELP, ThemeBuilder.CAPTION_LABEL)
	help.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(help)
	add_child(column)


## Montre le trio de cette collection, les cartes nommées d'après ce catalogue.
func show_trio(collection: CardCollection, catalog: CardCatalog) -> void:
	_count.text = Labels.TRIO_SLOTS % [collection.cache_trio.size(), CardCollection.TRIO_SIZE]
	Ui.clear(_slots)
	for attack: Dictionary in collection.cache_trio:
		var card := CardView.new(Look.TRIO_CARD_WIDTH)
		card.show_attack(attack, catalog, 0)
		card.pressed.connect(card_pressed.emit.bind(attack))
		card.details_requested.connect(card_pressed.emit.bind(attack))
		_slots.add_child(card)
	# Une place libre en pointillés, comme une carte à venir au draft.
	for _free in CardCollection.TRIO_SIZE - collection.cache_trio.size():
		_slots.add_child(CardView.placeholder(Look.TRIO_CARD_WIDTH, Labels.TRIO_FREE))
