class_name BoardMarks
extends Node3D
## Ce qui se pose ou passe sur les cases du plateau sans être une pièce :
## - la marque de la case survolée, et le bref voile d'une case refusée ;
## - les marques par sorte, réutilisées d'une vue à l'autre : le dernier coup adverse
##   (pointillés), les cases proposées pendant le choix d'une carte, les cases déjà
##   choisies, la ligne gagnante ;
## - les effets du déroulé d'une vue : l'éclat d'une case (attaque lancée, pion tué, mur
##   creusé) et le tir d'un garde de la reine.
##
## Le plateau lui dit où sont les cases (`ground`) et lesquelles sont dessinées
## (`is_drawn`) : une marque sur une case absente est cachée.

## Les marques par sorte.
enum Kind { LAST, CANDIDATE, CHOSEN, WIN }

## Le sol d'une case (`Vector2i -> Vector3`), et si elle est dessinée (`Vector2i -> bool`).
var ground: Callable
var is_drawn: Callable

var _hover: MeshInstance3D
var _refusal: MeshInstance3D
var _refusal_fade: Tween
var _pools: Dictionary[Kind, Array] = {}
var _materials: Dictionary[Kind, ShaderMaterial] = {}


func _ready() -> void:
	_hover = _flat(Look.MARK_SIZE, BoardMaterials.mark(Look.MARK_FILL, Look.MARK_BORDER))
	_refusal = _flat(Look.MARK_SIZE, BoardMaterials.mark(Look.REFUSAL_FILL, Look.REFUSAL_BORDER))
	_materials = {
		Kind.LAST: BoardMaterials.dashed_mark(Look.LAST_MARK, Look.LAST_MARK_DASHES),
		Kind.CANDIDATE: BoardMaterials.dotted_mark(Look.CANDIDATE_FILL, Look.CANDIDATE_DOT, Look.CANDIDATE_DOT_SIZE),
		Kind.CHOSEN: BoardMaterials.mark(Look.CHOSEN_FILL, Look.CHOSEN_BORDER),
		Kind.WIN: BoardMaterials.mark(Look.WIN_FILL, Look.WIN_BORDER),
	}
	for kind: Kind in Kind.values():
		_pools[kind] = []


## La marque de la case survolée, ou rien (`ViewData.NO_CELL`).
func show_hover(cell: Vector2i) -> void:
	_place(_hover, cell)


## Un bref voile terne sur cette case (rien n'y est permis), si elle est dessinée.
func flash_refusal(cell: Vector2i) -> void:
	if not _place(_refusal, cell):
		return
	if _refusal_fade != null:
		_refusal_fade.kill()
	_refusal.transparency = 0.0
	_refusal_fade = create_tween()
	_refusal_fade.tween_property(_refusal, "transparency", 1.0, Look.REFUSAL_TIME)
	_refusal_fade.tween_callback(func() -> void: _refusal.visible = false)


## Pose une marque de cette sorte sur chacune de ces cases, et cache les autres.
func show_marks(kind: Kind, cells: Array[Vector2i]) -> void:
	var pool: Array = _pools[kind]
	var size := Look.LAST_MARK_SIZE if kind == Kind.LAST else Look.MARK_SIZE
	while pool.size() < cells.size():
		pool.append(_flat(size, _materials[kind]))
	for index in pool.size():
		_place(pool[index], cells[index] if index < cells.size() else ViewData.NO_CELL)


## Un éclat à plat sur cette case, après ce délai : un anneau de cette couleur qui
## s'élargit en s'effaçant (attaque lancée, pion tué, mur creusé).
func pulse(cell: Vector2i, color: Color, delay: float) -> void:
	if not is_drawn.call(cell):
		return
	var ring := _flat(GridGeometry.CELL_SIZE, BoardMaterials.pulse(color))
	_place(ring, cell)
	ring.visible = false
	ring.scale = Vector3.ONE * Look.PULSE_START_SCALE
	var grow := create_tween()
	grow.tween_interval(delay)
	grow.tween_callback(ring.show)
	grow.set_parallel()
	grow.chain().tween_property(ring, "scale", Vector3.ONE * Look.PULSE_END_SCALE, Look.PULSE_TIME).set_ease(Tween.EASE_OUT)
	grow.tween_property(ring, "transparency", 1.0, Look.PULSE_TIME).set_ease(Tween.EASE_IN)
	grow.chain().tween_callback(ring.queue_free)


## Le tir d'un garde de la reine, après ce délai : une balle d'or qui vole en cloche de la
## reine à sa cible.
func shoot(from: Vector2i, to: Vector2i, delay: float) -> void:
	if not (is_drawn.call(from) and is_drawn.call(to)):
		return
	var ball := MeshInstance3D.new()
	ball.mesh = BoardMaterials.shot_mesh()
	ball.visible = false
	add_child(ball)
	var start: Vector3 = ground.call(from) + Vector3.UP * Look.PION_HEIGHT
	var end: Vector3 = ground.call(to) + Vector3.UP * Look.PION_HEIGHT
	var fly := func(progress: float) -> void:
		ball.position = BoardTimeline.along_arc(start, end, Look.SHOT_ARC, progress)
	fly.call(0.0)
	var flight := create_tween()
	flight.tween_interval(delay)
	flight.tween_callback(ball.show)
	flight.tween_method(fly, 0.0, 1.0, Look.SHOT_TIME)
	flight.tween_callback(ball.queue_free)


## Une marque à plat, cachée, de cette taille et ce matériau.
func _flat(size: float, material: ShaderMaterial) -> MeshInstance3D:
	var mark := MeshInstance3D.new()
	var plane := PlaneMesh.new()
	plane.size = Vector2.ONE * size
	mark.mesh = plane
	mark.material_override = material
	mark.visible = false
	add_child(mark)
	return mark


## Pose cette marque sur une case dessinée, ou la cache ; vrai si elle est montrée.
func _place(mark: MeshInstance3D, cell: Vector2i) -> bool:
	mark.visible = is_drawn.call(cell)
	if mark.visible:
		mark.position = ground.call(cell) + Vector3.UP * Look.FLAT_LIFT
	return mark.visible
