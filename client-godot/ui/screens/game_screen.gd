class_name GameScreen
extends Screen
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
##
## Le tour adverse : une carte adverse est annoncée en haut de l'écran ; sa cache révélée
## couvre l'écran jusqu'à un toucher (la file des vues attend) ; une fenêtre de réaction
## remplace l'arsenal (« Réagir ? ») ; pendant que l'adversaire décide, une pastille le dit
## (« Le bot réfléchit… »).
##
## Le journal des coups : en paysage, une colonne à gauche ; en portrait, une page qu'ouvre
## le bouton à côté du tour.
##
## Les vues passent par une file : chacune attend que la précédente ait fini de s'animer
## (le bot joue sans pause). Tant que des vues attendent, celle qui est affichée est
## périmée : aucun geste n'est pris.

## Le joueur demande ce coup, recopié de la vue.
signal play_requested(action: Variant)
## Le joueur revient à l'accueil : la flèche en haut à gauche (confirmée pendant la partie),
## ou le panneau de fin.
signal home_requested
## La partie est finie et le joueur en veut une autre.
signal replay_requested

## Les coups sans case qui deviennent l'action principale quand ils sont permis.
const MAIN_ACTIONS: Array[String] = ["EndTurn", "Pass"]
const HOURGLASS := "sablier"
const PLACE := "Place"
const HARE_STEP := "HareStep"
const CHOOSE := "Choose"
## Le texte de chaque avertissement de la session (`Session.Warning`).
const WARNING_TEXTS := {
	Session.Warning.OFFLINE: Labels.RECONNECTING,
	Session.Warning.BOT_DOWN: Labels.BOT_FAILED,
	Session.Warning.FRIEND_LEFT: Labels.FRIEND_LEFT,
}

## Le nom affiché de l'adversaire, s'il s'agit d'un bot, et le catalogue des cartes : ceux
## de la session.
var opponent_name: String:
	get:
		return session.opponent_name
var opponent_is_bot: bool:
	get:
		return session.against_bot()
var catalog: CardCatalog:
	get:
		return session.catalog

var _board: Board3D
var _view: ViewData
## Vrai tant que la connexion au serveur est ouverte.
var _online := true
## Le catalogue avec lequel l'arsenal a été dessiné.
var _shown_catalog: CardCatalog
var _waiting_reply := false
## Les coups d'une carte complète envoyés et pas encore confirmés : on n'affiche pas les
## vues intermédiaires (le choix du lanceur, des cibles), seulement la dernière, avec les
## événements de toutes.
var _sequence_left := 0
var _carried_events: Array = []
## Les vues reçues qui attendent leur tour (`[vue, événements]`), et la fin de l'animation
## de la vue affichée.
var _queue: Array[Array] = []
var _playing: Tween
var _landscape := false
var _layout: Control
var _hovered := ViewData.NO_CELL
var _was_my_turn := false
## Les pions du dernier coup adverse, par identité.
var _last_opponent_pions: Array[int] = []
## Le choix d'une carte en cours, ou `null`.
var _choice: CardChoice
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
var _journal := GameJournal.new()
var _attack_banner: AttackBanner
var _reveal: RevealOverlay
var _reaction: ReactionPanel
var _thinking: PanelContainer
var _thinking_label: Label
var _end_panel: EndPanel
## Vrai quand le panneau de fin est montré (après la dernière animation).
var _end_shown := false
var _quit_dialog: ConfirmDialog
var _journal_panel: JournalPanel
var _journal_button: Button
var _back_button: Button
## En portrait, la page du journal, par-dessus tout ; et la ligne du tour qui porte son bouton.
var _journal_page: PanelContainer
var _journal_page_column: VBoxContainer
var _top_row: HBoxContainer


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
	_build_journal()
	_build_opponent_turn()
	_end_panel = EndPanel.new()
	_end_panel.home_requested.connect(home_requested.emit)
	_end_panel.replay_requested.connect(replay_requested.emit)
	_quit_dialog = ConfirmDialog.new(Labels.QUIT_TITLE, Labels.QUIT_TEXT, Labels.STAY, Labels.QUIT)
	_quit_dialog.confirmed.connect(home_requested.emit)
	add_child(_quit_dialog)

	_board.cell_clicked.connect(_on_cell_clicked)
	_board.cell_hovered.connect(_on_cell_hovered)
	_board.empty_tapped.connect(_leave_hare_step)
	resized.connect(_on_resized)
	_arrange(Ui.is_landscape(self))
	_refresh_main_button()


