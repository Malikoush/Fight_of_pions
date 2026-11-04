class_name BadgeLayer
extends CanvasLayer
## Les badges des pions (statuts, gardes, jetons de pas), dessinés par-dessus le plateau et
## sous l'interface. Ils sont en 2D plutôt qu'en 3D : nets, et d'une taille lisible quel que
## soit le cadrage. Chaque pion qui en a reçoit une rangée, qui suit à chaque image le point
## que le plateau lui donne (le pion peut glisser ou tomber).

## Les badges de design/elements/effets.
const EFFECT_DIR := "res://assets/elements/effets/"

## La rangée de chaque pion qui a des badges, par identité, et ce qu'elle montre.
var _rows: Dictionary[int, HBoxContainer] = {}
var _shown: Dictionary[int, Array] = {}
var _root: Control
var _badge_size := 0.0


func _init() -> void:
	layer = Look.BADGE_CANVAS_LAYER
	_root = Control.new()
	_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_root.theme = ThemeBuilder.shared()
	add_child(_root)


## Montre les badges de ces pions (par identité) ; les autres n'en ont plus.
func show_statuses(statuses: Dictionary[int, PionStatus]) -> void:
	for id: int in _rows.keys():
		if not statuses.has(id) or _badges_of(statuses[id]).is_empty():
			_rows[id].queue_free()
			_rows.erase(id)
			_shown.erase(id)
	for id: int in statuses:
		var badges := _badges_of(statuses[id])
		if badges.is_empty() or _shown.get(id, []) == badges:
			continue
		if _rows.has(id):
			_rows[id].queue_free()
		_rows[id] = _row(badges)
		_shown[id] = badges
		_root.add_child(_rows[id])


## Place chaque rangée au point de son pion, à la taille qui convient à ce cadrage
## (`cell_pixels` : la largeur d'une case à l'écran).
func follow(camera: Camera3D, pions: Dictionary[int, Node3D], cell_pixels: float) -> void:
	var badge := clampf(cell_pixels * Look.BADGE_CELL_SHARE, Look.BADGE_MIN, Look.BADGE_MAX)
	var resized := not is_equal_approx(badge, _badge_size)
	_badge_size = badge
	for id: int in _rows:
		var row := _rows[id]
		if resized:
			_size_row(row)
		if not pions.has(id):
			row.visible = false
			continue
		var pion: Pion3D = pions[id]
		var point := camera.unproject_position(pion.badge_anchor())
		row.visible = true
		row.position = point - Vector2(0.0, row.size.y)


## Ce que montre la rangée d'un pion : ses badges (noms de fichiers), puis ses nombres (ses
## gardes, puis ses jetons de pas), dans un ordre fixe.
func _badges_of(status: PionStatus) -> Array:
	# Une liste sans type : elle mêle des noms et des nombres.
	var badges: Array = []
	badges.append_array(status.badges)
	if status.guards > 0:
		badges.append(status.guards)
	if status.step_tokens > 0:
		badges.append(status.step_tokens)
	return badges


func _row(badges: Array) -> HBoxContainer:
	var row := Ui.hbox(Look.BADGE_GAP)
	for badge: Variant in badges:
		if badge is int:
			row.add_child(NumberDiamond.new(badge))
		else:
			row.add_child(Ui.picture(EFFECT_DIR + str(badge) + ".svg", Vector2.ZERO, Color.WHITE))
	_size_row(row)
	return row


func _size_row(row: HBoxContainer) -> void:
	for item: Control in row.get_children():
		var share := Look.NUMBER_BADGE_SHARE if item is NumberDiamond else 1.0
		item.custom_minimum_size = Vector2.ONE * _badge_size * share
	row.reset_size()
