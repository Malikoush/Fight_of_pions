class_name DraftScreen
extends Screen
## Le draft, avant la partie (maquettes 1-5, p. 7 à 9), dans l'ordre du format : bans, picks
## en serpent, puis la cache.
## - En haut : la flèche (quitter la partie) et la barre des étapes.
## - L'adversaire : son palet, son nom, ce qu'il fait ou vient de faire, ses cartes.
## - Ce qu'il faut choisir, et pendant les picks, leur ordre en serpent.
## - Les cartes : pendant les bans et les picks, toutes les cartes tirées restent à leur
##   place, celles déjà prises ou bannies assombries et étiquetées (« TOI », « BOT »,
##   « BANNI ») ; pendant la cache, tout le catalogue, avec des filtres par rareté.
## - Mon arsenal et ses places encore vides, puis le bouton qui joue le choix.
##
## Rien n'est déduit : les étapes viennent du format de la partie (`config.draft`), ce qui a
## été choisi de l'historique de la vue (`draft_history`), les cartes qu'on peut toucher des
## coups permis (`Draft`).

## Le joueur joue ce coup du draft, recopié des coups permis.
signal play_requested(action: Variant)
## Le joueur quitte la partie.
signal home_requested

const DRAFT := "Draft"
## La carte (`{"kind", "rarity"}`) que montre chaque carte de la grille.
const ATTACK_META := &"attack"
## Les étapes dont le choix s'étiquette sur les cartes tirées (« BANNI », « TOI »…) : leur
## carte reste montrée à sa place. C'est de l'affichage : ce qu'on peut choisir vient
## toujours des coups permis.
const POOL_STEPS: Array[String] = ["Ban", "Pick"]
const ALL := ""

## Le nom affiché de l'adversaire, et le catalogue des cartes : ceux de la session.
var opponent_name: String:
	get:
		return session.opponent_name
var catalog: CardCatalog:
	get:
		return session.catalog

var _view: ViewData
var _waiting := false
var _selected: Variant = null
var _filter := ALL
## Le catalogue avec lequel l'écran a été dessiné.
var _shown_catalog: CardCatalog
var _quit_dialog: AlertDialog
var _card_sheet := CardSheet.new()

@onready var _page: MarginContainer = %Page
@onready var _back: Button = %Back
@onready var _steps: StepBar = %Steps
@onready var _opponent_chip: TextureRect = %OpponentChip
@onready var _opponent_name: Label = %OpponentName
@onready var _hourglass: TextureRect = %Hourglass
@onready var _status: Label = %Status
@onready var _opponent_cards: MiniArsenal = %OpponentCards
@onready var _title: Label = %Title
@onready var _help: Label = %Help
@onready var _snake: SnakeOrder = %Snake
@onready var _filters: ScrollContainer = %Filters
@onready var _filter_row: HBoxContainer = %FilterRow
@onready var _cards: ScrollContainer = %Cards
@onready var _grid: GridContainer = %Grid
@onready var _mine_title: Label = %MineTitle
@onready var _mine_plan: Label = %MinePlan
@onready var _my_cards: MiniArsenal = %MyCards
@onready var _play: Button = %Play


func _ready() -> void:
	Ui.fit_page(_page, Look.DRAFT_MAX_WIDTH)
	_back.icon = load(Ui.ICON_DIR + "retour.svg")
	_back.expand_icon = true
	_back.tooltip_text = Labels.BACK
	_back.custom_minimum_size = Vector2.ONE * Look.ICON_BUTTON_SIZE
	_back.pressed.connect(func() -> void: _quit_dialog.ask())
	_opponent_chip.custom_minimum_size = Look.CHIP_SIZE
	_hourglass.texture = load(Ui.ICON_DIR + Ui.HOURGLASS + ".svg")
	_hourglass.modulate = Look.GOLD
	_hourglass.custom_minimum_size = Vector2.ONE * Look.FACT_ICON_SIZE
	_mine_title.text = Labels.MY_ARSENAL
	_play.custom_minimum_size.y = Look.BUTTON_HEIGHT
	_play.pressed.connect(_on_play)
	_build_filters()
	_cards.resized.connect(_fit_columns)
	add_child(_card_sheet)
	_quit_dialog = AlertDialog.quit_game()
	_quit_dialog.confirmed.connect(home_requested.emit)
	add_child(_quit_dialog)


