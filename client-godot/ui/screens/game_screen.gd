class_name GameScreen
extends Control
## L'écran de partie : l'interface autour du plateau 3D. C'est le seul endroit qui
## transforme un geste en coup : il prend le coup dans `legal_actions` de la vue (ou d'un
## aperçu, pendant le choix d'une carte) et le demande tel quel (`play_requested`). Rien
## n'est calculé ici.
##
## En portrait (téléphone) : l'adversaire en haut, le tour, le plateau, l'arsenal, sa barre,
## et le bouton d'action sur toute la largeur, sous le pouce. En paysage (PC) : le plateau
## et l'arsenal à gauche, une colonne à droite. Pendant le choix d'une carte, le panneau de
## choix prend la place de l'arsenal (et, en portrait, de la barre).
##
## Les gestes :
## - une case où seule la pose est permise : le palet tombe (fantôme au survol, puis en
##   attente au-dessus de la case jusqu'à la réponse) ;
## - une case qui porte d'autres coups (pas de la reine, tir, pas du Lièvre) ou plusieurs :
##   un petit menu sur la case ; le pas du Lièvre demande ensuite sa case d'arrivée ;
## - une carte jouable : le choix commence (lanceur, puis cibles), chaque étape étant un
##   aperçu du serveur ; une fois complète, la carte part coup par coup ;
## - une case où rien n'est permis : un refus (le pion tremble, la case se voile).

## Le joueur demande ce coup, recopié de la vue.
signal play_requested(action: Variant)
## La partie est finie et le joueur revient à l'accueil (les écrans de fin viendront en E3g).
signal home_requested

## Les coups sans case qui deviennent l'action principale quand ils sont permis.
const MAIN_ACTIONS: Array[String] = ["EndTurn", "Pass"]
const HOURGLASS := "sablier"
const PLACE := "Place"
const HARE_STEP := "HareStep"
const CHOOSE := "Choose"

## Le nom affiché de l'adversaire (le bot, un ami).
var opponent_name := Labels.OPPONENT
## Le catalogue des cartes (noms et textes), quand il est arrivé.
var catalog: CardCatalog:
	set(value):
		catalog = value
		if _view != null and is_node_ready():
			_arsenal.show_arsenal(_view, _view.viewer(), catalog)

var _board: Board3D
var _view: ViewData
var _waiting_reply := false
## Les coups d'une carte complète envoyés et pas encore confirmés : on n'affiche pas les
## vues intermédiaires (le choix du lanceur, des cibles), seulement la dernière.
var _sequence_left := 0
var _landscape := false
var _layout: Control
var _hovered := ViewData.NO_CELL
var _was_my_turn := false
## Les pions du dernier coup adverse, par identité.
var _last_opponent_pions: Array[int] = []
## Le choix d'une carte en cours, ou `null`.
var _session: ChoiceSession
## Les réponses à venir d'aperçus demandés par un choix abandonné : on les ignore (les
## réponses arrivent dans l'ordre des demandes).
var _stale_previews := 0
## La case du menu ouvert, et celle du pion dont on choisit le pas du Lièvre.
var _menu_cell := ViewData.NO_CELL
var _hare_from := ViewData.NO_CELL

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
var _arsenal: ArsenalBar
var _choice_panel: ChoicePanel
var _action_menu: ActionMenu
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
	_arsenal = ArsenalBar.new()
	_arsenal.card_chosen.connect(_on_card_chosen)
	_choice_panel = ChoicePanel.new()
	_choice_panel.cancel_requested.connect(_end_choice)
	_choice_panel.finish_requested.connect(_choose_simple.bind("FinishAttack"))
	_choice_panel.tokens_requested.connect(_choose_simple.bind("ChooseEnemyTokens"))
	_build_banner()
	_action_menu = ActionMenu.new()
	_action_menu.option_chosen.connect(_on_menu_option)

	_board.cell_clicked.connect(_on_cell_clicked)
	_board.cell_hovered.connect(_on_cell_hovered)
	_board.empty_tapped.connect(_leave_hare_step)
	resized.connect(_on_resized)
	_arrange(_is_landscape())
	_refresh_main_button()


func _exit_tree() -> void:
	_board.cell_clicked.disconnect(_on_cell_clicked)
	_board.cell_hovered.disconnect(_on_cell_hovered)
	_board.empty_tapped.disconnect(_leave_hare_step)


