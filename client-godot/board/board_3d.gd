class_name Board3D
extends Node3D
## Le plateau en 3D, filmé en isométrique par une caméra orthographique. Il dessine la vue
## que lui donne l'écran de partie et annonce la case survolée ou touchée ; il ne sait pas
## ce qui est permis : l'écran de partie le lui dit (`show_mark`).
##
## Du mur, on ne voit que les cases qui touchent le plateau (la première rangée, et les
## voisines des cases creusées). La caméra cadre les cases jouables, plus une demi-case,
## dans le rectangle que l'écran lui laisse (`frame`) : sur un téléphone, c'est la largeur
## qui manque, et le mur peut déborder aux pointes du losange plutôt que de rétrécir les
## cases.

## Une case touchée ou cliquée (un toucher qui n'a presque pas bougé).
signal cell_clicked(cell: Vector2i)
## La case sous la souris a changé (`ViewData.NO_CELL` hors du plateau).
signal cell_hovered(cell: Vector2i)

enum Tile { CASE, CASE_ALT, WALL, DUG }

## Les huit voisines d'une case.
const NEIGHBORS: Array[Vector2i] = [
	Vector2i(-1, -1), Vector2i(0, -1), Vector2i(1, -1), Vector2i(-1, 0),
	Vector2i(1, 0), Vector2i(-1, 1), Vector2i(0, 1), Vector2i(1, 1),
]

var _geometry: GridGeometry
var _rig: Node3D
var _camera: Camera3D
var _layers: Dictionary[Tile, MultiMeshInstance3D] = {}
## Les cases dessinées, et ce qu'on y dessine.
var _drawn: Dictionary[Vector2i, Tile] = {}
var _bounds := AABB()
var _frame_rect := Rect2()
## Les pièces posées, par identité de pion (un pion vivant, ou le mort d'une tombe).
var _pions: Dictionary[int, Node3D] = {}
var _tombs: Dictionary[int, Node3D] = {}
var _pieces: Node3D
var _mark: MeshInstance3D
var _hovered := ViewData.NO_CELL
var _press_position := Vector2.ZERO
var _pressing := false


func _ready() -> void:
	_rig = Node3D.new()
	add_child(_rig)
	_camera = Camera3D.new()
	_camera.projection = Camera3D.PROJECTION_ORTHOGONAL
	_camera.position = Vector3(0.0, 0.0, Look.CAMERA_DISTANCE)
	_camera.far = Look.CAMERA_DISTANCE * 2.0
	_rig.add_child(_camera)

	for tile: Tile in Tile.values():
		var layer := MultiMeshInstance3D.new()
		layer.multimesh = MultiMesh.new()
		layer.multimesh.transform_format = MultiMesh.TRANSFORM_3D
		layer.multimesh.mesh = _tile_mesh(tile)
		add_child(layer)
		_layers[tile] = layer

	_pieces = Node3D.new()
	add_child(_pieces)

	_mark = MeshInstance3D.new()
	var plane := PlaneMesh.new()
	plane.size = Vector2.ONE * Look.MARK_SIZE
	_mark.mesh = plane
	_mark.material_override = BoardMaterials.flat(Look.MARK_FILL, Look.MARK_BORDER, false)
	_mark.visible = false
	add_child(_mark)
	get_viewport().size_changed.connect(_update_camera)


## Dessine cette vue : cases, mur visible, pions et tombes.
func show_view(view: ViewData) -> void:
	if _geometry == null or _geometry.size != view.grid_size():
		_geometry = GridGeometry.new(view.grid_size())
	_update_tiles(view)
	_update_pieces(view)
	_update_camera()


## Cadre le plateau dans ce rectangle de l'écran (en coordonnées de la fenêtre).
func frame(rect: Rect2) -> void:
	_frame_rect = rect
	_update_camera()


## Montre la marque sur cette case, ou la cache (`ViewData.NO_CELL`).
func show_mark(cell: Vector2i) -> void:
	_mark.visible = _drawn.has(cell)
	if _mark.visible:
		_mark.position = _geometry.center(cell) + Vector3.UP * (Look.CASE_HEIGHT + Look.FLAT_LIFT)


