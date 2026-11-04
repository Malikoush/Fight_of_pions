@tool
class_name StyleBoxDashed
extends StyleBox
## Un cadre arrondi en pointillés, sans fond : la place d'une action qui n'est pas encore à
## toi (le bouton « EN ATTENTE » des maquettes). Dessiné, net à toute taille. Tous ses
## réglages viennent de `Look`, par `ThemeBuilder` : sans eux, il ne dessine rien.

@export var color := Color.TRANSPARENT
@export var width := 0.0
@export var radius := 0.0
@export var dash := 0.0
@export var gap := 0.0
@export var corner_steps := 1


func _draw(to_canvas_item: RID, rect: Rect2) -> void:
	var path := _rounded_outline(rect.grow(-width / 2.0))
	var drawing := true
	var left := dash
	var stroke := PackedVector2Array([path[0]])
	for index in range(1, path.size()):
		var from := path[index - 1]
		var to := path[index]
		var length := from.distance_to(to)
		var done := 0.0
		while length - done > left:
			done += left
			var point := from.lerp(to, done / length)
			if drawing:
				stroke.append(point)
				_stroke(to_canvas_item, stroke)
			stroke = PackedVector2Array([point])
			drawing = not drawing
			left = dash if drawing else gap
		left -= length - done
		if drawing:
			stroke.append(to)
		else:
			stroke = PackedVector2Array([to])
	if drawing and stroke.size() > 1:
		_stroke(to_canvas_item, stroke)


func _stroke(item: RID, points: PackedVector2Array) -> void:
	RenderingServer.canvas_item_add_polyline(item, points, PackedColorArray([color]), width, true)


## Le tour du rectangle aux coins arrondis, fermé, dans le sens des aiguilles d'une montre.
func _rounded_outline(rect: Rect2) -> PackedVector2Array:
	var r := minf(radius, minf(rect.size.x, rect.size.y) / 2.0)
	var corners := [
		[Vector2(rect.end.x - r, rect.position.y + r), -PI / 2.0],
		[Vector2(rect.end.x - r, rect.end.y - r), 0.0],
		[Vector2(rect.position.x + r, rect.end.y - r), PI / 2.0],
		[Vector2(rect.position.x + r, rect.position.y + r), PI],
	]
	var points := PackedVector2Array()
	for corner: Array in corners:
		var center: Vector2 = corner[0]
		var start: float = corner[1]
		for step in corner_steps + 1:
			var angle := start + (PI / 2.0) * step / corner_steps
			points.append(center + Vector2(cos(angle), sin(angle)) * r)
	points.append(points[0])
	return points
