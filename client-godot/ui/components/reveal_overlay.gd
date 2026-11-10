class_name RevealOverlay
extends Control
## Une carte mise en lumière (maquettes, la cache adverse révélée) : par-dessus tout, un
## voile, des rayons d'or qui tournent lentement derrière la carte en grand, une étiquette,
## un titre, son nom et une précision. Elle sert à la cache adverse révélée (`reveal` :
## l'écran de partie attend le toucher pour continuer) et à la carte gagnée (`reward`). Un
## toucher n'importe où la ferme.

## Le joueur a touché l'écran : la page est fermée.
signal dismissed

var _tag: Label
var _title: Label
var _card: CardView
var _name: Label
var _detail: Label
var _fade: Tween


func _init() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	# Le focus clavier, pour que Entrée la ferme aussi sur PC.
	focus_mode = Control.FOCUS_ALL
	visible = false
	var dim := ColorRect.new()
	dim.color = Look.REVEAL_DIM
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(dim)
	var rays := Rays.new()
	rays.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(rays)

	var column := Ui.vbox(Look.GAP_M)
	column.alignment = BoxContainer.ALIGNMENT_CENTER
	_tag = Ui.centered_label("", ThemeBuilder.GOLD_CAPS_LABEL)
	column.add_child(_tag)
	_title = Ui.centered_label("", ThemeBuilder.HEADING_LABEL)
	column.add_child(_title)
	_card = CardView.new(Look.REVEAL_CARD_WIDTH)
	_card.focus_mode = Control.FOCUS_NONE
	_card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_card.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	column.add_child(_card)
	_name = Ui.centered_label("", ThemeBuilder.NAME_LABEL)
	column.add_child(_name)
	_detail = Ui.centered_label("", ThemeBuilder.MUTED_LABEL)
	column.add_child(_detail)
	column.add_child(Ui.gap(Vector2(0, Look.GAP_XL)))
	column.add_child(Ui.centered_label(Labels.TAP_TO_CONTINUE, ThemeBuilder.CAPTION_LABEL))
	add_child(Ui.page(column))


## Montre la carte cachée (`{"kind", "rarity"}`) que ce joueur vient de lancer.
func reveal(attack: Dictionary, who: String, catalog: CardCatalog, cost: int) -> void:
	var rarity := _rarity_name(attack, catalog)
	_spotlight(Labels.REVEAL_TAG, Labels.REVEAL_TITLE % who, attack, catalog, cost, Labels.REVEAL_DETAIL % [rarity, cost])


## Montre la carte (`{"kind", "rarity"}`) que la victoire vient de faire gagner.
func reward(attack: Dictionary, catalog: CardCatalog) -> void:
	_spotlight(Labels.REWARD_TAG, Labels.REWARD_TITLE, attack, catalog, 0, Labels.REWARD_DETAIL % _rarity_name(attack, catalog))


func _spotlight(tag: String, title: String, attack: Dictionary, catalog: CardCatalog, cost: int, detail: String) -> void:
	var kind: String = attack.get("kind", "")
	_tag.text = tag
	_title.text = title
	_card.show_attack(attack, catalog, cost)
	_name.text = catalog.family_name(kind) if catalog != null else kind
	_detail.text = detail
	visible = true
	grab_focus()
	modulate.a = 0.0
	if _fade != null:
		_fade.kill()
	_fade = create_tween()
	_fade.tween_property(self, "modulate:a", 1.0, Look.REVEAL_IN)


static func _rarity_name(attack: Dictionary, catalog: CardCatalog) -> String:
	var rarity: String = attack.get("rarity", "")
	return catalog.rarity_name(attack.get("kind", ""), rarity) if catalog != null else rarity


func _gui_input(event: InputEvent) -> void:
	var tapped: bool = event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT
	if tapped or event.is_action_pressed(&"ui_accept"):
		accept_event()
		release_focus()
		visible = false
		dismissed.emit()


## Les rayons d'or derrière la carte : des pointes qui partent du centre, un sur deux
## dessiné, et qui tournent lentement.
class Rays:
	extends Control

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _process(delta: float) -> void:
		if is_visible_in_tree():
			rotation += TAU * Look.REVEAL_RAY_SPEED * delta
			pivot_offset = size / 2.0

	func _draw() -> void:
		var center := size / 2.0
		var length := size.length() * Look.REVEAL_RAY_LENGTH
		var step := TAU / (Look.REVEAL_RAYS * 2)
		for index in Look.REVEAL_RAYS:
			var start := step * index * 2
			draw_colored_polygon(PackedVector2Array([
				center,
				center + Vector2.from_angle(start) * length,
				center + Vector2.from_angle(start + step) * length,
			]), Look.REVEAL_RAY_COLOR)