func _exit_tree() -> void:
	_board.cell_clicked.disconnect(_on_cell_clicked)
	_board.cell_hovered.disconnect(_on_cell_hovered)
	_board.empty_tapped.disconnect(_leave_hare_step)


## Affiche cette vue et ce qui s'est passé depuis la précédente, dès que les vues reçues
## avant elle ont fini de s'animer.
func on_view(view: ViewData, events: Array) -> void:
	_carried_events.append_array(events)
	if _sequence_left > 0:
		_sequence_left -= 1
		if _sequence_left > 0:
			return
	_queue.append([view, _carried_events])
	_carried_events = []
	if _playing == null or not _playing.is_running():
		_play_next()


## Affiche la vue suivante de la file, puis attend la fin de son animation.
func _play_next() -> void:
	while not _queue.is_empty():
		# La cache révélée attend un toucher : la suite viendra après.
		if _reveal.visible:
			return
		var next: Array = _queue.pop_front()
		var duration := _apply_view(next[0], next[1])
		if duration > 0.0:
			_playing = create_tween()
			_playing.tween_interval(duration)
			_playing.tween_callback(_play_next)
			return


## Vrai si des vues attendent leur tour : celle qui est affichée est périmée.
func _is_behind() -> bool:
	return not _queue.is_empty()


func _apply_view(view: ViewData, events: Array) -> float:
	var turn_before := _view.turn() if _view != null else view.turn()
	var lines := _journal.lines(events, view, turn_before, catalog)
	_journal_panel.set_names(view.viewer(), {view.viewer(): Labels.YOU, view.opponent(): opponent_name})
	_journal_panel.add_entries(lines)
	# Le coup adverse de cette vue, raconté : ce à quoi on réagit (vide à la reprise).
	var opponent_act := ""
	for line in lines:
		if line.get("acts", false) and line.get("player") == view.opponent():
			opponent_act = "%s %s" % [opponent_name, line.get("text", "")]
	_view = view
	_waiting_reply = false
	_end_choice()
	_leave_hare_step()
	_action_menu.close()
	var me := view.viewer()
	var opponent_placed: Array[int] = []
	for pion in GameEvents.placed_pions(events):
		if pion.get("owner") != me:
			opponent_placed.append(pion.get("id", -1))
	if not opponent_placed.is_empty():
		_last_opponent_pions = opponent_placed

	var duration := _board.show_view(view, events)
	_play_sound(events)
	_board.show_last(_last_opponent_pions)
	_announce_opponent_cards(view, events)
	_opponent_panel.show_side(view, view.opponent(), opponent_name, Labels.THINKS if opponent_is_bot else Labels.PLAYS)
	_my_panel.show_side(view, me, Labels.YOU)
	_arsenal.show_arsenal(view, me, catalog)
	_shown_catalog = catalog
	var my_turn := view.current_player() == me and not view.is_over()
	_pill_label.text = Labels.GAME_OVER if view.is_over() else Labels.turn_pill(view.turn(), my_turn)
	_pill_icon.visible = not my_turn and not view.is_over()
	_refresh_main_button()
	_on_cell_hovered(_hovered)
	if _is_reacting():
		_reaction.show_window(view, opponent_act, catalog)
	if view.is_over() and not _end_shown:
		# La fin se montre une fois la dernière animation jouée : la ligne gagnante s'allume,
		# puis le panneau de fin prend la place de l'arsenal.
		_end_panel.show_end(view)
		var reveal := create_tween()
		reveal.tween_interval(duration + Look.END_PANEL_DELAY)
		reveal.tween_callback(_show_end.bind(view))
	_apply_mode()
	_refresh_thinking()
	_refresh_warning()

	# Le bandeau n'annonce que le début de son tour, pas une réaction pendant celui de
	# l'adversaire.
	var my_main_turn := my_turn and view.decision_player() == me
	if my_main_turn and not _was_my_turn:
		_announce_turn()
	_was_my_turn = my_main_turn
	return duration


