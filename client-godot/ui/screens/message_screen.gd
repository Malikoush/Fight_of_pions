class_name MessageScreen
extends Screen
## Un écran qui n'est qu'une fenêtre (maquettes : « Partie ouverte ailleurs ») : ce qui se
## passe, sur le fond, et deux choix (« JOUER ICI » ou « ACCUEIL »). Échap ne la ferme pas :
## il n'y a rien dessous.

signal primary_pressed
signal secondary_pressed

var _dialog: AlertDialog


## `icon` : le nom d'une icône des éléments fournis, de braise.
func _init(title: String, text: String, primary: String, secondary: String, icon: String) -> void:
	_dialog = AlertDialog.new(title, text, secondary, primary, icon, true)
	_dialog.keep_on_cancel = true
	_dialog.confirmed.connect(primary_pressed.emit)
	_dialog.declined.connect(secondary_pressed.emit)


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_dialog)
	_dialog.ask()
