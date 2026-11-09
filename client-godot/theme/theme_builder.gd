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
const CODE_INPUT := &"CodeInput"
const CODE_LABEL := &"CodeLabel"
const OVERLAY := &"Overlay"
## Le menu (maquettes 1-5) : le nom du jeu, la barre de navigation, la pastille du joueur,
## les panneaux de l'accueil, les cases du code d'une salle, la barre de chargement.
const HERO_LABEL := &"HeroLabel"
const LOGO_LABEL := &"LogoLabel"
const SLOGAN_LABEL := &"SloganLabel"
const SECTION_LABEL := &"SectionLabel"
const CHOICE_TITLE_LABEL := &"ChoiceTitleLabel"
const NAV_BAR := &"NavBar"
const NAV_BAR_TOP := &"NavBarTop"
const NAV_TAB := &"NavTab"
const NAV_TAB_WIDE := &"NavTabWide"
const PILL_BUTTON := &"PillButton"
const BOARD_PANEL := &"BoardPanel"
const ROOM_CARD := &"RoomCard"
const TIP_PANEL := &"TipPanel"
const DIVIDER := &"Divider"
const CODE_BOX := &"CodeBox"
const CODE_BOX_ACTIVE := &"CodeBoxActive"
const CODE_LETTER_LABEL := &"CodeLetterLabel"
const HIDDEN_INPUT := &"HiddenInput"
const PROGRESS_TRACK := &"ProgressTrack"
const PROGRESS_FILL := &"ProgressFill"
## Les salles : le titre d'une page, « contre », un refus, la case refusée, les grandes
## lettres du code, et un bouton qui n'est qu'un mot (« FERMER LA SALLE »).
const TITLE_LABEL := &"TitleLabel"
const VERSUS_LABEL := &"VersusLabel"
const ERROR_TEXT := &"ErrorText"
const CODE_BOX_REFUSED := &"CodeBoxRefused"
const CODE_LETTER_LARGE_LABEL := &"CodeLetterLargeLabel"
const TEXT_BUTTON := &"TextButton"
## Le draft : les étiquettes d'une carte (crème : l'adversaire l'a prise ; braise : bannie),
## le contour de la carte choisie, les filtres en pastilles, les places de l'ordre des picks.
const TAG_PLAIN := &"TagPlain"
const TAG_PLAIN_LABEL := &"TagPlainLabel"
const TAG_THREAT := &"TagThreat"
const TAG_THREAT_LABEL := &"TagThreatLabel"
const CARD_OUTLINE := &"CardOutline"
const FILTER_PILL := &"FilterPill"
const SNAKE_SLOT := &"SnakeSlot"
const SNAKE_SLOT_CURRENT := &"SnakeSlotCurrent"
const SLOT_LABEL := &"SlotLabel"
const CARD_GRID := &"CardGrid"
const CARD_GRID_INSET := &"CardGridInset"
## Les boîtes des scènes : l'écart entre leurs enfants (colonne ou rangée), et une marge
## intérieure. Une scène n'écrit ainsi aucun nombre : elle choisit une variante.
const STACK := &"Stack"
const COLUMN_XS := &"ColumnXS"
const COLUMN_S := &"ColumnS"
const COLUMN_M := &"ColumnM"
const COLUMN_L := &"ColumnL"
const COLUMN_XL := &"ColumnXL"
const ROW_XS := &"RowXS"
const ROW_S := &"RowS"
const ROW_M := &"RowM"
const ROW_L := &"RowL"
const INSET := &"Inset"
const GRID_XL := &"GridXL"

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
	# Leur taille suit celle de la carte (`CardView`, `Look.CARD_TEXT_SHARE`).
	_label(theme, CARD_NAME_LABEL, caps, Look.FONT_BODY, Look.INK, false)
	_label(theme, CARD_COST_LABEL, caps, Look.FONT_BODY, Look.INK, false)
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
	_label(theme, CODE_LABEL, caps, Look.FONT_CODE, Look.GOLD, false)
	_code_input(theme, caps)
	_menu(theme, display, caps)
	_rooms(theme, display, caps)
	_draft(theme, caps)
	_boxes(theme)
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