# --- Le tour adverse ---

## Les cartes que l'adversaire vient de lancer : annoncées en haut de l'écran ; sa cache,
## révélée en grand.
func _announce_opponent_cards(view: ViewData, events: Array) -> void:
	var opponent := view.opponent()
	for event: Variant in events:
		var data := GameEvents.payload(event)
		if data.get("player") != opponent or not (data.get("attack") is Dictionary):
			continue
		var attack: Dictionary = data["attack"]
		var cost: int = view.stats_of(attack).get("cost", 0)
		match GameEvents.kind(event):
			"AttackUsed":
				var launcher: Variant = data.get("launcher")
				var cell := Labels.cell_name(ViewData.to_cell(launcher)) if launcher is Dictionary else "?"
				_attack_banner.announce(attack, opponent_name, cell, catalog, cost)
			"CardRevealed":
				_reveal.reveal(attack, opponent_name, catalog, cost)


func _show_end(view: ViewData) -> void:
	_end_shown = true
	_board.show_winning_line(view.winning_line())
	_apply_mode()


## Un son pour ce qui vient de se passer, le plus fort seulement : un kill, une attaque,
## une pose.
func _play_sound(events: Array) -> void:
	var kinds: Array[String] = []
	for event: Variant in events:
		kinds.append(GameEvents.kind(event))
	if "PionKilled" in kinds:
		Sounds.play("kill")
	elif "AttackUsed" in kinds:
		Sounds.play("attack")
	elif "PionPlaced" in kinds:
		Sounds.play("place")


## La flèche : pendant la partie, on confirme avant de la quitter.
func _on_back() -> void:
	if _view != null and _view.is_over():
		home_requested.emit()
	else:
		_quit_dialog.ask()


## La cache révélée est fermée : la file des vues reprend.
func _on_reveal_dismissed() -> void:
	if _playing == null or not _playing.is_running():
		_play_next()


## Vrai si c'est à soi de réagir au coup adverse, et qu'aucune réaction n'est en cours de
## choix.
func _is_reacting() -> bool:
	if _view == null or _choice != null:
		return false
	return _view.decision_kind() == "React" and _view.decision_player() == _view.viewer()


func _on_pass() -> void:
	if _view == null or _waiting_reply or _is_behind():
		return
	var pass_action: Variant = _view.simple_action("Pass")
	if pass_action != null:
		_request(pass_action)


## La pastille « Le bot réfléchit… », tant que l'adversaire décide et que la vue affichée
## est la dernière.
func _refresh_thinking() -> void:
	var deciding := _view != null and not _view.is_over() and _view.decision_player() == _view.opponent()
	_thinking.visible = deciding and not _is_behind()
	_thinking_label.text = Labels.BOT_THINKING if opponent_is_bot else Labels.OPPONENT_PLAYING


## La réponse du serveur à l'aperçu du choix en cours.
func on_preview(view: ViewData) -> void:
	if _stale_previews > 0:
		_stale_previews -= 1
	elif _choice != null:
		_choice.on_preview(view)


func uses_board() -> bool:
	return true


## Le serveur a refusé la dernière demande : le joueur peut rejouer. La vue est à redemander
## si le refus dit qu'elle est en retard, ou si une carte était en train de partir.
func on_refused(category: ServerErrors.Category, _message: String) -> Reply:
	var was_sending := _sequence_left > 0
	_waiting_reply = false
	_sequence_left = 0
	_carried_events = []
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_end_choice()
	_refresh_main_button()
	_on_cell_hovered(_hovered)
	if was_sending or category == ServerErrors.Category.STALE_VIEW:
		return Reply.RESYNC
	return Reply.HANDLED


## La connexion est revenue, ou s'est coupée : l'avertissement le dit.
func on_connection(status: ServerLink.Status) -> void:
	_online = status == ServerLink.Status.ONLINE
	if not _online:
		_lose_connection()
	_refresh_warning()


## L'ami est arrivé ou parti, le bot ne joue plus, ou le catalogue est arrivé (les noms
## des cartes de l'arsenal changent alors).
func on_session_changed() -> void:
	_refresh_warning()
	if catalog != _shown_catalog and _view != null and is_node_ready():
		_shown_catalog = catalog
		_arsenal.show_arsenal(_view, _view.viewer(), catalog)


