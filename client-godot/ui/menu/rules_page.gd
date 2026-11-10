class_name RulesPage
extends MenuPage
## L'essentiel des règles en un carrousel (maquettes 12-17, p. 7 à 12) : une page à la fois
## (`Labels.RULES`, tirées du carnet de règles), avec son dessin, son titre et son texte,
## des points qui disent où l'on en est, et « PRÉCÉDENT » / « SUIVANT » ; à la dernière,
## « JOUER » ramène à l'accueil. Le détail de chaque carte est dans la page Cartes.
##
## Les dessins sont faits des éléments du jeu : le plateau (l'aperçu de l'accueil), les
## palets, les losanges des points, les cartes du catalogue.

## « JOUER », à la dernière page.
signal play_requested

## Ce que montre le dessin de chaque page, dans l'ordre de `Labels.RULES`.
enum Picture { BOARD, TURN, CARDS, REACTION, GOLDEN_RULE, DRAFT }
const PICTURES: Array[Picture] = [Picture.BOARD, Picture.TURN, Picture.CARDS, Picture.REACTION, Picture.GOLDEN_RULE, Picture.DRAFT]

var _catalog: CardCatalog
var _page := 0
var _picture: PanelContainer
var _preview: BoardPreview
var _page_title: Label
var _text: Label
var _dots: HBoxContainer
var _previous: Button
var _next: Button


func _init(catalog: CardCatalog) -> void:
	super(Labels.RULES_TITLE)
	_catalog = catalog


func _fill() -> void:
	_picture = PanelContainer.new()
	_picture.theme_type_variation = ThemeBuilder.BOARD_PANEL
	_picture.clip_children = CanvasItem.CLIP_CHILDREN_AND_DRAW
	_picture.custom_minimum_size.y = Look.RULES_PICTURE_HEIGHT
	body.add_child(_picture)
	_page_title = Ui.label("", ThemeBuilder.LOGO_LABEL)
	body.add_child(_page_title)
	_text = _paragraph("", ThemeBuilder.STRONG_LABEL)
	body.add_child(_text)
	# Les points et les boutons restent en bas de la page, hors de ce qui défile : ils ne
	# bougent pas d'une page à l'autre.
	var footer := Ui.vbox(Look.GAP_M)
	_dots = Ui.hbox(Look.GAP_XS)
	_dots.alignment = BoxContainer.ALIGNMENT_CENTER
	footer.add_child(_dots)
	var buttons := Ui.hbox(Look.GAP_M)
	_previous = Ui.button(Labels.PREVIOUS, ThemeBuilder.SECONDARY_BUTTON)
	_previous.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_previous.pressed.connect(func() -> void: _show_page(_page - 1))
	buttons.add_child(_previous)
	_next = Ui.button(Labels.NEXT, ThemeBuilder.PRIMARY_BUTTON)
	_next.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_next.pressed.connect(_on_next)
	buttons.add_child(_next)
	footer.add_child(buttons)
	add_child(footer)
	_show_page(0)


func _on_next() -> void:
	if _page == Labels.RULES.size() - 1:
		play_requested.emit()
	else:
		_show_page(_page + 1)


func _show_page(page: int) -> void:
	_page = clampi(page, 0, Labels.RULES.size() - 1)
	var section: Array = Labels.RULES[_page]
	_page_title.text = section[0]
	_text.text = section[1]
	set_aside(Labels.RULES_PAGE % [_page + 1, Labels.RULES.size()])
	_previous.disabled = _page == 0
	_next.text = Labels.PLAY if _page == Labels.RULES.size() - 1 else Labels.NEXT
	# Les points : celui de la page, d'or et plus long.
	Ui.clear(_dots)
	for index in Labels.RULES.size():
		var dot := Panel.new()
		dot.theme_type_variation = ThemeBuilder.PROGRESS_FILL if index == _page else ThemeBuilder.PROGRESS_TRACK
		dot.custom_minimum_size = Look.RULES_DOT_CURRENT if index == _page else Look.RULES_DOT
		_dots.add_child(dot)
	_show_picture(PICTURES[_page])


