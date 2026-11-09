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