## L'avertissement de la session (connexion, bot, ami) sous le tour, ou rien.
func _refresh_warning() -> void:
	var text: String = WARNING_TEXTS.get(session.warning(_online), "")
	set_notice(text, text != "")


## La connexion est coupée : ce qui attendait une réponse (un coup, une carte qui partait,
## un aperçu) n'en aura pas. La vue envoyée à la reprise fera foi.
func _lose_connection() -> void:
	_waiting_reply = false
	_sequence_left = 0
	_carried_events = []
	_end_choice()
	_stale_previews = 0
	_leave_hare_step()
	_action_menu.close()
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_thinking.visible = false
	_refresh_main_button()


## Un message sous le tour, ou rien : discret (le pas du Lièvre), ou en braise pour un
## avertissement (`warning` : reconnexion, ami parti, bot qui ne répond plus).
func set_notice(text: String, warning: bool = false) -> void:
	_notice.text = text
	_notice.theme_type_variation = ThemeBuilder.THREAT_LABEL if warning else ThemeBuilder.MUTED_LABEL
	_notice.visible = text != ""


# --- Le choix d'une carte ---

func _on_card_chosen(slot: int) -> void:
	if _view == null or _waiting_reply or _choice != null or _is_behind():
		return
	var use_attack: Variant = _view.use_attack(slot)
	if use_attack == null:
		return
	_leave_hare_step()
	_action_menu.close()
	_board.show_mark(ViewData.NO_CELL)
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_choice = CardChoice.new(slot, use_attack)
	_choice.changed.connect(_refresh_choice)
	_choice.complete.connect(_play_sequence)
	_arsenal.select(slot)
	_apply_mode()
	_refresh_choice()


## Montre les choix suivants : les cases proposées, celles déjà choisies, et le panneau.
func _refresh_choice() -> void:
	if _choice == null:
		return
	var choices := _choice.choices
	_board.show_candidates(choices.cells_of(CHOOSE) if choices != null else ([] as Array[Vector2i]))
	_board.show_chosen(_choice.chosen_cells())
	var slot: Variant = _view.arsenal(_view.viewer())[_choice.slot]
	var attack: Dictionary = slot.get("attack", {}) if slot is Dictionary else {}
	_choice_panel.show_choice(_choice, attack, catalog, _view.stats_of(attack).get("cost", 0))


## Choisit le coup sans case de ce nom (`FinishAttack`, `ChooseEnemyTokens`), s'il est proposé.
func _choose_simple(kind: String) -> void:
	if _choice == null or _choice.is_waiting():
		return
	var action: Variant = _choice.choices.simple_action(kind)
	if action != null:
		_choice.choose(action)


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
	if _choice == null:
		return
	_stale_previews += _choice.pending
	_choice = null
	_board.show_candidates([] as Array[Vector2i])
	_board.show_chosen([] as Array[Vector2i])
	_arsenal.select(-1)
	_apply_mode()


# --- Les gestes sur le plateau ---

func _on_cell_clicked(cell: Vector2i) -> void:
	if _view == null or _waiting_reply or _is_behind():
		return
	if _choice != null:
		_click_during_choice(cell)
	elif _hare_from != ViewData.NO_CELL:
		_click_hare_destination(cell)
	else:
		_click_cell(cell)


func _click_during_choice(cell: Vector2i) -> void:
	if _choice.is_waiting():
		return
	var action: Variant = _choice.choices.action_at(cell, CHOOSE)
	if action == null:
		_board.flash_refusal(cell)
	else:
		_choice.choose(action)


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
	if _view == null or _waiting_reply or _is_behind():
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
	_refresh_warning()


func _place(cell: Vector2i) -> void:
	var action: Variant = _view.action_at(cell, PLACE)
	if action == null:
		return
	_board.show_mark(ViewData.NO_CELL)
	_board.show_ghost(cell, _view.viewer(), true)
	_request(action)


