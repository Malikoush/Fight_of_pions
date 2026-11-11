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
	equal(link._pending, "", "la demande a eu sa réponse")
	return true


func test_a_refusal_answers_the_pending_account_request() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var heard: Array = []
	var listen := func(category: ServerErrors.Category, _message: String) -> void: heard.append(category)
	link.server_error.connect(listen)
	# Sans connexion, la demande est perdue : rien n'attend de réponse.
	link.set_pseudo("Martin")
	equal(link._pending, "", "rien d'envoyé, rien n'attend")
	link._pending = "set_pseudo"
	link._on_error("storage", "le serveur n'a pas pu enregistrer")
	link._pending = "set_cache_trio"
	link._receive('{"type":"collection","cards":[],"cache_trio":[]}')
	equal(link._pending, "", "la collection répond au trio demandé")
	link._on_error("storage", "le serveur n'a pas pu enregistrer")
	link.server_error.disconnect(listen)
	equal(heard, [ServerErrors.Category.PSEUDO_REFUSED, ServerErrors.Category.OTHER], "la base parle du pseudo, puis de rien")
	return true


## Le compte de l'appareil, gardé le temps d'un test qui le change : les tests tournent
## sur le même `user://` que le jeu.
func _keep_account(link: Node) -> Dictionary:
	return {
		"token": link._token,
		"id": link.account_id,
		"pseudo": link.account_pseudo,
		"file": FileAccess.get_file_as_string(link.ACCOUNT_PATH),
	}


func _restore_account(link: Node, kept: Dictionary) -> void:
	link._token = kept["token"]
	link.account_id = kept["id"]
	link.account_pseudo = kept["pseudo"]
	if kept["file"] == "":
		DirAccess.remove_absolute(ProjectSettings.globalize_path(link.ACCOUNT_PATH))
		return
	var file := FileAccess.open(link.ACCOUNT_PATH, FileAccess.WRITE)
	file.store_string(kept["file"])
	file.close()


func test_the_account_is_created_once_then_logged_in_with_its_token() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var kept := _keep_account(link)
	link._token = ""
	equal(link._account_request(), {"type": "register"}, "sans jeton : un compte neuf")
	var heard: Array = []
	var listen := func(id: int, pseudo: String) -> void: heard.append([id, pseudo])
	link.account_received.connect(listen)
	link._receive('{"type":"account","id":12,"pseudo":"Pion0042","token":"secret"}')
	link._receive('{"type":"account","id":12,"pseudo":"Martin"}')
	link.account_received.disconnect(listen)
	equal(heard, [[12, "Pion0042"], [12, "Martin"]], "le compte annoncé, numéro entier")
	check(link.has_account(), "le jeton est gardé")
	equal(link._account_request(), {"type": "login", "token": "secret"}, "ensuite : login")
	var saved := ConfigFile.new()
	saved.load(link.ACCOUNT_PATH)
	equal(saved.get_value(link.ACCOUNT_SECTION, link.ACCOUNT_TOKEN, ""), "secret", "sur le disque")
	_restore_account(link, kept)
	return true


func test_an_unknown_account_is_replaced_without_a_refusal() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var kept := _keep_account(link)
	link._token = "perdu"
	var heard: Array = []
	var listen := func(category: ServerErrors.Category, _message: String) -> void: heard.append(category)
	link.server_error.connect(listen)
	link._receive('{"type":"error","kind":"unknown_account","message":"ce compte n\'existe pas"}')
	link.server_error.disconnect(listen)
	equal(heard, [], "rien d'annoncé")
	check(not link.has_account(), "le jeton est oublié : un compte neuf sera créé")
	_restore_account(link, kept)
	return true


func test_the_collection_and_a_reward_are_announced() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var collections: Array = []
	var rewards: Array = []
	var on_collection := func(cards: Array, trio: Array) -> void: collections.append([cards, trio])
	var on_reward := func(card: Dictionary) -> void: rewards.append(card)
	link.collection_received.connect(on_collection)
	link.reward_received.connect(on_reward)
	var ours := {"kind": "Ours", "rarity": "Common"}
	link._receive('{"type":"collection","cards":[{"kind":"Ours","rarity":"Common"}],"cache_trio":[]}')
	link._receive('{"type":"reward","card":{"kind":"Ours","rarity":"Common"}}')
	link._receive('{"type":"collection","cards":"rien"}')
	link._receive('{"type":"account","id":"douze","pseudo":"Martin"}')
	link.collection_received.disconnect(on_collection)
	link.reward_received.disconnect(on_reward)
	equal(collections, [[[ours], []]], "la collection, un message mal formé ignoré")
	equal(rewards, [ours], "la carte gagnée")
	return true


func test_in_a_browser_the_server_is_where_the_page_came_from() -> bool:
	var url_for_page := func(protocol: String, host: String, path: String) -> String:
		return Engine.get_main_loop().root.get_node("ServerLink").url_for_page(protocol, host, path)
	equal(url_for_page.call("https:", "optiplex.reseau.ts.net:8443", "/"), "wss://optiplex.reseau.ts.net:8443/ws", "https, port compris")
	equal(url_for_page.call("https:", "jeu.exemple.fr", "/index.html"), "wss://jeu.exemple.fr/ws", "la page nommée")
	equal(url_for_page.call("http:", "127.0.0.1:3000", "/"), "ws://127.0.0.1:3000/ws", "en local, sans chiffrement")
	equal(url_for_page.call("https:", "exemple.fr", "/pions/index.html"), "wss://exemple.fr/pions/ws", "dans un sous-dossier")
	equal(url_for_page.call("https:", "exemple.fr", ""), "wss://exemple.fr/ws", "sans chemin")
	equal(Engine.get_main_loop().root.get_node("ServerLink").server_url(), "ws://127.0.0.1:3000/ws", "hors navigateur")
	return true


