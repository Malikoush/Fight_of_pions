extends TestCase
## Le catalogue des cartes envoyé par le serveur.


func test_families_keep_the_server_order_and_their_texts() -> bool:
	var catalog := CardCatalog.new(fixture("cards")["cards"])
	check(catalog.families().size() > 0, "des familles")
	equal(catalog.family_name("CoupDuTaureau"), "Coup du taureau", "nom affiché")
	equal(catalog.rarity_name("Ours", "Epic"), "Épique", "nom de rareté")
	check(catalog.effect("Ours", "Rare") != "", "un effet par carte")
	check(catalog.is_reaction("Feintaille"), "une réaction")
	check(not catalog.is_reaction("Ours"), "une attaque")
	equal(CardCatalog.card_name(catalog, {"kind": "Ours", "rarity": "Rare"}), "Ours Rare", "nom complet")
	equal(CardCatalog.card_name(null, {"kind": "Ours", "rarity": "Rare"}), "Ours Rare", "sans catalogue : ses identifiants")
	return true
