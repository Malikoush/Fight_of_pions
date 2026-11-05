class_name ThemeBuilder
extends RefCounted
## Construit le thème de toute l'interface à partir de l'apparence (`Look`) : c'est le seul
## endroit qui transforme les couleurs et les tailles en styles Godot. Les composants
## choisissent un style par son nom de variante (`theme_type_variation`), jamais par une
## couleur.

const DISPLAY_FONT := "res://assets/fonts/GrenzeGotisch.ttf"
const BODY_FONT := "res://assets/fonts/BarlowSemiCondensed-Medium.ttf"
const STRONG_FONT := "res://assets/fonts/BarlowSemiCondensed-SemiBold.ttf"
const CAPS_FONT := "res://assets/fonts/BarlowCondensed-Bold.ttf"
const BOLD_FONT := "res://assets/fonts/BarlowSemiCondensed-Bold.ttf"

## Les variantes proposées aux composants.
const DISPLAY_LABEL := &"DisplayLabel"
const NAME_LABEL := &"NameLabel"
const STRONG_LABEL := &"StrongLabel"
const MUTED_LABEL := &"MutedLabel"
const NUMBER_LABEL := &"NumberLabel"
const PILL_LABEL := &"PillLabel"
const PRIMARY_BUTTON := &"PrimaryButton"
const WAITING_PANEL := &"WaitingPanel"
const WAITING_LABEL := &"WaitingLabel"
const BANNER := &"Banner"
const BANNER_LABEL := &"BannerLabel"
const CARD_BUTTON := &"CardButton"
const CARD_NAME_LABEL := &"CardNameLabel"
const CARD_COST_LABEL := &"CardCostLabel"
const TAG := &"Tag"
const TAG_LABEL := &"TagLabel"
const CAPS_LABEL := &"CapsLabel"
const SECONDARY_BUTTON := &"SecondaryButton"
const PANEL := &"GamePanel"
const PANEL_ACTIVE := &"GamePanelActive"
const PILL := &"Pill"
const BADGE_NUMBER_LABEL := &"BadgeNumberLabel"
const FACT_LABEL := &"FactLabel"
const SCORCHED_LABEL := &"ScorchedLabel"
const PANEL_SCORCHED := &"GamePanelScorched"
const JOURNAL_LINE := &"JournalLine"
const JOURNAL_TURN_LABEL := &"JournalTurnLabel"
const ICON_BUTTON := &"IconButton"
const PANEL_THREAT := &"GamePanelThreat"
const THREAT_LABEL := &"ThreatLabel"
const HEADING_LABEL := &"HeadingLabel"
const CAPTION_LABEL := &"CaptionLabel"
const GOLD_CAPS_LABEL := &"GoldCapsLabel"
const OVERLAY := &"Overlay"

static var _shared: Theme


## Le thème de toute l'interface, construit une fois et partagé (l'écran, les badges du
## plateau).
static func shared() -> Theme:
	if _shared == null:
		_shared = build()
	return _shared