func _update_tiles(view: ViewData) -> void:
	var by_tile := {}
	for tile: Tile in Tile.values():
		by_tile[tile] = []
	_drawn.clear()
	var size := view.grid_size()
	for row in size:
		for col in size:
			var cell := Vector2i(col, row)
			var tile := _tile_of(view, cell)
			if tile != -1:
				by_tile[tile].append(cell)
				_drawn[cell] = tile as Tile

	_bounds = AABB()
	var first := true
	for tile: Tile in Tile.values():
		var cells: Array = by_tile[tile]
		var multimesh: MultiMesh = _layers[tile].multimesh
		var height := _tile_height(tile)
		multimesh.instance_count = cells.size()
		for index in cells.size():
			var center := _geometry.center(cells[index])
			multimesh.set_instance_transform(index, Transform3D(Basis.IDENTITY, center + Vector3.UP * height / 2.0))
			if tile == Tile.WALL:
				continue
			var box := _geometry.box(cells[index], Look.WALL_HEIGHT, Look.FRAME_PADDING)
			_bounds = box if first else _bounds.merge(box)
			first = false


## Ce qu'il faut dessiner sur cette case, ou -1 pour un mur loin du plateau. Une case de la
## bande du mur qui n'est pas du mur a été creusée (glossaire, « Mur »).
func _tile_of(view: ViewData, cell: Vector2i) -> int:
	if not view.is_wall(cell):
		if view.is_in_wall_band(cell):
			return Tile.DUG
		return Tile.CASE if (cell.x + cell.y) % 2 == 0 else Tile.CASE_ALT
	for step in NEIGHBORS:
		var next := cell + step
		if _geometry.contains(next) and not view.is_wall(next):
			return Tile.WALL
	return -1


func _update_pieces(view: ViewData) -> void:
	var seen_pions := {}
	var seen_tombs := {}
	for cell: Vector2i in _drawn:
		var ground := _geometry.center(cell) + Vector3.UP * Look.CASE_HEIGHT
		var pion: Variant = view.pion_at(cell)
		if pion is Dictionary:
			seen_pions[pion.get("id", -1)] = true
			_place(_pions, pion, ground, Pion3D)
		var tomb: Variant = view.tomb_at(cell)
		if tomb is Dictionary:
			seen_tombs[tomb.get("id", -1)] = true
			_place(_tombs, tomb, ground, Tomb3D)
	_remove_missing(_pions, seen_pions)
	_remove_missing(_tombs, seen_tombs)


## Crée la pièce de ce pion si elle n'existe pas, sinon l'amène à sa case (`move_to` :
## un pion y glisse, une tombe y est posée).
func _place(pieces: Dictionary[int, Node3D], pion: Dictionary, ground: Vector3, kind: GDScript) -> void:
	var id: int = pion.get("id", -1)
	if pieces.has(id):
		pieces[id].move_to(ground)
		return
	var piece: Node3D = kind.new()
	piece.setup(pion)
	piece.position = ground
	_pieces.add_child(piece)
	pieces[id] = piece


func _remove_missing(pieces: Dictionary[int, Node3D], seen: Dictionary) -> void:
	for id: int in pieces.keys():
		if not seen.has(id):
			pieces[id].queue_free()
			pieces.erase(id)


