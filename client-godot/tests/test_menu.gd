extends TestCase
## Le menu : le code tapé dans ses cases, la position de l'aperçu du plateau.


func test_a_typed_code_keeps_letters_and_digits_in_capitals() -> bool:
	equal(CodeField.clean("zz9q-x"), "ZZ9QX", "tiret retiré, capitales")
	equal(CodeField.clean(" k7 "), "K7", "espaces retirés")
	equal(CodeField.clean("abcdefgh"), "ABCDE", "un code collé trop long est coupé")
	equal(CodeField.clean("é!?"), "", "rien à garder")
	return true


func test_the_home_board_is_a_view_the_client_reads() -> bool:
	var saved: Variant = Wire.normalize((load(BoardPreview.POSITION) as JSON).data)
	check(saved is Dictionary, "position lisible")
	var view := ViewData.new(saved.get("view", {}) if saved is Dictionary else {})
	equal(view.grid_size(), 16, "grille, mur compris")
	var pions := 0
	for row in view.grid_size():
		for col in view.grid_size():
			if view.pion_at(Vector2i(col, row)) != null:
				pions += 1
	equal(pions, 7, "les pions posés sur l'aperçu")
	check(view.legal_actions().is_empty(), "rien à jouer sur un aperçu")
	return true


## Partir de la salle d'attente demande confirmation, chez son créateur comme chez son ami :
## sur iPhone, le geste qui change d'application touchait « FERMER LA SALLE ».
func test_leaving_the_lobby_is_asked_first() -> bool:
	for hosting: bool in [true, false]:
		var session := Session.new()
		if hosting:
			session.start_host()
		else:
			session.start_guest()
		var lobby: LobbyScreen = load("res://ui/screens/lobby_screen.tscn").instantiate()
		lobby.session = session
		lobby.code = "K7QXM"
		var departures := [0]
		lobby.home_requested.connect(func() -> void: departures[0] += 1)
		Engine.get_main_loop().root.add_child(lobby)
		lobby._close.pressed.emit()
		lobby._header.back_pressed.emit()
		equal(departures[0], 0, "un appui ne fait que poser la question (créateur : %s)" % hosting)
		check(lobby._leave_dialog.visible, "la question est posée (créateur : %s)" % hosting)
		lobby._leave_dialog._answer(lobby._leave_dialog.declined)
		equal(departures[0], 0, "« RESTER » garde dans la salle (créateur : %s)" % hosting)
		check(not lobby._leave_dialog.visible, "et ferme la question (créateur : %s)" % hosting)
		lobby._close.pressed.emit()
		lobby._leave_dialog._answer(lobby._leave_dialog.confirmed)
		equal(departures[0], 1, "partir une fois la réponse donnée (créateur : %s)" % hosting)
		lobby.free()
	return true


## En portrait (téléphone), le bouton du bas qui ferme la salle est caché : la flèche suffit.
func test_the_lobby_close_button_is_only_shown_in_landscape() -> bool:
	var root: Window = Engine.get_main_loop().root
	var saved_size: Vector2i = root.size
	var session := Session.new()
	session.start_host()
	var lobby: LobbyScreen = load("res://ui/screens/lobby_screen.tscn").instantiate()
	lobby.session = session
	root.add_child(lobby)
	for window: Vector2i in [Vector2i(1280, 720), Vector2i(390, 844)]:
		root.size = window
		lobby._arrange()
		equal(lobby._close.visible, window.x > window.y, "bouton du bas en %s" % window)
	root.size = saved_size
	lobby.free()
	return true


## La page Réglages : le code de récupération se montre une fois créé, et un code tapé est
## remis à l'écran du menu, qui demande confirmation.
func test_the_settings_show_the_recovery_code_and_ask_before_changing_account() -> bool:
	var page := SettingsPage.new("Ana")
	var asked: Array = []
	page.recovery_asked.connect(func(code: String) -> void: asked.append(code))
	Engine.get_main_loop().root.add_child(page)
	check(not page._code_row.visible, "pas de code avant d'en créer un")
	page.show_recovery("K7QX-M2PA-9ZTR-4HWD", false, "Ana")
	check(page._code_row.visible, "le code créé est montré")
	equal(page._code_shown.text, "K7QX-M2PA-9ZTR-4HWD", "le code, à copier")
	equal(page._create_code.text, Labels.RECOVERY_RENEW, "le bouton en crée un nouveau")
	page._ask_recovery()
	equal(asked, [], "rien sans code tapé")
	page._recover_field.text = " k7qx m2pa 9ztr 4hwd "
	page._ask_recovery()
	equal(asked, ["k7qx m2pa 9ztr 4hwd"], "le code tapé, sans espace autour, à confirmer par le menu")
	page.show_recovery_refusal("aucun compte n'a ce code de récupération")
	check(page._recover_status.visible, "le refus, sous le champ")
	page.show_recovery("", true, "Bob")
	equal(page._recover_status.text, Labels.RECOVERED % "Bob", "le compte retrouvé")
	page.free()
	return true


## L'écran du menu pose la question avant de changer de compte par un code, par-dessus
## tout l'écran ; il ne transmet le code qu'une fois la réponse donnée.
func test_the_menu_asks_before_changing_account_by_a_code() -> bool:
	var menu: MenuScreen = load("res://ui/screens/menu_screen.tscn").instantiate()
	menu.session = Session.new()
	var requested: Array = []
	menu.recovery_requested.connect(func(code: String) -> void: requested.append(code))
	Engine.get_main_loop().root.add_child(menu)
	menu._ask_recovery("K7QX-M2PA-9ZTR-4HWD")
	equal(requested, [], "rien avant la réponse")
	check(menu._recover_dialog.visible, "la question est posée")
	equal(menu._recover_dialog.get_index(), menu.get_child_count() - 1, "par-dessus le reste de l'écran")
	menu._recover_dialog._answer(menu._recover_dialog.confirmed)
	equal(requested, ["K7QX-M2PA-9ZTR-4HWD"], "le code, une fois la réponse donnée")
	menu.free()
	return true
