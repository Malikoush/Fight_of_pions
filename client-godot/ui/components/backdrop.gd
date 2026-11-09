class_name Backdrop
extends Control
## Le fond des écrans (maquettes) : sur le sapin, un quadrillage en losanges à peine
## visible. Il reste en place quand les écrans passent de l'un à l'autre ; derrière le
## plateau 3D, il se cache (`ScreenRouter`).


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	resized.connect(queue_redraw)


## Deux familles de lignes à 45°, à `BACKDROP_SPACING` l'une de l'autre, tracées d'un seul
## appel d'un bord à l'autre de l'écran.
func _draw() -> void:
	var step := Look.BACKDROP_SPACING * sqrt(2.0)
	var lines := PackedVector2Array()
	var start := -size.y
	while start <= size.x + size.y:
		lines.append_array([Vector2(start, 0.0), Vector2(start + size.y, size.y)])
		lines.append_array([Vector2(start, 0.0), Vector2(start - size.y, size.y)])
		start += step
	draw_multiline(lines, Look.BACKDROP_LINE, Look.BACKDROP_WIDTH, true)
