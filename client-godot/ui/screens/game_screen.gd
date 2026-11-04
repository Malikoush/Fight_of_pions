class_name GameScreen
extends Control
## L'écran de partie : l'interface autour du plateau 3D. C'est le seul endroit qui
## transforme un geste en coup : il prend le coup dans `legal_actions` de la vue et le
## demande tel quel (`play_requested`). Rien n'est calculé ici.
##
## En portrait (téléphone) : l'adversaire en haut, le tour, le plateau, sa barre, et le
## bouton d'action sur toute la largeur, sous le pouce. En paysage (PC) : le plateau à
## gauche, une colonne à droite. Le plateau prend toute la largeur qui lui
## revient, marges de la page comprises : seule la rangée de mur peut en déborder.

## Le joueur demande ce coup, recopié de la vue.
signal play_requested(action: Variant)
## La partie est finie et le joueur revient à l'accueil (les écrans de fin viendront en E3g).
signal home_requested

## Les coups sans case qui deviennent l'action principale quand ils sont permis.
const MAIN_ACTIONS: Array[String] = ["EndTurn", "Pass"]

## Le nom affiché de l'adversaire (le bot, un ami).
var opponent_name := Labels.OPPONENT

var _board: Board3D
var _view: ViewData
var _waiting_reply := false
var _landscape := false
var _layout: Control

var _opponent_panel: PlayerPanel
var _my_panel: PlayerPanel
var _pill: PanelContainer
var _pill_label: Label
var _notice: Label
var _main_button: Button
var _main_action: Variant = null
var _board_area: Control


func _init(board: Board3D) -> void:
	_board = board


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_opponent_panel = PlayerPanel.new()
	_my_panel = PlayerPanel.new()
	_pill = PanelContainer.new()
	_pill.theme_type_variation = ThemeBuilder.PILL
	_pill.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_pill.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_pill_label = Ui.label("", ThemeBuilder.PILL_LABEL)
	_pill.add_child(_pill_label)
	_notice = Ui.label("", ThemeBuilder.MUTED_LABEL)
	_notice.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_notice.visible = false
	_main_button = Ui.button(Labels.WAITING, ThemeBuilder.PRIMARY_BUTTON)
	_main_button.pressed.connect(_on_main_button)
	_board_area = Ui.spacer()
	_board_area.resized.connect(_frame_board)

	_board.cell_clicked.connect(_on_cell_clicked)
	_board.cell_hovered.connect(_on_cell_hovered)
	resized.connect(_on_resized)
	_arrange(_is_landscape())


func _exit_tree() -> void:
	_board.cell_clicked.disconnect(_on_cell_clicked)
	_board.cell_hovered.disconnect(_on_cell_hovered)


## Affiche cette vue : plateau, joueurs, tour et action principale.
func show_view(view: ViewData) -> void:
	_view = view
	_waiting_reply = false
	_board.show_view(view)
	var me := view.viewer()
	_opponent_panel.show_side(view, view.opponent(), opponent_name)
	_my_panel.show_side(view, me, Labels.YOU)
	_pill_label.text = Labels.GAME_OVER if view.is_over() else Labels.turn_pill(view.turn(), view.current_player() == me)
	_refresh_main_button()


## Le serveur a refusé la dernière demande : le joueur peut rejouer.
func on_refused() -> void:
	_waiting_reply = false
	_refresh_main_button()


## Un message discret sous le tour (reconnexion…), ou rien.
func set_notice(text: String) -> void:
	_notice.text = text
	_notice.visible = text != ""


func _refresh_main_button() -> void:
	_main_action = null
	_main_button.text = Labels.WAITING
	if _view != null and _view.is_over():
		_main_button.text = Labels.HOME
		_main_button.disabled = false
		return
	if _view != null:
		for kind in MAIN_ACTIONS:
			var action: Variant = _view.simple_action(kind)
			if action != null:
				_main_action = action
				_main_button.text = Labels.ACTION_LABELS[kind]
				break
	_main_button.disabled = _main_action == null or _waiting_reply


func _on_main_button() -> void:
	if _view != null and _view.is_over():
		home_requested.emit()
	elif _main_action != null:
		_request(_main_action)


func _on_cell_clicked(cell: Vector2i) -> void:
	var action: Variant = _placement_at(cell)
	if action != null:
		_request(action)


func _on_cell_hovered(cell: Vector2i) -> void:
	_board.show_mark(cell if _placement_at(cell) != null else ViewData.NO_CELL)


## La pose permise sur cette case, ou `null`. Les cartes viendront en E3c.
func _placement_at(cell: Vector2i) -> Variant:
	if _view == null or _waiting_reply:
		return null
	return _view.action_at(cell, "Place")


func _request(action: Variant) -> void:
	_waiting_reply = true
	_board.show_mark(ViewData.NO_CELL)
	_refresh_main_button()
	play_requested.emit(action)


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
	for item: Control in [_opponent_panel, _my_panel, _pill, _notice, _main_button, _board_area]:
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
		column.add_child(_main_button)
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
		column.add_child(_main_button)
		_my_panel.set_identity_visible(false)
		_layout = Ui.page(column)
	add_child(_layout)


func _frame_board() -> void:
	if not _board_area.is_inside_tree():
		return
	var area := _board_area.get_global_rect()
	# Les marges de la page servent au texte, pas au plateau : il va jusqu'au bord gauche, et
	# jusqu'au bord droit en portrait.
	var right := area.end.x if _landscape else get_viewport_rect().size.x
	_board.frame(Rect2(0.0, area.position.y, right, area.size.y))