## Affiche cette vue et ce qui s'est passé depuis la précédente.
func show_view(view: ViewData, events: Array) -> void:
	if _sequence_left > 0:
		_sequence_left -= 1
		if _sequence_left > 0:
			return
	_view = view
	_waiting_reply = false
	_end_choice()
	_leave_hare_step()
	_action_menu.close()
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
	_arsenal.show_arsenal(view, me, catalog)
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


## La réponse du serveur à l'aperçu du choix en cours.
func on_preview(view: ViewData) -> void:
	if _stale_previews > 0:
		_stale_previews -= 1
	elif _session != null:
		_session.on_preview(view)


## Le serveur a refusé la dernière demande : le joueur peut rejouer. Vrai si une carte
## était en train de partir : sa vue est alors peut-être périmée, à redemander.
func on_refused() -> bool:
	var was_sending := _sequence_left > 0
	_waiting_reply = false
	_sequence_left = 0
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_end_choice()
	_refresh_main_button()
	_on_cell_hovered(_hovered)
	return was_sending


## La connexion est coupée : ce qui attendait une réponse (un coup, une carte qui partait,
## un aperçu) n'en aura pas. La vue envoyée à la reprise fera foi.
func on_connection_lost() -> void:
	_waiting_reply = false
	_sequence_left = 0
	_end_choice()
	_stale_previews = 0
	_leave_hare_step()
	_action_menu.close()
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_refresh_main_button()


## Un message discret sous le tour (reconnexion…), ou rien.
func set_notice(text: String) -> void:
	_notice.text = text
	_notice.visible = text != ""


# --- Le choix d'une carte ---

func _on_card_chosen(slot: int) -> void:
	if _view == null or _waiting_reply or _session != null:
		return
	var use_attack: Variant = _view.use_attack(slot)
	if use_attack == null:
		return
	_leave_hare_step()
	_action_menu.close()
	_board.show_mark(ViewData.NO_CELL)
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_session = ChoiceSession.new(slot, use_attack)
	_session.changed.connect(_refresh_choice)
	_session.complete.connect(_play_sequence)
	_arsenal.select(slot)
	_apply_mode()
	_refresh_choice()


## Montre les choix suivants : les cases proposées, celles déjà choisies, et le panneau.
func _refresh_choice() -> void:
	if _session == null:
		return
	var choices := _session.choices
	_board.show_candidates(choices.cells_of(CHOOSE) if choices != null else ([] as Array[Vector2i]))
	_board.show_chosen(_session.chosen_cells())
	var slot: Variant = _view.arsenal(_view.viewer())[_session.slot]
	var attack: Dictionary = slot.get("attack", {}) if slot is Dictionary else {}
	_choice_panel.show_choice(_session, attack, catalog, _view.stats_of(attack).get("cost", 0))


## Choisit le coup sans case de ce nom (`FinishAttack`, `ChooseEnemyTokens`), s'il est proposé.
func _choose_simple(kind: String) -> void:
	if _session == null or _session.is_waiting():
		return
	var action: Variant = _session.choices.simple_action(kind)
	if action != null:
		_session.choose(action)


## La carte est complète : ses coups partent, dans l'ordre.
func _play_sequence(actions: Array) -> void:
	_end_choice()
	_waiting_reply = true
	_sequence_left = actions.size()
	_refresh_main_button()
	for action: Variant in actions:
		play_requested.emit(action)


## Oublie le choix en cours (annuler ne coûte rien : rien n'a été joué).
func _end_choice() -> void:
	if _session == null:
		return
	_stale_previews += _session.pending
	_session = null
	_board.show_candidates([] as Array[Vector2i])
	_board.show_chosen([] as Array[Vector2i])
	_arsenal.select(-1)
	_apply_mode()


# --- Les gestes sur le plateau ---

func _on_cell_clicked(cell: Vector2i) -> void:
	if _view == null or _waiting_reply:
		return
	if _session != null:
		_click_during_choice(cell)
	elif _hare_from != ViewData.NO_CELL:
		_click_hare_destination(cell)
	else:
		_click_cell(cell)


func _click_during_choice(cell: Vector2i) -> void:
	if _session.is_waiting():
		return
	var action: Variant = _session.choices.action_at(cell, CHOOSE)
	if action == null:
		_board.flash_refusal(cell)
	else:
		_session.choose(action)


func _click_cell(cell: Vector2i) -> void:
	var kinds := _kinds_at(cell)
	if kinds.is_empty():
		_board.flash_refusal(cell)
	elif kinds == [PLACE]:
		_place(cell)
	elif kinds == [CHOOSE]:
		# Une partie reprise au milieu du choix d'une carte : chaque choix part tel quel.
		_request(_view.action_at(cell, CHOOSE))
	else:
		_menu_cell = cell
		_action_menu.open_at(_board.screen_point(cell), kinds)