static func _caps_font(spacing: int = Look.CAPS_SPACING) -> FontVariation:
	var font := FontVariation.new()
	font.base_font = load(CAPS_FONT)
	font.spacing_glyph = spacing
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
	# Son icône éventuelle (« CRÉER UNE SALLE ») est d'or.
	_icon_colors(theme, variation, Look.GOLD, Look.GOLD)
	theme.set_constant(&"icon_max_width", variation, Look.ICON_SIZE)
	theme.set_constant(&"h_separation", variation, Look.GAP_S)


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
	theme.set_stylebox(&"panel", TAG, _tag(Look.GOLD))


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


## Le champ du code d'une salle : une plaque crème, lettres d'encre en capitales, bord d'or
## quand on tape (maquettes).
static func _code_input(theme: Theme, font: Font) -> void:
	theme.set_type_variation(CODE_INPUT, &"LineEdit")
	for state in [&"normal", &"focus", &"read_only"]:
		var style := _rounded(Look.CREAM, Look.RADIUS_CONTROL)
		style.set_border_width_all(Look.BORDER_STRONG)
		style.border_color = Look.GOLD if state == &"focus" else Look.CASE_SIDE_A
		style.set_content_margin_all(Look.GAP_M)
		theme.set_stylebox(state, CODE_INPUT, style)
	theme.set_font(&"font", CODE_INPUT, font)
	theme.set_font_size(&"font_size", CODE_INPUT, Look.FONT_CODE)
	theme.set_color(&"font_color", CODE_INPUT, Look.INK)
	theme.set_color(&"caret_color", CODE_INPUT, Look.INK)
	theme.set_color(&"font_placeholder_color", CODE_INPUT, Look.CASE_SIDE_A)


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


## Le menu (maquettes 1-5) : tout ce que l'accueil, la barre de navigation et le démarrage
## ajoutent au thème.
static func _menu(theme: Theme, display: Font, caps: Font) -> void:
	_label(theme, HERO_LABEL, display, Look.FONT_HERO, Look.GOLD, true)
	theme.set_constant(&"line_spacing", HERO_LABEL, -roundi(Look.FONT_HERO * Look.DISPLAY_LINE_TIGHTEN))
	_label(theme, LOGO_LABEL, display, Look.FONT_LOGO, Look.GOLD, true)
	_label(theme, SLOGAN_LABEL, display, Look.FONT_SLOGAN, Look.CREAM, true)
	theme.set_constant(&"line_spacing", SLOGAN_LABEL, -roundi(Look.FONT_SLOGAN * Look.DISPLAY_LINE_TIGHTEN))
	_label(theme, SECTION_LABEL, _caps_font(Look.SECTION_SPACING), Look.FONT_NAV, Look.MUTED, false)
	_label(theme, CHOICE_TITLE_LABEL, caps, Look.FONT_BUTTON, Look.CREAM, false)
	_label(theme, CODE_LETTER_LABEL, caps, Look.FONT_CODE, Look.INK, false)
	_nav(theme, caps)
	_menu_panels(theme)
	_code_boxes(theme)
	# La barre de chargement : une piste arrondie, et l'or qui y court.
	for variation: StringName in [PROGRESS_TRACK, PROGRESS_FILL]:
		theme.set_type_variation(variation, &"Panel")
		var fill := Look.GOLD if variation == PROGRESS_FILL else Look.FOREST_RAISED
		theme.set_stylebox(&"panel", variation, _rounded(fill, Look.RADIUS_PILL))


