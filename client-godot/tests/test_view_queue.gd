extends TestCase
## La file des vues de l'écran de partie : l'ordre, l'attente, une carte qui part.

## Ce que la file a montré : `[vue, événements]`, dans l'ordre.
var shown: Array = []


func _queue() -> ViewQueue:
	var queue := ViewQueue.new()
	queue.show = func(view: ViewData, events: Array) -> float:
		shown.append([view, events])
		return 0.0
	return queue


func test_views_are_shown_in_order() -> bool:
	var queue := _queue()
	var first := view_of("game_view")
	var second := view_of("placed_view")
	queue.push(first, ["a"])
	queue.push(second, ["b"])
	equal(shown.size(), 2, "les deux vues")
	equal(shown[0][0], first, "la première d'abord")
	equal(shown[1][1], ["b"], "chacune avec ses événements")
	check(not queue.is_behind(), "rien n'attend")
	queue.free()
	return true


func test_a_blocked_queue_waits_then_resumes() -> bool:
	var queue := _queue()
	var blocked := [true]
	queue.blocked = func() -> bool: return blocked[0]
	queue.push(view_of("game_view"), [])
	equal(shown.size(), 0, "la cache révélée retient la vue")
	check(queue.is_behind(), "la vue affichée est périmée")
	blocked[0] = false
	queue.resume()
	equal(shown.size(), 1, "la vue passe une fois la cache fermée")
	check(not queue.is_behind(), "plus rien n'attend")
	queue.free()
	return true


func test_a_card_sent_move_by_move_shows_only_its_last_view() -> bool:
	var queue := _queue()
	queue.expect_sequence(3)
	check(queue.is_sending(), "la carte part")
	queue.push(view_of("game_view"), ["lanceur"])
	queue.push(view_of("game_view"), ["cible"])
	equal(shown.size(), 0, "les vues intermédiaires ne sont pas montrées")
	var last := view_of("placed_view")
	queue.push(last, ["fin"])
	equal(shown.size(), 1, "seule la dernière")
	equal(shown[0][0], last, "la vue du dernier coup")
	equal(shown[0][1], ["lanceur", "cible", "fin"], "avec les événements de toutes")
	check(not queue.is_sending(), "la carte est partie")
	queue.free()
	return true


func test_a_cancelled_card_forgets_its_carried_events() -> bool:
	var queue := _queue()
	queue.expect_sequence(2)
	queue.push(view_of("game_view"), ["lanceur"])
	queue.cancel_sequence()
	queue.push(view_of("placed_view"), ["suivante"])
	equal(shown.size(), 1, "la vue suivante passe")
	equal(shown[0][1], ["suivante"], "sans les événements de la carte abandonnée")
	queue.free()
	return true
