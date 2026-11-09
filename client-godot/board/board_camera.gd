class_name BoardCamera
extends Node3D
## La caméra du plateau : orthographique, portée par ce pivot qui la tourne en plongée
## vers le plateau, droit face à ses rangées.
##
## Elle cadre une emprise (les cases jouables, plus une demi-case) dans un rectangle de
## l'écran. Parmi les plongées permises (`Look.CAMERA_PITCHES_DEGREES`), elle prend celle
## qui donne les plus grandes cases à l'écran : la surface d'une case y vaut le sinus de la
## plongée sur le carré de la hauteur filmée. Quand la largeur manque et que la hauteur
## abonde (téléphone en portrait), une plongée plus forte grandit les cases en hauteur.
##
## Elle dit aussi où tombe un point de l'écran sur le plan du dessus des cases, et où se
## trouve un point du monde à l'écran.

var camera: Camera3D
var _reframe: Tween


func _init() -> void:
	camera = Camera3D.new()
	camera.projection = Camera3D.PROJECTION_ORTHOGONAL
	camera.position = Vector3(0.0, 0.0, Look.CAMERA_DISTANCE)
	camera.far = Look.CAMERA_DISTANCE * 2.0
	add_child(camera)


## Cadre cette emprise dans ce rectangle de l'écran (vide : tout l'écran). `animated` : la
## caméra glisse vers son cadrage (le plateau a changé), sinon elle y saute (la fenêtre a
## changé).
func frame_bounds(bounds: AABB, frame_rect: Rect2, animated: bool) -> void:
	if not bounds.has_volume():
		return
	var viewport := get_viewport().get_visible_rect().size
	var rect := frame_rect if frame_rect.has_area() else Rect2(Vector2.ZERO, viewport)
	var start_rotation := rotation_degrees
	var start_position := position
	position = bounds.get_center()
	var best_pitch: float = Look.CAMERA_PITCHES_DEGREES[0]
	var best := Rect2()
	var best_height := INF
	var best_area := 0.0
	for pitch: float in Look.CAMERA_PITCHES_DEGREES:
		rotation_degrees = Vector3(pitch, Look.CAMERA_YAW_DEGREES, 0.0)
		var extent := _projected_bounds(bounds)
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
	var target_position := position
	if not animated:
		rotation_degrees = target_rotation
		camera.size = target_size
		camera.h_offset = target_offset.x
		camera.v_offset = target_offset.y
		return
	rotation_degrees = start_rotation
	position = start_position
	_reframe = create_tween().set_parallel().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_IN_OUT)
	_reframe.tween_property(self, "position", target_position, Look.REFRAME_TIME)
	_reframe.tween_property(self, "rotation_degrees", target_rotation, Look.REFRAME_TIME)
	_reframe.tween_property(camera, "size", target_size, Look.REFRAME_TIME)
	_reframe.tween_property(camera, "h_offset", target_offset.x, Look.REFRAME_TIME)
	_reframe.tween_property(camera, "v_offset", target_offset.y, Look.REFRAME_TIME)


## Le point du plan à cette hauteur (le dessus des cases) sous ce point de l'écran, ou
## `null` si le rayon lui est parallèle.
func ground_hit(screen_position: Vector2, height: float) -> Variant:
	var origin := camera.project_ray_origin(screen_position)
	var direction := camera.project_ray_normal(screen_position)
	if is_zero_approx(direction.y):
		return null
	return origin + direction * ((height - origin.y) / direction.y)


## Le point de l'écran où se trouve ce point du monde.
func screen_point(world: Vector3) -> Vector2:
	return camera.unproject_position(world)


## L'emprise à l'écran de ces cases, dans le plan de la caméra telle qu'elle est tournée
## (en unités du monde, autour du centre du cadrage).
func _projected_bounds(bounds: AABB) -> Rect2:
	var right := global_basis.x
	var up := global_basis.y
	var low := Vector2(INF, INF)
	var high := Vector2(-INF, -INF)
	for index in 8:
		var offset := bounds.get_endpoint(index) - position
		var projected := Vector2(offset.dot(right), offset.dot(up))
		low = low.min(projected)
		high = high.max(projected)
	return Rect2(low, high - low)