## La barre de navigation, ses onglets, et la pastille du joueur.
static func _nav(theme: Theme, caps: Font) -> void:
	# La barre : sombre, bordée d'un trait d'or du côté de la page (dessus en portrait,
	# dessous en paysage).
	for variation: StringName in [NAV_BAR, NAV_BAR_TOP]:
		theme.set_type_variation(variation, &"PanelContainer")
		var bar := StyleBoxFlat.new()
		bar.bg_color = Look.FOREST_DEEP
		bar.border_color = Look.PANEL_BORDER
		if variation == NAV_BAR:
			bar.border_width_top = Look.BORDER
			bar.content_margin_top = Look.GAP_M
			bar.content_margin_bottom = Look.GAP_L
		else:
			bar.border_width_bottom = Look.BORDER
			bar.content_margin_left = Look.GAP_XL
			bar.content_margin_right = Look.GAP_XL
		theme.set_stylebox(&"panel", variation, bar)
	# Un onglet : son icône et son nom, ternes ; l'onglet actif a l'icône d'or et le nom
	# crème. En bas, l'icône est au-dessus du nom ; en haut, à sa gauche.
	for variation: StringName in [NAV_TAB, NAV_TAB_WIDE]:
		var wide := variation == NAV_TAB_WIDE
		theme.set_type_variation(variation, &"Button")
		for state in [&"normal", &"hover", &"pressed", &"hover_pressed", &"disabled"]:
			var style := StyleBoxEmpty.new()
			style.set_content_margin_all(Look.GAP_XS)
			if wide:
				style.content_margin_left = Look.GAP_M
				style.content_margin_right = Look.GAP_M
				style.content_margin_top = Look.GAP_L
				style.content_margin_bottom = Look.GAP_L
			theme.set_stylebox(state, variation, style)
		theme.set_stylebox(&"focus", variation, _focus())
		theme.set_font(&"font", variation, caps)
		theme.set_font_size(&"font_size", variation, Look.FONT_NAV)
		theme.set_color(&"font_color", variation, Look.MUTED)
		theme.set_color(&"font_focus_color", variation, Look.MUTED)
		for state in [&"font_hover_color", &"font_pressed_color", &"font_hover_pressed_color"]:
			theme.set_color(state, variation, Look.CREAM)
		_icon_colors(theme, variation, Look.MUTED, Look.GOLD)
		theme.set_color(&"icon_hover_color", variation, Look.CREAM)
		theme.set_constant(&"icon_max_width", variation, Look.NAV_ICON_SIZE_WIDE if wide else Look.NAV_ICON_SIZE)
		theme.set_constant(&"h_separation", variation, Look.GAP_S if wide else Look.GAP_XS)

	# La pastille du joueur : son palet et son pseudo, sur un fond de panneau arrondi.
	theme.set_type_variation(PILL_BUTTON, &"Button")
	for state in [&"normal", &"hover", &"pressed", &"hover_pressed", &"disabled"]:
		var pill := _rounded(Look.FOREST_RAISED if state == &"hover" else Look.FOREST_PANEL, Look.RADIUS_PILL)
		pill.content_margin_left = Look.GAP_M
		pill.content_margin_right = Look.GAP_L
		pill.content_margin_top = Look.GAP_XS
		pill.content_margin_bottom = Look.GAP_XS
		theme.set_stylebox(state, PILL_BUTTON, pill)
	theme.set_stylebox(&"focus", PILL_BUTTON, _focus())
	theme.set_font(&"font", PILL_BUTTON, load(BOLD_FONT))
	theme.set_font_size(&"font_size", PILL_BUTTON, Look.FONT_BODY)
	for state in [&"font_color", &"font_hover_color", &"font_pressed_color", &"font_hover_pressed_color", &"font_focus_color"]:
		theme.set_color(state, PILL_BUTTON, Look.CREAM)
	theme.set_constant(&"icon_max_width", PILL_BUTTON, int(Look.PILL_CHIP.x))
	theme.set_constant(&"h_separation", PILL_BUTTON, Look.GAP_S)