## Montre cette vue du draft. Les événements ne servent pas : l'historique de la vue dit
## tout ce qui a été choisi, même après une reprise.
func on_view(view: ViewData, _events: Array) -> void:
	_waiting = false
	_redraw(view)


## Le serveur a refusé le choix : on peut rechoisir, sur une vue redemandée.
func on_refused(_category: ServerErrors.Category, _message: String) -> Reply:
	_waiting = false
	if _view != null:
		_refresh_button(Wire.tag(_view.draft_step()))
	return Reply.RESYNC


## Le catalogue est arrivé : les noms des cartes changent, l'écran se redessine en gardant
## la carte choisie. Le reste de la session (présence, bot) ne change rien ici.
func on_session_changed() -> void:
	if catalog == _shown_catalog or _view == null:
		return
	var selected: Variant = _selected
	_redraw(_view)
	if selected != null:
		for card: Node in _grid.get_children():
			var shown := card as CardView
			if shown != null and shown.get_meta(ATTACK_META, null) == selected:
				_select(selected, shown)


## Redessine l'écran pour cette vue, sans toucher à l'attente d'une réponse (un filtre
## changé ne doit pas permettre de rejouer).
func _redraw(view: ViewData) -> void:
	_view = view
	_shown_catalog = catalog
	_selected = null
	var step: Variant = view.draft_step()
	var kind := Wire.tag(step)
	var mine: bool = Wire.content(step) == view.viewer()
	_show_steps(view)
	_show_opponent(view)
	_filters.visible = kind == "Cache" and mine
	_title.text = Labels.DRAFT_TITLES.get(kind, "") if mine else Labels.DRAFT_WAIT % opponent_name
	_help.text = _help_text(view, kind)
	_show_snake(view, kind)
	_show_cards(view, kind, mine)
	_show_mine(view)
	_refresh_button(kind)


# --- Les parties de l'écran ---

## La barre des étapes : une par phase du format (des étapes d'une même sorte à la suite),
## celle de l'étape en cours allumée.
func _show_steps(view: ViewData) -> void:
	var names: Array[String] = []
	var current := 0
	var previous := ""
	var steps := view.draft_steps()
	for index in steps.size():
		var kind := Wire.tag(steps[index])
		if kind != previous:
			names.append(Labels.DRAFT_PHASES.get(kind, kind))
			previous = kind
		if index == view.draft_step_index():
			current = names.size() - 1
	_steps.show_phases(names, current)


## L'adversaire : son palet, son nom, ce qu'il choisit (avec un sablier) ou vient de faire,
## et ses cartes avec ses places encore vides.
func _show_opponent(view: ViewData) -> void:
	var opponent := view.opponent()
	_opponent_chip.texture = Ui.chip_texture(opponent)
	_opponent_name.text = opponent_name
	var step: Variant = view.draft_step()
	var thinking: bool = Wire.content(step) == opponent
	_hourglass.visible = thinking
	_status.text = Labels.DRAFT_THINKING.get(Wire.tag(step), "") if thinking else _last_choices(view, opponent)
	_opponent_cards.show_arsenal(view, opponent, catalog, _remaining(view, opponent), Look.OPPONENT_CARD_WIDTH, false)


## Ce que ce joueur vient de faire : ses derniers choix d'une même sorte, à la suite
## (« a pris Attila, Médecin »), ou rien.
func _last_choices(view: ViewData, player: String) -> String:
	var history := view.draft_history()
	var last := history.size() - 1
	while last >= 0 and Wire.content(history[last].get("step")) != player:
		last -= 1
	if last < 0:
		return ""
	var kind := Wire.tag(history[last].get("step"))
	if not Labels.DRAFT_DONE.has(kind):
		return Labels.DRAFT_CACHED
	var names: Array[String] = []
	var index := last
	while index >= 0 and Wire.content(history[index].get("step")) == player and Wire.tag(history[index].get("step")) == kind:
		names.push_front(_family(history[index].get("attack")))
		index -= 1
	return Labels.DRAFT_DONE[kind] % ", ".join(names)


