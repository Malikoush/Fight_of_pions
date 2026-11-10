class_name MenuPage
extends VBoxContainer
## Une page du menu qui se lit (Cartes, Règles, Réglages) : son titre, une précision à sa
## droite (« 12 familles · 5 réactions », « 1 / 6 »), puis un contenu qui défile (`body`),
## rempli par la page elle-même (`_fill`). La barre de navigation du menu mène d'une page à
## l'autre.

var body: VBoxContainer
var _title := ""
var _aside: Label


func _init(title: String) -> void:
	_title = title


func _ready() -> void:
	theme_type_variation = ThemeBuilder.COLUMN_L
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var header := Ui.hbox(Look.GAP_M)
	var title := Ui.label(_title, ThemeBuilder.LOGO_LABEL)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(title)
	_aside = Ui.label("", ThemeBuilder.CAPTION_LABEL)
	_aside.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	header.add_child(_aside)
	add_child(header)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	# Sans barre (maquettes) : la page défile au doigt ou à la molette.
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	body = Ui.vbox(Look.GAP_L)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)
	add_child(scroll)
	_fill()


## La précision à droite du titre.
func set_aside(text: String) -> void:
	_aside.text = text


## Remplit `body` : à écrire par chaque page.
func _fill() -> void:
	pass


## Un paragraphe qui passe à la ligne.
func _paragraph(text: String, variation: StringName = ThemeBuilder.MUTED_LABEL) -> Label:
	var label := Ui.label(text, variation)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	return label


## Une rangée de filtres en pastilles (`Ui.add_filter_pills`), qui défile au doigt.
func _filter_row(choices: Dictionary, current: Variant, chosen: Callable) -> ScrollContainer:
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	var row := Ui.hbox(Look.GAP_S)
	Ui.add_filter_pills(row, choices, current, chosen)
	scroll.add_child(row)
	return scroll
