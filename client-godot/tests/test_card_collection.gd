extends TestCase
## La collection du compte : ce qu'il possède, son trio, et le trio à demander au serveur.

const OURS := {"kind": "Ours", "rarity": "Common"}
const LIEVRE := {"kind": "Lievre", "rarity": "Common"}
const HIBOU := {"kind": "Hibou", "rarity": "Common"}
const REINE := {"kind": "Reine", "rarity": "Common"}
const OURS_RARE := {"kind": "Ours", "rarity": "Rare"}


func test_a_collection_reads_only_cards_once() -> bool:
	var collection := CardCollection.new([OURS, OURS, {"kind": "Ours"}, "Ours", {"kind": "Lievre", "rarity": "Common", "extra": 1}], [OURS, OURS])
	equal(collection.cards, [OURS, LIEVRE], "les cartes lisibles, sans doublon ni autre champ")
	equal(collection.cache_trio, [OURS], "le trio sans doublon")
	check(collection.owns(OURS), "possédée")
	check(not collection.owns(OURS_RARE), "la rareté compte")
	equal(CardCollection.new().size(), 0, "vide")
	return true


func test_the_trio_to_ask_adds_or_removes_one_card() -> bool:
	var collection := CardCollection.new([OURS, LIEVRE, HIBOU, REINE], [LIEVRE])
	equal(collection.trio_with(OURS), [LIEVRE, OURS], "ajoutée en dernier")
	equal(collection.trio_with(LIEVRE), [LIEVRE], "déjà dedans")
	equal(collection.trio_with(OURS_RARE), [LIEVRE], "pas à soi : rien ne change")
	equal(collection.trio_without(LIEVRE), [], "retirée")
	equal(collection.cache_trio, [LIEVRE], "la collection, elle, attend le serveur")
	var full := CardCollection.new([OURS, LIEVRE, HIBOU, REINE], [OURS, LIEVRE, HIBOU])
	check(full.trio_full(), "trio complet")
	check(not full.can_add_to_trio(REINE), "pas de quatrième")
	equal(full.trio_with(REINE), [OURS, LIEVRE, HIBOU], "le trio ne déborde pas")
	equal(CardCollection.new([], [OURS, LIEVRE, HIBOU, REINE]).cache_trio.size(), CardCollection.TRIO_SIZE, "un trio reçu trop long est coupé")
	return true
