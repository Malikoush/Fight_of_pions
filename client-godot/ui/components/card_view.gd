class_name CardView
extends Button
## Une carte : le cadre de sa rareté (ou celui des réactions), son nom sur le ruban, son coût
## dans le losange, un sablier et le nombre de tours quand elle se recharge, et l'étiquette
## « CACHE » pour sa carte cachée. Une carte qu'on ne peut pas voir montre son dos.
##
## Tout ce qu'elle montre lui est donné : le coût vient de la configuration de la partie, le
## nom du catalogue, et « jouable » de la présence de son coup dans les coups permis.

const FRAME_DIR := "res://assets/elements/cartes/"
const FRAMES := {
	"Common": "cadre-commune",
	"Rare": "cadre-rare",
	"Epic": "cadre-epique",
	"Legendary": "cadre-legendaire",
}
const REACTION_FRAME := "cadre-reaction"
const HIDDEN_FRAME := "cadre-dos"
const HOURGLASS := "sablier"

## L'emplacement de la carte dans l'arsenal.
var slot := -1

var _face: Control
var _frame: TextureRect
var _name: Label
var _cost: Label
var _cooldown: Control
var _cooldown_turns: Label
var _tag: PanelContainer
var _lift: Tween
var _selected := false


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
	_tag.theme_type_variation = ThemeBuilder.TAG
	_tag.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_tag.add_child(Ui.label(Labels.SECRET_TAG, ThemeBuilder.TAG_LABEL))
	_tag.visible = false
	_face.add_child(_tag)


## Montre cette carte. `attack` vaut `{"kind", "rarity"}`, ou `null` pour une carte qu'on ne
## peut pas voir (le dos).
func show_card(attack: Variant, display_name: String, cost: int, reaction: bool, cooldown_left: int, secret: bool, playable: bool) -> void:
	if not (attack is Dictionary):
		_frame.texture = load(FRAME_DIR + HIDDEN_FRAME + ".svg")
		_name.text = ""
		_cost.text = ""
		_cooldown.visible = false
		_tag.visible = false
		disabled = true
		_face.modulate = Color.WHITE
		return
	var frame: String = REACTION_FRAME if reaction else FRAMES.get(attack.get("rarity"), FRAMES["Common"])
	_frame.texture = load(FRAME_DIR + frame + ".svg")
	_name.text = display_name
	_cost.text = str(cost) if cost > 0 else ""
	_cooldown.visible = cooldown_left > 0
	_cooldown_turns.text = str(cooldown_left)
	_tag.visible = secret
	_tag.reset_size()
	_tag.position = Vector2(_face.size.x - _tag.size.x, -_tag.size.y / 2.0)
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
## permis.
func show_slot(view: ViewData, player: String, index: int, catalog: CardCatalog) -> void:
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
		view.use_attack(index) != null,
	)


## Lève la carte choisie au-dessus des autres.
func set_selected(selected: bool) -> void:
	_selected = selected
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
	var column := Ui.vbox(Look.GAP_XS)
	column.alignment = BoxContainer.ALIGNMENT_CENTER
	var hourglass := Ui.icon(HOURGLASS, Look.CREAM)
	hourglass.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	column.add_child(hourglass)
	_cooldown_turns = Ui.centered_label("", ThemeBuilder.NUMBER_LABEL)
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
	label.position = rect.position
	label.size = rect.size
	_face.add_child(label)
	return label


## La place que prend une carte de cette largeur : sa face, plus, au-dessus, la place où
## elle se lève quand elle est choisie.
static func footprint(width: float = Look.CARD_WIDTH) -> Vector2:
	return Look.CARD_CANVAS * (width / Look.CARD_CANVAS.x) + Vector2(0.0, Look.CARD_LIFT)


static func _scaled(rect: Rect2, factor: float) -> Rect2:
	return Rect2(rect.position * factor, rect.size * factor)
