class_name Board3D
extends Node3D
## Le plateau en 3D, filmé de face et en plongée par une caméra orthographique. Il dessine la vue
## que lui donne l'écran de partie, anime ce que celui-ci lui demande, et annonce la case
## survolée ou touchée ; il ne sait pas ce qui est permis : l'écran de partie le lui dit
## (`show_mark`, `show_ghost`, `flash_refusal`).
##
## Ce qui se passe s'anime d'après les événements joints à la vue, dans leur ordre
## (`BoardTimeline`) : l'éclat de l'attaque lancée, le tir de la reine, les pions tués, les
## tombes, les sauts et les échanges, le mur creusé, les poses. L'état final est toujours
## celui de la vue.
##
## Les effets continus se voient sur les pions (`PionStatus`) : anneau, taille d'enfant,
## couronne en 3D ; leurs badges, dans un calque 2D par-dessus (`BadgeLayer`).
##
## Du mur, on ne voit que les cases qui touchent le plateau (la première rangée, et les
## voisines des cases creusées). La caméra cadre les cases jouables, plus une demi-case,
## dans le rectangle que l'écran lui laisse (`frame`) : sur un téléphone, c'est la largeur
## qui manque, et le mur peut déborder sur les côtés plutôt que de rétrécir les cases.

## Une case touchée ou cliquée (un toucher qui n'a presque pas bougé).
signal cell_clicked(cell: Vector2i)
## La case sous la souris a changé (`ViewData.NO_CELL` hors du plateau).
signal cell_hovered(cell: Vector2i)
## Un toucher qui ne désigne aucune case (hors du plateau, ou loin des cases proposées).
signal empty_tapped

enum Tile { CASE, CASE_ALT, WALL, DUG }
## Les marques posées sur des cases, par sorte : le dernier coup adverse, les cases
## proposées pendant le choix d'une carte, les cases déjà choisies.
enum Marks { LAST, CANDIDATE, CHOSEN }

## Les huit voisines d'une case.
const NEIGHBORS: Array[Vector2i] = [
	Vector2i(-1, -1), Vector2i(0, -1), Vector2i(1, -1), Vector2i(-1, 0),
	Vector2i(1, 0), Vector2i(-1, 1), Vector2i(0, 1), Vector2i(1, 1),
]

var _geometry: GridGeometry
var _rig: Node3D
var _camera: Camera3D
var _reframe: Tween
var _layers: Dictionary[Tile, MultiMeshInstance3D] = {}
## Les cases dessinées, et ce qu'on y dessine.
var _drawn: Dictionary[Vector2i, Tile] = {}
var _bounds := AABB()
var _frame_rect := Rect2()
## Les pièces posées, par identité de pion (un pion vivant, ou le mort d'une tombe).
var _pions: Dictionary[int, Node3D] = {}
var _tombs: Dictionary[int, Node3D] = {}
## La case de chaque pion vivant, d'après la dernière vue.
var _pion_cells: Dictionary[int, Vector2i] = {}
var _pieces: Node3D
var _badges: BadgeLayer
## Les statuts de la vue en cours, montrés quand son déroulé a fini.
var _statuses_later: Tween
var _mark: MeshInstance3D
var _refusal: MeshInstance3D
var _refusal_fade: Tween
## Les marques de chaque sorte (réutilisées d'une vue à l'autre), et leur matériau.
var _marks: Dictionary[Marks, Array] = {}
var _mark_materials: Dictionary[Marks, ShaderMaterial] = {}
## Les pions du dernier coup adverse : leur marque les suit s'ils bougent.
var _last_ids: Array[int] = []
## Les cases vers lesquelles un toucher s'aimante (les cases proposées d'un choix).
var _magnet: Array[Vector2i] = []
var _ghost: Pion3D
var _ghost_player := ""
var _ghost_cell := ViewData.NO_CELL
var _ghost_lifted := false
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
	_badges = BadgeLayer.new()
	_badges.visible = visible
	add_child(_badges)
	# Un calque 2D ne suit pas la visibilité d'un nœud 3D : on la lui recopie.
	visibility_changed.connect(func() -> void: _badges.visible = visible)
	_mark = _flat_mark(Look.MARK_SIZE, BoardMaterials.mark(Look.MARK_FILL, Look.MARK_BORDER))
	_refusal = _flat_mark(Look.MARK_SIZE, BoardMaterials.mark(Look.REFUSAL_FILL, Look.REFUSAL_BORDER))
	_mark_materials = {
		Marks.LAST: BoardMaterials.dashed_mark(Look.LAST_MARK, Look.LAST_MARK_DASHES),
		Marks.CANDIDATE: BoardMaterials.dotted_mark(Look.CANDIDATE_FILL, Look.CANDIDATE_DOT, Look.CANDIDATE_DOT_SIZE),
		Marks.CHOSEN: BoardMaterials.mark(Look.CHOSEN_FILL, Look.CHOSEN_BORDER),
	}
	for kind: Marks in Marks.values():
		_marks[kind] = []
	get_viewport().size_changed.connect(_update_camera.bind(false))


