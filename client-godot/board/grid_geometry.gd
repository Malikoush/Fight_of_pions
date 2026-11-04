class_name GridGeometry
extends RefCounted
## La seule conversion entre une case de la grille et le monde 3D. Une case fait
## `CELL_SIZE` de côté, posée sur le plan y = 0 ; la colonne avance vers +x, la rangée vers
## +z, et la grille est centrée sur l'origine. La caméra regarde le plateau depuis +z, face
## à ses rangées : A1 est le coin en haut à gauche, comme sur un damier.

const CELL_SIZE := 1.0

var size: int


func _init(grid_size: int) -> void:
	size = grid_size


func contains(cell: Vector2i) -> bool:
	return cell.x >= 0 and cell.y >= 0 and cell.x < size and cell.y < size


## Le centre de la case, au niveau du sol.
func center(cell: Vector2i) -> Vector3:
	var middle := (size - 1) / 2.0
	return Vector3(cell.x - middle, 0.0, cell.y - middle) * CELL_SIZE


## La boîte posée sur cette case, de cette hauteur, élargie de `padding` cases de chaque
## côté.
func box(cell: Vector2i, height: float, padding: float = 0.0) -> AABB:
	var half := CELL_SIZE * (0.5 + padding)
	return AABB(center(cell) - Vector3(half, 0.0, half), Vector3(half * 2.0, height, half * 2.0))


## La case sous ce point du sol, ou `ViewData.NO_CELL` hors de la grille.
func cell_at(point: Vector3) -> Vector2i:
	var middle := (size - 1) / 2.0
	var cell := Vector2i(roundi(point.x / CELL_SIZE + middle), roundi(point.z / CELL_SIZE + middle))
	return cell if contains(cell) else ViewData.NO_CELL