func _on_cell_hovered(cell: Vector2i) -> void:
	_hovered = cell
	if _view == null or _waiting_reply or _is_behind():
		return
	if _choice != null or _hare_from != ViewData.NO_CELL:
		# Le survol (aimanté) désigne la case proposée qui serait choisie.
		_board.show_mark(cell)
		_board.show_ghost(ViewData.NO_CELL, "", false)
		return
	var placeable := _view.action_at(cell, PLACE) != null
	var shown := cell if placeable else ViewData.NO_CELL
	_board.show_mark(shown)
	_board.show_ghost(shown, _view.viewer(), false)


func _unhandled_input(event: InputEvent) -> void:
	# Échap (PC) ferme la page du journal, ou annule le choix en cours ou le pas du Lièvre.
	if event.is_action_pressed(&"ui_cancel") and _journal_page.visible:
		_show_journal_page(false)
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed(&"ui_cancel") and (_choice != null or _hare_from != ViewData.NO_CELL):
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
				_show_main_button(Labels.ACTION_LABELS[kind], not _waiting_reply and not _is_behind())
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
	if _is_behind():
		return
	if _view != null and _view.is_over():
		home_requested.emit()
	elif _main_action != null:
		_request(_main_action)


# --- Le bandeau du tour ---

## Le bandeau « À toi ! » au milieu du plateau, et une brève vibration sur téléphone.
func _announce_turn() -> void:
	Settings.vibrate(Look.TURN_VIBRATION_MS)
	Sounds.play("turn")
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


func _build_journal() -> void:
	_journal_panel = JournalPanel.new()
	_journal_button = Ui.icon_button("journal", Labels.JOURNAL_TITLE)
	_journal_button.pressed.connect(_show_journal_page.bind(true))
	_back_button = Ui.icon_button("retour", Labels.BACK)
	_back_button.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	_back_button.pressed.connect(_on_back)
	_top_row = Ui.hbox(0)
	_journal_page = PanelContainer.new()
	_journal_page.theme_type_variation = ThemeBuilder.OVERLAY
	_journal_page.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	# La page arrête les touchers : le plateau, dessous, n'en reçoit aucun.
	_journal_page.mouse_filter = Control.MOUSE_FILTER_STOP
	_journal_page.visible = false
	_journal_page_column = Ui.vbox(Look.GAP_M)
	var close := Ui.button(Labels.CLOSE, ThemeBuilder.SECONDARY_BUTTON)
	close.pressed.connect(_show_journal_page.bind(false))
	_journal_page_column.add_child(close)
	_journal_page.add_child(Ui.page(_journal_page_column))
	add_child(_journal_page)


## Ouvre ou ferme la page du journal (portrait).
func _show_journal_page(shown: bool) -> void:
	_journal_page.visible = shown and not _landscape


func _build_opponent_turn() -> void:
	_attack_banner = AttackBanner.new()
	_attack_banner.set_anchors_and_offsets_preset(Control.PRESET_TOP_WIDE)
	var insets := Ui.safe_insets()
	_attack_banner.offset_left = Look.PAGE_MARGIN + insets.position.x
	_attack_banner.offset_right = -Look.PAGE_MARGIN - insets.size.x
	_attack_banner.offset_top = Look.PAGE_MARGIN + insets.position.y
	add_child(_attack_banner)
	_reveal = RevealOverlay.new()
	_reveal.dismissed.connect(_on_reveal_dismissed)
	add_child(_reveal)
	_reaction = ReactionPanel.new()
	_reaction.pass_requested.connect(_on_pass)
	_reaction.reaction_chosen.connect(_on_card_chosen)
	_thinking = PanelContainer.new()
	_thinking.theme_type_variation = ThemeBuilder.PILL
	_thinking.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_thinking.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	var row := Ui.hbox(Look.GAP_S)
	row.add_child(Ui.icon(HOURGLASS, Look.GOLD))
	_thinking_label = Ui.label("", ThemeBuilder.PILL_LABEL)
	row.add_child(_thinking_label)
	_thinking.add_child(row)
	_thinking.visible = false


func _build_banner() -> void:
	_banner = PanelContainer.new()
	_banner.theme_type_variation = ThemeBuilder.BANNER
	_banner.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_banner.visible = false
	_banner.add_child(Ui.label(Labels.YOUR_TURN_BANNER, ThemeBuilder.BANNER_LABEL))
	add_child(_banner)


