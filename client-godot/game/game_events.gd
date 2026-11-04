class_name GameEvents
extends RefCounted
## La lecture des événements que le serveur joint à chaque vue (`Event` du moteur, filtrés
## pour ce joueur). Ils ne servent qu'à animer et à raconter : l'état affiché vient toujours
## de la vue. Un événement s'écrit `{"PionPlaced": {…}}`, ou `"…"` sans donnée.


## La sorte d'un événement (`PionPlaced`, `TurnEnded`…).
static func kind(event: Variant) -> String:
	return Wire.tag(event)


## Les données d'un événement, ou un dictionnaire vide.
static func payload(event: Variant) -> Dictionary:
	var data: Variant = Wire.content(event)
	return data if data is Dictionary else {}


## Les pions posés dans ces événements (`{"id": 3, "owner": "One"}`), dans l'ordre.
static func placed_pions(events: Array) -> Array[Dictionary]:
	var pions: Array[Dictionary] = []
	for event: Variant in events:
		if kind(event) == "PionPlaced":
			var pion: Variant = payload(event).get("pion")
			if pion is Dictionary:
				pions.append(pion)
	return pions
