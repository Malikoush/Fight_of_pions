class_name Pion3D
extends Node3D
## Un pion sur le plateau : un cylindre pour J1, un prisme à 8 faces pour J2 (la même
## primitive), et son ombre. L'origine du nœud est au sol de sa case ; un modèle plus riche
## pourra remplacer le corps sans changer le reste.

const DISC_SEGMENTS := 48
const OCTAGON_SEGMENTS := 8

static var _meshes := {}
static var _shadow_mesh: PlaneMesh
static var _shadow_material: ShaderMaterial

var _move: Tween


func setup(pion: Dictionary) -> void:
	var player: String = pion.get("owner", ViewData.PLAYER_ONE)
	var body := MeshInstance3D.new()
	body.mesh = _body_mesh(player)
	body.material_override = BoardMaterials.pion(player)
	body.position.y = Look.PION_HEIGHT / 2.0
	if BoardMaterials.is_octagon(player):
		body.rotation.y = Look.PION_OCTAGON_TURN
	add_child(body)

	var shadow := MeshInstance3D.new()
	shadow.mesh = _shadow()
	shadow.material_override = _shadow_material
	shadow.position.y = Look.FLAT_LIFT
	add_child(shadow)


## Glisse jusqu'à ce point.
func move_to(target: Vector3) -> void:
	if target.is_equal_approx(position):
		return
	if _move != null:
		_move.kill()
	_move = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	_move.tween_property(self, "position", target, Look.PION_MOVE)


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
