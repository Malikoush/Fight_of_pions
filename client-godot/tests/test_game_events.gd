extends TestCase
## Les événements joints à une vue.


func test_the_placed_pions_are_read_from_the_events() -> bool:
	var events: Array = fixture("placed_view").get("events", [])
	equal(GameEvents.kind(events[0]), "PionPlaced", "sorte")
	var placed := GameEvents.placed_pions(events)
	equal(placed.size(), 1, "un pion posé")
	equal(placed[0].get("owner"), ViewData.PLAYER_ONE, "le mien")
	equal(GameEvents.payload("EndTurn"), {}, "un événement sans donnée")
	return true