static func build() -> Theme:
	var theme := Theme.new()
	var display := _display_font()
	var body: FontFile = load(BODY_FONT)
	var strong: FontFile = load(STRONG_FONT)
	var caps := _caps_font()
	theme.default_font = body
	theme.default_font_size = Look.FONT_BODY
	theme.set_color(&"font_color", &"Label", Look.CREAM)

	_label(theme, DISPLAY_LABEL, display, Look.FONT_DISPLAY, Look.GOLD, true)
	_label(theme, BANNER_LABEL, display, Look.FONT_DISPLAY, Look.INK, false)
	_label(theme, NAME_LABEL, display, Look.FONT_HEADING, Look.GOLD, false)
	_label(theme, STRONG_LABEL, strong, Look.FONT_BODY, Look.CREAM, false)
	_label(theme, MUTED_LABEL, body, Look.FONT_BODY, Look.MUTED, false)
	_label(theme, NUMBER_LABEL, caps, Look.FONT_NUMBER, Look.CREAM, false)
	_label(theme, PILL_LABEL, caps, Look.FONT_BODY, Look.CREAM, false)
	_label(theme, WAITING_LABEL, caps, Look.FONT_BUTTON, Look.MUTED, false)
	_label(theme, CARD_NAME_LABEL, caps, Look.FONT_CARD_NAME, Look.INK, false)
	_label(theme, CARD_COST_LABEL, caps, Look.FONT_CARD_COST, Look.INK, false)
	_label(theme, TAG_LABEL, caps, Look.FONT_CARD_TAG, Look.GOLD, false)
	_label(theme, CAPS_LABEL, caps, Look.FONT_BODY, Look.MUTED, false)
	_label(theme, BADGE_NUMBER_LABEL, caps, Look.FONT_BODY, Look.CREAM, false)
	_label(theme, FACT_LABEL, strong, Look.FONT_FACT, Look.CREAM, false)
	_label(theme, SCORCHED_LABEL, strong, Look.FONT_FACT, Look.SCORCHED, false)
	_label(theme, JOURNAL_TURN_LABEL, caps, Look.FONT_FACT, Look.MUTED, false)
	_label(theme, THREAT_LABEL, caps, Look.FONT_BODY, Look.THREAT, false)
	_label(theme, HEADING_LABEL, display, Look.FONT_HEADING, Look.CREAM, false)
	_label(theme, CAPTION_LABEL, body, Look.FONT_FACT, Look.MUTED, false)
	_label(theme, GOLD_CAPS_LABEL, caps, Look.FONT_BODY, Look.GOLD, false)
	_journal(theme, strong)
	_icon_button(theme)

	_primary_button(theme, caps)
	_secondary_button(theme, caps)
	_waiting_panel(theme)
	_panels(theme)
	_banner(theme)
	_cards(theme)
	return theme


static func _display_font() -> FontVariation:
	var font := FontVariation.new()
	font.base_font = load(DISPLAY_FONT)
	font.variation_opentype = {TextServerManager.get_primary_interface().name_to_tag("wght"): Look.DISPLAY_WEIGHT}
	return font


static func _caps_font() -> FontVariation:
	var font := FontVariation.new()
	font.base_font = load(CAPS_FONT)
	font.spacing_glyph = Look.CAPS_SPACING
	return font


static func _label(theme: Theme, variation: StringName, font: Font, size: int, color: Color, shadow: bool) -> void:
	theme.set_type_variation(variation, &"Label")
	theme.set_font(&"font", variation, font)
	theme.set_font_size(&"font_size", variation, size)
	theme.set_color(&"font_color", variation, color)
	if shadow:
		theme.set_color(&"font_shadow_color", variation, Look.INK)
		theme.set_constant(&"shadow_offset_x", variation, int(Look.TITLE_SHADOW.x))
		theme.set_constant(&"shadow_offset_y", variation, int(Look.TITLE_SHADOW.y))


static func _primary_button(theme: Theme, font: Font) -> void:
	var variation := PRIMARY_BUTTON
	theme.set_type_variation(variation, &"Button")
	theme.set_stylebox(&"normal", variation, _hex(Look.GOLD, Look.GOLD_SHADE, false))
	theme.set_stylebox(&"hover", variation, _hex(Look.GOLD.lightened(Look.HOVER_LIGHTEN), Look.GOLD_SHADE, false))
	theme.set_stylebox(&"pressed", variation, _hex(Look.GOLD_SHADE, Look.GOLD_DEEP, true))
	theme.set_stylebox(&"hover_pressed", variation, _hex(Look.GOLD_SHADE, Look.GOLD_DEEP, true))
	theme.set_stylebox(&"disabled", variation, _hex(Look.DISABLED_FILL, Look.DISABLED_FILL, false))
	theme.set_stylebox(&"focus", variation, _focus())
	_button_text(theme, variation, font, Look.INK)
	theme.set_color(&"font_disabled_color", variation, Look.DISABLED_TEXT)


static func _secondary_button(theme: Theme, font: Font) -> void:
	var variation := SECONDARY_BUTTON
	theme.set_type_variation(variation, &"Button")
	theme.set_stylebox(&"normal", variation, _outlined(Look.FOREST_PANEL, Look.GOLD))
	theme.set_stylebox(&"hover", variation, _outlined(Look.FOREST_RAISED, Look.GOLD))
	theme.set_stylebox(&"pressed", variation, _outlined(Look.FOREST_DEEP, Look.GOLD_SHADE))
	theme.set_stylebox(&"hover_pressed", variation, _outlined(Look.FOREST_DEEP, Look.GOLD_SHADE))
	theme.set_stylebox(&"disabled", variation, _outlined(Look.FOREST_PANEL, Look.DISABLED_FILL))
	theme.set_stylebox(&"focus", variation, _focus())
	_button_text(theme, variation, font, Look.CREAM)
	theme.set_color(&"font_disabled_color", variation, Look.DISABLED_TEXT)


