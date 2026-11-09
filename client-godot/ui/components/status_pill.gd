class_name StatusPill
extends PanelContainer
## Une pastille d'état de la partie (maquettes) : un sablier d'or et un texte en capitales.
## Le tour (« TOUR 7 · À TOI », sablier quand on attend) ; « Le bot réfléchit… ».

var _hourglass: TextureRect
var _label: Label


func _init() -> void:
	theme_type_variation = ThemeBuilder.PILL
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	var row := Ui.hbox(Look.GAP_S)
	_hourglass = Ui.icon(Ui.HOURGLASS, Look.GOLD)
	row.add_child(_hourglass)
	_label = Ui.label("", ThemeBuilder.PILL_LABEL)
	row.add_child(_label)
	add_child(row)


## Ce texte, avec ou sans le sablier.
func show_status(text: String, hourglass: bool) -> void:
	_label.text = text
	_hourglass.visible = hourglass