## Dessine cette vue (cases, mur visible, pions et tombes) en animant ce que racontent ses
## événements. Rend la durée de l'animation, en secondes (0 sans rien à animer).
func show_view(view: ViewData, events: Array = []) -> float:
	if _geometry == null or _geometry.size != view.grid_size():
		_geometry = GridGeometry.new(view.grid_size())
	var timeline := BoardTimeline.of(events)
	var framed := _bounds
	_update_tiles(view)
	_update_pieces(view, timeline)
	for pulse in timeline.pulses:
		_pulse(pulse["cell"], pulse["color"], pulse["delay"])
	for shot in timeline.shots:
		_shoot(shot["from"], shot["to"], shot["delay"])
	# Un palet en attente que cette vue n'a pas posé (le coup a abouti ailleurs) disparaît.
	if _ghost_lifted:
		show_ghost(ViewData.NO_CELL, "", false)
	_update_last_marks()
	_update_camera(framed.has_volume() and framed != _bounds)
	return timeline.length


## Cadre le plateau dans ce rectangle de l'écran (en coordonnées de la fenêtre).
func frame(rect: Rect2) -> void:
	_frame_rect = rect
	_update_camera(false)


## Montre la marque sur cette case, ou la cache (`ViewData.NO_CELL`).
func show_mark(cell: Vector2i) -> void:
	_place_flat(_mark, cell)


## Montre le palet fantôme de ce joueur sur cette case : posé (survol) ou flottant au-dessus
## (`lifted` : un coup envoyé, en attente de la réponse). `ViewData.NO_CELL` le cache.
func show_ghost(cell: Vector2i, player: String, lifted: bool) -> void:
	if not _drawn.has(cell):
		if _ghost != null:
			_ghost.visible = false
		_ghost_cell = ViewData.NO_CELL
		_ghost_lifted = false
		return
	if _ghost == null or player != _ghost_player:
		if _ghost != null:
			_ghost.queue_free()
		_ghost = Pion3D.new()
		_ghost.setup({"owner": player}, true)
		_pieces.add_child(_ghost)
		_ghost_player = player
	_ghost.visible = true
	_ghost_cell = cell
	_ghost_lifted = lifted
	_ghost.position = _ground(cell) + Vector3.UP * (Look.PION_PENDING_LIFT if lifted else 0.0)


## Rien n'est permis sur cette case touchée : son pion tremble, ou, sur une case vide, un
## bref voile terne la couvre.
func flash_refusal(cell: Vector2i) -> void:
	for id: int in _pion_cells:
		if _pion_cells[id] == cell and _pions.has(id):
			_pions[id].wiggle()
			return
	if not _place_flat(_refusal, cell):
		return
	if _refusal_fade != null:
		_refusal_fade.kill()
	_refusal.transparency = 0.0
	_refusal_fade = create_tween()
	_refusal_fade.tween_property(_refusal, "transparency", 1.0, Look.REFUSAL_TIME)
	_refusal_fade.tween_callback(func() -> void: _refusal.visible = false)


## Entoure de pointillés ces pions (par identité) : le dernier coup adverse.
func show_last(ids: Array[int]) -> void:
	_last_ids = ids
	_update_last_marks()


## Le point de l'écran où se trouve le centre de cette case.
func screen_point(cell: Vector2i) -> Vector2:
	return _camera.unproject_position(_ground(cell))


## Marque ces cases proposées (le choix d'une carte) ; un toucher proche s'y aimante.
func show_candidates(cells: Array[Vector2i]) -> void:
	_magnet = cells
	_show_marks(Marks.CANDIDATE, cells)


## Marque ces cases déjà choisies (le lanceur et les cibles d'une carte en cours).
func show_chosen(cells: Array[Vector2i]) -> void:
	_show_marks(Marks.CHOSEN, cells)


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