## La place du bouton d'action quand on attend l'adversaire : un cadre en pointillés, sans
## fond (maquettes). Son sablier et son texte ternes sont centrés ensemble.
static func _waiting_panel(theme: Theme) -> void:
	theme.set_type_variation(WAITING_PANEL, &"PanelContainer")
	theme.set_stylebox(&"panel", WAITING_PANEL, _dashed(Color.TRANSPARENT, Look.WAITING_BORDER, Look.BORDER, Look.RADIUS_CONTROL))


static func _dashed(fill: Color, color: Color, width: float, radius: float) -> StyleBoxDashed:
	var dashed := StyleBoxDashed.new()
	dashed.fill = fill
	dashed.color = color
	dashed.width = width
	dashed.radius = radius
	dashed.dash = Look.DASH_LENGTH
	dashed.gap = Look.DASH_GAP
	dashed.corner_steps = Look.CORNER_STEPS
	return dashed


static func _button_text(theme: Theme, variation: StringName, font: Font, color: Color) -> void:
	theme.set_font(&"font", variation, font)
	theme.set_font_size(&"font_size", variation, Look.FONT_BUTTON)
	for state in [&"font_color", &"font_hover_color", &"font_pressed_color", &"font_hover_pressed_color", &"font_focus_color"]:
		theme.set_color(state, variation, color)


static func _hex(fill: Color, shade: Color, sunk: bool) -> StyleBoxHex:
	var style := StyleBoxHex.new()
	style.fill = fill
	style.shade = shade
	style.point_width = Look.BUTTON_POINT
	style.shade_height = Look.BUTTON_SHADE
	style.sunk = sunk
	style.content_margin_left = Look.BUTTON_PADDING
	style.content_margin_right = Look.BUTTON_PADDING
	# Le texte descend avec le dessus quand le bouton s'enfonce.
	style.content_margin_top = Look.BUTTON_SHADE if sunk else 0.0
	style.content_margin_bottom = 0.0 if sunk else Look.BUTTON_SHADE
	return style


static func _outlined(fill: Color, border: Color) -> StyleBoxFlat:
	var style := _rounded(fill, Look.RADIUS_CONTROL)
	style.set_border_width_all(Look.BORDER_STRONG)
	style.border_color = border
	style.content_margin_left = Look.BUTTON_PADDING
	style.content_margin_right = Look.BUTTON_PADDING
	return style


## Le repère du clavier (PC) : un trait crème autour du bouton qui a le focus.
static func _focus() -> StyleBoxFlat:
	var style := _rounded(Color.TRANSPARENT, Look.RADIUS_CONTROL)
	style.draw_center = false
	style.set_border_width_all(Look.BORDER)
	style.border_color = Look.CREAM
	style.set_expand_margin_all(Look.GAP_XS)
	return style


static func _panels(theme: Theme) -> void:
	for variation in [PANEL, PANEL_ACTIVE]:
		theme.set_type_variation(variation, &"PanelContainer")
		var style := _rounded(Look.FOREST_PANEL, Look.RADIUS_PANEL)
		style.set_border_width_all(Look.BORDER_STRONG if variation == PANEL_ACTIVE else Look.BORDER)
		style.border_color = Look.PANEL_BORDER_ACTIVE if variation == PANEL_ACTIVE else Look.PANEL_BORDER
		style.set_content_margin_all(Look.GAP_L)
		theme.set_stylebox(&"panel", variation, style)
	# La menace adverse (l'annonce d'une carte, la fenêtre de réaction) : un bord de braise.
	theme.set_type_variation(PANEL_THREAT, &"PanelContainer")
	var threat := _rounded(Look.FOREST_DEEP, Look.RADIUS_PANEL)
	threat.set_border_width_all(Look.BORDER_STRONG)
	threat.border_color = Look.THREAT
	threat.set_content_margin_all(Look.GAP_L)
	theme.set_stylebox(&"panel", PANEL_THREAT, threat)
	# Sous Terre brûlée, le panneau du joueur qui attend a un bord de braise en pointillés
	# (maquettes) ; celui qui joue garde son bord d'or.
	theme.set_type_variation(PANEL_SCORCHED, &"PanelContainer")
	var scorched := _dashed(Look.FOREST_PANEL, Look.SCORCHED, Look.BORDER_STRONG, Look.RADIUS_PANEL)
	scorched.set_content_margin_all(Look.GAP_L)
	theme.set_stylebox(&"panel", PANEL_SCORCHED, scorched)

	theme.set_type_variation(PILL, &"PanelContainer")
	var pill := _rounded(Look.FOREST_DEEP, Look.RADIUS_PILL)
	pill.set_border_width_all(Look.BORDER)
	pill.border_color = Look.PANEL_BORDER
	pill.content_margin_left = Look.GAP_XL
	pill.content_margin_right = Look.GAP_XL
	pill.content_margin_top = Look.GAP_S
	pill.content_margin_bottom = Look.GAP_S
	theme.set_stylebox(&"panel", PILL, pill)


