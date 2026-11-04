class_name Wire
extends RefCounted
## Le format du fil : un message JSON par trame texte (docs/PROTOCOLE.md).
##
## Le JSON de Godot peut lire les nombres comme des flottants : renvoyé tel quel, un coup
## deviendrait `{"row":3.0}`, que le serveur refuse. Tout ce qui arrive passe donc par
## `normalize`, qui remet les nombres entiers en entiers. Les coups permis sont ensuite
## renvoyés exactement comme le serveur les a donnés.


## Le message lu dans cette trame, ou `null` si ce n'est pas du JSON.
static func decode(text: String) -> Variant:
	var json := JSON.new()
	if json.parse(text) != OK:
		return null
	return normalize(json.data)


static func encode(message: Dictionary) -> String:
	return JSON.stringify(message)


## La même valeur, où chaque nombre entier écrit en flottant redevient un entier.
static func normalize(value: Variant) -> Variant:
	match typeof(value):
		TYPE_FLOAT:
			var number: float = value
			if is_finite(number) and number == floorf(number):
				return int(number)
			return number
		TYPE_ARRAY:
			var items: Array = []
			for item: Variant in value:
				items.append(normalize(item))
			return items
		TYPE_DICTIONARY:
			var fields := {}
			for key: Variant in value:
				fields[key] = normalize(value[key])
			return fields
	return value