## Place la caméra pour que les cases cadrées tiennent dans le rectangle de cadrage. Parmi
## les plongées permises, elle prend celle qui donne les plus grandes cases à l'écran : la
## surface d'une case y vaut le sinus de la plongée sur le carré de la hauteur filmée.
## Quand la largeur manque et que la hauteur abonde (téléphone en portrait), une plongée
## plus forte grandit les cases en hauteur.
func _update_camera() -> void:
	if _camera == null or _bounds.size == Vector3.ZERO:
		return
	var viewport := get_viewport().get_visible_rect().size
	var rect := _frame_rect if _frame_rect.has_area() else Rect2(Vector2.ZERO, viewport)
	_rig.position = _bounds.get_center()
	var best_pitch: float = Look.CAMERA_PITCHES_DEGREES[0]
	var best := Rect2()
	var best_height := INF
	var best_area := 0.0
	for pitch: float in Look.CAMERA_PITCHES_DEGREES:
		_rig.rotation_degrees = Vector3(pitch, Look.CAMERA_YAW_DEGREES, 0.0)
		var extent := _projected_bounds()
		var world_height := maxf(extent.size.y * viewport.y / rect.size.y, extent.size.x * viewport.y / rect.size.x)
		var cell_area := absf(sin(deg_to_rad(pitch))) / (world_height * world_height)
		if cell_area > best_area:
			best_area = cell_area
			best_height = world_height
			best_pitch = pitch
			best = extent
	_rig.rotation_degrees = Vector3(best_pitch, Look.CAMERA_YAW_DEGREES, 0.0)
	_camera.size = best_height * Look.FRAME_MARGIN
	var per_point := _camera.size / viewport.y
	var shift := rect.get_center() - viewport / 2.0
	var middle := best.get_center()
	_camera.h_offset = middle.x - shift.x * per_point
	_camera.v_offset = middle.y + shift.y * per_point


## L'emprise à l'écran des cases cadrées, dans le plan de la caméra telle qu'elle est
## tournée (en unités du monde, autour du centre du cadrage).
func _projected_bounds() -> Rect2:
	var right := _rig.global_basis.x
	var up := _rig.global_basis.y
	var low := Vector2(INF, INF)
	var high := Vector2(-INF, -INF)
	for index in 8:
		var offset := _bounds.get_endpoint(index) - _rig.position
		var projected := Vector2(offset.dot(right), offset.dot(up))
		low = low.min(projected)
		high = high.max(projected)
	return Rect2(low, high - low)


func _unhandled_input(event: InputEvent) -> void:
	if _geometry == null or not visible:
		return
	if event is InputEventMouseMotion:
		var cell := _pick(event.position)
		if cell != _hovered:
			_hovered = cell
			cell_hovered.emit(cell)
	elif event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			_pressing = true
			_press_position = event.position
		elif _pressing:
			_pressing = false
			if event.position.distance_to(_press_position) <= Look.TAP_SLOP:
				var cell := _pick(event.position)
				if cell != ViewData.NO_CELL:
					cell_clicked.emit(cell)


## La case dessinée sous ce point de l'écran, lue sur le plan du dessus des cases.
func _pick(screen_position: Vector2) -> Vector2i:
	var origin := _camera.project_ray_origin(screen_position)
	var direction := _camera.project_ray_normal(screen_position)
	if is_zero_approx(direction.y):
		return ViewData.NO_CELL
	var hit := origin + direction * ((Look.CASE_HEIGHT - origin.y) / direction.y)
	var cell := _geometry.cell_at(hit)
	return cell if _drawn.has(cell) else ViewData.NO_CELL


static func _tile_height(tile: Tile) -> float:
	return Look.WALL_HEIGHT if tile == Tile.WALL else Look.CASE_HEIGHT


static func _tile_mesh(tile: Tile) -> BoxMesh:
	var size := Vector3(GridGeometry.CELL_SIZE, _tile_height(tile), GridGeometry.CELL_SIZE)
	var mesh := BoxMesh.new()
	mesh.size = size
	match tile:
		Tile.CASE:
			mesh.material = BoardMaterials.prism(size, Look.CASE_TOP, Look.CASE_SIDE_A, Look.CASE_SIDE_B, Look.CASE_EDGE)
		Tile.CASE_ALT:
			mesh.material = BoardMaterials.prism(size, Look.CASE_ALT_TOP, Look.CASE_SIDE_A, Look.CASE_SIDE_B, Look.CASE_EDGE)
		Tile.WALL:
			mesh.material = BoardMaterials.prism(size, Look.WALL_TOP, Look.WALL_SIDE_A, Look.WALL_SIDE_B, Look.WALL_EDGE)
		Tile.DUG:
			mesh.material = BoardMaterials.prism(size, Look.DUG_TOP, Look.DUG_SIDE_A, Look.DUG_SIDE_B, Look.CASE_EDGE)
	return mesh
