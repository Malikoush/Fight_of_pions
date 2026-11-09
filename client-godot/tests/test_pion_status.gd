extends TestCase
## Ce qu'on montre d'un pion d'après les effets qui le visent.


func _effect(kind: Variant, end: Variant) -> Dictionary:
	return {"id": 0, "kind": kind, "pion": 7, "creator": "One", "play": 0, "end": end}


func test_the_strongest_status_draws_the_ring() -> bool:
	var status := PionStatus.of([_effect({"Pact": {"friend": 3}}, "Never"), _effect("Immobilized", {"Turn": 5})])
	equal(status.ring, "Immobilized", "immobilisé avant le pacte")
	equal(status.badges, ["badge-immobilise", "badge-pacte"] as Array[String], "un badge par sorte")
	return true


func test_a_framed_protection_is_a_child() -> bool:
	var status := PionStatus.of([_effect("Protected", {"Framed": [1, 2]})])
	check(status.child, "un enfant")
	equal(status.badges, ["badge-enfant"] as Array[String], "son badge")
	return true


func test_a_queen_counts_her_guards_and_a_hare_its_tokens() -> bool:
	var queen := PionStatus.of([_effect("Crown", "Never"), _effect("Guard", "Never"), _effect("Guard", "Never")])
	check(queen.crown, "couronnée")
	equal(queen.guards, 2, "deux gardes")
	var hare := PionStatus.of([_effect("StepTokens", {"Tokens": 3})])
	equal(hare.step_tokens, 3, "trois jetons de pas")
	check(PionStatus.of([]).is_plain(), "sans effet, rien à montrer")
	return true
