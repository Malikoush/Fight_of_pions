class_name ReactionPanel
extends PanelContainer
## La fenêtre de réaction (maquettes), à la place de l'arsenal et de la barre : ce que
## l'adversaire vient de jouer, « Réagir ? », tes cartes de réaction (celles que le serveur
## permet sont vives, avec ce qu'elles font), et « PASSER » ou « JOUER … ». Toucher une carte
## vive la choisit ; la jouer ouvre son choix, comme une carte de l'arsenal.

## Le joueur passe : il ne réagit pas.
signal pass_requested
## Le joueur joue la réaction de cet emplacement.
signal reaction_chosen(slot: int)

var _title: Label
var _cards: HBoxContainer
var _play: Button
var _chosen := -1
var _names := {}


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL_THREAT
	var column := Ui.vbox(Look.GAP_M)
	_title = Ui.centered_label("", ThemeBuilder.HEADING_LABEL)
	column.add_child(_title)
	column.add_child(Ui.centered_label(Labels.REACT_QUESTION, ThemeBuilder.THREAT_LABEL))
	# Les cartes défilent au doigt quand elles ne tiennent pas (les variantes d'essai donnent
	# toutes les réactions), comme l'arsenal.
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	column.add_child(scroll)
	_cards = Ui.hbox(Look.GAP_M)
	_cards.alignment = BoxContainer.ALIGNMENT_CENTER
	_cards.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_cards)

	var buttons := Ui.hbox(Look.GAP_M)
	var skip := Ui.button(Labels.ACTION_LABELS["Pass"], ThemeBuilder.SECONDARY_BUTTON)
	skip.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	skip.pressed.connect(pass_requested.emit)
	buttons.add_child(skip)
	_play = Ui.button("", ThemeBuilder.PRIMARY_BUTTON)
	_play.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_play.size_flags_stretch_ratio = Look.PLAY_BUTTON_STRETCH
	# Un nom de carte long (« JOUER QUEUE DU LÉZARD ») s'abrège plutôt que d'élargir le bouton.
	_play.clip_text = true
	_play.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	_play.pressed.connect(_on_play)
	buttons.add_child(_play)
	column.add_child(buttons)
	add_child(column)


## Montre la fenêtre pour cette vue : `trigger` dit ce que l'adversaire vient de jouer (une
## ligne du journal, vide si on ne le sait pas, comme à la reprise d'une partie).
func show_window(view: ViewData, trigger: String, catalog: CardCatalog) -> void:
	_title.text = trigger if trigger != "" else Labels.REACT_DEFAULT_TITLE
	for old in _cards.get_children():
		_cards.remove_child(old)
		old.queue_free()
	_names.clear()
	var me := view.viewer()
	var playable: Array[int] = []
	var others: Array[int] = []
	var slots := view.arsenal(me)
	for index in slots.size():
		var slot: Variant = slots[index]
		if not (slot is Dictionary):
			continue
		var kind: String = slot.get("attack", {}).get("kind", "")
		var usable := view.use_attack(index) != null
		# Sans catalogue, on ne sait pas quelles cartes sont des réactions : on montre celles
		# que le serveur permet, qui en sont forcément pendant le tour adverse.
		if not (usable or (catalog != null and catalog.is_reaction(kind))):
			continue
		_names[index] = catalog.family_name(kind) if catalog != null else kind
		if usable:
			playable.append(index)
		else:
			others.append(index)
	# Les réactions jouables d'abord, les autres ensuite, plus ternes.
	for index in playable + others:
		_cards.add_child(_card_column(view, me, index, catalog, index in playable))
	_choose(playable[0] if playable.size() == 1 else -1)


func _card_column(view: ViewData, me: String, index: int, catalog: CardCatalog, usable: bool) -> VBoxContainer:
	var column := Ui.vbox(Look.GAP_XS)
	var card := CardView.new(Look.REACTION_CARD_WIDTH)
	card.slot = index
	card.show_slot(view, me, index, catalog)
	card.pressed.connect(_choose.bind(index))
	column.add_child(card)
	var kind: String = view.arsenal(me)[index].get("attack", {}).get("kind", "")
	var caption := Ui.centered_label(catalog.summary(kind) if usable and catalog != null else Labels.NOT_PLAYABLE_NOW, ThemeBuilder.CAPTION_LABEL)
	caption.custom_minimum_size.x = Look.REACTION_CARD_WIDTH
	caption.max_lines_visible = Look.CAPTION_LINES
	caption.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	column.add_child(caption)
	return column


## Choisit la réaction de cet emplacement (-1 : aucune) : sa carte se lève, et le bouton
## dit son nom.
func _choose(slot: int) -> void:
	_chosen = slot
	for column in _cards.get_children():
		var card: CardView = column.get_child(0)
		card.set_selected(card.slot == slot)
	_play.disabled = slot < 0
	_play.text = Labels.PLAY_REACTION % str(_names.get(slot, "")).to_upper() if slot >= 0 else Labels.PLAY_CARD


func _on_play() -> void:
	if _chosen >= 0:
		reaction_chosen.emit(_chosen)
