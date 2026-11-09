class_name MenuPage
extends VBoxContainer
## Une page du menu qui se lit (Cartes, Règles, Réglages) : son titre, puis un contenu qui
## défile (`body`), rempli par la page elle-même (`_fill`). La barre de navigation du menu
## mène d'une page à l'autre. Leur contenu sera refait d'après les maquettes 12-17 (R8).

var body: VBoxContainer
var _title := ""


func _init(title: String) -> void:
	_title = title


func _ready() -> void:
	theme_type_variation = ThemeBuilder.COLUMN_L
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(Ui.label(_title, ThemeBuilder.LOGO_LABEL))
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	body = Ui.vbox(Look.GAP_L)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)
	add_child(scroll)
	_fill()


## Remplit `body` : à écrire par chaque page.
func _fill() -> void:
	pass


## Un paragraphe qui passe à la ligne.
func _paragraph(text: String, variation: StringName = ThemeBuilder.MUTED_LABEL) -> Label:
	var label := Ui.label(text, variation)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	return label
