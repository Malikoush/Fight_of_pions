class_name CodeBoxes
extends HBoxContainer
## Le code d'une salle, une case crème par caractère (maquettes) : la case de la prochaine
## lettre bordée d'or, toutes bordées de braise quand le serveur a refusé le code. En grand
## (`large`, rejoindre et salle d'attente), les cases se partagent la largeur.
##
## Elles ne font que montrer : la salle d'attente y montre son code ; le champ du code
## (`CodeField`) y montre ce qu'on tape.

## En grand : à régler avant l'entrée dans l'arbre.
@export var large := false

var _boxes: Array[PanelContainer] = []
var _letters: Array[Label] = []


func _ready() -> void:
	theme_type_variation = ThemeBuilder.ROW_S
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	for index in ServerLink.ROOM_CODE_LENGTH:
		var box := PanelContainer.new()
		box.mouse_filter = Control.MOUSE_FILTER_IGNORE
		if large:
			box.custom_minimum_size.y = Look.CODE_BOX_LARGE_HEIGHT
			box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		else:
			box.custom_minimum_size = Look.CODE_BOX_SIZE
		var letter := Ui.centered_label("", ThemeBuilder.CODE_LETTER_LARGE_LABEL if large else ThemeBuilder.CODE_LETTER_LABEL)
		letter.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		box.add_child(letter)
		add_child(box)
		_boxes.append(box)
		_letters.append(letter)
	show_code("")


## Montre ce code : la case `next` (celle de la prochaine lettre, -1 pour aucune) bordée
## d'or, avec le curseur ; toutes de braise si le code a été refusé.
func show_code(code: String, next := -1, refused := false) -> void:
	for index in _boxes.size():
		var letter := code[index] if index < code.length() else ""
		_letters[index].text = Labels.CODE_CARET if index == next and letter == "" else letter
		var variation := ThemeBuilder.CODE_BOX
		if refused:
			variation = ThemeBuilder.CODE_BOX_REFUSED
		elif index == next:
			variation = ThemeBuilder.CODE_BOX_ACTIVE
		_boxes[index].theme_type_variation = variation
