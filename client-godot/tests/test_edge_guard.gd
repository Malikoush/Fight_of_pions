extends TestCase
## Le geste du système en bas d'un téléphone : un toucher qui commence dans la bande du bas
## n'appuie sur rien (`EdgeGuard`), ni lui, ni la souris que Godot en déduit.

const HEIGHT := 844.0
const EDGE := 24.0
## Dans la bande, et juste au-dessus.
const IN_EDGE := Vector2(200, 835)
const ABOVE := Vector2(200, 800)


func test_a_touch_starting_in_the_bottom_edge_is_ignored_until_lifted() -> bool:
	var guard := EdgeGuard.new()
	check(guard.blocks(_touch(0, IN_EDGE, true), HEIGHT, EDGE), "le toucher posé en bas")
	check(guard.blocks(_drag(0, ABOVE), HEIGHT, EDGE), "son glissé vers le haut")
	check(guard.blocks(_touch(0, ABOVE, false), HEIGHT, EDGE), "son lever, même plus haut")
	check(not guard.blocks(_touch(0, ABOVE, true), HEIGHT, EDGE), "le toucher suivant, au-dessus, passe")
	check(not guard.blocks(_touch(0, ABOVE, false), HEIGHT, EDGE), "et son lever aussi")
	guard.free()
	return true


func test_only_a_touch_that_starts_in_the_edge_is_ignored() -> bool:
	var guard := EdgeGuard.new()
	check(not guard.blocks(_touch(0, ABOVE, true), HEIGHT, EDGE), "posé au-dessus")
	check(not guard.blocks(_drag(0, IN_EDGE), HEIGHT, EDGE), "puis glissé dans la bande : il compte")
	check(not guard.blocks(_touch(0, IN_EDGE, false), HEIGHT, EDGE), "et son lever dans la bande aussi")
	check(guard.blocks(_touch(0, Vector2(200, HEIGHT - EDGE), true), HEIGHT, EDGE), "le bord exact de la bande en fait partie")
	var cancel := _touch(0, IN_EDGE, false)
	cancel.canceled = true
	check(guard.blocks(cancel, HEIGHT, EDGE), "un toucher annulé par le système est un lever ignoré")
	check(guard.blocks(_touch(0, IN_EDGE, true), HEIGHT, EDGE), "deux doigts : le premier dans la bande")
	check(not guard.blocks(_touch(1, ABOVE, true), HEIGHT, EDGE), "le second, au-dessus, passe")
	check(not guard.blocks(_touch(1, ABOVE, false), HEIGHT, EDGE), "et se lève")
	check(guard.blocks(_touch(0, IN_EDGE, false), HEIGHT, EDGE), "le premier reste ignoré jusqu'à son lever")
	guard.free()
	return true


func test_the_mouse_godot_derives_from_that_touch_is_ignored_too() -> bool:
	var guard := EdgeGuard.new()
	check(guard.blocks(_mouse(IN_EDGE, true, InputEvent.DEVICE_ID_EMULATION), HEIGHT, EDGE), "l'appui déduit")
	var motion := InputEventMouseMotion.new()
	motion.device = InputEvent.DEVICE_ID_EMULATION
	motion.position = ABOVE
	check(guard.blocks(motion, HEIGHT, EDGE), "son mouvement")
	check(guard.blocks(_mouse(ABOVE, false, InputEvent.DEVICE_ID_EMULATION), HEIGHT, EDGE), "son lever : le bouton ne s'enclenche pas")
	check(not guard.blocks(_mouse(ABOVE, true, InputEvent.DEVICE_ID_EMULATION), HEIGHT, EDGE), "un appui au-dessus passe")
	check(not guard.blocks(_mouse(ABOVE, false, InputEvent.DEVICE_ID_EMULATION), HEIGHT, EDGE), "et son lever")
	guard.free()
	return true


func test_nothing_is_ignored_without_an_edge_or_with_a_real_mouse() -> bool:
	var guard := EdgeGuard.new()
	check(not guard.blocks(_touch(0, IN_EDGE, true), HEIGHT, 0.0), "hors d'un téléphone, pas de bande")
	check(not guard.blocks(_mouse(IN_EDGE, true, 0), HEIGHT, EDGE), "une vraie souris, tout en bas")
	check(not guard.blocks(_mouse(IN_EDGE, false, 0), HEIGHT, EDGE), "et son lever")
	equal(EdgeGuard.edge_height(Vector2(390, HEIGHT)), 0.0, "sur PC, la bande est nulle")
	guard.free()
	return true


static func _touch(index: int, position: Vector2, pressed: bool) -> InputEventScreenTouch:
	var event := InputEventScreenTouch.new()
	event.index = index
	event.position = position
	event.pressed = pressed
	return event


static func _drag(index: int, position: Vector2) -> InputEventScreenDrag:
	var event := InputEventScreenDrag.new()
	event.index = index
	event.position = position
	return event


static func _mouse(position: Vector2, pressed: bool, device: int) -> InputEventMouseButton:
	var event := InputEventMouseButton.new()
	event.device = device
	event.button_index = MOUSE_BUTTON_LEFT
	event.position = position
	event.pressed = pressed
	return event
