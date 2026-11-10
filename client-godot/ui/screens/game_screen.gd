class_name GameScreen
extends Screen
## L'écran de partie : l'interface autour du plateau 3D. Il ne calcule rien : ce qu'il
## montre vient de la vue, et un geste ne devient un coup que s'il est dans `legal_actions`
## de la vue (ou d'un aperçu, pendant le choix d'une carte), demandé tel quel
## (`play_requested`).
##
## Il est fait de trois morceaux, qu'il relie :
## - la mise en page (`GameLayout`) : ses éléments, en portrait et en paysage ;
## - les gestes sur le plateau (`BoardGestures`) : pose, menu de case, pas du Lièvre ;
## - la file des vues (`ViewQueue`) : chaque vue attend la fin de l'animation de la
##   précédente ; tant que des vues attendent, aucun geste n'est pris.
##
## Il garde pour lui : montrer une vue, le choix d'une carte (lanceur, puis cibles, chaque
## étape étant un aperçu du serveur ; une fois complète, la carte part coup par coup), le
## tour adverse (annonce d'une carte, cache révélée, « Réagir ? », « Le bot réfléchit… »),
## le bouton d'action, la fin et la carte qu'elle fait gagner.

## Le joueur demande ce coup, recopié de la vue.
signal play_requested(action: Variant)
## Le joueur revient à l'accueil : la flèche en haut à gauche (confirmée pendant la partie),
## ou le panneau de fin.
signal home_requested
## La partie est finie et le joueur en veut une autre.
signal replay_requested
## La connexion est perdue et le joueur demande d'essayer tout de suite.
signal reconnect_requested

## Les coups sans case qui deviennent l'action principale quand ils sont permis.
const MAIN_ACTIONS: Array[String] = ["EndTurn", "Pass"]

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
var _ui := GameLayout.new()
var _queue := ViewQueue.new()
var _gestures: BoardGestures
var _journal := GameJournal.new()
var _view: ViewData
## Vrai tant que la connexion au serveur est ouverte.
var _online := true
## Le catalogue avec lequel l'arsenal a été dessiné.
var _shown_catalog: CardCatalog
var _waiting_reply := false
var _was_my_turn := false
## Les pions du dernier coup adverse, par identité.
var _last_opponent_pions: Array[int] = []
## Le choix d'une carte en cours, ou `null`.
var _choice: CardChoice
## Les réponses à venir d'aperçus demandés par un choix abandonné : on les ignore (les
## réponses arrivent dans l'ordre des demandes).
var _stale_previews := 0
var _main_action: Variant = null
## Vrai quand le panneau de fin est montré (après la dernière animation).
var _end_shown := false
## La carte gagnée, annoncée avant que la fin soit montrée : elle attend la fin.
var _reward: Dictionary = {}
## Vrai depuis que le bot ne répond plus, jusqu'à ce qu'il joue (la vue n'attend plus sa
## décision) : le serveur le relance toutes les quelques secondes, et chaque relance ratée
## ne doit pas rouvrir sa fenêtre.
var _bot_stuck := false
## Vrai quand le joueur a choisi d'attendre le bot muet : sa fenêtre ne revient pas tant
## qu'il ne joue pas, un message le dit sous le tour.
var _waiting_for_bot := false


