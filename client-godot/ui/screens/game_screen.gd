class_name GameScreen
extends Control
## L'écran de partie : l'interface autour du plateau 3D. C'est le seul endroit qui
## transforme un geste en coup : il prend le coup dans `legal_actions` de la vue et le
## demande tel quel (`play_requested`). Rien n'est calculé ici.
##
## En portrait (téléphone) : l'adversaire en haut, le tour, le plateau, sa barre, et le
## bouton d'action sur toute la largeur, sous le pouce. En paysage (PC) : le plateau à
## gauche, une colonne à droite. Le plateau prend toute la largeur qui lui revient, marges
## de la page comprises : seule la rangée de mur peut en déborder.
##
## Ce que voit le joueur, sans attendre le serveur : le palet fantôme sous la souris, le
## palet qui flotte sur la case touchée jusqu'à la réponse, un voile terne sur une case où
## rien n'est permis. Puis, d'après la vue et ses événements : la chute des pions posés, le
## dernier coup adverse entouré, le bandeau « À toi ! » au début de son tour.

## Le joueur demande ce coup, recopié de la vue.
signal play_requested(action: Variant)
## La partie est finie et le joueur revient à l'accueil (les écrans de fin viendront en E3g).
signal home_requested

## Les coups sans case qui deviennent l'action principale quand ils sont permis.
const MAIN_ACTIONS: Array[String] = ["EndTurn", "Pass"]
const HOURGLASS := "sablier"

## Le nom affiché de l'adversaire (le bot, un ami).
var opponent_name := Labels.OPPONENT

var _board: Board3D
var _view: ViewData
var _waiting_reply := false
var _landscape := false
var _layout: Control
var _hovered := ViewData.NO_CELL
var _was_my_turn := false
## Les pions du dernier coup adverse, par identité.
var _last_opponent_pions: Array[int] = []

var _opponent_panel: PlayerPanel
var _my_panel: PlayerPanel
var _pill: PanelContainer
var _pill_icon: TextureRect
var _pill_label: Label
var _notice: Label
var _main_button: Button
var _main_action: Variant = null
## La place du bouton d'action : le bouton, ou le cadre « EN ATTENTE ».
var _action_slot: MarginContainer
var _waiting: PanelContainer
var _board_area: Control
var _banner: PanelContainer
var _banner_show: Tween


func _init(board: Board3D) -> void:
	_board = board


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_opponent_panel = PlayerPanel.new()
	_my_panel = PlayerPanel.new()
	_build_pill()
	_notice = Ui.centered_label("", ThemeBuilder.MUTED_LABEL)
	_notice.visible = false
	_build_action_slot()
	_board_area = Ui.spacer()
	_board_area.resized.connect(_frame_board)
	_build_banner()

	_board.cell_clicked.connect(_on_cell_clicked)
	_board.cell_hovered.connect(_on_cell_hovered)
	resized.connect(_on_resized)
	_arrange(_is_landscape())
	_refresh_main_button()


func _exit_tree() -> void:
	_board.cell_clicked.disconnect(_on_cell_clicked)
	_board.cell_hovered.disconnect(_on_cell_hovered)


## Affiche cette vue et ce qui s'est passé depuis la précédente.
func show_view(view: ViewData, events: Array) -> void:
	_view = view
	_waiting_reply = false
	var me := view.viewer()
	var dropped: Array[int] = []
	var opponent_placed: Array[int] = []
	for pion in GameEvents.placed_pions(events):
		var id: int = pion.get("id", -1)
		dropped.append(id)
		if pion.get("owner") != me:
			opponent_placed.append(id)
	if not opponent_placed.is_empty():
		_last_opponent_pions = opponent_placed

	_board.show_view(view, dropped)
	_board.show_last(_last_opponent_pions)
	_opponent_panel.show_side(view, view.opponent(), opponent_name)
	_my_panel.show_side(view, me, Labels.YOU)
	var my_turn := view.current_player() == me and not view.is_over()
	_pill_label.text = Labels.GAME_OVER if view.is_over() else Labels.turn_pill(view.turn(), my_turn)
	_pill_icon.visible = not my_turn and not view.is_over()
	_refresh_main_button()
	_on_cell_hovered(_hovered)

	# Le bandeau n'annonce que le début de son tour, pas une réaction pendant celui de
	# l'adversaire.
	var my_main_turn := my_turn and view.decision_player() == me
	if my_main_turn and not _was_my_turn:
		_announce_turn()
	_was_my_turn = my_main_turn


