class_name Pion3D
extends Node3D
## Un pion sur le plateau : un cylindre pour J1, un prisme à 8 faces pour J2 (la même
## primitive), et son ombre. L'origine du nœud est au sol de sa case ; un modèle plus riche
## pourra remplacer le corps sans changer le reste.
##
## Ce que les effets lui donnent (`show_status`) : un anneau à ses pieds, une taille
## d'enfant, une couronne. Les badges, eux, sont dessinés par-dessus le plateau
## (`BadgeLayer`), au point que donne `badge_anchor`.
##
## En fantôme, le même palet transparent et sans ombre montre où tombera un pion.

const DISC_SEGMENTS := 48
const OCTAGON_SEGMENTS := 8

static var _meshes := {}
static var _shadow_mesh: PlaneMesh
static var _shadow_material: ShaderMaterial

## Ce qui prend la taille du pion (enfant ou non) : le corps, l'ombre, l'anneau.
var _pivot: Node3D
var _body: MeshInstance3D
var _ring: MeshInstance3D
var _crown: Crown3D
var _move: Tween
var _resize: Tween
var _ring_kind := ""
var _scale_target := Vector3.ONE


func setup(pion: Dictionary, ghost: bool = false) -> void:
	var player: String = pion.get("owner", ViewData.PLAYER_ONE)
	_pivot = Node3D.new()
	add_child(_pivot)
	_body = MeshInstance3D.new()
	_body.mesh = _body_mesh(player)
	_body.material_override = BoardMaterials.ghost(player) if ghost else BoardMaterials.pion(player)
	_body.position.y = Look.PION_HEIGHT / 2.0
	if BoardMaterials.is_octagon(player):
		_body.rotation.y = Look.PION_OCTAGON_TURN
	_pivot.add_child(_body)
	if ghost:
		return
	var shadow := MeshInstance3D.new()
	shadow.mesh = _shadow()
	shadow.material_override = _shadow_material
	shadow.position.y = Look.FLAT_LIFT
	_pivot.add_child(shadow)


## Montre ce que les effets donnent à ce pion : son anneau, sa taille, sa couronne.
func show_status(status: PionStatus) -> void:
	_show_ring(status.ring)
	var scale_target := Vector3.ONE * (Look.CHILD_SCALE if status.child else 1.0)
	if not scale_target.is_equal_approx(_scale_target):
		_scale_target = scale_target
		if _resize != null:
			_resize.kill()
		_resize = create_tween().set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
		_resize.tween_property(_pivot, "scale", scale_target, Look.STATUS_CHANGE_TIME)
	if status.crown and _crown == null:
		_crown = Crown3D.new()
		_crown.position.y = Look.PION_HEIGHT / 2.0
		# Le palet octogonal est tourné : la couronne ne l'est pas, sa pierre reste devant.
		_crown.rotation.y = -_body.rotation.y
		_body.add_child(_crown)
	elif not status.crown and _crown != null:
		_crown.queue_free()
		_crown = null


## Le point au-dessus du pion où se posent ses badges : le haut du palet (ou de sa
## couronne), sur son bord droit, tel qu'il est maintenant (il peut glisser, tomber,
## rapetisser).
func badge_anchor() -> Vector3:
	var above := Look.PION_HEIGHT / 2.0 + (Crown3D.height() if _crown != null else 0.0)
	var point := _body.position + Vector3(Look.PION_RADIUS * Look.BADGE_ANCHOR_SIDE, above * _body.scale.y, 0.0)
	return _pivot.global_transform * point


func _show_ring(kind: String) -> void:
	if kind == _ring_kind:
		return
	_ring_kind = kind
	if kind == "":
		_ring.visible = false
		return
	if _ring == null:
		_ring = MeshInstance3D.new()
		var plane := PlaneMesh.new()
		plane.size = Vector2.ONE * GridGeometry.CELL_SIZE
		_ring.mesh = plane
		_ring.position.y = Look.FLAT_LIFT
		_pivot.add_child(_ring)
	_ring.material_override = BoardMaterials.status_ring(kind)
	_ring.visible = true


## Va jusqu'à ce point : glisse, ou saute en arc de cette hauteur (`arc`), après ce délai
## (`delay` : le déroulé de la vue, `BoardTimeline`).
func move_to(target: Vector3, delay: float = 0.0, arc: float = 0.0) -> void:
	if target.is_equal_approx(position):
		return
	var move := _restart_move()
	move.tween_interval(delay)
	_travel(move, position, target, arc)