## Les panneaux de l'accueil : celui du plateau, les cartes de salle, le conseil, le trait.
static func _menu_panels(theme: Theme) -> void:
	# Le panneau du plateau : le plateau y va jusqu'au bord, rogné par ses coins
	# (`clip_children`).
	theme.set_type_variation(BOARD_PANEL, &"PanelContainer")
	var board := _rounded(Look.FOREST_PANEL, Look.RADIUS_PANEL)
	board.set_border_width_all(Look.BORDER)
	board.border_color = Look.PANEL_BORDER
	theme.set_stylebox(&"panel", BOARD_PANEL, board)
	# Une carte de salle (créer une salle, rejoindre) : un panneau qu'on touche.
	theme.set_type_variation(ROOM_CARD, &"Button")
	for state in [&"normal", &"hover", &"pressed", &"hover_pressed", &"disabled"]:
		var fill := Look.FOREST_PANEL
		if state == &"hover":
			fill = Look.FOREST_RAISED
		elif state in [&"pressed", &"hover_pressed"]:
			fill = Look.FOREST_DEEP
		var card := _rounded(fill, Look.RADIUS_PANEL)
		card.set_border_width_all(Look.BORDER)
		card.border_color = Look.PANEL_BORDER
		card.set_content_margin_all(Look.GAP_L)
		theme.set_stylebox(state, ROOM_CARD, card)
	theme.set_stylebox(&"focus", ROOM_CARD, _focus())
	# Un conseil, en bas d'un panneau : une plaque sombre.
	theme.set_type_variation(TIP_PANEL, &"PanelContainer")
	var tip := _rounded(Look.FOREST_DEEP, Look.RADIUS_CONTROL)
	tip.set_content_margin_all(Look.GAP_L)
	theme.set_stylebox(&"panel", TIP_PANEL, tip)
	# Le trait de part et d'autre de « ou ».
	theme.set_type_variation(DIVIDER, &"HSeparator")
	var line := StyleBoxLine.new()
	line.color = Look.PANEL_BORDER
	line.thickness = Look.DIVIDER_WIDTH
	theme.set_stylebox(&"separator", DIVIDER, line)


## Les cases du code d'une salle, et le champ invisible qui reçoit la frappe.
static func _code_boxes(theme: Theme) -> void:
	# Une case : une plaque crème ; celle où va la prochaine lettre est bordée d'or ; celles
	# d'un code refusé, de braise.
	var borders := {
		CODE_BOX: [Look.CASE_SIDE_A, Look.BORDER],
		CODE_BOX_ACTIVE: [Look.GOLD, Look.BORDER_STRONG],
		CODE_BOX_REFUSED: [Look.THREAT, Look.BORDER_STRONG],
	}
	for variation: StringName in borders:
		theme.set_type_variation(variation, &"PanelContainer")
		var box := _rounded(Look.CREAM, Look.RADIUS_CODE_BOX)
		box.border_color = borders[variation][0]
		box.set_border_width_all(borders[variation][1])
		theme.set_stylebox(&"panel", variation, box)
	# Le champ est invisible : il ouvre le clavier d'un téléphone, et les cases montrent
	# ce qu'on y tape.
	theme.set_type_variation(HIDDEN_INPUT, &"LineEdit")
	for state in [&"normal", &"focus", &"read_only"]:
		theme.set_stylebox(state, HIDDEN_INPUT, StyleBoxEmpty.new())
	for color in [&"font_color", &"caret_color", &"selection_color", &"font_selected_color", &"font_placeholder_color"]:
		theme.set_color(color, HIDDEN_INPUT, Color.TRANSPARENT)