## Le serveur a refusé la dernière demande : le joueur peut rejouer.
func on_refused() -> void:
	_waiting_reply = false
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_refresh_main_button()
	_on_cell_hovered(_hovered)


## Un message discret sous le tour (reconnexion…), ou rien.
func set_notice(text: String) -> void:
	_notice.text = text
	_notice.visible = text != ""


func _refresh_main_button() -> void:
	_main_action = null
	if _view != null and _view.is_over():
		_show_main_button(Labels.HOME, true)
		return
	if _view != null:
		for kind in MAIN_ACTIONS:
			var action: Variant = _view.simple_action(kind)
			if action != null:
				_main_action = action
				_show_main_button(Labels.ACTION_LABELS[kind], not _waiting_reply)
				return
	_show_waiting()


## Le bouton d'action doré, avec ce texte (grisé le temps que le serveur réponde).
func _show_main_button(text: String, enabled: bool) -> void:
	_waiting.visible = false
	_main_button.visible = true
	_main_button.text = text
	_main_button.disabled = not enabled


## À la place du bouton d'action, le cadre en pointillés : on attend l'adversaire.
func _show_waiting() -> void:
	_main_button.visible = false
	_waiting.visible = true


func _on_main_button() -> void:
	if _view != null and _view.is_over():
		home_requested.emit()
	elif _main_action != null:
		_request(_main_action)


func _on_cell_clicked(cell: Vector2i) -> void:
	if _view == null or _waiting_reply:
		return
	var action: Variant = _placement_at(cell)
	if action == null:
		_board.flash_refusal(cell)
		return
	_board.show_mark(ViewData.NO_CELL)
	_board.show_ghost(cell, _view.viewer(), true)
	_request(action)


func _on_cell_hovered(cell: Vector2i) -> void:
	_hovered = cell
	if _waiting_reply:
		return
	var placeable := _placement_at(cell) != null
	var shown := cell if placeable else ViewData.NO_CELL
	_board.show_mark(shown)
	_board.show_ghost(shown, _view.viewer() if _view != null else "", false)


## La pose permise sur cette case, ou `null`. Les cartes viendront en E3c.
func _placement_at(cell: Vector2i) -> Variant:
	if _view == null or _waiting_reply:
		return null
	return _view.action_at(cell, "Place")


func _request(action: Variant) -> void:
	_waiting_reply = true
	_refresh_main_button()
	play_requested.emit(action)


## Le bandeau « À toi ! » au milieu du plateau, et une brève vibration sur téléphone.
func _announce_turn() -> void:
	Input.vibrate_handheld(Look.TURN_VIBRATION_MS)
	if _banner_show != null:
		_banner_show.kill()
	_banner.reset_size()
	_center_banner()
	_banner.scale = Vector2.ONE * Look.TURN_BANNER_START_SCALE
	_banner.modulate.a = 0.0
	_banner.visible = true
	_banner_show = create_tween()
	_banner_show.set_parallel().set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
	_banner_show.tween_property(_banner, "scale", Vector2.ONE, Look.TURN_BANNER_IN)
	_banner_show.tween_property(_banner, "modulate:a", 1.0, Look.TURN_BANNER_IN)
	_banner_show.chain().tween_property(_banner, "modulate:a", 0.0, Look.TURN_BANNER_OUT).set_delay(Look.TURN_BANNER_HOLD)
	_banner_show.chain().tween_callback(func() -> void: _banner.visible = false)


