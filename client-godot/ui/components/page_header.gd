class_name PageHeader
extends HBoxContainer
## L'en-tête d'une page (maquettes) : le bouton retour, un carré bordé d'or avec sa flèche,
## et le titre de la page.

signal back_pressed

## Le titre : à régler avant l'entrée dans l'arbre, ou par `set_title`.
@export var title := ""

var _title: Label


func _ready() -> void:
	theme_type_variation = ThemeBuilder.ROW_L
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var back := Ui.icon_button("retour", Labels.BACK)
	back.custom_minimum_size = Vector2.ONE * Look.HEADER_BUTTON_SIZE
	back.pressed.connect(back_pressed.emit)
	add_child(back)
	_title = Ui.label(title, ThemeBuilder.TITLE_LABEL)
	_title.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	add_child(_title)


func set_title(text: String) -> void:
	title = text
	if _title != null:
		_title.text = text
