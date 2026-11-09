class_name GameLayout
extends Control
## La mise en page de l'écran de partie : ses éléments, et leurs boîtes selon l'orientation.
## Elle ne décide de rien : l'écran de partie (`GameScreen`) lui dit quoi montrer, et écoute
## ses boutons.
##
## En portrait (téléphone) : l'adversaire en haut, le tour, le plateau, l'arsenal, sa barre,
## et le bouton d'action sur toute la largeur, sous le pouce. En paysage (PC) : le journal à
## gauche, le plateau et l'arsenal au milieu, une colonne à droite. Pendant le choix d'une
## carte, la réaction au coup adverse ou la fin, leur panneau prend la place de l'arsenal
## (et, en portrait, de la barre). Par-dessus tout : le bandeau du tour, le menu de case, la
## page du journal (portrait), l'annonce d'une carte adverse, la cache révélée, la question
## pour quitter.

## Le bouton d'action, la flèche de retour (en haut à gauche).
signal main_pressed
signal back_pressed
## Le rectangle de la place du plateau a changé (le plateau doit s'y recadrer).
signal board_area_changed

var opponent_panel := PlayerPanel.new()
var my_panel := PlayerPanel.new()
## Le tour (« TOUR 7 · À TOI »), et « Le bot réfléchit… ».
var turn_pill := StatusPill.new()
var thinking := StatusPill.new()
var arsenal := ArsenalBar.new()
var choice_panel := ChoicePanel.new()
var reaction := ReactionPanel.new()
var end_panel := EndPanel.new()
var journal_panel := JournalPanel.new()
var action_menu := ActionMenu.new()
var attack_banner := AttackBanner.new()
var reveal := RevealOverlay.new()
var quit_dialog := ConfirmDialog.quit_game()

var _landscape := false
var _arranged := false
## Le dernier mode demandé, réappliqué quand l'orientation change.
var _choosing := false
var _reacting := false
var _ending := false
var _layout: Control
## Un message sous le tour : discret (le pas du Lièvre), ou en braise (un avertissement).
var _notice: Label
var _main_button: Button
## La place du bouton d'action : le bouton, ou le cadre « EN ATTENTE ».
var _action_slot: MarginContainer
var _waiting: PanelContainer
## La place du plateau : le plateau 3D est cadré dans son rectangle.
var _board_area: Control
var _banner: PanelContainer
var _banner_show: Tween
var _journal_button: Button
var _back_button: Button
## En portrait, la page du journal, par-dessus tout ; et la ligne du tour qui porte son bouton.
var _journal_page: PanelContainer
var _journal_page_column: VBoxContainer
var _top_row: HBoxContainer


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_notice = Ui.centered_label("", ThemeBuilder.MUTED_LABEL)
	_notice.visible = false
	thinking.visible = false
	_build_action_slot()
	_board_area = Ui.spacer()
	_board_area.resized.connect(_on_board_area_resized)
	_build_banner()
	_build_journal()
	_build_attack_banner()
	add_child(reveal)
	add_child(quit_dialog)
	resized.connect(_on_resized)
	_on_resized()


func is_landscape() -> bool:
	return _landscape


## Le rectangle de l'écran où cadrer le plateau. En portrait, les marges de la page servent
## au texte, pas au plateau : il va d'un bord de l'écran à l'autre. En paysage, il reste
## entre le journal et la colonne de droite. Vide tant que la place n'est pas dans l'arbre.
func board_rect() -> Rect2:
	if not _board_area.is_inside_tree():
		return Rect2()
	var area := _board_area.get_global_rect()
	var left := area.position.x if _landscape else 0.0
	var right := area.end.x if _landscape else get_viewport_rect().size.x
	return Rect2(left, area.position.y, right - left, area.size.y)


