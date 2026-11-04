class_name Crown3D
extends Node3D
## La couronne d'une reine : une bande d'or à pointes, posée sur son palet, et une pierre
## rouge sur le devant. Son origine est à sa base. Une seule mesure, partagée par toutes
## les couronnes.

static var _band_mesh: ArrayMesh
static var _jewel_mesh: SphereMesh


func _init() -> void:
	_prepare()
	var band := MeshInstance3D.new()
	band.mesh = _band_mesh
	band.material_override = BoardMaterials.crown()
	add_child(band)
	var jewel := MeshInstance3D.new()
	jewel.mesh = _jewel_mesh
	jewel.material_override = BoardMaterials.solid(Look.CROWN_JEWEL)
	jewel.position = Vector3(0.0, Look.CROWN_BASE / 2.0, Look.CROWN_RADIUS)
	add_child(jewel)


## La hauteur totale de la couronne, pointes comprises.
static func height() -> float:
	return Look.CROWN_PEAK


static func _prepare() -> void:
	if _band_mesh != null:
		return
	_band_mesh = _band()
	_jewel_mesh = SphereMesh.new()
	_jewel_mesh.radius = Look.CROWN_JEWEL_RADIUS
	_jewel_mesh.height = Look.CROWN_JEWEL_RADIUS * 2.0


## La bande : un tour de quadrilatères verticaux, dont le haut alterne pointe et creux.
static func _band() -> ArrayMesh:
	var columns := Look.CROWN_POINTS * 2
	var tool := SurfaceTool.new()
	tool.begin(Mesh.PRIMITIVE_TRIANGLES)
	for index in columns:
		var left := _column(index, columns)
		var right := _column(index + 1, columns)
		# Deux triangles : bas gauche, haut gauche, haut droite ; bas gauche, haut droite, bas droite.
		for corner: Array in [[left, 0], [left, 1], [right, 1], [left, 0], [right, 1], [right, 0]]:
			var column: Dictionary = corner[0]
			var top: int = corner[1]
			tool.set_normal(column["normal"])
			tool.set_uv(Vector2(column["turn"], top))
			tool.set_uv2(Vector2(column["height"], 0.0))
			tool.add_vertex(column["normal"] * Look.CROWN_RADIUS + Vector3.UP * column["height"] * top)
	return tool.commit()


## Une colonne de la bande : sa direction depuis l'axe, sa hauteur (pointe ou creux), et
## sa place dans le tour.
static func _column(index: int, columns: int) -> Dictionary:
	var angle := TAU * index / columns
	return {
		"normal": Vector3(sin(angle), 0.0, cos(angle)),
		"height": Look.CROWN_PEAK if index % 2 == 0 else Look.CROWN_BASE,
		"turn": float(index) / columns,
	}
