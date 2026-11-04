@tool
class_name StyleBoxHex
extends StyleBox
## Le fond des boutons d'action des maquettes : un hexagone allongé, une pointe de chaque
## côté, avec une tranche plus sombre en bas qui lui donne de l'épaisseur. Il est dessiné,
## pas étiré depuis une image : il reste net à toute taille et sur tout écran.
##
## Enfoncé (`sunk`), le dessus descend sur sa tranche : le bouton s'écrase sous le doigt.

@export var fill := Color.WHITE
@export var shade := Color.BLACK
@export var point_width := 14.0
@export var shade_height := 4.0
@export var sunk := false


func _draw(to_canvas_item: RID, rect: Rect2) -> void:
	var face := Rect2(rect.position, Vector2(rect.size.x, rect.size.y - shade_height))
	if sunk:
		face.position.y += shade_height
	else:
		_fill(to_canvas_item, _hexagon(Rect2(face.position + Vector2(0, shade_height), face.size)), shade)
	_fill(to_canvas_item, _hexagon(face), fill)


func _hexagon(rect: Rect2) -> PackedVector2Array:
	var point := minf(point_width, rect.size.x / 2.0)
	var middle := rect.position.y + rect.size.y / 2.0
	return PackedVector2Array([
		Vector2(rect.position.x + point, rect.position.y),
		Vector2(rect.end.x - point, rect.position.y),
		Vector2(rect.end.x, middle),
		Vector2(rect.end.x - point, rect.end.y),
		Vector2(rect.position.x + point, rect.end.y),
		Vector2(rect.position.x, middle),
	])


## Un polygone plein, bordé d'un trait de même couleur lissé : sans lui, les pointes
## seraient crénelées (les polygones ne sont pas lissés).
func _fill(item: RID, points: PackedVector2Array, color: Color) -> void:
	RenderingServer.canvas_item_add_polygon(item, points, PackedColorArray([color]))
	var outline := points.duplicate()
	outline.append(points[0])
	RenderingServer.canvas_item_add_polyline(item, outline, PackedColorArray([color]), 1.0, true)
