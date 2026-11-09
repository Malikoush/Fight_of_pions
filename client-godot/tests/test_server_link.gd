extends TestCase
## Le lien avec le serveur (autoload `ServerLink`, chargé aussi sans fenêtre) : comment il
## classe un refus selon la demande en cours. Rien n'est envoyé : sans connexion, la
## demande `join` attend dans la file d'envoi.


func test_a_refusal_during_a_join_is_a_refused_code() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var heard: Array = []
	var listen := func(category: ServerErrors.Category, _message: String) -> void: heard.append(category)
	link.server_error.connect(listen)
	link.join_room("ZZZZZ")
	link._on_error("unknown_room", "aucune salle n'a le code ZZZZZ")
	link.server_error.disconnect(listen)
	equal(heard, [ServerErrors.Category.JOIN_REFUSED], "un code tapé qui ne mène nulle part")
	check(not link._joining, "la demande a eu sa réponse")
	return true
