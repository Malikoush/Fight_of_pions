extends TestCase
## La vue : ses lectures typées, sur de vraies vues du serveur.


func test_a_draft_view_tells_its_step_and_its_cards() -> bool:
	var view := view_of("draft_view")
	equal(view.decision_kind(), "Draft", "décision")
	equal(Wire.tag(view.draft_step()), "Ban", "étape")
	equal(Wire.content(view.draft_step()), ViewData.PLAYER_ONE, "joueur de l'étape")
	equal(view.decision_player(), ViewData.PLAYER_ONE, "qui décide")
	equal(view.draft_pool().size(), 10, "cartes tirées")
	equal(view.legal_actions().size(), 10, "une carte permise par carte tirée")
	equal(Wire.tag(view.draft_steps()[-1]), "Cache", "la cache en dernier")
	return true


func test_a_game_view_sorts_the_moves_by_cell() -> bool:
	var view := view_of("game_view")
	equal(view.viewer(), ViewData.PLAYER_ONE, "qui regarde")
	equal(view.opponent(), ViewData.PLAYER_TWO, "l'adversaire")
	equal(view.grid_size(), 16, "grille, mur compris")
	var corner := Vector2i(3, 3)
	check(view.action_at(corner, "Place") != null, "pose permise sur la première case jouable")
	check(view.action_at(Vector2i(0, 0), "Place") == null, "rien sur le mur")
	check(view.is_wall(Vector2i(0, 0)), "le coin est du mur")
	check(not view.is_wall(corner), "la case jouable n'est pas du mur")
	check(view.simple_action("EndTurn") != null, "finir son tour est permis")
	equal(view.cells_of("Place").size(), view.legal_actions().size() - 1, "une pose par case, plus finir son tour")
	return true


func test_the_opponent_cache_is_hidden_in_the_arsenal() -> bool:
	var view := view_of("game_view")
	var mine: Variant = view.arsenal(ViewData.PLAYER_ONE)[-1]
	var theirs: Variant = view.arsenal(ViewData.PLAYER_TWO)[-1]
	check(mine is Dictionary and mine.get("secret", false), "ma cache, marquée secrète")
	equal(theirs, null, "la cache adverse, invisible")
	return true


func test_a_placed_pion_is_found_by_cell_and_by_identity() -> bool:
	var view := view_of("placed_view")
	var cell := Vector2i(3, 3)
	var pion: Variant = view.pion_at(cell)
	check(pion is Dictionary and pion.get("owner") == ViewData.PLAYER_ONE, "le pion posé")
	equal(view.cell_of(pion.get("id", -1)), cell, "sa case, par son identité")
	equal(view.cell_of(999), ViewData.NO_CELL, "un pion absent")
	equal(Labels.cell_name(cell), "D4", "le nom de sa case, comme le moteur")
	return true