func _init(board: Board3D) -> void:
	_board = board


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_ui.arsenal.card_chosen.connect(_on_card_chosen)
	_ui.arsenal.card_details.connect(_show_card_sheet)
	_ui.choice_panel.cancel_requested.connect(_end_choice)
	_ui.choice_panel.finish_requested.connect(_choose_simple.bind("FinishAttack"))
	_ui.choice_panel.tokens_requested.connect(_choose_simple.bind("ChooseEnemyTokens"))
	_ui.reaction.pass_requested.connect(_on_pass)
	_ui.reaction.reaction_chosen.connect(_on_card_chosen)
	_ui.end_panel.home_requested.connect(home_requested.emit)
	_ui.end_panel.replay_requested.connect(replay_requested.emit)
	_ui.quit_dialog.confirmed.connect(home_requested.emit)
	_ui.connection_dialog.confirmed.connect(reconnect_requested.emit)
	_ui.connection_dialog.declined.connect(home_requested.emit)
	_ui.bot_dialog.declined.connect(home_requested.emit)
	_ui.bot_dialog.confirmed.connect(func() -> void:
		_waiting_for_bot = true
		_refresh_warning())
	_ui.reveal.dismissed.connect(_queue.resume)
	_ui.main_pressed.connect(_on_main_button)
	_ui.back_pressed.connect(_on_back)
	_ui.board_area_changed.connect(_frame_board)
	add_child(_ui)

	_queue.show = _apply_view
	# La cache révélée attend un toucher : la file reprend quand elle se ferme.
	_queue.blocked = func() -> bool: return _ui.reveal.visible
	add_child(_queue)

	_gestures = BoardGestures.new(_board, _ui.action_menu)
	_gestures.can_act = func() -> bool: return not _waiting_reply and not _queue.is_behind()
	_gestures.requested.connect(_request)
	_gestures.hare_step_changed.connect(func(active: bool) -> void:
		if active:
			_ui.set_notice(Labels.HARE_DESTINATION)
		else:
			_refresh_warning())
	_gestures.attach()
	_refresh_main_button()


func _exit_tree() -> void:
	_gestures.detach()


func uses_board() -> bool:
	return true


## Affiche cette vue et ce qui s'est passé depuis la précédente, dès que les vues reçues
## avant elle ont fini de s'animer.
func on_view(view: ViewData, events: Array) -> void:
	_queue.push(view, events)


## La réponse du serveur à l'aperçu du choix en cours.
func on_preview(view: ViewData) -> void:
	if _stale_previews > 0:
		_stale_previews -= 1
	elif _choice != null:
		_choice.on_preview(view)


## Le serveur a refusé la dernière demande : le joueur peut rejouer. La vue est à redemander
## si le refus dit qu'elle est en retard, ou si une carte était en train de partir.
func on_refused(category: ServerErrors.Category, _message: String) -> Reply:
	var was_sending := _queue.is_sending()
	_waiting_reply = false
	_queue.cancel_sequence()
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_end_choice()
	_refresh_main_button()
	_gestures.refresh_hover()
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
		_ui.arsenal.show_arsenal(_view, _view.viewer(), catalog)


# --- Une vue ---

## Montre cette vue (la file l'appelle) et rend la durée de son animation.
func _apply_view(view: ViewData, events: Array) -> float:
	var turn_before := _view.turn() if _view != null else view.turn()
	var lines := _journal.lines(events, view, turn_before, catalog)
	_ui.journal_panel.set_names(view.viewer(), {view.viewer(): Labels.YOU, view.opponent(): opponent_name})
	_ui.journal_panel.add_entries(lines)
	# Le coup adverse de cette vue, raconté : ce à quoi on réagit (vide à la reprise).
	var opponent_act := ""
	for line in lines:
		if line.get("acts", false) and line.get("player") == view.opponent():
			opponent_act = "%s %s" % [opponent_name, line.get("text", "")]
	_view = view
	_gestures.view = view
	# Le bot a joué (ce n'est plus à lui de décider) : il ne bloque plus la partie.
	if view.is_over() or view.decision_player() != view.opponent():
		_bot_stuck = false
		_waiting_for_bot = false
	_waiting_reply = false
	_end_choice()
	_gestures.reset()
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
	_ui.opponent_panel.show_side(view, view.opponent(), opponent_name, Labels.THINKS if opponent_is_bot else Labels.PLAYS, catalog)
	_ui.my_panel.show_side(view, me, Labels.YOU)
	_ui.arsenal.show_arsenal(view, me, catalog)
	_shown_catalog = catalog
	var my_turn := view.current_player() == me and not view.is_over()
	_ui.turn_pill.show_status(Labels.GAME_OVER if view.is_over() else Labels.turn_pill(view.turn(), my_turn), not my_turn and not view.is_over())
	_refresh_main_button()
	_gestures.refresh_hover()
	if _is_reacting():
		_ui.reaction.show_window(view, opponent_act, catalog)
	if view.is_over() and not _end_shown:
		# La fin se montre une fois la dernière animation jouée : la ligne gagnante s'allume,
		# puis le panneau de fin prend la place de l'arsenal.
		_ui.end_panel.show_end(view, opponent_name)
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


