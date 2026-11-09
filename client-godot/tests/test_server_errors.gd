extends TestCase
## Les refus du serveur, classés par ce qu'ils demandent au client.


func test_each_kind_has_its_category() -> bool:
	equal(ServerErrors.category("bad_key"), ServerErrors.Category.SEAT_LOST, "mauvaise clé")
	equal(ServerErrors.category("seat_taken"), ServerErrors.Category.SEAT_TAKEN, "place reprise")
	equal(ServerErrors.category("not_your_turn"), ServerErrors.Category.STALE_VIEW, "pas son tour")
	equal(ServerErrors.category("refused_move"), ServerErrors.Category.STALE_VIEW, "coup refusé")
	equal(ServerErrors.category("bot_failed"), ServerErrors.Category.BOT_DOWN, "bot muet")
	equal(ServerErrors.category("server_full"), ServerErrors.Category.SERVER_FULL, "serveur complet")
	equal(ServerErrors.category("bad_message"), ServerErrors.Category.OTHER, "bug du client")
	return true


func test_a_typed_code_that_leads_nowhere_is_a_refused_join() -> bool:
	equal(ServerErrors.category("unknown_room", true), ServerErrors.Category.JOIN_REFUSED, "code inconnu")
	equal(ServerErrors.category("room_full", true), ServerErrors.Category.JOIN_REFUSED, "salle pleine")
	equal(ServerErrors.category("unknown_room", false), ServerErrors.Category.SEAT_LOST, "à la reprise : place perdue")
	return true


func test_a_server_message_reads_as_a_sentence() -> bool:
	equal(Labels.sentence("aucune salle n'a le code K7QXM"), "Aucune salle n'a le code K7QXM.", "capitale et point")
	equal(Labels.sentence("Déjà une phrase."), "Déjà une phrase.", "rien à ajouter")
	equal(Labels.sentence("  "), "", "vide")
	return true