func _update_pieces(view: ViewData, timeline: BoardTimeline) -> void:
	var seen_tombs := {}
	var statuses: Dictionary[int, PionStatus] = {}
	_pion_cells.clear()
	for cell: Vector2i in _drawn:
		var pion: Variant = view.pion_at(cell)
		if pion is Dictionary:
			var id: int = pion.get("id", -1)
			_pion_cells[id] = cell
			_place_pion(pion, cell, timeline)
			statuses[id] = PionStatus.of(view.effects_on(id))
		var tomb: Variant = view.tomb_at(cell)
		if tomb is Dictionary:
			var dead: int = tomb.get("id", -1)
			seen_tombs[dead] = true
			if not _tombs.has(dead):
				var stone := Tomb3D.new()
				stone.setup(tomb)
				_pieces.add_child(stone)
				_tombs[dead] = stone
				if timeline.tombs.has(dead):
					stone.rise_in(timeline.tombs[dead])
			_tombs[dead].position = _ground(cell)
	# Un pion tué part en animation, puis se libère tout seul ; les autres absents, aussitôt.
	for id: int in _pions.keys():
		if not _pion_cells.has(id) and timeline.kills.has(id):
			var kill: Dictionary = timeline.kills[id]
			var cell: Vector2i = kill["cell"]
			var at := _ground(cell) if _drawn.has(cell) else _pions[id].position
			(_pions[id] as Pion3D).vanish(kill["delay"], at, timeline.moves.get(id, {}))
			_pions.erase(id)
	_remove_missing(_pions, _pion_cells)
	_remove_missing(_tombs, seen_tombs)
	# Les statuts changent une fois le déroulé fini : l'anneau d'une cible ne tombe pas
	# avant qu'on ait vu l'attaque.
	if _statuses_later != null:
		_statuses_later.kill()
	if timeline.length > 0.0:
		_statuses_later = create_tween()
		_statuses_later.tween_interval(timeline.length)
		_statuses_later.tween_callback(_show_statuses.bind(statuses))
	else:
		_show_statuses(statuses)


func _show_statuses(statuses: Dictionary[int, PionStatus]) -> void:
	for id: int in statuses:
		if _pions.has(id):
			(_pions[id] as Pion3D).show_status(statuses[id])
	_badges.show_statuses(statuses)


func _process(_delta: float) -> void:
	if visible and _camera != null and _camera.size > 0.0:
		# La caméra est orthographique et garde sa hauteur : `size` est la hauteur filmée.
		var cell_pixels := get_viewport().get_visible_rect().size.y / _camera.size * GridGeometry.CELL_SIZE
		_badges.follow(_camera, _pions, cell_pixels)


## Crée la pièce de ce pion si elle n'existe pas, sinon l'amène à sa case (en sautant, si
## le déroulé le dit). Un pion qui vient d'être posé tombe : du palet fantôme qui
## l'attendait, s'il est là.
func _place_pion(pion: Dictionary, cell: Vector2i, timeline: BoardTimeline) -> void:
	var id: int = pion.get("id", -1)
	if _pions.has(id):
		var jump: Dictionary = timeline.moves.get(id, {})
		_pions[id].move_to(_ground(cell), jump.get("delay", 0.0), jump.get("arc", 0.0))
		return
	var piece := Pion3D.new()
	piece.setup(pion)
	piece.position = _ground(cell)
	_pieces.add_child(piece)
	_pions[id] = piece
	if timeline.drops.has(id):
		var from_ghost := _ghost_cell == cell and _ghost_lifted
		piece.drop_in(Look.PION_PENDING_LIFT if from_ghost else Look.PION_DROP_HEIGHT, timeline.drops[id])
		if from_ghost:
			show_ghost(ViewData.NO_CELL, "", false)


func _remove_missing(pieces: Dictionary[int, Node3D], seen: Dictionary) -> void:
	for id: int in pieces.keys():
		if not seen.has(id):
			pieces[id].queue_free()
			pieces.erase(id)


func _update_last_marks() -> void:
	var cells: Array[Vector2i] = []
	for id in _last_ids:
		if _pion_cells.has(id):
			cells.append(_pion_cells[id])
	_show_marks(Marks.LAST, cells)


## Pose une marque de cette sorte sur chacune de ces cases, et cache les autres.
func _show_marks(kind: Marks, cells: Array[Vector2i]) -> void:
	var pool: Array = _marks[kind]
	var size := Look.LAST_MARK_SIZE if kind == Marks.LAST else Look.MARK_SIZE
	while pool.size() < cells.size():
		pool.append(_flat_mark(size, _mark_materials[kind]))
	for index in pool.size():
		_place_flat(pool[index], cells[index] if index < cells.size() else ViewData.NO_CELL)


## Un éclat à plat sur cette case, après ce délai : un anneau de cette couleur qui
## s'élargit en s'effaçant (attaque lancée, pion tué, mur creusé).
func _pulse(cell: Vector2i, color: Color, delay: float) -> void:
	if not _drawn.has(cell):
		return
	var ring := _flat_mark(GridGeometry.CELL_SIZE, BoardMaterials.pulse(color))
	_place_flat(ring, cell)
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
func _shoot(from: Vector2i, to: Vector2i, delay: float) -> void:
	if not (_drawn.has(from) and _drawn.has(to)):
		return
	var ball := MeshInstance3D.new()
	ball.mesh = BoardMaterials.shot_mesh()
	ball.visible = false
	add_child(ball)
	var start := _ground(from) + Vector3.UP * Look.PION_HEIGHT
	var end := _ground(to) + Vector3.UP * Look.PION_HEIGHT
	var fly := func(progress: float) -> void:
		ball.position = BoardTimeline.along_arc(start, end, Look.SHOT_ARC, progress)
	fly.call(0.0)
	var flight := create_tween()
	flight.tween_interval(delay)
	flight.tween_callback(ball.show)
	flight.tween_method(fly, 0.0, 1.0, Look.SHOT_TIME)
	flight.tween_callback(ball.queue_free)