## Le bandeau au milieu du plateau. Rappelé quand la mise en page change : au premier
## affichage, le plateau n'a pas encore sa place.
func _center_banner() -> void:
	var area := _board_area.get_global_rect()
	_banner.position = area.get_center() - _banner.size / 2.0
	_banner.pivot_offset = _banner.size / 2.0


func _build_action_slot() -> void:
	_action_slot = MarginContainer.new()
	_action_slot.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_main_button = Ui.button(Labels.WAITING, ThemeBuilder.PRIMARY_BUTTON)
	_main_button.pressed.connect(_on_main_button)
	_action_slot.add_child(_main_button)
	_waiting = PanelContainer.new()
	_waiting.theme_type_variation = ThemeBuilder.WAITING_PANEL
	_waiting.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_waiting.custom_minimum_size.y = Look.BUTTON_HEIGHT
	var row := Ui.hbox(Look.GAP_S)
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	row.add_child(Ui.icon(HOURGLASS, Look.MUTED))
	row.add_child(Ui.label(Labels.WAITING, ThemeBuilder.WAITING_LABEL))
	_waiting.add_child(row)
	_waiting.visible = false
	_action_slot.add_child(_waiting)


func _build_pill() -> void:
	_pill = PanelContainer.new()
	_pill.theme_type_variation = ThemeBuilder.PILL
	_pill.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_pill.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	var row := Ui.hbox(Look.GAP_S)
	_pill_icon = Ui.icon(HOURGLASS, Look.GOLD)
	row.add_child(_pill_icon)
	_pill_label = Ui.label("", ThemeBuilder.PILL_LABEL)
	row.add_child(_pill_label)
	_pill.add_child(row)


func _build_banner() -> void:
	_banner = PanelContainer.new()
	_banner.theme_type_variation = ThemeBuilder.BANNER
	_banner.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_banner.visible = false
	_banner.add_child(Ui.label(Labels.YOUR_TURN_BANNER, ThemeBuilder.BANNER_LABEL))
	add_child(_banner)


func _is_landscape() -> bool:
	var area := get_viewport_rect().size
	return area.x > area.y


func _on_resized() -> void:
	var landscape := _is_landscape()
	if landscape != _landscape:
		_arrange(landscape)
	_frame_board()


## Range les éléments pour cette orientation. Les éléments restent les mêmes : seules
## leurs boîtes changent.
func _arrange(landscape: bool) -> void:
	_landscape = landscape
	for item: Control in [_opponent_panel, _my_panel, _pill, _notice, _action_slot, _board_area]:
		if item.get_parent() != null:
			item.get_parent().remove_child(item)
	if _layout != null:
		_layout.queue_free()

	if landscape:
		var row := Ui.hbox(Look.GAP_XL)
		row.add_child(_board_area)
		var column := Ui.vbox(Look.GAP_M)
		column.custom_minimum_size.x = Look.SIDE_COLUMN_WIDTH
		column.add_child(_pill)
		column.add_child(_notice)
		column.add_child(_opponent_panel)
		column.add_child(_my_panel)
		column.add_child(Ui.spacer())
		column.add_child(_action_slot)
		row.add_child(column)
		_my_panel.set_identity_visible(true)
		_layout = Ui.page(row)
	else:
		var column := Ui.vbox(Look.GAP_M)
		column.add_child(_opponent_panel)
		column.add_child(_pill)
		column.add_child(_notice)
		column.add_child(_board_area)
		column.add_child(_my_panel)
		column.add_child(_action_slot)
		_my_panel.set_identity_visible(false)
		_layout = Ui.page(column)
	add_child(_layout)
	# Le bandeau passe devant la nouvelle mise en page.
	move_child(_banner, -1)


func _frame_board() -> void:
	if not _board_area.is_inside_tree():
		return
	var area := _board_area.get_global_rect()
	# Les marges de la page servent au texte, pas au plateau : il va jusqu'au bord gauche, et
	# jusqu'au bord droit en portrait.
	var right := area.end.x if _landscape else get_viewport_rect().size.x
	_board.frame(Rect2(0.0, area.position.y, right, area.size.y))
	if _banner.visible:
		_center_banner()
