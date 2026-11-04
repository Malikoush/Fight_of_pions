class_name ActionMenu
extends PanelContainer
## Le petit menu d'une case qui porte autre chose qu'une simple pose (pas de la reine, tir,
## pas du Lièvre), ou plusieurs coups : un bouton par sorte de coup permis, près de la case.
## Toucher ailleurs le ferme.

## Le joueur choisit cette sorte de coup (`StepQueen`, `Shoot`…).
signal option_chosen(kind: String)

var _options: VBoxContainer


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL_ACTIVE
	_options = Ui.vbox(Look.GAP_S)
	add_child(_options)
	visible = false


## Ouvre le menu près de ce point de l'écran, avec ces sortes de coups, dans l'ordre.
func open_at(point: Vector2, kinds: Array[String]) -> void:
	for old in _options.get_children():
		old.queue_free()
	for kind in kinds:
		var option := Ui.button(Labels.CELL_ACTIONS.get(kind, kind), ThemeBuilder.SECONDARY_BUTTON)
		option.pressed.connect(_choose.bind(kind))
		_options.add_child(option)
	visible = true
	reset_size()
	# Au-dessus de la case, sans sortir de l'écran.
	var area := get_viewport_rect().size
	var corner := point - Vector2(size.x / 2.0, size.y + Look.GAP_L)
	position = corner.clamp(Vector2.ONE * Look.PAGE_MARGIN, area - size - Vector2.ONE * Look.PAGE_MARGIN)


func close() -> void:
	visible = false


func _choose(kind: String) -> void:
	close()
	option_chosen.emit(kind)


func _input(event: InputEvent) -> void:
	# Toucher hors du menu le ferme, sans rien jouer.
	if visible and event is InputEventMouseButton and event.pressed and not get_global_rect().has_point(event.position):
		close()
		get_viewport().set_input_as_handled()
