class_name CardCollection
extends RefCounted
## Ce que possède le compte du joueur, tel que le serveur le dit (message `collection`,
## docs/PROTOCOLE.md) : ses cartes et son trio de cache, des `{"kind", "rarity"}`. Le
## serveur en est la seule source : le client ne la change jamais lui-même, il demande un
## nouveau trio (`trio_with`, `trio_without`) et attend la collection qui lui répond.
##
## La collection ne limite que la cache : au draft, la cache se choisit dans le trio, ou,
## sans trio, parmi 3 cartes de la collection tirées au hasard par le serveur.

## Le nombre maximal de cartes d'un trio (`TRIO_SIZE` du serveur).
const TRIO_SIZE := 3

## Les cartes possédées, dans l'ordre du serveur.
var cards: Array[Dictionary] = []
## Le trio de cache, dans l'ordre où le joueur l'a rangé ; vide : tiré au hasard à chaque
## partie.
var cache_trio: Array[Dictionary] = []

## Les cartes possédées, par clé (`key`), pour `owns`.
var _owned := {}


## Lit les cartes et le trio d'un message `collection` ; ce qui n'est pas une carte est
## ignoré, un doublon compte une fois.
func _init(raw_cards: Array = [], raw_trio: Array = []) -> void:
	cards = _read(raw_cards)
	for attack: Dictionary in cards:
		_owned[CardCatalog.key(attack)] = true
	for attack: Dictionary in _read(raw_trio):
		if cache_trio.size() < TRIO_SIZE:
			cache_trio.append(attack)


func size() -> int:
	return cards.size()


## Vrai si le compte possède cette carte.
func owns(attack: Dictionary) -> bool:
	return _owned.has(CardCatalog.key(attack))


## Vrai si cette carte est dans le trio de cache.
func in_trio(attack: Dictionary) -> bool:
	return cache_trio.any(func(card: Dictionary) -> bool: return CardCatalog.key(card) == CardCatalog.key(attack))


func trio_full() -> bool:
	return cache_trio.size() >= TRIO_SIZE


## Vrai si cette carte peut entrer dans le trio : possédée, pas déjà dedans, trio pas plein.
func can_add_to_trio(attack: Dictionary) -> bool:
	return owns(attack) and not in_trio(attack) and not trio_full()


## Le trio à demander pour y ajouter cette carte (en dernier).
func trio_with(attack: Dictionary) -> Array[Dictionary]:
	var trio: Array[Dictionary] = cache_trio.duplicate()
	if can_add_to_trio(attack):
		trio.append(_card(attack))
	return trio


## Le trio à demander pour en retirer cette carte.
func trio_without(attack: Dictionary) -> Array[Dictionary]:
	var trio: Array[Dictionary] = []
	for card: Dictionary in cache_trio:
		if CardCatalog.key(card) != CardCatalog.key(attack):
			trio.append(card)
	return trio


## Les cartes lisibles de cette liste, sans doublon.
static func _read(raw: Array) -> Array[Dictionary]:
	var read: Array[Dictionary] = []
	var seen := {}
	for card: Variant in raw:
		if card is Dictionary and card.get("kind") is String and card.get("rarity") is String:
			var attack := _card(card)
			if not seen.has(CardCatalog.key(attack)):
				seen[CardCatalog.key(attack)] = true
				read.append(attack)
	return read


## La carte seule, sans autre champ : telle que le protocole l'écrit.
static func _card(attack: Dictionary) -> Dictionary:
	return {"kind": attack.get("kind", ""), "rarity": attack.get("rarity", "")}