func _on_resized() -> void:
	var landscape := Ui.is_landscape(self)
	if landscape != _landscape:
		_arrange(landscape)
	_frame_board()


## Range les éléments pour cette orientation. Les éléments restent les mêmes : seules
## leurs boîtes changent.
func _arrange(landscape: bool) -> void:
	_landscape = landscape
	for item: Control in [_opponent_panel, _my_panel, _pill, _thinking, _notice, _action_slot, _board_area, _arsenal, _choice_panel, _reaction, _end_panel, _journal_panel, _journal_button, _back_button]:
		if item.get_parent() != null:
			item.get_parent().remove_child(item)
	if _top_row.get_parent() != null:
		_top_row.get_parent().remove_child(_top_row)
	for old in _top_row.get_children():
		old.queue_free()
	if _layout != null:
		_layout.queue_free()
	_show_journal_page(false)

	if landscape:
		var row := Ui.hbox(Look.GAP_XL)
		_journal_panel.custom_minimum_size.x = Look.JOURNAL_WIDTH
		row.add_child(_journal_panel)
		var left := Ui.vbox(Look.GAP_M)
		left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		left.add_child(_board_area)
		left.add_child(_arsenal)
		left.add_child(_choice_panel)
		left.add_child(_reaction)
		left.add_child(_end_panel)
		row.add_child(left)
		var column := Ui.vbox(Look.GAP_M)
		column.custom_minimum_size.x = Look.SIDE_COLUMN_WIDTH
		column.add_child(_back_button)
		column.add_child(_pill)
		column.add_child(_thinking)
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
		# La pastille du tour reste au centre ; le bouton du journal se range à droite.
		var right := Ui.hbox(0)
		right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		right.alignment = BoxContainer.ALIGNMENT_END
		right.add_child(_journal_button)
		var left := Ui.hbox(0)
		left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		left.add_child(_back_button)
		_top_row.add_child(left)
		_top_row.add_child(_pill)
		_top_row.add_child(right)
		column.add_child(_top_row)
		column.add_child(_thinking)
		column.add_child(_notice)
		column.add_child(_board_area)
		column.add_child(_arsenal)
		column.add_child(_my_panel)
		column.add_child(_action_slot)
		column.add_child(_choice_panel)
		column.add_child(_reaction)
		column.add_child(_end_panel)
		_my_panel.set_identity_visible(false)
		_journal_panel.custom_minimum_size.x = 0.0
		_journal_page_column.add_child(_journal_panel)
		_journal_page_column.move_child(_journal_panel, 0)
		_layout = Ui.page(column)
	add_child(_layout)
	_apply_mode()
	# Le bandeau et le menu de case passent devant la nouvelle mise en page.
	if _action_menu.get_parent() == null:
		add_child(_action_menu)
	move_child(_banner, -1)
	move_child(_action_menu, -1)
	move_child(_journal_page, -1)
	move_child(_attack_banner, -1)
	move_child(_reveal, -1)
	move_child(_quit_dialog, -1)


## Montre l'arsenal et la barre ; à leur place, pendant le choix d'une carte, le panneau de
## choix, et quand on peut réagir au coup adverse, la fenêtre de réaction.
func _apply_mode() -> void:
	var choosing := _choice != null
	var reacting := _is_reacting()
	var ending := _end_shown
	_choice_panel.visible = choosing and not ending
	_reaction.visible = reacting and not ending
	_end_panel.visible = ending
	var busy := choosing or reacting or ending
	_arsenal.visible = not busy
	_action_slot.visible = not busy
	_my_panel.visible = _landscape or not busy


func _frame_board() -> void:
	if not _board_area.is_inside_tree():
		return
	var area := _board_area.get_global_rect()
	# En portrait, les marges de la page servent au texte, pas au plateau : il va d'un bord
	# de l'écran à l'autre. En paysage, il reste entre le journal et la colonne de droite.
	var left := area.position.x if _landscape else 0.0
	var right := area.end.x if _landscape else get_viewport_rect().size.x
	_board.frame(Rect2(left, area.position.y, right - left, area.size.y))
	if _banner.visible:
		_center_banner()