## La pose : après ce délai, le palet tombe de `height` au-dessus de sa case, s'écrase un
## peu en touchant le plateau, puis reprend sa forme. Il est caché tant qu'il attend.
func drop_in(height: float, delay: float = 0.0) -> void:
	var ground := position
	position = ground + Vector3.UP * height
	var move := _restart_move()
	if delay > 0.0:
		visible = false
		move.tween_interval(delay)
		move.tween_callback(show)
	move.tween_property(self, "position", ground, Look.PION_DROP_TIME).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)
	_land(move)


## Un pion tué au point `at`, où il a pu sauter juste avant (`jump` : `{"delay", "arc"}` de
## son saut dans le déroulé) : après ce délai, il gonfle un instant, puis se ratatine en
## s'élevant, et disparaît.
func vanish(delay: float, at: Vector3, jump: Dictionary = {}) -> void:
	if _resize != null:
		_resize.kill()
	var move := _restart_move()
	var elapsed := 0.0
	if not at.is_equal_approx(position):
		var jump_delay: float = jump.get("delay", 0.0)
		move.tween_interval(jump_delay)
		elapsed = jump_delay + _travel(move, position, at, jump.get("arc", 0.0))
	move.tween_interval(maxf(delay - elapsed, 0.0))
	var pop := Look.KILL_TIME * Look.KILL_POP_SHARE
	var shrink := Look.KILL_TIME - pop
	move.tween_property(_pivot, "scale", _pivot.scale * Look.KILL_POP, pop)
	move.tween_property(_pivot, "scale", Vector3.ONE * Look.TINY_SCALE, shrink).set_ease(Tween.EASE_IN)
	move.parallel().tween_property(self, "position", at + Vector3.UP * Look.KILL_RISE, shrink)
	move.tween_callback(queue_free)


## Ajoute à ce mouvement le trajet de `from` à `target` : une glissade, ou un saut en arc
## suivi de l'arrivée au sol. Rend sa durée.
func _travel(move: Tween, from: Vector3, target: Vector3, arc: float) -> float:
	if is_zero_approx(arc):
		move.tween_property(self, "position", target, Look.PION_MOVE)
		return Look.PION_MOVE
	var hop := func(progress: float) -> void:
		position = BoardTimeline.along_arc(from, target, arc, progress)
	move.tween_method(hop, 0.0, 1.0, Look.JUMP_TIME).set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
	_land(move)
	return Look.JUMP_TIME + Look.PION_SQUASH_TIME


## L'arrivée au sol : le palet s'écrase un peu, puis reprend sa forme.
func _land(move: Tween) -> void:
	move.tween_property(_body, "scale", Vector3(1.0, Look.PION_SQUASH, 1.0), 0.0)
	move.tween_property(_body, "scale", Vector3.ONE, Look.PION_SQUASH_TIME).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)


## Un bref tremblement sur place : rien n'est permis ici. Un pion qui tombe ou glisse
## encore ne tremble pas : l'interrompre le laisserait en l'air ou à mi-chemin.
func wiggle() -> void:
	if _move != null and _move.is_running():
		return
	var move := _restart_move()
	# Chaque aller-retour fait deux pas, et le retour au centre un dernier.
	var swing := Look.WIGGLE_TIME / (Look.WIGGLE_SWINGS * 2.0 + 1.0)
	for index in Look.WIGGLE_SWINGS:
		move.tween_property(_body, "position:x", Look.WIGGLE_DISTANCE, swing)
		move.tween_property(_body, "position:x", -Look.WIGGLE_DISTANCE, swing)
	move.tween_property(_body, "position:x", 0.0, swing)


func _restart_move() -> Tween:
	if _move != null:
		_move.kill()
	_body.scale = Vector3.ONE
	_body.position.x = 0.0
	_move = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	return _move


static func _body_mesh(player: String) -> CylinderMesh:
	if _meshes.has(player):
		return _meshes[player]
	var mesh := CylinderMesh.new()
	var radius := BoardMaterials.pion_radius(player)
	mesh.top_radius = radius
	mesh.bottom_radius = radius
	mesh.height = Look.PION_HEIGHT
	mesh.radial_segments = OCTAGON_SEGMENTS if BoardMaterials.is_octagon(player) else DISC_SEGMENTS
	mesh.rings = 1
	_meshes[player] = mesh
	return mesh


static func _shadow() -> PlaneMesh:
	if _shadow_mesh == null:
		_shadow_mesh = PlaneMesh.new()
		_shadow_mesh.size = Vector2.ONE * Look.PION_SHADOW_SIZE
		_shadow_material = BoardMaterials.shadow(Color(Look.INK, Look.PION_SHADOW_ALPHA))
	return _shadow_mesh
