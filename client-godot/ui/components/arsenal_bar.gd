class_name ArsenalBar
extends ScrollContainer
## L'arsenal du joueur : une carte par emplacement, dans l'ordre de la vue, centrées. Une
## carte n'est vive que si le coup qui la lance (`UseAttack` de son emplacement) est permis
## maintenant. Quand les cartes ne tiennent pas dans la largeur (les variantes d'essai, où
## chacun a tout le catalogue), la rangée défile au doigt.

## Le joueur touche la carte de cet emplacement, et elle est jouable.
signal card_chosen(slot: int)

var _row: HBoxContainer
var _cards: Array[CardView] = []


func _init() -> void:
	horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	mouse_filter = Control.MOUSE_FILTER_PASS
	_row = Ui.hbox(Look.CARD_GAP)
	_row.alignment = BoxContainer.ALIGNMENT_CENTER
	_row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_row)
	# La rangée prend la hauteur d'une carte (qui réserve déjà sa place levée).
	custom_minimum_size.y = CardView.footprint().y


## Montre l'arsenal de ce joueur, tel que la vue le décrit.
func show_arsenal(view: ViewData, player: String, catalog: CardCatalog) -> void:
	var slots := view.arsenal(player)
	while _cards.size() < slots.size():
		var card := CardView.new()
		card.slot = _cards.size()
		card.pressed.connect(card_chosen.emit.bind(card.slot))
		_row.add_child(card)
		_cards.append(card)
	for index in _cards.size():
		var card := _cards[index]
		card.visible = index < slots.size()
		if not card.visible:
			continue
		var slot: Variant = slots[index]
		if not (slot is Dictionary):
			card.show_card(null, "", 0, false, 0, false, false)
			continue
		var attack: Dictionary = slot.get("attack", {})
		var kind: String = attack.get("kind", "")
		card.show_card(
			attack,
			catalog.family_name(kind) if catalog != null else kind,
			view.stats_of(attack).get("cost", 0),
			catalog != null and catalog.is_reaction(kind),
			slot.get("cooldown_left", 0),
			slot.get("secret", false),
			view.use_attack(index) != null,
		)


## Lève la carte de cet emplacement (une carte en cours de choix), ou aucune (-1).
func select(slot: int) -> void:
	for card in _cards:
		card.set_selected(card.slot == slot)
