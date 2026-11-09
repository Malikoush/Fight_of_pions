class_name RoomCard
extends Button
## Une carte de salle de l'accueil (maquettes : « CRÉER UNE SALLE », « REJOINDRE ») : un
## panneau qu'on touche, avec son icône d'or, son titre et ce qu'il fait, dessous. C'est un
## bouton : survol, appui et clavier viennent avec.

var _icon_name := ""
var _title := ""
var _detail := ""
var _column: VBoxContainer


func _init(icon_name: String, title: String, detail: String) -> void:
	_icon_name = icon_name
	_title = title
	_detail = detail


func _ready() -> void:
	theme_type_variation = ThemeBuilder.ROOM_CARD
	_column = VBoxContainer.new()
	_column.theme_type_variation = ThemeBuilder.COLUMN_S
	_column.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var picture := Ui.icon(_icon_name, Look.GOLD, Look.ROOM_ICON_SIZE)
	picture.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	_column.add_child(picture)
	_column.add_child(Ui.label(_title, ThemeBuilder.CHOICE_TITLE_LABEL))
	var detail := Ui.label(_detail, ThemeBuilder.CAPTION_LABEL)
	detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_column.add_child(detail)
	add_child(_column)
	# Un bouton ne range pas ses enfants : la colonne suit ses marges, et le bouton
	# grandit avec elle.
	var style := get_theme_stylebox(&"normal")
	_column.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_column.offset_left = style.content_margin_left
	_column.offset_top = style.content_margin_top
	_column.offset_right = -style.content_margin_right
	_column.offset_bottom = -style.content_margin_bottom
	_column.minimum_size_changed.connect(_fit)
	_fit()


func _fit() -> void:
	var style := get_theme_stylebox(&"normal")
	custom_minimum_size.y = _column.get_combined_minimum_size().y + style.content_margin_top + style.content_margin_bottom
