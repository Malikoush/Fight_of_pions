class_name Ui
extends RefCounted
## Les briques de l'interface, construites d'une seule façon : un libellé, un bouton, une
## icône, des boîtes. Les styles viennent du thème, par leur nom de variante.

const ICON_DIR := "res://assets/elements/icones/"
const PION_DIR := "res://assets/elements/pions/"


static func label(text: String, variation: StringName = &"") -> Label:
	var item := Label.new()
	item.text = text
	item.theme_type_variation = variation
	item.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return item


## Un libellé centré, qui passe à la ligne quand la place manque (titres, accroches).
static func centered_label(text: String, variation: StringName) -> Label:
	var item := label(text, variation)
	item.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	item.autowrap_mode = TextServer.AUTOWRAP_WORD
	return item


static func button(text: String, variation: StringName) -> Button:
	var item := Button.new()
	item.text = text
	item.theme_type_variation = variation
	item.custom_minimum_size.y = Look.BUTTON_HEIGHT
	return item


## Une icône blanche des éléments fournis, teintée de cette couleur.
static func icon(name: String, color: Color, size: float = Look.ICON_SIZE) -> TextureRect:
	return picture(ICON_DIR + name + ".svg", Vector2.ONE * size, color)


## Le palet d'un joueur, en image (panneaux de joueur), recadré sur le palet : le SVG a
## la toile commune des éléments du plateau, bien plus grande que lui.
static func chip(player: String) -> TextureRect:
	var file := "pion-j1.svg" if player == ViewData.PLAYER_ONE else "pion-j2.svg"
	var item := picture(PION_DIR + file, Look.CHIP_SIZE, Color.WHITE)
	var full: Texture2D = item.texture
	var cropped := AtlasTexture.new()
	cropped.atlas = full
	var scale := full.get_width() / Look.ELEMENT_CANVAS.x
	cropped.region = Rect2(Look.CHIP_REGION.position * scale, Look.CHIP_REGION.size * scale)
	item.texture = cropped
	return item


static func picture(path: String, size: Vector2, color: Color) -> TextureRect:
	var item := TextureRect.new()
	item.texture = load(path)
	item.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	item.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	item.custom_minimum_size = size
	item.modulate = color
	item.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return item


static func hbox(separation: int = Look.GAP_M) -> HBoxContainer:
	var box := HBoxContainer.new()
	box.add_theme_constant_override(&"separation", separation)
	box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return box


static func vbox(separation: int = Look.GAP_M) -> VBoxContainer:
	var box := VBoxContainer.new()
	box.add_theme_constant_override(&"separation", separation)
	box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return box


## Centre ce contrôle dans la largeur, sans dépasser `max_width` (un bouton d'accueil ne
## doit pas s'étirer sur tout un écran de PC).
static func limit_width(child: Control, max_width: float) -> CenterContainer:
	var center := CenterContainer.new()
	center.mouse_filter = Control.MOUSE_FILTER_IGNORE
	center.add_child(child)
	var fit := func() -> void:
		var available := center.size.x
		child.custom_minimum_size.x = minf(max_width, available) if available > 0.0 else max_width
	center.resized.connect(fit)
	return center


## Un vide qui prend toute la place libre de sa boîte.
static func spacer() -> Control:
	var item := Control.new()
	item.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	item.size_flags_vertical = Control.SIZE_EXPAND_FILL
	item.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return item


## Un vide de taille fixe, pour espacer deux éléments plus que ne le fait leur boîte.
static func gap(minimum: Vector2) -> Control:
	var item := Control.new()
	item.custom_minimum_size = minimum
	item.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return item


## Une marge autour de la page, élargie par les bords inaccessibles d'un téléphone
## (encoche, barre de geste).
static func page(child: Control) -> MarginContainer:
	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var insets := safe_insets()
	margin.add_theme_constant_override(&"margin_left", Look.PAGE_MARGIN + int(insets.position.x))
	margin.add_theme_constant_override(&"margin_top", Look.PAGE_MARGIN + int(insets.position.y))
	margin.add_theme_constant_override(&"margin_right", Look.PAGE_MARGIN + int(insets.size.x))
	margin.add_theme_constant_override(&"margin_bottom", Look.PAGE_MARGIN + int(insets.size.y))
	margin.add_child(child)
	return margin


## Les bords inaccessibles de l'écran, en points : à gauche et en haut dans `position`, à
## droite et en bas dans `size`. Rien sur PC : la zone sûre y décrit l'écran, pas la fenêtre.
static func safe_insets() -> Rect2:
	if not OS.has_feature("mobile"):
		return Rect2()
	var screen := Vector2(DisplayServer.window_get_size())
	var safe := Rect2(DisplayServer.get_display_safe_area())
	if screen.x <= 0.0 or safe.size.x <= 0.0:
		return Rect2()
	var root := (Engine.get_main_loop() as SceneTree).root
	var scale := root.get_visible_rect().size / screen
	return Rect2(safe.position * scale, (screen - safe.end) * scale)
