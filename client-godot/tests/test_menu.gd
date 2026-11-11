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
