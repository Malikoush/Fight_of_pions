class_name LobbySeat
extends VBoxContainer
## Un joueur de la salle d'attente (maquettes) : sa case avec son palet posé dessus, son nom,
## et « PRÊT » en or ; une place encore libre est le contour d'une case en pointillés, avec
## un sablier et « EN ATTENTE ».

var _art: ElementArt
var _name: Label
var _hourglass: TextureRect
var _status: Label
## Ce que montre la place (`[joueur, nom, présent]`) : elle n'est redessinée que si ça change.
var _shown: Array = []


func _ready() -> void:
	theme_type_variation = ThemeBuilder.COLUMN_XS
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_art = ElementArt.new()
	_art.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	add_child(_art)
	_name = Ui.centered_label("", ThemeBuilder.STRONG_LABEL)
	add_child(_name)
	var status := HBoxContainer.new()
	status.theme_type_variation = ThemeBuilder.ROW_XS
	status.alignment = BoxContainer.ALIGNMENT_CENTER
	status.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_hourglass = Ui.icon("sablier", Look.MUTED, Look.FACT_ICON_SIZE)
	status.add_child(_hourglass)
	_status = Ui.label("", ThemeBuilder.CAPS_LABEL)
	status.add_child(_status)
	add_child(status)


## Le joueur `player`, nommé `who` : assis et prêt (`present`), ou sa place encore libre.
func show_seat(player: String, who: String, present: bool) -> void:
	if _shown == [player, who, present]:
		return
	_shown = [player, who, present]
	_art.clear()
	var center := ElementArt.first_anchor(Look.SEAT_ART_SCALE)
	if present:
		_art.add_tile(center, Look.SEAT_ART_SCALE)
		_art.add_pion(player, center, Look.SEAT_ART_SCALE)
	else:
		_art.add_free_place(center, Look.SEAT_ART_SCALE)
	_name.text = who
	_hourglass.visible = not present
	_status.text = Labels.PLAYER_READY if present else Labels.PLAYER_WAITING
	_status.theme_type_variation = ThemeBuilder.GOLD_CAPS_LABEL if present else ThemeBuilder.CAPS_LABEL