## Montre l'arsenal et la barre ; à leur place, pendant le choix d'une carte, le panneau de
## choix ; quand on peut réagir au coup adverse, la fenêtre de réaction ; à la fin, le
## panneau de fin.
func apply_mode(choosing: bool, reacting: bool, ending: bool) -> void:
	_choosing = choosing
	_reacting = reacting
	_ending = ending
	choice_panel.visible = choosing and not ending
	reaction.visible = reacting and not ending
	end_panel.visible = ending
	var busy := choosing or reacting or ending
	arsenal.visible = not busy
	_action_slot.visible = not busy
	my_panel.visible = _landscape or not busy


## Un message sous le tour, ou rien : discret, ou en braise pour un avertissement.
func set_notice(text: String, warning: bool = false) -> void:
	_notice.text = text
	_notice.theme_type_variation = ThemeBuilder.THREAT_LABEL if warning else ThemeBuilder.MUTED_LABEL
	_notice.visible = text != ""


## Le bouton d'action doré, avec ce texte (grisé le temps que le serveur réponde).
func show_main_button(text: String, enabled: bool) -> void:
	_waiting.visible = false
	_main_button.visible = true
	_main_button.text = text
	_main_button.disabled = not enabled


## À la place du bouton d'action, le cadre en pointillés : on attend l'adversaire.
func show_waiting() -> void:
	_main_button.visible = false
	_waiting.visible = true


## Ouvre ou ferme la page du journal (portrait seulement).
func show_journal_page(shown: bool) -> void:
	_journal_page.visible = shown and not _landscape


func is_journal_page_open() -> bool:
	return _journal_page.visible


## Le bandeau « À toi ! » au milieu du plateau : il grandit en apparaissant, reste un
## instant, puis s'efface.
func play_turn_banner() -> void:
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


# --- La construction ---

func _build_action_slot() -> void:
	_action_slot = MarginContainer.new()
	_action_slot.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_main_button = Ui.button(Labels.WAITING, ThemeBuilder.PRIMARY_BUTTON)
	_main_button.pressed.connect(main_pressed.emit)
	_action_slot.add_child(_main_button)
	_waiting = PanelContainer.new()
	_waiting.theme_type_variation = ThemeBuilder.WAITING_PANEL
	_waiting.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_waiting.custom_minimum_size.y = Look.BUTTON_HEIGHT
	var row := Ui.hbox(Look.GAP_S)
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	row.add_child(Ui.icon(Ui.HOURGLASS, Look.MUTED))
	row.add_child(Ui.label(Labels.WAITING, ThemeBuilder.WAITING_LABEL))
	_waiting.add_child(row)
	_waiting.visible = false
	_action_slot.add_child(_waiting)


func _build_journal() -> void:
	_journal_button = Ui.icon_button("journal", Labels.JOURNAL_TITLE)
	_journal_button.pressed.connect(show_journal_page.bind(true))
	_back_button = Ui.icon_button("retour", Labels.BACK)
	_back_button.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	_back_button.pressed.connect(back_pressed.emit)
	_top_row = Ui.hbox(0)
	_journal_page = PanelContainer.new()
	_journal_page.theme_type_variation = ThemeBuilder.OVERLAY
	_journal_page.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	# La page arrête les touchers : le plateau, dessous, n'en reçoit aucun.
	_journal_page.mouse_filter = Control.MOUSE_FILTER_STOP
	_journal_page.visible = false
	_journal_page_column = Ui.vbox(Look.GAP_M)
	var close := Ui.button(Labels.CLOSE, ThemeBuilder.SECONDARY_BUTTON)
	close.pressed.connect(show_journal_page.bind(false))
	_journal_page_column.add_child(close)
	_journal_page.add_child(Ui.page(_journal_page_column))
	add_child(_journal_page)


