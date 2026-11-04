class_name NumberDiamond
extends Control
## Un nombre dans un losange sombre bordé d'or, comme sur les maquettes : les gardes d'une
## reine, les jetons de pas d'un pion. Dessiné, net à toute taille ; sa taille suit
## `custom_minimum_size`.

var _number: Label


func _init(value: int) -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_number = Ui.label(str(value), ThemeBuilder.BADGE_NUMBER_LABEL)
	_number.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_number.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	_number.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(_number)
	resized.connect(_on_resized)


func _on_resized() -> void:
	_number.add_theme_font_size_override(&"font_size", maxi(1, roundi(size.y * Look.BADGE_NUMBER_SHARE)))
	queue_redraw()


func _draw() -> void:
	var path := DiamondStrip.outline(size / 2.0, minf(size.x, size.y) / 2.0 - Look.DIAMOND_OUTLINE)
	draw_colored_polygon(path.slice(0, 4), Look.FOREST_DEEP)
	draw_polyline(path, Look.GOLD, Look.DIAMOND_OUTLINE, true)
