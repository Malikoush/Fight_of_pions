extends TestCase
## La session : contre qui on joue, l'attente du lancement, l'avertissement.


func test_each_start_resets_the_previous_game() -> bool:
	var session := Session.new()
	session.start_host()
	session.note_opponent_present(true)
	session.note_bot_failed(true)
	session.start_bot()
	check(session.against_bot(), "contre le bot")
	equal(session.opponent_name, Labels.BOT, "son nom")
	check(not session.opponent_present, "présence oubliée")
	check(not session.bot_failed, "panne oubliée")
	check(not session.waiting_launch(), "rien à lancer")
	return true


func test_the_account_outlives_each_game() -> bool:
	var session := Session.new()
	equal(session.pseudo, Labels.DEFAULT_PLAYER_NAME, "avant le compte")
	check(session.collection == null, "pas encore de collection")
	var owned := CardCollection.new([{"kind": "Ours", "rarity": "Common"}])
	session.note_account("Martin")
	session.note_collection(owned)
	session.start_bot()
	session.reset()
	equal(session.pseudo, "Martin", "le pseudo reste")
	equal(session.collection, owned, "la collection reste")
	return true


func test_a_friends_room_holds_its_last_view_until_launch() -> bool:
	for start: String in ["start_host", "start_guest"]:
		var session := Session.new()
		session.call(start)
		check(session.waiting_launch(), "%s : la salle attend" % start)
		equal(session.hosting(), start == "start_host", "%s : seul le créateur lance" % start)
		var first := view_of("draft_view")
		var last := view_of("draft_view")
		session.hold(first, [])
		session.hold(last, ["x"])
		equal(session.note_launched(false), [], "%s : pas encore lancée" % start)
		check(session.waiting_launch(), "%s : elle attend encore" % start)
		var held := session.note_launched(true)
		equal(held[0], last, "%s : seule la dernière vue compte" % start)
		equal(held[1], ["x"], "%s : avec ses événements" % start)
		check(not session.waiting_launch(), "%s : lancée" % start)
		equal(session.note_launched(true), [], "%s : rien de plus à rendre" % start)
	var against_bot := Session.new()
	against_bot.start_bot()
	against_bot.note_launched(false)
	check(not against_bot.waiting_launch(), "contre le bot, rien n'attend")
	return true


func test_the_warning_puts_the_connection_first() -> bool:
	var session := Session.new()
	session.start_guest()
	equal(session.warning(true), Session.Warning.FRIEND_LEFT, "ami pas encore là")
	session.note_opponent_present(true)
	equal(session.warning(true), Session.Warning.NONE, "tout va bien")
	session.note_bot_failed(true)
	equal(session.warning(false), Session.Warning.OFFLINE, "la coupure d'abord")
	equal(session.warning(true), Session.Warning.BOT_DOWN, "puis le bot")
	session.start_bot()
	equal(session.warning(true), Session.Warning.NONE, "contre le bot, pas d'ami à attendre")
	return true


func test_a_saved_session_comes_back_after_a_restart() -> bool:
	var path := "user://test_session.cfg"
	var first := Session.new(path)
	first.start_host()
	var restarted := Session.new(path)
	restarted.restore()
	equal(restarted.mode, Session.Mode.HOST, "le mode revient")
	equal(restarted.opponent_name, Labels.FRIEND, "contre un ami")
	check(restarted.waiting_launch(), "la salle attend encore son lancement")
	first.note_launched(true)
	restarted.restore()
	check(not restarted.waiting_launch(), "une salle lancée ne l'attend plus")
	first.start_bot()
	restarted.restore()
	check(restarted.against_bot(), "contre le bot")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
	return true


func test_a_session_without_file_restores_nothing() -> bool:
	var session := Session.new()
	session.start_bot()
	session.restore()
	check(session.against_bot(), "rien n'est relu")
	return true


func test_a_damaged_session_file_is_ignored() -> bool:
	var path := "user://test_session_damaged.cfg"
	var file := ConfigFile.new()
	file.set_value(Session.SAVE_SECTION, "mode", 3)
	file.save(path)
	var session := Session.new(path)
	session.restore()
	equal(session.mode, Session.Mode.NONE, "un mode qui n'est pas un nom")
	file.set_value(Session.SAVE_SECTION, "mode", "INCONNU")
	file.save(path)
	session.restore()
	equal(session.mode, Session.Mode.NONE, "un mode inconnu")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
	return true


func test_a_friend_is_named_by_its_pseudo() -> bool:
	var session := Session.new()
	session.start_guest()
	equal(session.opponent_name, Labels.FRIEND, "avant son pseudo")
	session.note_opponent_present(true, "Bob")
	equal(session.opponent_name, "Bob", "son pseudo")
	session.note_opponent_present(false)
	equal(session.opponent_name, "Bob", "parti, il garde son nom")
	session.start_bot()
	session.note_opponent_present(true, "Bob")
	equal(session.opponent_name, Labels.BOT, "le bot garde le sien")
	return true