func _build_attack_banner() -> void:
	attack_banner.set_anchors_and_offsets_preset(Control.PRESET_TOP_WIDE)
	var insets := Ui.safe_insets()
	attack_banner.offset_left = Look.PAGE_MARGIN + insets.position.x
	attack_banner.offset_right = -Look.PAGE_MARGIN - insets.size.x
	attack_banner.offset_top = Look.PAGE_MARGIN + insets.position.y
	add_child(attack_banner)


func _build_banner() -> void:
	_banner = PanelContainer.new()
	_banner.theme_type_variation = ThemeBuilder.BANNER
	_banner.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_banner.visible = false
	_banner.add_child(Ui.label(Labels.YOUR_TURN_BANNER, ThemeBuilder.BANNER_LABEL))
	add_child(_banner)


# --- Le rangement ---

func _on_resized() -> void:
	var landscape := Ui.is_landscape(self)
	if not _arranged or landscape != _landscape:
		_arrange(landscape)
	_on_board_area_resized()


## La place du plateau a changé : il s'y recadre, et le bandeau du tour reste en son milieu.
func _on_board_area_resized() -> void:
	board_area_changed.emit()
	if _banner.visible:
		_center_banner()


## Range les éléments pour cette orientation. Les éléments restent les mêmes : seules
## leurs boîtes changent.
func _arrange(landscape: bool) -> void:
	_arranged = true
	_landscape = landscape
	for item: Control in [opponent_panel, my_panel, turn_pill, thinking, _notice, _action_slot, _board_area, arsenal, choice_panel, reaction, end_panel, journal_panel, _journal_button, _back_button]:
		if item.get_parent() != null:
			item.get_parent().remove_child(item)
	if _top_row.get_parent() != null:
		_top_row.get_parent().remove_child(_top_row)
	for old in _top_row.get_children():
		old.queue_free()
	if _layout != null:
		_layout.queue_free()
	show_journal_page(false)
	_layout = _landscape_page() if landscape else _portrait_page()
	add_child(_layout)
	apply_mode(_choosing, _reacting, _ending)
	# Ce qui passe par-dessus la page, dans cet ordre.
	if action_menu.get_parent() == null:
		add_child(action_menu)
	for overlay: Control in [_banner, action_menu, _journal_page, attack_banner, reveal, quit_dialog]:
		move_child(overlay, -1)


func _landscape_page() -> Control:
	var row := Ui.hbox(Look.GAP_XL)
	journal_panel.custom_minimum_size.x = Look.JOURNAL_WIDTH
	row.add_child(journal_panel)
	var left := Ui.vbox(Look.GAP_M)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for item: Control in [_board_area, arsenal, choice_panel, reaction, end_panel]:
		left.add_child(item)
	row.add_child(left)
	var column := Ui.vbox(Look.GAP_M)
	column.custom_minimum_size.x = Look.SIDE_COLUMN_WIDTH
	for item: Control in [_back_button, turn_pill, thinking, _notice, opponent_panel, my_panel, Ui.spacer(), _action_slot]:
		column.add_child(item)
	row.add_child(column)
	my_panel.set_identity_visible(true)
	return Ui.page(row)


func _portrait_page() -> Control:
	var column := Ui.vbox(Look.GAP_M)
	column.add_child(opponent_panel)
	# La pastille du tour reste au centre ; le bouton du journal se range à droite.
	var right := Ui.hbox(0)
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right.alignment = BoxContainer.ALIGNMENT_END
	right.add_child(_journal_button)
	var left := Ui.hbox(0)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.add_child(_back_button)
	_top_row.add_child(left)
	_top_row.add_child(turn_pill)
	_top_row.add_child(right)
	for item: Control in [_top_row, thinking, _notice, _board_area, arsenal, my_panel, _action_slot, choice_panel, reaction, end_panel]:
		column.add_child(item)
	my_panel.set_identity_visible(false)
	journal_panel.custom_minimum_size.x = 0.0
	_journal_page_column.add_child(journal_panel)
	_journal_page_column.move_child(journal_panel, 0)
	return Ui.page(column)
