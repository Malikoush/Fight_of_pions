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
## Deux morceaux l'aident : la caméra (`BoardCamera` : le cadrage, et le point de l'écran
## sous un toucher) et les marques (`BoardMarks` : survol, refus, marques par sorte, éclats
## et tirs du déroulé).
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

## Les huit voisines d'une case.
const NEIGHBORS: Array[Vector2i] = [
	Vector2i(-1, -1), Vector2i(0, -1), Vector2i(1, -1), Vector2i(-1, 0),
	Vector2i(1, 0), Vector2i(-1, 1), Vector2i(0, 1), Vector2i(1, 1),
]

var _geometry: GridGeometry
## La caméra, et ce qui se pose ou passe sur les cases sans être une pièce.
var _view_camera: BoardCamera
var _marks: BoardMarks
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
	_view_camera = BoardCamera.new()
	add_child(_view_camera)

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
	_marks = BoardMarks.new()
	_marks.ground = _ground
	_marks.is_drawn = func(cell: Vector2i) -> bool: return _drawn.has(cell)
	add_child(_marks)
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
		_marks.pulse(pulse["cell"], pulse["color"], pulse["delay"])
	for shot in timeline.shots:
		_marks.shoot(shot["from"], shot["to"], shot["delay"])
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
	_marks.show_hover(cell)


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
	_marks.flash_refusal(cell)


## Entoure de pointillés ces pions (par identité) : le dernier coup adverse.
func show_last(ids: Array[int]) -> void:
	_last_ids = ids
	_update_last_marks()


## Le point de l'écran où se trouve le centre de cette case.
func screen_point(cell: Vector2i) -> Vector2:
	return _view_camera.screen_point(_ground(cell))


## Marque ces cases proposées (le choix d'une carte) ; un toucher proche s'y aimante.
func show_candidates(cells: Array[Vector2i]) -> void:
	_magnet = cells
	_marks.show_marks(BoardMarks.Kind.CANDIDATE, cells)


## Allume ces cases : la ligne gagnante, que donne la vue (rien n'est calculé ici).
func show_winning_line(cells: Array[Vector2i]) -> void:
	_marks.show_marks(BoardMarks.Kind.WIN, cells)


## Marque ces cases déjà choisies (le lanceur et les cibles d'une carte en cours).
func show_chosen(cells: Array[Vector2i]) -> void:
	_marks.show_marks(BoardMarks.Kind.CHOSEN, cells)


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
	var camera := _view_camera.camera if _view_camera != null else null
	if visible and camera != null and camera.size > 0.0:
		# La caméra est orthographique et garde sa hauteur : `size` est la hauteur filmée.
		var cell_pixels := get_viewport().get_visible_rect().size.y / camera.size * GridGeometry.CELL_SIZE
		_badges.follow(camera, _pions, cell_pixels)


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
	_marks.show_marks(BoardMarks.Kind.LAST, cells)


## Le sol d'une case : le dessus de la case, là où se posent les pièces.
func _ground(cell: Vector2i) -> Vector3:
	return _geometry.center(cell) + Vector3.UP * Look.CASE_HEIGHT


## Cadre les cases jouables, plus une demi-case, dans le rectangle de cadrage
## (`BoardCamera`). `animated` : la caméra glisse vers son cadrage (le plateau a changé).
func _update_camera(animated: bool) -> void:
	if _view_camera != null:
		_view_camera.frame_bounds(_bounds, _frame_rect, animated)


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
	var found: Variant = _view_camera.ground_hit(screen_position, Look.CASE_HEIGHT)
	if found == null:
		return ViewData.NO_CELL
	var hit: Vector3 = found
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
