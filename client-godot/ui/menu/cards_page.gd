class_name CardsPage
extends MenuPage
## La collection des cartes (maquettes 12-17, p. 5), telle que le serveur la décrit : le
## compte des familles et des réactions, deux rangées de filtres, puis chaque famille avec
## les cartes qui passent les filtres, de la Commune à la Légendaire.
## - Ce qu'on voit : ses cartes (par défaut), ou tout le catalogue, les cartes pas encore
##   gagnées grisées (« À GAGNER »).
## - Lesquelles : toutes, une rareté, ou les réactions.
##
## En haut, l'encart du trio de cache (`TrioPanel`) expose ses cartes, toujours visible quels
## que soient les filtres. Toucher une carte (de l'encart ou de la liste) ouvre sa fiche
## (`CardSheet`), d'où l'on met une de ses cartes dans son trio de cache, ou l'en retire. Le trio est gardé par le serveur : la page demande le
## nouveau trio (`cache_trio_submitted`) et attend la collection qui lui répond (`refresh`).
## Les cartes du trio portent « TA CACHE ».

## Le joueur demande ce trio de cache (`{"kind", "rarity"}`, de 0 à 3 cartes).
signal cache_trio_submitted(cards: Array)

## Ce qu'on voit : ses cartes, ou tout le catalogue.
enum Scope { OWNED, CATALOG }

## Les filtres de la seconde rangée : toutes les cartes, une rareté, ou les réactions.
const ALL := ""
const REACTIONS := "reactions"

var _session: Session
var _scope := Scope.OWNED
var _kind := ALL
var _trio := TrioPanel.new()
var _error: Label
var _filters: VBoxContainer
var _sections: VBoxContainer
var _sheet := CardSheet.new()
## La carte dont la fiche est ouverte.
var _shown: Dictionary = {}
## Vrai entre une demande de trio et la collection qui lui répond : le bouton de la fiche
## attend.
var _waiting := false


func _init(session: Session) -> void:
	super(Labels.CARDS_TITLE)
	_session = session


func _fill() -> void:
	# La fiche passe par-dessus tout l'écran, hors de la mise en page.
	_sheet.top_level = true
	_sheet.action_pressed.connect(_toggle_shown)
	add_child(_sheet)
	var catalog := _session.catalog
	if catalog == null:
		body.add_child(_paragraph(Labels.CARDS_LOADING))
		return
	var families := catalog.families()
	var reactions := families.filter(func(family: Variant) -> bool: return family is Dictionary and family.get("reaction", false))
	set_aside(Labels.CARDS_COUNT % [families.size(), reactions.size()])
	_trio.card_pressed.connect(_open)
	body.add_child(_trio)
	_error = _paragraph("", ThemeBuilder.THREAT_LABEL)
	_error.visible = false
	body.add_child(_error)
	_filters = Ui.vbox(Look.GAP_S)
	body.add_child(_filters)
	_sections = Ui.vbox(Look.GAP_L)
	body.add_child(_sections)
	refresh()


## La collection a changé (ou vient d'arriver) : l'encart du trio, les filtres (leurs
## nombres), les cartes et la fiche ouverte la reprennent ; une demande de trio a sa réponse.
func refresh() -> void:
	if _sections == null:
		return
	_waiting = false
	_trio.show_trio(_collection(), _session.catalog)
	_show_filters()
	_show_families()
	if _sheet.visible:
		_open(_shown)


## Le serveur refuse le trio demandé : son message, sous l'encart ; la collection redemandée
## (par la racine) remettra la page d'aplomb.
func show_refusal(message: String) -> void:
	if _error == null:
		return
	_error.text = Labels.sentence(message)
	_error.visible = true


## La collection reçue, ou vide tant qu'elle n'est pas là.
func _collection() -> CardCollection:
	return _session.collection if _session.collection != null else CardCollection.new()


## Les deux rangées de filtres, avec le nombre de ses cartes et de toutes.
func _show_filters() -> void:
	Ui.clear(_filters)
	var scopes := {
		Scope.OWNED: Labels.OWNED_FILTER % _collection().size(),
		Scope.CATALOG: Labels.CATALOG_FILTER % _catalog_cards().size(),
	}
	_filters.add_child(_filter_row(scopes, _scope, func(scope: Scope) -> void:
		_scope = scope
		_show_families()))
	var kinds := {ALL: Labels.ALL_RARITIES}
	kinds.merge(Labels.RARITY_FILTERS)
	kinds[REACTIONS] = Labels.REACTIONS_FILTER
	_filters.add_child(_filter_row(kinds, _kind, func(kind: String) -> void:
		_kind = kind
		_show_families()))


