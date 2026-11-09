class_name NavBar
extends PanelContainer
## La barre de navigation du menu (maquettes) : un onglet par page, l'actif allumé.
## - En portrait, elle est en bas, sous le pouce : l'icône au-dessus du nom, un losange d'or
##   posé sur le trait au-dessus de l'onglet actif.
## - En paysage, elle est en haut : le logo, les onglets soulignés d'or, la pastille du
##   joueur.
##
## Elle ne sait pas ce que montrent les onglets : elle dit lequel est choisi.

signal tab_selected(index: int)
signal player_pressed

var _buttons: Array[Button] = []
var _group := ButtonGroup.new()
var _wide := false
var _selected := 0

@onready var _logo: Label = %Logo
@onready var _tabs: HBoxContainer = %Tabs
@onready var _pill: PlayerPill = %Pill
@onready var _top_inset: Control = %TopInset
@onready var _bottom_inset: Control = %BottomInset


func _ready() -> void:
	_logo.text = Labels.GAME_NAME
	_pill.pressed.connect(player_pressed.emit)
	set_wide(_wide)


## Les onglets, dans l'ordre : l'icône (`assets/elements/icones`) et le nom de chacun.
func setup(entries: Array) -> void:
	for index in entries.size():
		var entry: Array = entries[index]
		var button := Button.new()
		button.icon = load(Ui.ICON_DIR + str(entry[0]) + ".svg")
		button.text = entry[1]
		button.toggle_mode = true
		button.button_group = _group
		button.focus_mode = Control.FOCUS_NONE
		button.pressed.connect(_on_pressed.bind(index))
		# Le repère suit l'onglet quand la barre change de taille.
		button.item_rect_changed.connect(queue_redraw)
		_tabs.add_child(button)
		_buttons.append(button)
	_style_tabs()
	select(_selected)


## Allume cet onglet, sans l'annoncer (l'écran l'a déjà ouvert).
func select(index: int) -> void:
	_selected = index
	if index < _buttons.size():
		_buttons[index].set_pressed_no_signal(true)
	queue_redraw()


## En haut, en paysage (`true`), ou en bas, en portrait.
func set_wide(wide: bool) -> void:
	_wide = wide
	theme_type_variation = ThemeBuilder.NAV_BAR_TOP if wide else ThemeBuilder.NAV_BAR
	_logo.visible = wide
	_pill.visible = wide
	_style_tabs()
	queue_redraw()


## Les bords inaccessibles du téléphone que la barre couvre (en haut en paysage, en bas en
## portrait) : son fond y va, ses onglets restent hors d'eux.
func set_insets(top: float, bottom: float) -> void:
	_top_inset.custom_minimum_size.y = top
	_bottom_inset.custom_minimum_size.y = bottom


func _style_tabs() -> void:
	for button in _buttons:
		button.theme_type_variation = ThemeBuilder.NAV_TAB_WIDE if _wide else ThemeBuilder.NAV_TAB
		button.icon_alignment = HORIZONTAL_ALIGNMENT_LEFT if _wide else HORIZONTAL_ALIGNMENT_CENTER
		button.vertical_icon_alignment = VERTICAL_ALIGNMENT_CENTER if _wide else VERTICAL_ALIGNMENT_TOP
		# En bas, les onglets se partagent la largeur ; en haut, ils se suivent après le logo.
		button.size_flags_horizontal = Control.SIZE_FILL if _wide else Control.SIZE_EXPAND_FILL


func _on_pressed(index: int) -> void:
	_selected = index
	queue_redraw()
	tab_selected.emit(index)


## Le repère de l'onglet actif : un trait d'or sous lui, en haut ; un losange d'or sur le
## trait de la barre, au-dessus de lui, en bas.
func _draw() -> void:
	if _selected >= _buttons.size():
		return
	var button := _buttons[_selected]
	var area := Rect2(button.global_position - global_position, button.size)
	if _wide:
		draw_rect(Rect2(area.position.x, size.y - Look.NAV_UNDERLINE, area.size.x, Look.NAV_UNDERLINE), Look.GOLD)
		return
	var center := Vector2(area.get_center().x, Look.BORDER / 2.0)
	var half := Look.NAV_MARKER / 2.0
	var diamond := PackedVector2Array([
		center + Vector2(0.0, -half), center + Vector2(half, 0.0),
		center + Vector2(0.0, half), center + Vector2(-half, 0.0),
	])
	draw_colored_polygon(diamond, Look.GOLD)
	diamond.append(diamond[0])
	draw_polyline(diamond, Look.GOLD, Look.SMOOTH_EDGE, true)
