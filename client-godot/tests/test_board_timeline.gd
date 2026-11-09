extends TestCase
## Le déroulé d'une vue : quand commence chaque animation.


func test_a_placement_drops_at_once() -> bool:
	var timeline := BoardTimeline.of(fixture("placed_view")["events"])
	equal(timeline.drops.get(0), 0.0, "le pion posé tombe tout de suite")
	equal(timeline.length, Look.PION_DROP_TIME + Look.PION_SQUASH_TIME, "la durée d'une chute")
	return true


func test_kills_play_together_and_their_tombs_rise_after() -> bool:
	var events := [
		{"AttackUsed": {"player": "One", "attack": {"kind": "Ours", "rarity": "Epic"}, "launcher": {"row": 6, "col": 6}}},
		{"PionKilled": {"pion": {"id": 1, "owner": "Two"}, "at": {"row": 5, "col": 5}, "by": 0}},
		{"TombAdded": {"pion": {"id": 1, "owner": "Two"}, "at": {"row": 5, "col": 5}}},
		{"PionKilled": {"pion": {"id": 2, "owner": "Two"}, "at": {"row": 5, "col": 7}, "by": 0}},
		{"TombAdded": {"pion": {"id": 2, "owner": "Two"}, "at": {"row": 5, "col": 7}}},
	]
	var timeline := BoardTimeline.of(events)
	equal(timeline.kills[1]["delay"], Look.ATTACK_BEAT, "le premier kill, après l'éclat de l'attaque")
	equal(timeline.kills[2]["delay"], Look.ATTACK_BEAT, "le second, en même temps")
	equal(timeline.tombs[1], Look.ATTACK_BEAT + Look.KILL_TIME, "la tombe, à la fin de son kill")
	equal(timeline.length, Look.ATTACK_BEAT + Look.KILL_TIME + Look.TOMB_RISE_TIME, "la durée du tout")
	return true


func test_a_far_jump_is_higher_than_a_step() -> bool:
	var step := BoardTimeline.of([{"PionMoved": {"pion": {"id": 3, "owner": "One"}, "from": {"row": 4, "col": 4}, "to": {"row": 4, "col": 5}}}])
	var jump := BoardTimeline.of([{"PionMoved": {"pion": {"id": 3, "owner": "One"}, "from": {"row": 4, "col": 4}, "to": {"row": 4, "col": 7}}}])
	check(jump.moves[3]["arc"] > step.moves[3]["arc"], "plus haut quand il va plus loin")
	check(jump.moves[3]["arc"] <= Look.JUMP_ARC_MAX, "jusqu'à une borne")
	equal(BoardTimeline.along_arc(Vector3.ZERO, Vector3.RIGHT, 1.0, 0.5), Vector3(0.5, 1.0, 0.0), "le sommet de la cloche")
	return true
