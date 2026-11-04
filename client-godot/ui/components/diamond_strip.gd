class_name DiamondStrip
extends Control
## Une rangée de losanges, les premiers pleins : le symbole des points d'action et de la
## banque. Dessinés, pas en image : nets à toute taille, aux couleurs de l'apparence (`Look`).

var count := 1:
	set(value):
		count = value
		_resize()
var filled := 0:
	set(value):
		filled = value
		queue_redraw()
var diamond := Look.BANK_PIP:
	set(value):
		diamond = value
		_resize()


func _init() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	size_flags_vertical = Control.SIZE_SHRINK_CENTER
	_resize()


func _resize() -> void:
	custom_minimum_size = Vector2(count * diamond + maxi(count - 1, 0) * Look.GAP_XS, diamond)
	queue_redraw()


func _draw() -> void:
	var half := diamond / 2.0
	for index in count:
		var center := Vector2(half + index * (diamond + Look.GAP_XS), size.y / 2.0)
		var outline := PackedVector2Array([
			center + Vector2(0, -half), center + Vector2(half, 0),
			center + Vector2(0, half), center + Vector2(-half, 0), center + Vector2(0, -half),
		])
		if index < filled:
			draw_colored_polygon(outline.slice(0, 4), Look.GOLD)
			draw_polyline(outline, Look.INK, Look.DIAMOND_OUTLINE / 2.0, true)
		else:
			draw_polyline(outline, Look.GOLD, Look.DIAMOND_OUTLINE, true)
