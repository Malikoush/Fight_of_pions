extends SceneTree
## Lance les tests du client sans fenêtre :
##
##     godot --headless --path client-godot --script res://tests/run_tests.gd
##
## Chaque fichier de `SUITES` est un `TestCase` ; chacune de ses méthodes `test_…` est
## lancée sur un objet neuf, et doit rendre `true` : un test qui plante en route rend `null`
## et compte en échec. Un fichier qui ne se compile pas compte aussi en échec (sinon ses
## tests seraient sautés sans bruit). Le code de sortie vaut 1 si un test échoue.
##
## Les tests partent à la première image, l'arbre une fois actif : un écran ajouté par un
## test y est prêt (`_ready`, ses `@onready`), comme dans le jeu.

const SUITES := [
	"res://tests/test_wire.gd",
	"res://tests/test_server_errors.gd",
	"res://tests/test_view_data.gd",
	"res://tests/test_game_events.gd",
	"res://tests/test_game_journal.gd",
	"res://tests/test_board_timeline.gd",
	"res://tests/test_pion_status.gd",
	"res://tests/test_card_catalog.gd",
	"res://tests/test_card_collection.gd",
	"res://tests/test_session.gd",
	"res://tests/test_server_link.gd",
	"res://tests/test_menu.gd",
	"res://tests/test_view_queue.gd",
	"res://tests/test_edge_guard.gd",
]


var _done := false


func _process(_delta: float) -> bool:
	if not _done:
		_done = true
		_run_all()
	return false


func _run_all() -> void:
	var passed := 0
	var failed := 0
	for path: String in SUITES:
		var suite: GDScript = load(path)
		if suite == null or not suite.can_instantiate():
			failed += 1
			print("ÉCHEC %s · le fichier ne se compile pas" % path.get_file())
			continue
		for method: Dictionary in suite.get_script_method_list():
			var test_name: String = method["name"]
			if not test_name.begins_with("test_"):
				continue
			var case: TestCase = suite.new()
			if case.call(test_name) != true:
				case.failures.append("le test s'est arrêté avant la fin (erreur de script ?)")
			if case.failures.is_empty():
				passed += 1
			else:
				failed += 1
				print("ÉCHEC %s · %s" % [path.get_file(), test_name])
				for failure in case.failures:
					print("    ", failure)
	print("%d tests réussis, %d en échec" % [passed, failed])
	quit(1 if failed > 0 else 0)
