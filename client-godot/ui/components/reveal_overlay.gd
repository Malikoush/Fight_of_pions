class_name RevealOverlay
extends Control
## La cache adverse révélée (maquettes) : par-dessus tout, un voile, des rayons d'or qui
## tournent lentement derrière la carte en grand, son nom, sa rareté et son coût. Un toucher
## n'importe où la ferme ; l'écran de partie attend ce toucher pour continuer.

## Le joueur a touché l'écran : la page est fermée.
signal dismissed

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
	column.add_child(Ui.centered_label(Labels.REVEAL_TAG, ThemeBuilder.GOLD_CAPS_LABEL))
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
	var kind: String = attack.get("kind", "")
	var rarity: String = attack.get("rarity", "")
	_title.text = Labels.REVEAL_TITLE % who
	_card.show_attack(attack, catalog, cost)
	_name.text = catalog.family_name(kind) if catalog != null else kind
	_detail.text = Labels.REVEAL_DETAIL % [catalog.rarity_name(kind, rarity) if catalog != null else rarity, cost]
	visible = true
	grab_focus()
	modulate.a = 0.0
	if _fade != null:
		_fade.kill()
	_fade = create_tween()
	_fade.tween_property(self, "modulate:a", 1.0, Look.REVEAL_IN)


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
