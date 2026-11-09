class_name MessageScreen
extends Screen
## Une page qui dit ce qui se passe et propose quoi faire (« Partie ouverte ailleurs » :
## « REPRENDRE ICI » ou « ACCUEIL »).

signal primary_pressed
signal secondary_pressed

var _title := ""
var _text := ""
var _primary := ""
var _secondary := ""


func _init(title: String, text: String, primary: String, secondary: String) -> void:
	_title = title
	_text = text
	_primary = primary
	_secondary = secondary


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var column := Ui.vbox(Look.GAP_L)
	column.alignment = BoxContainer.ALIGNMENT_CENTER
	column.add_child(Ui.centered_label(_title, ThemeBuilder.HEADING_LABEL))
	column.add_child(Ui.centered_label(_text, ThemeBuilder.MUTED_LABEL))
	var primary := Ui.button(_primary, ThemeBuilder.PRIMARY_BUTTON)
	primary.pressed.connect(primary_pressed.emit)
	column.add_child(primary)
	var secondary := Ui.button(_secondary, ThemeBuilder.SECONDARY_BUTTON)
	secondary.pressed.connect(secondary_pressed.emit)
	column.add_child(secondary)
	add_child(Ui.page(Ui.limit_width(column, Look.CONTENT_MAX_WIDTH)))