func _show_end(view: ViewData) -> void:
	_end_shown = true
	_board.show_winning_line(view.winning_line())
	_apply_mode()
	if not _reward.is_empty():
		_ui.reveal.reward(_reward, catalog)
		_reward = {}


## La carte gagnée se montre par-dessus la fin : tout de suite si elle est déjà montrée,
## sinon avec elle (le serveur l'annonce juste après le dernier coup).
func on_reward(card: Dictionary) -> void:
	if _end_shown:
		_ui.reveal.reward(card, catalog)
	else:
		_reward = card


## Montre l'arsenal et la barre, ou à leur place le panneau du moment (choix, réaction, fin).
func _apply_mode() -> void:
	_ui.apply_mode(_choice != null, _is_reacting(), _end_shown)


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


## Le bandeau « À toi ! » au milieu du plateau, un son, et une brève vibration sur téléphone.
func _announce_turn() -> void:
	Settings.vibrate(Look.TURN_VIBRATION_MS)
	Sounds.play("turn")
	_ui.play_turn_banner()


func _frame_board() -> void:
	var rect := _ui.board_rect()
	if rect.has_area():
		_board.frame(rect)


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
				_ui.attack_banner.announce(attack, opponent_name, cell, catalog, cost)
			"CardRevealed":
				_ui.reveal.reveal(attack, opponent_name, catalog, cost)


## Vrai si c'est à soi de réagir au coup adverse, et qu'aucune réaction n'est en cours de
## choix.
func _is_reacting() -> bool:
	if _view == null or _choice != null:
		return false
	return _view.decision_kind() == "React" and _view.decision_player() == _view.viewer()


func _on_pass() -> void:
	if _view == null or _waiting_reply or _queue.is_behind():
		return
	var pass_action: Variant = _view.simple_action("Pass")
	if pass_action != null:
		_request(pass_action)


## La pastille « Le bot réfléchit… », tant que l'adversaire décide et que la vue affichée
## est la dernière.
func _refresh_thinking() -> void:
	var deciding := _view != null and not _view.is_over() and _view.decision_player() == _view.opponent()
	_ui.thinking.visible = deciding and not _queue.is_behind()
	_ui.thinking.show_status(Labels.BOT_THINKING if opponent_is_bot else Labels.OPPONENT_PLAYING, true)


# --- La connexion et les avertissements ---

## L'avertissement de la session : la connexion perdue et le bot muet en fenêtre (le bot,
## jusqu'à ce qu'il joue, sauf si l'on a choisi de l'attendre : un message le dit alors sous
## le tour), l'ami parti sous le tour.
func _refresh_warning() -> void:
	var warning := session.warning(_online)
	if warning == Session.Warning.BOT_DOWN:
		_bot_stuck = true
	if warning == Session.Warning.OFFLINE:
		if not _ui.connection_dialog.visible:
			_ui.connection_dialog.ask(true)
	else:
		_ui.connection_dialog.hide()
	var bot_alert := _bot_stuck and not _waiting_for_bot and _online
	if bot_alert and not _ui.bot_dialog.visible:
		_ui.bot_dialog.ask()
	elif not bot_alert:
		_ui.bot_dialog.hide()
	var text := ""
	if warning == Session.Warning.FRIEND_LEFT:
		text = Labels.FRIEND_LEFT
	elif _bot_stuck and _waiting_for_bot:
		text = Labels.BOT_FAILED
	_ui.set_notice(text, text != "")


## La connexion est coupée : ce qui attendait une réponse (un coup, une carte qui partait,
## un aperçu) n'en aura pas. La vue envoyée à la reprise fera foi.
func _lose_connection() -> void:
	_waiting_reply = false
	_queue.cancel_sequence()
	_end_choice()
	_stale_previews = 0
	_gestures.reset()
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_ui.thinking.visible = false
	_refresh_main_button()


# --- Le choix d'une carte ---