## L'aide de l'étape ; pendant les picks, leur rang (« Pick 4 sur 6 · en serpent »).
func _help_text(view: ViewData, kind: String) -> String:
	if kind != "Pick":
		return Labels.DRAFT_HELP.get(kind, "")
	var picks := _pick_players(view)
	return Labels.DRAFT_PICK_RANK % [_pick_rank(view) + 1, picks.size()]


## Pendant les picks, leur ordre en serpent.
func _show_snake(view: ViewData, kind: String) -> void:
	_snake.visible = kind == "Pick"
	if _snake.visible:
		_snake.show_order(_pick_players(view), _pick_rank(view), view.viewer(), _opponent_tag())


## Le joueur de chaque pick du format, dans l'ordre.
func _pick_players(view: ViewData) -> Array[String]:
	var players: Array[String] = []
	for step: Variant in view.draft_steps():
		if Wire.tag(step) == "Pick":
			players.append(Wire.content(step))
	return players


## Le rang du pick en cours parmi les picks (le nombre de picks déjà faits).
func _pick_rank(view: ViewData) -> int:
	var rank := 0
	for entry: Variant in view.draft_history():
		if Wire.tag(entry.get("step")) == "Pick":
			rank += 1
	return rank


## Le nom court de l'adversaire sur les étiquettes : « BOT » ou « AMI ».
func _opponent_tag() -> String:
	return Labels.TAG_BOT if session.against_bot() else Labels.TAG_FRIEND


func _build_filters() -> void:
	var choices := {ALL: Labels.ALL_RARITIES}
	choices.merge(Labels.RARITY_FILTERS)
	Ui.add_filter_pills(_filter_row, choices, _filter, func(rarity: String) -> void:
		_filter = rarity
		_redraw(_view))


## La grille. Pendant la cache, mon choix : tout le catalogue (les coups permis), filtré par
## rareté. Sinon, toutes les cartes tirées à leur place (l'ordre du catalogue) : celles qui
## restent, et celles déjà bannies ou prises, assombries et étiquetées.
func _show_cards(view: ViewData, kind: String, mine: bool) -> void:
	Ui.clear(_grid)
	if kind == "Cache" and mine:
		# Le trio de cache, choisi dans la page Cartes, passe en tête, étiqueté.
		var shown: Array = []
		for action: Variant in view.legal_actions():
			var attack: Variant = Wire.content(action) if Wire.tag(action) == DRAFT else null
			if attack is Dictionary and (_filter == ALL or attack.get("rarity") == _filter):
				shown.append(attack)
		var trio := shown.filter(func(attack: Dictionary) -> bool: return Settings.in_trio(attack))
		for attack: Dictionary in trio + shown.filter(func(attack: Dictionary) -> bool: return not Settings.in_trio(attack)):
			var card := _add_card(view, attack, true)
			if Settings.in_trio(attack):
				card.set_tag(Labels.SHEET_YOUR_CACHE)
		return
	# Les cartes tirées déjà choisies, et l'étape qui les a choisies, par carte.
	var drawn: Array = view.draft_pool().duplicate()
	var taken := {}
	for entry: Variant in view.draft_history():
		var attack: Variant = entry.get("attack")
		if attack is Dictionary and Wire.tag(entry.get("step")) in POOL_STEPS:
			drawn.append(attack)
			taken[_key(attack)] = entry.get("step")
	drawn.sort_custom(func(a: Variant, b: Variant) -> bool: return view.catalog_index(a) < view.catalog_index(b))
	# Une carte se choisit si son coup est permis, et seulement alors.
	var allowed := {}
	for action: Variant in view.legal_actions():
		if Wire.tag(action) == DRAFT and Wire.content(action) is Dictionary:
			allowed[_key(Wire.content(action))] = true
	for attack: Variant in drawn:
		var key := _key(attack)
		var card := _add_card(view, attack, allowed.has(key))
		if taken.has(key):
			_mark_taken(card, taken[key], view.viewer())


## Une carte (`{"kind", "rarity"}`), en clé de dictionnaire.
static func _key(attack: Dictionary) -> String:
	return "%s/%s" % [attack.get("kind", ""), attack.get("rarity", "")]


## Le nom de la famille d'une carte (« Familia »), comme sur le bouton et l'étiquette.
func _family(attack: Variant) -> String:
	var kind: String = attack.get("kind", "") if attack is Dictionary else ""
	return catalog.family_name(kind) if catalog != null else kind


