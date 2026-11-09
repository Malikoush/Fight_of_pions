class_name ScreenRouter
extends RefCounted
## L'écran affiché, et le passage de l'un à l'autre : l'ancien s'efface en fondu, le nouveau
## apparaît, avec la session en cours. Derrière un écran qui montre le plateau 3D, le
## plateau est visible ; derrière les autres, le fond quadrillé.

## L'écran affiché.
var current: Screen

var _container: Control
var _board: Node3D
var _backdrop: Control
var _session: Session


func _init(container: Control, board: Node3D, backdrop: Control, session: Session) -> void:
	_container = container
	_board = board
	_backdrop = backdrop
	_session = session


## Affiche cet écran, déjà câblé, à la place du précédent.
func show(screen: Screen) -> void:
	if current != null:
		var old := current
		var fade := old.create_tween()
		fade.tween_property(old, "modulate:a", 0.0, Look.SCREEN_FADE)
		fade.tween_callback(old.queue_free)
	current = screen
	screen.session = _session
	_board.visible = screen.uses_board()
	_backdrop.visible = not screen.uses_board()
	screen.modulate.a = 0.0
	_container.add_child(screen)
	screen.create_tween().tween_property(screen, "modulate:a", 1.0, Look.SCREEN_FADE)
