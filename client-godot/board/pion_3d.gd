class_name Pion3D
extends Node3D
## Un pion sur le plateau : un cylindre pour J1, un prisme à 8 faces pour J2 (la même
## primitive), et son ombre. L'origine du nœud est au sol de sa case ; un modèle plus riche
## pourra remplacer le corps sans changer le reste.
##
## En fantôme, le même palet transparent et sans ombre montre où tombera un pion.

const DISC_SEGMENTS := 48
const OCTAGON_SEGMENTS := 8

static var _meshes := {}
static var _shadow_mesh: PlaneMesh
static var _shadow_material: ShaderMaterial

var _body: MeshInstance3D
var _move: Tween


func setup(pion: Dictionary, ghost: bool = false) -> void:
	var player: String = pion.get("owner", ViewData.PLAYER_ONE)
	_body = MeshInstance3D.new()
	_body.mesh = _body_mesh(player)
	_body.material_override = BoardMaterials.ghost(player) if ghost else BoardMaterials.pion(player)
	_body.position.y = Look.PION_HEIGHT / 2.0
	if BoardMaterials.is_octagon(player):
		_body.rotation.y = Look.PION_OCTAGON_TURN
	add_child(_body)
	if ghost:
		return
	var shadow := MeshInstance3D.new()
	shadow.mesh = _shadow()
	shadow.material_override = _shadow_material
	shadow.position.y = Look.FLAT_LIFT
	add_child(shadow)


## Glisse jusqu'à ce point.
func move_to(target: Vector3) -> void:
	if target.is_equal_approx(position):
		return
	_restart_move().tween_property(self, "position", target, Look.PION_MOVE)


## La pose : le palet tombe de `height` au-dessus de sa case, s'écrase un peu en touchant
## le plateau, puis reprend sa forme.
func drop_in(height: float) -> void:
	var ground := position
	position = ground + Vector3.UP * height
	var move := _restart_move()
	move.tween_property(self, "position", ground, Look.PION_DROP_TIME).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)
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
		_shadow_material = BoardMaterials.flat(Color(Look.INK, Look.PION_SHADOW_ALPHA), Color.TRANSPARENT, true)
	return _shadow_mesh
