class_name ElementArt
extends Control
## Un petit dessin fait des éléments du plateau fournis (design/elements) : des cases, des
## palets, et des places libres (un losange en pointillés, de la taille d'une case).
##
## Toutes les images ont la même toile, ancrée au centre du dessus de la case
## (`Look.ELEMENT_ANCHOR`) : poser une image, c'est placer son ancre. Le dessin grandit pour
## contenir tout ce qu'on y pose ; les images se superposent dans l'ordre où on les pose.

var _free_places: Array[Vector2] = []
var _free_scale := 1.0


func _init() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE


## Une case du plateau, son centre en ce point.
func add_tile(at: Vector2, scale_factor: float) -> void:
	_add_picture(Ui.BOARD_DIR + "case.svg", at, scale_factor)


## Le palet d'un joueur, posé en ce point.
func add_pion(player: String, at: Vector2, scale_factor: float) -> void:
	_add_picture(Ui.pion_path(player), at, scale_factor)


## Une place libre : le contour en pointillés d'une case, en ce point.
func add_free_place(at: Vector2, scale_factor: float) -> void:
	_free_places.append(at)
	_free_scale = scale_factor
	_grow_to(at, scale_factor)
	queue_redraw()


## Efface tout, pour redessiner.
func clear() -> void:
	for child in get_children():
		child.queue_free()
	_free_places.clear()
	custom_minimum_size = Vector2.ZERO
	queue_redraw()


## Le point d'ancrage d'une image posée tout en haut à gauche du dessin.
static func first_anchor(scale_factor: float) -> Vector2:
	return Look.ELEMENT_ANCHOR * scale_factor


func _add_picture(path: String, at: Vector2, scale_factor: float) -> void:
	var canvas := Look.ELEMENT_CANVAS * scale_factor
	var item := Ui.picture(path, canvas, Color.WHITE)
	item.position = at - Look.ELEMENT_ANCHOR * scale_factor
	item.size = canvas
	add_child(item)
	_grow_to(at, scale_factor)


## Le dessin contient la toile entière de l'image ancrée en ce point.
func _grow_to(at: Vector2, scale_factor: float) -> void:
	var corner := at - Look.ELEMENT_ANCHOR * scale_factor + Look.ELEMENT_CANVAS * scale_factor
	custom_minimum_size = custom_minimum_size.max(corner)


func _draw() -> void:
	var half := Vector2(Look.ELEMENT_TILE_WIDTH, Look.ELEMENT_TILE_WIDTH / 2.0) * _free_scale / 2.0
	for at in _free_places:
		var corners := [at + Vector2(0.0, -half.y), at + Vector2(half.x, 0.0), at + Vector2(0.0, half.y), at + Vector2(-half.x, 0.0)]
		for index in corners.size():
			draw_dashed_line(corners[index], corners[(index + 1) % corners.size()], Look.FREE_PLACE, Look.BORDER, Look.DASH_LENGTH)