## Le dessin de la page : le plateau (gardé d'une page à l'autre), ou des éléments du jeu.
func _show_picture(picture: Picture) -> void:
	for child in _picture.get_children():
		if child != _preview:
			child.queue_free()
	var board := picture in [Picture.BOARD, Picture.GOLDEN_RULE]
	if board and _preview == null:
		_preview = BoardPreview.new()
		_picture.add_child(_preview)
	if _preview != null:
		_preview.visible = board
	if board:
		return
	var center := CenterContainer.new()
	center.mouse_filter = Control.MOUSE_FILTER_IGNORE
	match picture:
		Picture.TURN:
			center.add_child(_turn_picture())
		Picture.CARDS:
			center.add_child(_cards_row(_rarities_of(_first_family(false))))
		Picture.REACTION:
			center.add_child(_reaction_picture())
		Picture.DRAFT:
			center.add_child(_draft_picture())
	_picture.add_child(center)


## Ton tour : un palet (« 1 POSE ») plus un losange (« 1 POINT »), et la banque.
func _turn_picture() -> Control:
	var column := Ui.vbox(Look.GAP_M)
	var row := Ui.hbox(Look.GAP_XL)
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	var place := Ui.vbox(Look.GAP_XS)
	place.add_child(Ui.chip(ViewData.PLAYER_ONE, Look.RULES_CHIP))
	place.add_child(_caption(Labels.RULES_ONE_PLACE))
	row.add_child(place)
	var point := Ui.vbox(Look.GAP_XS)
	var diamond := DiamondStrip.new()
	diamond.diamond = Look.RULES_DIAMOND
	diamond.filled = 1
	diamond.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	point.add_child(diamond)
	point.add_child(_caption(Labels.RULES_ONE_POINT))
	row.add_child(point)
	column.add_child(row)
	var bank := Ui.hbox(Look.GAP_S)
	bank.alignment = BoxContainer.ALIGNMENT_CENTER
	bank.add_child(Ui.label(Labels.RULES_BANK, ThemeBuilder.SECTION_LABEL))
	var pips := DiamondStrip.new()
	pips.count = Look.RULES_BANK_PIPS
	pips.filled = Look.RULES_BANK_PIPS - 1
	bank.add_child(pips)
	column.add_child(bank)
	return column


## Réactions : « Réagir ? », et une carte de réaction.
func _reaction_picture() -> Control:
	var column := Ui.vbox(Look.GAP_S)
	column.add_child(Ui.centered_label(Labels.REACT_QUESTION, ThemeBuilder.THREAT_LABEL))
	var cards := _cards_row(_rarities_of(_first_family(true)).slice(0, 1))
	column.add_child(cards)
	return column


## Le draft : trois cartes prises, puis la cache, en dos.
func _draft_picture() -> Control:
	var column := Ui.vbox(Look.GAP_S)
	var row := _cards_row(_rarities_of(_first_family(false)).slice(0, Look.RULES_DRAFT_PICKS))
	var cache := CardView.new(Look.RULES_CARD_WIDTH)
	cache.mouse_filter = Control.MOUSE_FILTER_IGNORE
	cache.show_card(null, "", 0, false, 0, false, false)
	row.add_child(cache)
	column.add_child(row)
	column.add_child(Ui.centered_label(Labels.RULES_DRAFT_PLAN, ThemeBuilder.SECTION_LABEL))
	return column


## Une légende d'un dessin, sur une ligne.
func _caption(text: String) -> Label:
	var label := Ui.centered_label(text, ThemeBuilder.PILL_LABEL)
	label.autowrap_mode = TextServer.AUTOWRAP_OFF
	return label


## Une rangée de ces cartes, en petit.
func _cards_row(attacks: Array) -> HBoxContainer:
	var row := Ui.hbox(Look.CARD_GAP)
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	for attack: Variant in attacks:
		var card := CardView.new(Look.RULES_CARD_WIDTH)
		card.mouse_filter = Control.MOUSE_FILTER_IGNORE
		card.show_attack(attack, _catalog, 0)
		row.add_child(card)
	return row


## La première famille du catalogue (une réaction, ou non), ou rien sans catalogue.
func _first_family(reaction: bool) -> Dictionary:
	if _catalog == null:
		return {}
	for family: Variant in _catalog.families():
		if family is Dictionary and family.get("reaction", false) == reaction:
			return family
	return {}


## Les cartes de cette famille, de la Commune à la Légendaire.
func _rarities_of(family: Dictionary) -> Array:
	var attacks: Array = []
	for rarity: Variant in family.get("rarities", []):
		if rarity is Dictionary:
			attacks.append({"kind": family.get("kind", ""), "rarity": rarity.get("rarity", "")})
	return attacks
