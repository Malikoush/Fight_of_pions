class_name Tomb3D
extends Node3D
## La tombe d'un pion à l'infirmerie : une stèle grise au fond de sa case, barrée de la
## couleur de son joueur. Elle reste visible quand un pion se pose sur la même case.

static var _stone_mesh: BoxMesh
static var _stone_material: ShaderMaterial
static var _band_meshes := {}


func setup(dead: Dictionary) -> void:
	var player: String = dead.get("owner", ViewData.PLAYER_ONE)
	_prepare()
	var stone := MeshInstance3D.new()
	stone.mesh = _stone_mesh
	stone.material_override = _stone_material
	stone.position = Vector3(Look.TOMB_OFFSET.x, Look.TOMB_SIZE.y / 2.0, Look.TOMB_OFFSET.y)
	add_child(stone)

	var band := MeshInstance3D.new()
	band.mesh = _band(player)
	band.material_override = band.mesh.material
	band.position = stone.position
	add_child(band)



static func _prepare() -> void:
	if _stone_mesh != null:
		return
	_stone_mesh = BoxMesh.new()
	_stone_mesh.size = Look.TOMB_SIZE
	_stone_material = BoardMaterials.prism(Look.TOMB_SIZE, Look.TOMB_TOP, Look.TOMB_SIDE_A, Look.TOMB_SIDE_B, Look.INK)


## La bande de couleur du joueur, un peu plus large que la stèle pour dépasser de ses faces.
static func _band(player: String) -> BoxMesh:
	if _band_meshes.has(player):
		return _band_meshes[player]
	var color := Look.J1_TOP if player == ViewData.PLAYER_ONE else Look.J2_MOTIF
	var size := Vector3(Look.TOMB_SIZE.x, Look.TOMB_BAND, Look.TOMB_SIZE.z) * Vector3(Look.TOMB_BAND_OVERHANG, 1.0, Look.TOMB_BAND_OVERHANG)
	var mesh := BoxMesh.new()
	mesh.size = size
	mesh.material = BoardMaterials.prism(size, color, color, color.darkened(Look.SIDE_DARKEN), Look.INK)
	_band_meshes[player] = mesh
	return mesh
