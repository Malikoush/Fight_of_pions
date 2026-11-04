class_name GameJournal
extends RefCounted
## Le journal des coups : ce qui s'est passé dans la partie, raconté à partir des seuls
## événements joints aux vues (rien n'est déduit). Chaque ligne dit son tour, le joueur
## concerné, et ce qui s'est passé. Une ligne « d'action » est faite par ce joueur (il pose,
## il lance une carte) ; une ligne de conséquence le concerne (son pion est tué, déplacé,
## immobilisé…).
##
## Le journal ne commence qu'avec la connexion : à la reprise d'une partie, le serveur
## n'envoie pas les coups passés.

## Une ligne s'écrit `{"turn", "player", "acts", "text"}` (`player` vide si elle ne concerne
## personne en particulier, comme un mur creusé). Les lignes sont gardées par le panneau qui
## les affiche (`JournalPanel`).

var _catalog: CardCatalog


## Les lignes des événements de cette vue, de la plus ancienne à la plus récente.
## `turn_before` est le tour de la vue précédente : les événements d'avant une fin de tour
## sont de ce tour-là, ceux d'après sont du tour de cette vue.
func lines(events: Array, view: ViewData, turn_before: int, catalog: CardCatalog) -> Array[Dictionary]:
	_catalog = catalog
	var added: Array[Dictionary] = []
	var turn := turn_before
	for event: Variant in events:
		var data := GameEvents.payload(event)
		var kind := GameEvents.kind(event)
		if kind == "TurnEnded":
			turn = view.turn()
			continue
		var line := _line(kind, data, view)
		if not line.is_empty():
			line["turn"] = turn
			added.append(line)
	return added


## La ligne d'un événement, ou rien s'il ne se raconte pas (points, recharges…).
func _line(kind: String, data: Dictionary, view: ViewData) -> Dictionary:
	match kind:
		"PionPlaced":
			return _act(_owner(data, "pion"), Labels.JOURNAL_PLACE % _cell(data, "at"))
		"CardRevealed":
			return _act(data.get("player", ""), Labels.JOURNAL_REVEAL % _card(data.get("attack")))
		"AttackUsed":
			return _act(data.get("player", ""), Labels.JOURNAL_ATTACK % [_card(data.get("attack")), _cell(data, "launcher")])
		"QueenShot":
			var queen: Variant = data.get("queen")
			var shooter: Variant = view.pion_at(ViewData.to_cell(queen)) if queen is Dictionary else null
			var player: String = shooter.get("owner", "") if shooter is Dictionary else ""
			return _act(player, Labels.JOURNAL_SHOT % _cell(data, "target"))
		"PionKilled":
			return _about(_owner(data, "pion"), Labels.JOURNAL_KILLED % _cell(data, "at"))
		"PionMoved":
			return _about(_owner(data, "pion"), Labels.JOURNAL_MOVED % [_cell(data, "from"), _cell(data, "to")])
		"PionsSwapped":
			return _about("", Labels.JOURNAL_SWAPPED % [_cell(data, "first_at"), _cell(data, "second_at")])
		"WallDug":
			return _about("", Labels.JOURNAL_DUG % _cell(data, "at"))
		"GraveyardEntered":
			# La donnée de l'événement est le pion lui-même.
			return _about(_owner_of(data), Labels.JOURNAL_GRAVEYARD)
		"InfirmaryLeft":
			return _about(_owner(data, "pion"), Labels.JOURNAL_HEALED)
		"ScorchedEarthStarted":
			return _about(data.get("player", ""), Labels.JOURNAL_SCORCHED % data.get("last_turn", 0))
		"TokenPileStarted":
			return _about(data.get("owner", ""), Labels.JOURNAL_TOKENS % Labels.token_pile(data.get("tokens", 0), data.get("reward")))
		"EffectStarted":
			return _effect(data, view)
	return {}


## Un effet continu qui commence, sur le pion qu'il vise (lu dans la vue, s'il y est encore).
func _effect(effect: Dictionary, view: ViewData) -> Dictionary:
	var kind := Wire.tag(effect.get("kind"))
	if not Labels.JOURNAL_EFFECTS.has(kind):
		return {}
	var id: int = effect.get("pion", -1)
	var cell := view.cell_of(id)
	var where := Labels.cell_name(cell) if cell != ViewData.NO_CELL else "?"
	var text: String = Labels.JOURNAL_EFFECTS[kind]
	if kind == PionStatus.PROTECTED and Wire.tag(effect.get("end")) == PionStatus.FRAMED:
		text = Labels.JOURNAL_CHILD
	var pion: Variant = view.pion_at(cell)
	var owner: String = pion.get("owner", "") if pion is Dictionary else ""
	return _about(owner, text % where)


static func _act(player: String, text: String) -> Dictionary:
	return {"player": player, "acts": true, "text": text}


static func _about(player: String, text: String) -> Dictionary:
	return {"player": player, "acts": false, "text": text}


## Le nom d'une carte (`{"kind", "rarity"}`) : « Ours Rare ». Il vient du catalogue du
## serveur : un crochet y est échappé, pour ne pas passer pour une balise du journal.
func _card(attack: Variant) -> String:
	if not (attack is Dictionary):
		return "?"
	var kind: String = attack.get("kind", "")
	var rarity: String = attack.get("rarity", "")
	var card := "%s %s" % [kind, rarity]
	if _catalog != null:
		card = "%s %s" % [_catalog.family_name(kind), _catalog.rarity_name(kind, rarity)]
	return card.replace("[", "[lb]")


static func _cell(data: Dictionary, field: String) -> String:
	var position: Variant = data.get(field)
	return Labels.cell_name(ViewData.to_cell(position)) if position is Dictionary else "?"


static func _owner(data: Dictionary, field: String) -> String:
	return _owner_of(data.get(field))


static func _owner_of(pion: Variant) -> String:
	return pion.get("owner", "") if pion is Dictionary else ""