func _add_card(view: ViewData, attack: Dictionary, choosable: bool) -> CardView:
	var card := CardView.new(Look.DRAFT_CARD_WIDTH)
	card.show_attack(attack, catalog, view.stats_of(attack).get("cost", 0))
	card.outlined = true
	card.disabled = not choosable
	card.set_meta(ATTACK_META, attack)
	card.pressed.connect(_select.bind(attack, card))
	card.details_requested.connect(func() -> void: _card_sheet.show_card(attack, catalog, _view.stats_of(attack), false))
	_grid.add_child(card)
	return card


## Une carte déjà choisie : prise (« TOI » d'or, ou le nom de l'adversaire) ou bannie
## (« BANNI », de braise, avec le nom de l'adversaire si c'est lui).
func _mark_taken(card: CardView, step: Variant, viewer: String) -> void:
	var by_me: bool = Wire.content(step) == viewer
	if Wire.tag(step) == "Ban":
		card.set_tag(Labels.TAG_BANNED if by_me else Labels.TAG_BANNED_BY % _opponent_tag(), CardView.Tag.THREAT)
		card.dim(Look.CARD_BANNED)
	else:
		card.set_tag(Labels.TAG_YOU if by_me else _opponent_tag(), CardView.Tag.GOLD if by_me else CardView.Tag.PLAIN)
		card.dim(Look.CARD_UNPLAYABLE)


## Mon arsenal : ce que le draft y ajoute (« 3 picks + 1 cache »), mes cartes, et mes places
## encore vides, nommées.
func _show_mine(view: ViewData) -> void:
	var counts := {}
	for step: Variant in view.draft_steps():
		var kind := Wire.tag(step)
		if Wire.content(step) == view.viewer() and Labels.DRAFT_PLAN_PARTS.has(kind):
			counts[kind] = counts.get(kind, 0) + 1
	var parts: Array[String] = []
	for kind: String in counts:
		var forms: Array = Labels.DRAFT_PLAN_PARTS[kind]
		parts.append((forms[0] if counts[kind] == 1 else forms[1]) % counts[kind])
	_mine_plan.text = Labels.DRAFT_PLAN_JOIN.join(parts)
	_my_cards.show_arsenal(view, view.viewer(), catalog, _remaining(view, view.viewer()), Look.DRAFT_MINI_CARD, true)


## Les sortes des choix qui restent à ce joueur et lui donnent une carte, dans l'ordre.
func _remaining(view: ViewData, player: String) -> Array[String]:
	var kinds: Array[String] = []
	var steps := view.draft_steps()
	for index in range(view.draft_step_index(), steps.size()):
		var kind := Wire.tag(steps[index])
		if Wire.content(steps[index]) == player and Labels.DRAFT_SLOTS.has(kind):
			kinds.append(kind)
	return kinds


# --- Le choix ---

func _select(attack: Dictionary, card: CardView) -> void:
	if _waiting:
		return
	_selected = attack
	var kind := Wire.tag(_view.draft_step())
	for node: Node in _grid.get_children():
		var other := node as CardView
		if other.disabled:
			continue
		other.set_selected(other == card)
		# Une carte à bannir le dit au-dessus d'elle (maquettes : « BANNIR ? ») ; aux autres
		# étapes, les étiquettes restent (le trio de cache).
		if kind == "Ban":
			other.set_tag(Labels.TAG_TO_BAN if other == card else "", CardView.Tag.THREAT)
	_refresh_button(kind)


func _refresh_button(kind: String) -> void:
	var mine: bool = _view != null and Wire.content(_view.draft_step()) == _view.viewer()
	_play.visible = mine
	if _selected is Dictionary:
		var card_name := _family(_selected)
		var template: String = Labels.DRAFT_ACTIONS.get(kind, "%s")
		_play.text = template % card_name.to_upper() if template.contains("%s") else template
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


## Autant de colonnes de cartes que la largeur de la grille en laisse tenir (la largeur
## vient de la page, jamais des cartes : rien ne s'élargit en boucle).
func _fit_columns() -> void:
	var cell := CardView.footprint(Look.DRAFT_CARD_WIDTH).x + Look.CARD_GAP
	var width := _cards.size.x - 2 * Look.CARD_GRID_SIDE
	_grid.columns = maxi(1, int((width + Look.CARD_GAP) / cell))
