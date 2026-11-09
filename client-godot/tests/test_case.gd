class_name TestCase
extends RefCounted
## La base d'un fichier de tests : chaque méthode `test_…` est lancée par `run_tests.gd`, note
## ses échecs avec `check` et `equal`, et rend `true` à sa dernière ligne : un test qui
## plante en route (une erreur de script) rend `null`, et le lanceur le compte en échec.
## Les vues de test sont de vraies vues du serveur, enregistrées dans `tests/fixtures/`
## (voir `capture_fixtures.py`).

const FIXTURES := "res://tests/fixtures/"

## Les échecs du test en cours.
var failures: Array[String] = []


func check(condition: bool, what: String) -> void:
	if not condition:
		failures.append(what)


func equal(actual: Variant, expected: Variant, what: String) -> void:
	if actual != expected:
		failures.append("%s : attendu %s, obtenu %s" % [what, expected, actual])


## Le message enregistré sous ce nom (`draft_view`, `game_view`…), lu comme le client lit
## le fil. Un fichier absent ou illisible fait échouer le test.
func fixture(name: String) -> Dictionary:
	var text := FileAccess.get_file_as_string(FIXTURES + name + ".json")
	var message: Variant = Wire.decode(text)
	if not (message is Dictionary):
		failures.append("vue de test absente ou illisible : %s" % name)
		return {}
	return message


## La vue d'un message enregistré.
func view_of(name: String) -> ViewData:
	return ViewData.new(fixture(name).get("view", {}))