## Le sol d'une case : le dessus de la case, là où se posent les pièces.
func _ground(cell: Vector2i) -> Vector3:
	return _geometry.center(cell) + Vector3.UP * Look.CASE_HEIGHT


## Une marque à plat, cachée, de cette taille et ce matériau.
func _flat_mark(size: float, material: ShaderMaterial) -> MeshInstance3D:
	var mark := MeshInstance3D.new()
	var plane := PlaneMesh.new()
	plane.size = Vector2.ONE * size
	mark.mesh = plane
	mark.material_override = material
	mark.visible = false
	add_child(mark)
	return mark


## Pose cette marque sur une case dessinée, ou la cache ; vrai si elle est montrée.
func _place_flat(mark: MeshInstance3D, cell: Vector2i) -> bool:
	mark.visible = _drawn.has(cell)
	if mark.visible:
		mark.position = _ground(cell) + Vector3.UP * Look.FLAT_LIFT
	return mark.visible


## Place la caméra pour que les cases cadrées tiennent dans le rectangle de cadrage. Parmi
## les plongées permises, elle prend celle qui donne les plus grandes cases à l'écran : la
## surface d'une case y vaut le sinus de la plongée sur le carré de la hauteur filmée.
## Quand la largeur manque et que la hauteur abonde (téléphone en portrait), une plongée
## plus forte grandit les cases en hauteur. `animated` : la caméra glisse vers son cadrage
## (le plateau a changé), sinon elle y saute (la fenêtre a changé).
func _update_camera(animated: bool) -> void:
	if _camera == null or not _bounds.has_volume():
		return
	var viewport := get_viewport().get_visible_rect().size
	var rect := _frame_rect if _frame_rect.has_area() else Rect2(Vector2.ZERO, viewport)
	var start_rotation := _rig.rotation_degrees
	var start_position := _rig.position
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
	var target_size := best_height * Look.FRAME_MARGIN
	var per_point := target_size / viewport.y
	var shift := rect.get_center() - viewport / 2.0
	var middle := best.get_center()
	var target_rotation := Vector3(best_pitch, Look.CAMERA_YAW_DEGREES, 0.0)
	var target_offset := Vector2(middle.x - shift.x * per_point, middle.y + shift.y * per_point)

	if _reframe != null:
		_reframe.kill()
	var target_position := _rig.position
	if not animated:
		_rig.rotation_degrees = target_rotation
		_camera.size = target_size
		_camera.h_offset = target_offset.x
		_camera.v_offset = target_offset.y
		return
	_rig.rotation_degrees = start_rotation
	_rig.position = start_position
	_reframe = create_tween().set_parallel().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_IN_OUT)
	_reframe.tween_property(_rig, "position", target_position, Look.REFRAME_TIME)
	_reframe.tween_property(_rig, "rotation_degrees", target_rotation, Look.REFRAME_TIME)
	_reframe.tween_property(_camera, "size", target_size, Look.REFRAME_TIME)
	_reframe.tween_property(_camera, "h_offset", target_offset.x, Look.REFRAME_TIME)
	_reframe.tween_property(_camera, "v_offset", target_offset.y, Look.REFRAME_TIME)


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
				else:
					empty_tapped.emit()


## La case dessinée sous ce point de l'écran, lue sur le plan du dessus des cases. Pendant
## un choix, la case proposée la plus proche, si elle est assez près : viser juste une
## petite case au doigt est difficile, et seules ces cases comptent alors.
func _pick(screen_position: Vector2) -> Vector2i:
	var origin := _camera.project_ray_origin(screen_position)
	var direction := _camera.project_ray_normal(screen_position)
	if is_zero_approx(direction.y):
		return ViewData.NO_CELL
	var hit := origin + direction * ((Look.CASE_HEIGHT - origin.y) / direction.y)
	if not _magnet.is_empty():
		return _nearest_candidate(hit)
	var cell := _geometry.cell_at(hit)
	return cell if _drawn.has(cell) else ViewData.NO_CELL


func _nearest_candidate(hit: Vector3) -> Vector2i:
	var best := ViewData.NO_CELL
	var best_distance := Look.MAGNET_RADIUS * GridGeometry.CELL_SIZE
	for cell in _magnet:
		var offset := _ground(cell) - hit
		var distance := Vector2(offset.x, offset.z).length()
		if distance <= best_distance:
			best = cell
			best_distance = distance
	return best


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
