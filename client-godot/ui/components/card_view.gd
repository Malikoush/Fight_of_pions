class_name CardView
extends Button
## Une carte : le cadre de sa rareté (ou celui des réactions), son nom sur le ruban, son coût
## dans le losange, un sablier et le nombre de tours quand elle se recharge, et une
## étiquette au-dessus de son coin (« CACHE » pour sa carte cachée ; au draft, qui l'a
## prise ou bannie). Une carte qu'on ne peut pas voir montre son dos.
##
## Tout ce qu'elle montre lui est donné : le coût vient de la configuration de la partie, le
## nom du catalogue, et « jouable » de la présence de son coup dans les coups permis.
##
## Un clic droit, ou un appui long au doigt, demande sa fiche (`details_requested`) : l'appui
## long ne la joue pas.

## Le joueur veut la fiche de cette carte.
signal details_requested

const FRAME_DIR := "res://assets/elements/cartes/"
const FRAMES := {
	"Common": "cadre-commune",
	"Rare": "cadre-rare",
	"Epic": "cadre-epique",
	"Legendary": "cadre-legendaire",
}
const REACTION_FRAME := "cadre-reaction"
const HIDDEN_FRAME := "cadre-dos"
## Les sortes d'étiquette : d'or (« CACHE », « TOI »), crème (prise par l'adversaire), de
## braise (bannie, à bannir).
enum Tag { GOLD, PLAIN, THREAT }
const TAG_STYLES := {
	Tag.GOLD: [ThemeBuilder.TAG, ThemeBuilder.TAG_LABEL],
	Tag.PLAIN: [ThemeBuilder.TAG_PLAIN, ThemeBuilder.TAG_PLAIN_LABEL],
	Tag.THREAT: [ThemeBuilder.TAG_THREAT, ThemeBuilder.TAG_THREAT_LABEL],
}

## L'emplacement de la carte dans l'arsenal.
var slot := -1
## Vrai pour qu'une carte choisie soit aussi entourée d'or (au draft).
var outlined := false

var _face: Control
var _frame: TextureRect
var _name: Label
var _cost: Label
var _cooldown: Control
var _cooldown_turns: Label
var _tag: PanelContainer
var _tag_label: Label
var _outline: Panel
var _lift: Tween
var _selected := false
## L'appui en cours (pour reconnaître un appui long), et si la carte est jouable : c'est ce
## que l'appui long, qui la désactive le temps du relâcher, lui rend ensuite.
var _press := 0
var _long_pressed := false
var _playable := false


func _init(width: float = Look.CARD_WIDTH) -> void:
	theme_type_variation = ThemeBuilder.CARD_BUTTON
	var scale_factor := width / Look.CARD_CANVAS.x
	var face_size := Look.CARD_CANVAS * scale_factor
	custom_minimum_size = footprint(width)
	focus_mode = Control.FOCUS_ALL
	# Tout le dessin vit dans `_face`, qui se lève quand la carte est choisie.
	_face = Control.new()
	_face.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_face.size = face_size
	_face.position.y = Look.CARD_LIFT
	add_child(_face)

	_outline = Panel.new()
	_outline.theme_type_variation = ThemeBuilder.CARD_OUTLINE
	_outline.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_outline.position = -Vector2.ONE * Look.GAP_XS
	_outline.size = face_size + Vector2.ONE * Look.GAP_XS * 2.0
	_outline.visible = false
	_face.add_child(_outline)

	_frame = TextureRect.new()
	_frame.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_frame.stretch_mode = TextureRect.STRETCH_SCALE
	_frame.size = face_size
	_frame.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_face.add_child(_frame)

	_name = _placed_label(ThemeBuilder.CARD_NAME_LABEL, _scaled(Look.CARD_RIBBON, scale_factor))
	var cost_size := Vector2.ONE * Look.CARD_COST_SIZE
	_cost = _placed_label(ThemeBuilder.CARD_COST_LABEL, _scaled(Rect2(Look.CARD_COST_CENTER - cost_size / 2.0, cost_size), scale_factor))

	_cooldown = _build_cooldown(_scaled(Look.CARD_WINDOW, scale_factor))
	_face.add_child(_cooldown)

	_tag = PanelContainer.new()
	_tag.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_tag_label = Ui.label("", ThemeBuilder.TAG_LABEL)
	_tag.add_child(_tag_label)
	_tag.visible = false
	_face.add_child(_tag)


