class_name Ui
extends RefCounted
## Les briques de l'interface, construites d'une seule façon : un libellé, un bouton, une
## icône, des boîtes. Les styles viennent du thème, par leur nom de variante.

const ICON_DIR := "res://assets/elements/icones/"
const PION_DIR := "res://assets/elements/pions/"
const BOARD_DIR := "res://assets/elements/plateau/"


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


## Un bouton carré qui ne montre qu'une icône des éléments fournis (`ThemeBuilder.ICON_BUTTON`).
static func icon_button(name: String, tooltip: String) -> Button:
	var item := Button.new()
	item.theme_type_variation = ThemeBuilder.ICON_BUTTON
	item.icon = load(ICON_DIR + name + ".svg")
	item.expand_icon = true
	item.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	item.tooltip_text = tooltip
	item.custom_minimum_size = Vector2.ONE * Look.ICON_BUTTON_SIZE
	return item


## Une icône blanche des éléments fournis, teintée de cette couleur.
static func icon(name: String, color: Color, size: float = Look.ICON_SIZE) -> TextureRect:
	return picture(ICON_DIR + name + ".svg", Vector2.ONE * size, color)


## Le palet d'un joueur, en image (panneaux de joueur), recadré sur le palet : le SVG a
## la toile commune des éléments du plateau, bien plus grande que lui.
static func chip(player: String, size: Vector2 = Look.CHIP_SIZE) -> TextureRect:
	var item := picture(pion_path(player), size, Color.WHITE)
	item.texture = chip_texture(player)
	return item


## L'image du palet d'un joueur, recadrée (l'icône d'un bouton, comme la pastille du joueur).
static func chip_texture(player: String) -> Texture2D:
	var full: Texture2D = load(pion_path(player))
	var cropped := AtlasTexture.new()
	cropped.atlas = full
	var scale := full.get_width() / Look.ELEMENT_CANVAS.x
	cropped.region = Rect2(Look.CHIP_REGION.position * scale, Look.CHIP_REGION.size * scale)
	return cropped


## Le fichier du pion d'un joueur, sur la toile commune des éléments du plateau.
static func pion_path(player: String) -> String:
	return PION_DIR + ("pion-j1.svg" if player == ViewData.PLAYER_ONE else "pion-j2.svg")


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
	set_page_margins(margin)
	margin.add_child(child)
	return margin


## Donne à ce conteneur les marges d'une page (`side_margin` de chaque côté), élargies par les bords
## inaccessibles du téléphone ; en haut et en bas, seulement si rien d'autre ne les couvre
## (une barre de navigation y prend elle-même ces bords).
static func set_page_margins(margin: MarginContainer, top_edge := true, bottom_edge := true, side_margin := Look.PAGE_MARGIN) -> void:
	var insets := safe_insets()
	margin.add_theme_constant_override(&"margin_left", side_margin + int(insets.position.x))
	margin.add_theme_constant_override(&"margin_top", side_margin + (int(insets.position.y) if top_edge else 0))
	margin.add_theme_constant_override(&"margin_right", side_margin + int(insets.size.x))
	margin.add_theme_constant_override(&"margin_bottom", side_margin + (int(insets.size.y) if bottom_edge else 0))


## Les marges d'une page dont le contenu ne dépasse pas `max_width` : sur un grand écran, il
## reste centré, les marges des côtés s'élargissent. Elles suivent la taille du conteneur.
static func fit_page(margin: MarginContainer, max_width: float) -> void:
	var fit := func() -> void:
		set_page_margins(margin)
		var insets := safe_insets()
		var inner := margin.size.x - 2 * Look.PAGE_MARGIN - insets.position.x - insets.size.x
		var extra := int(maxf(0.0, (inner - max_width) / 2.0))
		margin.add_theme_constant_override(&"margin_left", Look.PAGE_MARGIN + int(insets.position.x) + extra)
		margin.add_theme_constant_override(&"margin_right", Look.PAGE_MARGIN + int(insets.size.x) + extra)
	margin.resized.connect(fit)
	fit.call()


## Vrai si la fenêtre est plus large que haute : les écrans passent alors à leur mise en
## page en paysage.
static func is_landscape(item: CanvasItem) -> bool:
	var area := item.get_viewport_rect().size
	return area.x > area.y


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