## Une carte est un bouton sans fond : son cadre est son dessin. L'étiquette « CACHE » est
## une petite plaque sombre bordée d'or (maquettes).
static func _cards(theme: Theme) -> void:
	theme.set_type_variation(CARD_BUTTON, &"Button")
	for state in [&"normal", &"hover", &"pressed", &"hover_pressed", &"disabled"]:
		theme.set_stylebox(state, CARD_BUTTON, StyleBoxEmpty.new())
	theme.set_stylebox(&"focus", CARD_BUTTON, _focus())
	theme.set_type_variation(TAG, &"PanelContainer")
	var tag := StyleBoxFlat.new()
	tag.bg_color = Look.FOREST_DEEP
	tag.set_border_width_all(Look.BORDER)
	tag.border_color = Look.GOLD
	tag.content_margin_left = Look.GAP_XS
	tag.content_margin_right = Look.GAP_XS
	theme.set_stylebox(&"panel", TAG, tag)


## Une ligne du journal : un texte riche (le nom du joueur en couleur), sur plusieurs lignes
## si besoin ; et la page qui couvre l'écran (le journal en portrait).
static func _journal(theme: Theme, font: Font) -> void:
	theme.set_type_variation(JOURNAL_LINE, &"RichTextLabel")
	theme.set_font(&"normal_font", JOURNAL_LINE, font)
	theme.set_font_size(&"normal_font_size", JOURNAL_LINE, Look.FONT_FACT)
	theme.set_font(&"bold_font", JOURNAL_LINE, load(BOLD_FONT))
	theme.set_font_size(&"bold_font_size", JOURNAL_LINE, Look.FONT_FACT)
	theme.set_color(&"default_color", JOURNAL_LINE, Look.CREAM)
	theme.set_type_variation(OVERLAY, &"PanelContainer")
	var overlay := StyleBoxFlat.new()
	overlay.bg_color = Look.OVERLAY_FILL
	theme.set_stylebox(&"panel", OVERLAY, overlay)


## Un bouton carré à icône (le journal, en portrait) : fond sombre bordé d'or, icône d'or.
static func _icon_button(theme: Theme) -> void:
	theme.set_type_variation(ICON_BUTTON, &"Button")
	for state in [&"normal", &"hover", &"pressed", &"hover_pressed", &"disabled"]:
		var style := _rounded(Look.FOREST_RAISED if state == &"hover" else Look.FOREST_DEEP, Look.RADIUS_CONTROL)
		style.set_border_width_all(Look.BORDER)
		style.border_color = Look.GOLD
		style.set_content_margin_all(Look.GAP_S)
		theme.set_stylebox(state, ICON_BUTTON, style)
	theme.set_stylebox(&"focus", ICON_BUTTON, _focus())
	for state in [&"icon_normal_color", &"icon_hover_color", &"icon_pressed_color", &"icon_hover_pressed_color", &"icon_focus_color"]:
		theme.set_color(state, ICON_BUTTON, Look.GOLD)


## Le bandeau qui annonce ton tour : un hexagone d'or, comme les boutons d'action.
static func _banner(theme: Theme) -> void:
	theme.set_type_variation(BANNER, &"PanelContainer")
	var style := _hex(Look.GOLD, Look.GOLD_SHADE, false)
	style.content_margin_left = Look.GAP_XXL
	style.content_margin_right = Look.GAP_XXL
	theme.set_stylebox(&"panel", BANNER, style)


static func _rounded(fill: Color, radius: int) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = fill
	style.set_corner_radius_all(radius)
	style.anti_aliasing = true
	return style