## Montre cette carte. `attack` vaut `{"kind", "rarity"}`, ou `null` pour une carte qu'on ne
## peut pas voir (le dos).
func show_card(attack: Variant, display_name: String, cost: int, reaction: bool, cooldown_left: int, secret: bool, playable: bool) -> void:
	_long_pressed = false
	_playable = attack is Dictionary and playable
	if not (attack is Dictionary):
		_frame.texture = load(FRAME_DIR + HIDDEN_FRAME + ".svg")
		_name.text = ""
		_cost.text = ""
		_cooldown.visible = false
		set_tag("")
		disabled = true
		_face.modulate = Color.WHITE
		return
	var frame: String = REACTION_FRAME if reaction else FRAMES.get(attack.get("rarity"), FRAMES["Common"])
	_frame.texture = load(FRAME_DIR + frame + ".svg")
	_name.text = display_name
	_cost.text = str(cost) if cost > 0 else ""
	_cooldown.visible = cooldown_left > 0
	_cooldown_turns.text = str(cooldown_left)
	set_tag(Labels.SECRET_TAG if secret else "")
	disabled = not playable
	_face.modulate = Color.WHITE if playable or _selected else Look.CARD_UNPLAYABLE
	tooltip_text = display_name


## Montre cette carte (`{"kind", "rarity"}`) face visible, hors de l'arsenal (un choix, une
## annonce) : son nom dans le catalogue, et ce coût.
func show_attack(attack: Dictionary, catalog: CardCatalog, cost: int) -> void:
	var kind: String = attack.get("kind", "")
	var reaction := catalog != null and catalog.is_reaction(kind)
	show_card(attack, catalog.family_name(kind) if catalog != null else kind, cost, reaction, 0, false, true)


## Montre la carte de cet emplacement de l'arsenal de ce joueur, telle que la vue la décrit :
## son coût lu dans la configuration, son nom dans le catalogue, vive si son `UseAttack` est
## permis, ou toujours vive (`vivid` : hors de la partie, comme au draft).
func show_slot(view: ViewData, player: String, index: int, catalog: CardCatalog, vivid: bool = false) -> void:
	var entry: Variant = view.arsenal(player)[index]
	if not (entry is Dictionary):
		show_card(null, "", 0, false, 0, false, false)
		return
	var attack: Dictionary = entry.get("attack", {})
	var kind: String = attack.get("kind", "")
	show_card(
		attack,
		catalog.family_name(kind) if catalog != null else kind,
		view.stats_of(attack).get("cost", 0),
		catalog != null and catalog.is_reaction(kind),
		entry.get("cooldown_left", 0),
		entry.get("secret", false),
		vivid or view.use_attack(index) != null,
	)


func _gui_input(event: InputEvent) -> void:
	var button := event as InputEventMouseButton
	if button == null:
		return
	if button.button_index == MOUSE_BUTTON_RIGHT and button.pressed:
		details_requested.emit()
		accept_event()
	elif button.button_index == MOUSE_BUTTON_LEFT and button.pressed:
		_press += 1
		get_tree().create_timer(Look.LONG_PRESS).timeout.connect(_on_long_press.bind(_press))
	elif button.button_index == MOUSE_BUTTON_LEFT and _long_pressed:
		_long_pressed = false
		disabled = not _playable


## Toujours appuyée après ce délai : la fiche s'ouvre, et le relâcher ne jouera pas la carte
## (elle est désactivée jusque-là).
func _on_long_press(press: int) -> void:
	if press != _press or not Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) or not get_global_rect().has_point(get_global_mouse_position()):
		return
	_long_pressed = true
	disabled = true
	details_requested.emit()


## L'étiquette au-dessus du coin de la carte, de cette sorte ; vide : aucune.
func set_tag(text: String, kind := Tag.GOLD) -> void:
	_tag.visible = text != ""
	if text == "":
		return
	_tag.theme_type_variation = TAG_STYLES[kind][0]
	_tag_label.theme_type_variation = TAG_STYLES[kind][1]
	_tag_label.text = text
	_tag.reset_size()
	_tag.position = Vector2(_face.size.x - _tag.size.x, -_tag.size.y / 2.0)


## Une carte qu'on ne peut plus choisir (prise, bannie) : son dessin assombri de cette
## teinte ; son étiquette reste vive.
func dim(shade: Color) -> void:
	_playable = false
	disabled = true
	for part: Node in _face.get_children():
		if part != _tag and part is CanvasItem:
			(part as CanvasItem).modulate = shade