## Chaque famille qui a une carte gardée par les filtres : son nom, un trait, puis ces
## cartes. Sans aucune, une phrase le dit.
func _show_families() -> void:
	Ui.clear(_sections)
	var collection := _collection()
	for family: Dictionary in _session.catalog.families():
		var cards := HFlowContainer.new()
		cards.add_theme_constant_override(&"h_separation", Look.CARD_GAP)
		cards.add_theme_constant_override(&"v_separation", Look.CARD_GAP)
		for attack: Dictionary in _cards_of(family):
			if _passes(family, attack, collection):
				cards.add_child(_card(attack, collection))
		if cards.get_child_count() == 0:
			cards.queue_free()
			continue
		_sections.add_child(_family_title(family.get("name", "")))
		_sections.add_child(cards)
	if _sections.get_child_count() == 0:
		var waiting := _session.collection == null and _scope == Scope.OWNED
		_sections.add_child(_paragraph(Labels.COLLECTION_LOADING if waiting else Labels.NO_CARD_MATCHES))


## Vrai si cette carte de cette famille passe les deux filtres.
func _passes(family: Dictionary, attack: Dictionary, collection: CardCollection) -> bool:
	if _scope == Scope.OWNED and not collection.owns(attack):
		return false
	match _kind:
		ALL:
			return true
		REACTIONS:
			return family.get("reaction", false)
	return attack["rarity"] == _kind


## Les cartes d'une famille du catalogue (`{"kind", "rarity"}`), de la Commune à la
## Légendaire.
static func _cards_of(family: Dictionary) -> Array[Dictionary]:
	var cards: Array[Dictionary] = []
	for rarity: Variant in family.get("rarities", []):
		if rarity is Dictionary and rarity.get("rarity") is String:
			cards.append({"kind": family.get("kind", ""), "rarity": rarity["rarity"]})
	return cards


## Toutes les cartes du catalogue.
func _catalog_cards() -> Array[Dictionary]:
	var cards: Array[Dictionary] = []
	for family: Dictionary in _session.catalog.families():
		cards.append_array(_cards_of(family))
	return cards


## Le nom d'une famille, suivi d'un trait.
func _family_title(family_name: String) -> HBoxContainer:
	var title := Ui.hbox(Look.GAP_M)
	title.add_child(Ui.label(family_name, ThemeBuilder.NAME_LABEL))
	var line := HSeparator.new()
	line.theme_type_variation = ThemeBuilder.DIVIDER
	line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	line.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	title.add_child(line)
	return title


## Une carte : étiquetée « TA CACHE » dans le trio, grisée et « À GAGNER » si elle n'est
## pas encore à soi.
func _card(attack: Dictionary, collection: CardCollection) -> CardView:
	var card := CardView.new(Look.DRAFT_CARD_WIDTH)
	card.show_attack(attack, _session.catalog, 0)
	if collection.in_trio(attack):
		card.set_tag(Labels.SHEET_YOUR_CACHE)
	elif not collection.owns(attack):
		card.set_tag(Labels.TAG_NOT_OWNED, CardView.Tag.PLAIN)
		card.dim(Look.CARD_UNPLAYABLE)
	card.pressed.connect(_open.bind(attack))
	card.details_requested.connect(_open.bind(attack))
	return card


## La fiche de cette carte, avec son bouton du trio. Le coût et la recharge viennent de la
## partie : ici, on ne les connaît pas.
func _open(attack: Dictionary) -> void:
	_shown = attack
	var collection := _collection()
	_sheet.show_card(attack, _session.catalog, {}, collection.in_trio(attack))
	if not collection.owns(attack):
		_sheet.set_action(Labels.TRIO_NOT_OWNED, false)
	elif collection.in_trio(attack):
		_sheet.set_action(Labels.TRIO_REMOVE, not _waiting)
	elif collection.trio_full():
		_sheet.set_action(Labels.TRIO_FULL, false)
	else:
		_sheet.set_action(Labels.TRIO_ADD, not _waiting)


## Demande le trio avec la carte de la fiche, ou sans elle ; le bouton attend la réponse.
func _toggle_shown() -> void:
	var collection := _collection()
	var trio := collection.trio_without(_shown) if collection.in_trio(_shown) else collection.trio_with(_shown)
	_waiting = true
	_error.visible = false
	cache_trio_submitted.emit(trio)
	_open(_shown)
