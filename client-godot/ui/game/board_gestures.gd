class_name BoardGestures
extends RefCounted
## Les gestes sur le plateau, pendant la partie : c'est ici qu'un toucher devient un coup,
## pris tel quel dans les coups permis de la vue (ou de l'aperçu, pendant le choix d'une
## carte). Rien n'est calculé.
## - Une case où seule la pose est permise : le palet tombe (fantôme au survol, puis en
##   attente au-dessus de la case jusqu'à la réponse).
## - Une case qui porte d'autres coups (pas de la reine, tir, pas du Lièvre) ou plusieurs :
##   un petit menu sur la case ; le pas du Lièvre demande ensuite sa case d'arrivée.
## - Pendant le choix d'une carte : la case proposée touchée est choisie.
## - Une case où rien n'est permis : un refus (le pion tremble, la case se voile).

## Le joueur demande ce coup, recopié de la vue.
signal requested(action: Variant)
## Le pas du Lièvre attend sa case d'arrivée (`true`), ou ne l'attend plus.
signal hare_step_changed(active: bool)

const PLACE := "Place"
const HARE_STEP := "HareStep"
const CHOOSE := "Choose"

## La vue affichée, et le choix d'une carte en cours (ou `null`) : donnés par l'écran.
var view: ViewData
var choice: CardChoice
## Vrai quand un geste peut être pris : aucune réponse attendue, et la vue affichée est la
## dernière.
var can_act: Callable = func() -> bool: return true

var _board: Board3D
var _menu: ActionMenu
var _hovered := ViewData.NO_CELL
## La case du menu ouvert, et celle du pion dont on choisit le pas du Lièvre.
var _menu_cell := ViewData.NO_CELL
var _hare_from := ViewData.NO_CELL


func _init(board: Board3D, menu: ActionMenu) -> void:
	_board = board
	_menu = menu
	_menu.option_chosen.connect(_on_menu_option)


## Écoute le plateau (partagé entre les écrans) : à l'entrée de l'écran de partie.
func attach() -> void:
	_board.cell_clicked.connect(click)
	_board.cell_hovered.connect(hover)
	_board.empty_tapped.connect(leave_hare_step)


## Cesse d'écouter le plateau : à la sortie de l'écran de partie.
func detach() -> void:
	_board.cell_clicked.disconnect(click)
	_board.cell_hovered.disconnect(hover)
	_board.empty_tapped.disconnect(leave_hare_step)


## Vrai pendant le pas du Lièvre, quand sa case d'arrivée est attendue.
func in_hare_step() -> bool:
	return _hare_from != ViewData.NO_CELL


## Oublie le geste en cours : le menu se ferme, le pas du Lièvre est abandonné.
func reset() -> void:
	leave_hare_step()
	_menu.close()


## Une case touchée ou cliquée.
func click(cell: Vector2i) -> void:
	if view == null or not can_act.call():
		return
	if choice != null:
		_click_during_choice(cell)
	elif in_hare_step():
		_click_hare_destination(cell)
	else:
		_click_cell(cell)


## La case sous la souris : la marque et le palet fantôme d'une pose possible ; pendant un
## choix ou le pas du Lièvre, la case proposée (aimantée) qui serait choisie.
func hover(cell: Vector2i) -> void:
	_hovered = cell
	if view == null or not can_act.call():
		return
	if choice != null or in_hare_step():
		_board.show_mark(cell)
		_board.show_ghost(ViewData.NO_CELL, "", false)
		return
	var placeable := view.action_at(cell, PLACE) != null
	var shown := cell if placeable else ViewData.NO_CELL
	_board.show_mark(shown)
	_board.show_ghost(shown, view.viewer(), false)


## Remontre le survol de la dernière case (la vue ou l'attente ont changé).
func refresh_hover() -> void:
	hover(_hovered)


func leave_hare_step() -> void:
	if not in_hare_step():
		return
	_hare_from = ViewData.NO_CELL
	_board.show_candidates([] as Array[Vector2i])
	hare_step_changed.emit(false)


func _click_during_choice(cell: Vector2i) -> void:
	if choice.is_waiting():
		return
	var action: Variant = choice.choices.action_at(cell, CHOOSE)
	if action == null:
		_board.flash_refusal(cell)
	else:
		choice.choose(action)


func _click_cell(cell: Vector2i) -> void:
	var kinds := _kinds_at(cell)
	if kinds.is_empty():
		_board.flash_refusal(cell)
	elif kinds == [PLACE]:
		_place(cell)
	elif kinds == [CHOOSE]:
		# Une partie reprise au milieu du choix d'une carte : chaque choix part tel quel.
		requested.emit(view.action_at(cell, CHOOSE))
	else:
		_menu_cell = cell
		_menu.open_at(_board.screen_point(cell), kinds)


## Les sortes de coups permis sur cette case, chacune une fois, dans l'ordre du serveur.
func _kinds_at(cell: Vector2i) -> Array[String]:
	var kinds: Array[String] = []
	for action: Variant in view.actions_at(cell):
		var kind := ViewData.action_kind(action)
		if not kinds.has(kind):
			kinds.append(kind)
	return kinds


func _on_menu_option(kind: String) -> void:
	if view == null or not can_act.call():
		return
	if kind == PLACE:
		_place(_menu_cell)
	elif kind == HARE_STEP:
		_hare_from = _menu_cell
		_board.show_candidates(_hare_destinations(_hare_from))
		hare_step_changed.emit(true)
	else:
		var action: Variant = view.action_at(_menu_cell, kind)
		if action != null:
			requested.emit(action)


func _hare_destinations(from: Vector2i) -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	for action: Variant in view.actions_at(from):
		var destination := ViewData.step_destination(action)
		if destination != ViewData.NO_CELL:
			cells.append(destination)
	return cells


func _click_hare_destination(cell: Vector2i) -> void:
	for action: Variant in view.actions_at(_hare_from):
		if ViewData.step_destination(action) == cell:
			leave_hare_step()
			requested.emit(action)
			return
	leave_hare_step()
	_board.flash_refusal(cell)


func _place(cell: Vector2i) -> void:
	var action: Variant = view.action_at(cell, PLACE)
	if action == null:
		return
	_board.show_mark(ViewData.NO_CELL)
	_board.show_ghost(cell, view.viewer(), true)
	requested.emit(action)