## Lève la carte choisie au-dessus des autres (et l'entoure d'or, si `outlined`).
func set_selected(selected: bool) -> void:
	_selected = selected
	_outline.visible = selected and outlined
	if _lift != null:
		_lift.kill()
	_lift = create_tween().set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	_lift.tween_property(_face, "position:y", 0.0 if selected else Look.CARD_LIFT, Look.CARD_LIFT_TIME)


func _build_cooldown(window: Rect2) -> Control:
	var shade := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Look.CARD_COOLDOWN_SHADE
	style.set_corner_radius_all(Look.RADIUS_CONTROL)
	shade.add_theme_stylebox_override(&"panel", style)
	shade.mouse_filter = Control.MOUSE_FILTER_IGNORE
	shade.position = window.position
	shade.size = window.size
	# Le sablier et le nombre de tours suivent la taille de la carte : ils tiennent dans sa
	# fenêtre, en petit (l'adversaire) comme en grand.
	var column := Ui.vbox(0)
	column.alignment = BoxContainer.ALIGNMENT_CENTER
	var hourglass := Ui.icon(Ui.HOURGLASS, Look.CREAM, window.size.x * Look.CARD_COOLDOWN_ICON_SHARE)
	hourglass.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	column.add_child(hourglass)
	_cooldown_turns = Ui.centered_label("", ThemeBuilder.NUMBER_LABEL)
	_cooldown_turns.add_theme_font_size_override(&"font_size", maxi(1, roundi(window.size.y * Look.CARD_COOLDOWN_NUMBER_SHARE)))
	column.add_child(_cooldown_turns)
	shade.add_child(column)
	shade.visible = false
	return shade


func _placed_label(variation: StringName, rect: Rect2) -> Label:
	var label := Ui.centered_label("", variation)
	label.autowrap_mode = TextServer.AUTOWRAP_OFF
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	label.clip_text = true
	# Le texte suit la taille de la carte : il tient dans son ruban ou son losange, que la
	# carte soit petite (draft) ou grande (cache révélée).
	label.add_theme_font_size_override(&"font_size", maxi(1, roundi(rect.size.y * Look.CARD_TEXT_SHARE)))
	label.position = rect.position
	label.size = rect.size
	# Hors de l'arbre, le libellé ne connaît pas encore le thème : il se croit plus haut que
	# sa zone. Il la reprend en y entrant, pour rester centré sur son ruban.
	label.ready.connect(func() -> void: label.size = rect.size)
	_face.add_child(label)
	return label


## La place que prend une carte de cette largeur : sa face, plus, au-dessus, la place où
## elle se lève quand elle est choisie.
static func footprint(width: float = Look.CARD_WIDTH) -> Vector2:
	return Look.CARD_CANVAS * (width / Look.CARD_CANVAS.x) + Vector2(0.0, Look.CARD_LIFT)


## Une place vide de la taille d'une carte de cette largeur (une carte à venir au draft, une
## place libre du trio) : des pointillés qui suivent le corps de la carte, là où elle serait
## posée, nommée au milieu par ce texte s'il y en a un.
static func placeholder(width: float, text := "") -> MarginContainer:
	var body := _scaled(Look.CARD_BODY, width / Look.CARD_CANVAS.x)
	var face := footprint(width) - Vector2(0.0, Look.CARD_LIFT)
	var margin := MarginContainer.new()
	margin.custom_minimum_size = footprint(width)
	margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	margin.add_theme_constant_override(&"margin_left", roundi(body.position.x))
	margin.add_theme_constant_override(&"margin_top", roundi(Look.CARD_LIFT + body.position.y))
	margin.add_theme_constant_override(&"margin_right", roundi(face.x - body.end.x))
	margin.add_theme_constant_override(&"margin_bottom", roundi(face.y - body.end.y))
	var place := PanelContainer.new()
	place.theme_type_variation = ThemeBuilder.WAITING_PANEL
	place.mouse_filter = Control.MOUSE_FILTER_IGNORE
	if text != "":
		place.add_child(Ui.centered_label(text, ThemeBuilder.SLOT_LABEL))
	margin.add_child(place)
	return margin


static func _scaled(rect: Rect2, factor: float) -> Rect2:
	return Rect2(rect.position * factor, rect.size * factor)