func _on_card_chosen(slot: int) -> void:
	if _view == null or _waiting_reply or _choice != null or _queue.is_behind():
		return
	var use_attack: Variant = _view.use_attack(slot)
	if use_attack == null:
		return
	_gestures.reset()
	_board.show_mark(ViewData.NO_CELL)
	_board.show_ghost(ViewData.NO_CELL, "", false)
	_choice = CardChoice.new(slot, use_attack)
	_choice.changed.connect(_refresh_choice)
	_choice.complete.connect(_play_sequence)
	_gestures.choice = _choice
	_ui.arsenal.select(slot)
	_apply_mode()
	_refresh_choice()


## La fiche de la carte de cet emplacement de mon arsenal.
func _show_card_sheet(slot: int) -> void:
	if _view == null:
		return
	var arsenal := _view.arsenal(_view.viewer())
	if slot < 0 or slot >= arsenal.size():
		return
	var entry: Variant = arsenal[slot]
	if not (entry is Dictionary) or not (entry.get("attack") is Dictionary):
		return
	var attack: Dictionary = entry["attack"]
	_ui.card_sheet.show_card(attack, catalog, _view.stats_of(attack), entry.get("secret", false))


## Montre les choix suivants : les cases proposées, celles déjà choisies, et le panneau.
func _refresh_choice() -> void:
	if _choice == null:
		return
	var choices := _choice.choices
	_board.show_candidates(choices.cells_of(BoardGestures.CHOOSE) if choices != null else ([] as Array[Vector2i]))
	_board.show_chosen(_choice.chosen_cells())
	var slot: Variant = _view.arsenal(_view.viewer())[_choice.slot]
	var attack: Dictionary = slot.get("attack", {}) if slot is Dictionary else {}
	_ui.choice_panel.show_choice(_choice, attack, catalog, _view.stats_of(attack).get("cost", 0))


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
	_queue.expect_sequence(actions.size())
	_refresh_main_button()
	for action: Variant in actions:
		play_requested.emit(action)


## Oublie le choix en cours (annuler ne coûte rien : rien n'a été joué).
func _end_choice() -> void:
	if _choice == null:
		return
	_stale_previews += _choice.pending
	_choice = null
	_gestures.choice = null
	_board.show_candidates([] as Array[Vector2i])
	_board.show_chosen([] as Array[Vector2i])
	_ui.arsenal.select(-1)
	_apply_mode()


# --- Les demandes ---

func _unhandled_input(event: InputEvent) -> void:
	# Entrée (PC) : le bouton d'action, comme le dit sa légende en paysage.
	var key := event as InputEventKey
	if key != null and key.pressed and not key.echo and key.keycode in [KEY_ENTER, KEY_KP_ENTER] and _ui.is_main_button_ready():
		_on_main_button()
		get_viewport().set_input_as_handled()
		return
	# Échap (PC) ferme la page du journal, ou annule le choix en cours ou le pas du Lièvre.
	if not event.is_action_pressed(&"ui_cancel"):
		return
	if _ui.is_journal_page_open():
		_ui.show_journal_page(false)
		get_viewport().set_input_as_handled()
	elif _choice != null or _gestures.in_hare_step():
		_end_choice()
		_gestures.leave_hare_step()
		get_viewport().set_input_as_handled()


func _request(action: Variant) -> void:
	_waiting_reply = true
	_refresh_main_button()
	play_requested.emit(action)


## La flèche : pendant la partie, on confirme avant de la quitter.
func _on_back() -> void:
	if _view != null and _view.is_over():
		home_requested.emit()
	else:
		_ui.quit_dialog.ask()


## Le bouton d'action : l'action principale permise (finir son tour, passer), ou l'accueil
## une fois la partie finie ; le cadre « EN ATTENTE » sinon.
func _refresh_main_button() -> void:
	_main_action = null
	if _view != null and _view.is_over():
		_ui.show_main_button(Labels.HOME, true)
		return
	if _view != null:
		for kind in MAIN_ACTIONS:
			var action: Variant = _view.simple_action(kind)
			if action != null:
				_main_action = action
				_ui.show_main_button(Labels.ACTION_LABELS[kind], not _waiting_reply and not _queue.is_behind())
				return
	_ui.show_waiting()


func _on_main_button() -> void:
	if _queue.is_behind():
		return
	if _view != null and _view.is_over():
		home_requested.emit()
	elif _main_action != null:
		_request(_main_action)
