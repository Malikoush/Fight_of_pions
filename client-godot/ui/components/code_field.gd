class_name CodeField
extends MarginContainer
## Taper le code d'une salle (maquettes) : on tape au clavier, ou sur celui du téléphone,
## qui s'ouvre au toucher, et les cases (`CodeBoxes`) se remplissent toutes seules, en
## capitales. Effacer reprend la dernière lettre. Un code refusé borde les cases de braise,
## jusqu'à ce qu'on le corrige.
##
## La frappe va dans un champ invisible posé sur les cases : c'est lui qui a le focus et
## ouvre le clavier. Seuls les lettres et les chiffres sont gardés ; le serveur dit si le
## code existe.

## Le code a changé (en capitales, de 0 à `ServerLink.ROOM_CODE_LENGTH` caractères).
signal changed(code: String)
## Entrée, une fois le code complet.
signal submitted(code: String)

## En grand (rejoindre) : à régler avant l'entrée dans l'arbre.
@export var large := false

var _input: LineEdit
var _boxes: CodeBoxes
var _refused := false


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_boxes = CodeBoxes.new()
	_boxes.large = large
	add_child(_boxes)

	_input = LineEdit.new()
	_input.theme_type_variation = ThemeBuilder.HIDDEN_INPUT
	_input.max_length = ServerLink.ROOM_CODE_LENGTH
	_input.context_menu_enabled = false
	_input.caret_blink = false
	_input.text_changed.connect(_on_text_changed)
	_input.text_submitted.connect(func(_text: String) -> void:
		if is_complete():
			submitted.emit(code()))
	_input.focus_entered.connect(_refresh)
	_input.focus_exited.connect(_refresh)
	add_child(_input)
	_refresh()


func code() -> String:
	return _input.text


func is_complete() -> bool:
	return _input.text.length() == ServerLink.ROOM_CODE_LENGTH


## Donne le focus au champ : on peut taper sans toucher les cases.
func focus() -> void:
	_input.grab_focus()


## Le serveur a refusé ce code : les cases se bordent de braise, jusqu'à la prochaine lettre.
func show_refused() -> void:
	_refused = true
	_refresh()


## Ce qu'on garde d'un texte tapé ou collé : ses lettres et ses chiffres, en capitales,
## jusqu'à la longueur d'un code.
static func clean(text: String) -> String:
	var kept := ""
	for character in text.to_upper():
		if (character >= "A" and character <= "Z") or (character >= "0" and character <= "9"):
			kept += character
	return kept.left(ServerLink.ROOM_CODE_LENGTH)


func _on_text_changed(text: String) -> void:
	var kept := clean(text)
	_input.text = kept
	_input.caret_column = kept.length()
	_refused = false
	_refresh()
	changed.emit(kept)


## La case de la prochaine lettre est bordée d'or, tant que le champ a le focus.
func _refresh() -> void:
	var next := _input.text.length() if _input.has_focus() else -1
	_boxes.show_code(_input.text, next, _refused)
