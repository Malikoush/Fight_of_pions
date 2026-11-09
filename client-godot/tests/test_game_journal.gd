extends TestCase
## Le journal des coups, raconté à partir des événements.


func test_a_placement_is_told_as_an_action_of_its_player() -> bool:
	var message := fixture("placed_view")
	var view := ViewData.new(message["view"])
	var lines := GameJournal.new().lines(message["events"], view, 1, null)
	equal(lines.size(), 1, "une ligne : la pose (le reste ne se raconte pas)")
	equal(lines[0].get("player"), ViewData.PLAYER_ONE, "son joueur")
	check(lines[0].get("acts", false), "c'est une action")
	equal(lines[0].get("text"), Labels.JOURNAL_PLACE % "D4", "son texte")
	equal(lines[0].get("turn"), 1, "son tour")
	return true


func test_events_after_the_end_of_a_turn_belong_to_the_next_one() -> bool:
	var view := view_of("game_view")
	var events := [
		{"PionPlaced": {"pion": {"id": 4, "owner": "Two"}, "at": {"row": 5, "col": 5}}},
		{"TurnEnded": {"next": "One"}},
		{"WallDug": {"at": {"row": 2, "col": 6}}},
	]
	var lines := GameJournal.new().lines(events, view, 0, null)
	equal(lines[0].get("turn"), 0, "avant la fin du tour : le tour d'avant")
	equal(lines[1].get("turn"), view.turn(), "après : le tour de la vue")
	check(not lines[1].get("acts", true), "un mur creusé est une conséquence")
	return true


func test_a_card_is_named_by_the_catalog() -> bool:
	var catalog := CardCatalog.new(fixture("cards")["cards"])
	var view := view_of("game_view")
	var events := [{"AttackUsed": {"player": "Two", "attack": {"kind": "Ours", "rarity": "Rare"}, "launcher": {"row": 4, "col": 4}}}]
	var lines := GameJournal.new().lines(events, view, 1, catalog)
	equal(lines[0].get("text"), Labels.JOURNAL_ATTACK % ["Ours Rare", "E5"], "nom et rareté du catalogue")
	return true