## Les salles (maquettes 1-5, p. 4 à 6).
static func _rooms(theme: Theme, display: Font, caps: Font) -> void:
	_label(theme, TITLE_LABEL, display, Look.FONT_TITLE, Look.CREAM, true)
	_label(theme, VERSUS_LABEL, display, Look.FONT_TITLE, Look.MUTED, false)
	# Un refus : du texte riche, pour y mettre la croix devant le message.
	theme.set_type_variation(ERROR_TEXT, &"RichTextLabel")
	theme.set_font(&"normal_font", ERROR_TEXT, load(BOLD_FONT))
	theme.set_font_size(&"normal_font_size", ERROR_TEXT, Look.FONT_BODY)
	theme.set_color(&"default_color", ERROR_TEXT, Look.THREAT)
	_label(theme, CODE_LETTER_LARGE_LABEL, caps, Look.FONT_CODE_LARGE, Look.INK, false)
	# Un bouton qui n'est qu'un mot en capitales, terne, crème au survol.
	theme.set_type_variation(TEXT_BUTTON, &"Button")
	for state in [&"normal", &"hover", &"pressed", &"hover_pressed", &"disabled"]:
		var style := StyleBoxEmpty.new()
		style.set_content_margin_all(Look.GAP_M)
		theme.set_stylebox(state, TEXT_BUTTON, style)
	theme.set_stylebox(&"focus", TEXT_BUTTON, _focus())
	theme.set_font(&"font", TEXT_BUTTON, caps)
	theme.set_font_size(&"font_size", TEXT_BUTTON, Look.FONT_BODY)
	theme.set_color(&"font_color", TEXT_BUTTON, Look.MUTED)
	theme.set_color(&"font_focus_color", TEXT_BUTTON, Look.MUTED)
	for state in [&"font_hover_color", &"font_pressed_color", &"font_hover_pressed_color"]:
		theme.set_color(state, TEXT_BUTTON, Look.CREAM)


## Le draft (maquettes 1-5, p. 7 à 9).
static func _draft(theme: Theme, caps: Font) -> void:
	# Les étiquettes d'une carte : la plaque sombre de « CACHE », bordée de crème ou de braise.
	for pair: Array in [[TAG_PLAIN, TAG_PLAIN_LABEL, Look.CREAM], [TAG_THREAT, TAG_THREAT_LABEL, Look.THREAT]]:
		theme.set_type_variation(pair[0], &"PanelContainer")
		theme.set_stylebox(&"panel", pair[0], _tag(pair[2]))
		_label(theme, pair[1], caps, Look.FONT_CARD_TAG, pair[2], false)
	# La carte choisie : un contour d'or arrondi autour d'elle.
	theme.set_type_variation(CARD_OUTLINE, &"Panel")
	var outline := _rounded(Color.TRANSPARENT, Look.RADIUS_CONTROL)
	outline.draw_center = false
	outline.set_border_width_all(Look.BORDER_STRONG)
	outline.border_color = Look.GOLD
	theme.set_stylebox(&"panel", CARD_OUTLINE, outline)
	# Un filtre : une pastille bordée, d'or pleine quand il est choisi.
	theme.set_type_variation(FILTER_PILL, &"Button")
	for state in [&"normal", &"hover", &"pressed", &"hover_pressed", &"disabled"]:
		var chosen: bool = state in [&"pressed", &"hover_pressed"]
		var pill := _rounded(Look.GOLD if chosen else (Look.FOREST_RAISED if state == &"hover" else Look.FOREST_PANEL), Look.RADIUS_PILL)
		pill.set_border_width_all(Look.BORDER)
		pill.border_color = Look.GOLD if chosen else Look.FOREST_RAISED
		pill.content_margin_left = Look.GAP_L
		pill.content_margin_right = Look.GAP_L
		theme.set_stylebox(state, FILTER_PILL, pill)
	theme.set_stylebox(&"focus", FILTER_PILL, _focus())
	theme.set_font(&"font", FILTER_PILL, caps)
	theme.set_font_size(&"font_size", FILTER_PILL, Look.FONT_BODY)
	for state in [&"font_color", &"font_hover_color", &"font_focus_color"]:
		theme.set_color(state, FILTER_PILL, Look.CREAM)
	for state in [&"font_pressed_color", &"font_hover_pressed_color"]:
		theme.set_color(state, FILTER_PILL, Look.INK)
	# Le nom d'une place vide d'un arsenal (« PICK », « CACHE »), et la grille des cartes,
	# assez espacée pour leurs étiquettes.
	_label(theme, SLOT_LABEL, caps, Look.FONT_CARD_TAG, Look.MUTED, false)
	theme.set_type_variation(CARD_GRID, &"GridContainer")
	theme.set_constant(&"h_separation", CARD_GRID, Look.CARD_GAP)
	theme.set_constant(&"v_separation", CARD_GRID, Look.DRAFT_ROW_GAP)
	# Autour de la grille, la place de l'étiquette et du contour des cartes du bord.
	theme.set_type_variation(CARD_GRID_INSET, &"MarginContainer")
	theme.set_constant(&"margin_top", CARD_GRID_INSET, Look.GAP_L)
	for side in [&"margin_left", &"margin_right", &"margin_bottom"]:
		theme.set_constant(side, CARD_GRID_INSET, Look.CARD_GRID_SIDE)
	# Une place de l'ordre des picks : le pick en cours est bordé d'or.
	for variation: StringName in [SNAKE_SLOT, SNAKE_SLOT_CURRENT]:
		theme.set_type_variation(variation, &"PanelContainer")
		var slot := _rounded(Look.FOREST_PANEL, Look.RADIUS_CONTROL)
		slot.set_border_width_all(Look.BORDER_STRONG if variation == SNAKE_SLOT_CURRENT else Look.BORDER)
		slot.border_color = Look.GOLD if variation == SNAKE_SLOT_CURRENT else Look.FOREST_RAISED
		slot.set_content_margin_all(Look.GAP_XS)
		theme.set_stylebox(&"panel", variation, slot)


