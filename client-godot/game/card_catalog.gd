class_name CardCatalog
extends RefCounted
## Le catalogue des cartes envoyé par le serveur (`cards`, docs/PROTOCOLE.md) : le nom de
## chaque famille et de ses raretés, ce qu'elle fait, ses règles et l'effet de chaque
## carte. Le coût et la recharge, eux, viennent de la configuration de la partie (vue).

var _families := {}


func _init(families: Array) -> void:
	for family: Variant in families:
		if family is Dictionary and family.has("kind"):
			_families[family["kind"]] = family


## Le nom affiché d'une famille (« Coup du taureau »), ou son identifiant si le catalogue
## ne la connaît pas.
func family_name(kind: String) -> String:
	return _families.get(kind, {}).get("name", kind)


## Le nom affiché d'une rareté (« Épique »).
func rarity_name(kind: String, rarity: String) -> String:
	return _rarity(kind, rarity).get("name", rarity)


## L'effet de cette carte, à lire par le joueur ; vide si le catalogue ne le connaît pas.
func effect(kind: String, rarity: String) -> String:
	return _rarity(kind, rarity).get("effect", "")


## Vrai pour une réaction, jouée pendant le tour adverse.
func is_reaction(kind: String) -> bool:
	return _families.get(kind, {}).get("reaction", false)


func _rarity(kind: String, rarity: String) -> Dictionary:
	for card: Variant in _families.get(kind, {}).get("rarities", []):
		if card is Dictionary and card.get("rarity") == rarity:
			return card
	return {}