func test_the_launch_is_heard_when_seated_and_when_announced() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var kept_seat: Dictionary = link._seat.duplicate()
	var heard: Array = []
	var listen := func(launched: bool) -> void: heard.append(launched)
	link.room_launch.connect(listen)
	link._receive('{"type":"seated","code":"K7QXM","player":"Two","key":"cle","launched":false}')
	link._receive('{"type":"launched"}')
	link._receive('{"type":"seated","code":"K7QXM","player":"Two","key":"cle"}')
	link._receive('{"type":"seated","code":"K7QXM","player":"Two","key":"cle","launched":"oui"}')
	link.room_launch.disconnect(listen)
	equal(heard, [false, true, false], "assis, lancée, ancien serveur ; un message mal formé ignoré")
	equal(link.seat_player(), "Two", "la place tirée au sort")
	link.forget_seat()
	if not kept_seat.is_empty():
		link._save_seat(kept_seat)
	return true


func test_leaving_ignores_the_room_until_the_server_answers() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var kept_seat: Dictionary = link._seat.duplicate()
	var views: Array = []
	var listen := func(view: ViewData, _events: Array) -> void: views.append(view)
	link.state_received.connect(listen)
	# Sans connexion : `leave` partira à la prochaine ouverture.
	link.leave_room()
	check(link._leaving, "le départ attend sa réponse")
	link._receive('{"type":"state","view":{},"events":[]}')
	link._receive('{"type":"left"}')
	check(not link._leaving, "le serveur a répondu")
	link._receive('{"type":"state","view":{},"events":[]}')
	link.state_received.disconnect(listen)
	equal(views.size(), 1, "seule la vue d'après le départ compte")
	if not kept_seat.is_empty():
		link._save_seat(kept_seat)
	return true


func test_the_presence_of_the_opponent_comes_with_a_pseudo() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var heard: Array = []
	var listen := func(connected: bool, pseudo: String) -> void: heard.append([connected, pseudo])
	link.opponent_presence.connect(listen)
	link._receive('{"type":"opponent","connected":true,"pseudo":"Bob"}')
	link._receive('{"type":"opponent","connected":false}')
	link._receive('{"type":"opponent","connected":true,"pseudo":7}')
	link.opponent_presence.disconnect(listen)
	equal(heard, [[true, "Bob"], [false, ""]], "le pseudo, absent d'un ancien serveur ; mal formé ignoré")
	return true


func test_a_saved_seat_the_server_does_not_resume_is_lost() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var kept_seat: Dictionary = link._seat.duplicate()
	var kept := _keep_account(link)
	var heard: Array = []
	var listen := func(category: ServerErrors.Category, _message: String) -> void: heard.append(category)
	link.server_error.connect(listen)
	link._seat = {"code": "K7QXM", "player": "Two", "key": "cle"}
	link._logging_in = true
	link._receive('{"type":"account","id":12,"pseudo":"Martin","resumed":true}')
	check(link.has_saved_seat(), "reprise : la place reste")
	link._logging_in = true
	link._receive('{"type":"account","id":12,"pseudo":"Martin"}')
	link._receive('{"type":"account","id":12,"pseudo":"Martin le Pion"}')
	link.server_error.disconnect(listen)
	equal(heard, [ServerErrors.Category.SEAT_LOST], "pas reprise : perdue, une seule fois")
	check(not link.has_saved_seat(), "la place est oubliée")
	_restore_account(link, kept)
	if not kept_seat.is_empty():
		link._save_seat(kept_seat)
	return true


func test_leaving_still_hears_the_refusals_of_the_account() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var kept_seat: Dictionary = link._seat.duplicate()
	var kept := _keep_account(link)
	var heard: Array = []
	var listen := func(category: ServerErrors.Category, _message: String) -> void: heard.append(category)
	link.server_error.connect(listen)
	link.leave_room()
	link._receive('{"type":"error","kind":"seat_taken","message":"ta place a été reprise"}')
	link._receive('{"type":"error","kind":"storage","message":"base indisponible"}')
	link._token = "perdu"
	link._receive('{"type":"error","kind":"unknown_account","message":"ce compte n\'existe pas"}')
	link._receive('{"type":"left"}')
	link.server_error.disconnect(listen)
	equal(heard, [ServerErrors.Category.OTHER], "la salle quittée se tait, la base parle")
	check(not link.has_account(), "le jeton inconnu est oublié")
	_restore_account(link, kept)
	if not kept_seat.is_empty():
		link._save_seat(kept_seat)
	return true


func test_a_refused_login_waits_no_more() -> bool:
	var link: Node = Engine.get_main_loop().root.get_node("ServerLink")
	var kept := _keep_account(link)
	link._logging_in = true
	link._on_error("storage", "base indisponible")
	check(not link._logging_in, "la connexion au compte a eu sa réponse")
	_restore_account(link, kept)
	return true