## La plaque sombre d'une étiquette, bordée de cette couleur.
static func _tag(border: Color) -> StyleBoxFlat:
	var tag := StyleBoxFlat.new()
	tag.bg_color = Look.FOREST_DEEP
	tag.set_border_width_all(Look.BORDER)
	tag.border_color = border
	tag.content_margin_left = Look.GAP_XS
	tag.content_margin_right = Look.GAP_XS
	return tag


## Les boîtes des scènes, par l'écart entre leurs enfants, et la marge intérieure.
static func _boxes(theme: Theme) -> void:
	theme.set_type_variation(STACK, &"VBoxContainer")
	theme.set_constant(&"separation", STACK, 0)
	var columns := {
		COLUMN_XS: Look.GAP_XS, COLUMN_S: Look.GAP_S, COLUMN_M: Look.GAP_M,
		COLUMN_L: Look.GAP_L, COLUMN_XL: Look.GAP_XL,
	}
	for variation: StringName in columns:
		theme.set_type_variation(variation, &"VBoxContainer")
		theme.set_constant(&"separation", variation, columns[variation])
	var rows := {ROW_XS: Look.GAP_XS, ROW_S: Look.GAP_S, ROW_M: Look.GAP_M, ROW_L: Look.GAP_L}
	for variation: StringName in rows:
		theme.set_type_variation(variation, &"HBoxContainer")
		theme.set_constant(&"separation", variation, rows[variation])
	theme.set_type_variation(GRID_XL, &"GridContainer")
	theme.set_constant(&"h_separation", GRID_XL, Look.GAP_XL)
	theme.set_constant(&"v_separation", GRID_XL, Look.GAP_XL)
	theme.set_type_variation(INSET, &"MarginContainer")
	for side in [&"margin_left", &"margin_top", &"margin_right", &"margin_bottom"]:
		theme.set_constant(side, INSET, Look.GAP_L)


## Les couleurs de l'icône d'un bouton : au repos, et enfoncé (l'onglet actif).
static func _icon_colors(theme: Theme, variation: StringName, normal: Color, pressed: Color) -> void:
	for state in [&"icon_normal_color", &"icon_hover_color", &"icon_focus_color", &"icon_disabled_color"]:
		theme.set_color(state, variation, normal)
	for state in [&"icon_pressed_color", &"icon_hover_pressed_color"]:
		theme.set_color(state, variation, pressed)