## Les sortes de coups permis sur cette case, chacune une fois, dans l'ordre du serveur.
func _kinds_at(cell: Vector2i) -> Array[String]:
	var kinds: Array[String] = []
	for action: Variant in _view.actions_at(cell):
		var kind := ViewData.action_kind(action)
		if not kinds.has(kind):
			kinds.append(kind)
	return kinds


func _on_menu_option(kind: String) -> void:
	if _view == null or _waiting_reply:
		return
	if kind == PLACE:
		_place(_menu_cell)
	elif kind == HARE_STEP:
		_hare_from = _menu_cell
		_board.show_candidates(_hare_destinations(_hare_from))
		set_notice(Labels.HARE_DESTINATION)
	else:
		var action: Variant = _view.action_at(_menu_cell, kind)
		if action != null:
			_request(action)


func _hare_destinations(from: Vector2i) -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	for action: Variant in _view.actions_at(from):
		var destination := ViewData.step_destination(action)
		if destination != ViewData.NO_CELL:
			cells.append(destination)
	return cells


func _click_hare_destination(cell: Vector2i) -> void:
	for action: Variant in _view.actions_at(_hare_from):
		if ViewData.step_destination(action) == cell:
			_leave_hare_step()
			_request(action)
			return
	_leave_hare_step()
	_board.flash_refusal(cell)


func _leave_hare_step() -> void:
	if _hare_from == ViewData.NO_CELL:
		return
	_hare_from = ViewData.NO_CELL
	_board.show_candidates([] as Array[Vector2i])
	set_notice("")


func _place(cell: Vector2i) -> void:
	var action: Variant = _view.action_at(cell, PLACE)
	if action == null:
		return
	_board.show_mark(ViewData.NO_CELL)
	_board.show_ghost(cell, _view.viewer(), true)
	_request(action)


func _on_cell_hovered(cell: Vector2i) -> void:
	_hovered = cell
	if _view == null or _waiting_reply:
		return
	if _session != null or _hare_from != ViewData.NO_CELL:
		# Le survol (aimanté) désigne la case proposée qui serait choisie.
		_board.show_mark(cell)
		_board.show_ghost(ViewData.NO_CELL, "", false)
		return
	var placeable := _view.action_at(cell, PLACE) != null
	var shown := cell if placeable else ViewData.NO_CELL
	_board.show_mark(shown)
	_board.show_ghost(shown, _view.viewer(), false)


func _unhandled_input(event: InputEvent) -> void:
	# Échap (PC) annule le choix en cours ou le pas du Lièvre.
	if event.is_action_pressed(&"ui_cancel") and (_session != null or _hare_from != ViewData.NO_CELL):
		_end_choice()
		_leave_hare_step()
		get_viewport().set_input_as_handled()


func _request(action: Variant) -> void:
	_waiting_reply = true
	_refresh_main_button()
	play_requested.emit(action)


# --- Le bouton d'action ---

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


# --- Le bandeau du tour ---

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


# --- La construction et la mise en page ---

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
	for item: Control in [_opponent_panel, _my_panel, _pill, _notice, _action_slot, _board_area, _arsenal, _choice_panel]:
		if item.get_parent() != null:
			item.get_parent().remove_child(item)
	if _layout != null:
		_layout.queue_free()

	if landscape:
		var row := Ui.hbox(Look.GAP_XL)
		var left := Ui.vbox(Look.GAP_M)
		left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		left.add_child(_board_area)
		left.add_child(_arsenal)
		left.add_child(_choice_panel)
		row.add_child(left)
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
		column.add_child(_arsenal)
		column.add_child(_my_panel)
		column.add_child(_action_slot)
		column.add_child(_choice_panel)
		_my_panel.set_identity_visible(false)
		_layout = Ui.page(column)
	add_child(_layout)
	_apply_mode()
	# Le bandeau et le menu de case passent devant la nouvelle mise en page.
	if _action_menu.get_parent() == null:
		add_child(_action_menu)
	move_child(_banner, -1)
	move_child(_action_menu, -1)


## Montre l'arsenal et la barre, ou, pendant le choix d'une carte, le panneau de choix à
## leur place.
func _apply_mode() -> void:
	var choosing := _session != null
	_choice_panel.visible = choosing
	_arsenal.visible = not choosing
	_action_slot.visible = not choosing
	_my_panel.visible = _landscape or not choosing


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
