extends SceneTree
## Lance les tests du client sans fenêtre :
##
##     godot --headless --path client-godot --script res://tests/run_tests.gd
##
## Chaque fichier de `SUITES` est un `TestCase` ; chacune de ses méthodes `test_…` est
## lancée sur un objet neuf, et doit rendre `true` : un test qui plante en route rend `null`
## et compte en échec. Le code de sortie vaut 1 si un test échoue.

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
]


func _initialize() -> void:
	var passed := 0
	var failed := 0
	for path: String in SUITES:
		var suite: GDScript = load(path)
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
