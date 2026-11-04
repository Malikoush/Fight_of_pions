class_name ViewData
extends RefCounted
## La vue d'un joueur, telle que le serveur l'envoie (`PlayerView`, docs/PROTOCOLE.md), avec
## des lectures typées. Elle ne calcule aucune règle : elle relit les champs et range les
## coups permis par case, dans l'ordre où le serveur les donne.
##
## Une case s'écrit `Vector2i(colonne, rangée)`, à partir de 0 en haut à gauche de la
## grille, mur compris.

const PLAYER_ONE := "One"
const PLAYER_TWO := "Two"
const NO_CELL := Vector2i(-1, -1)

var raw: Dictionary
var _actions_by_cell := {}


func _init(raw_view: Dictionary) -> void:
	raw = raw_view
	for action: Variant in legal_actions():
		var cell := action_cell(action)
		if cell != NO_CELL:
			if not _actions_by_cell.has(cell):
				_actions_by_cell[cell] = []
			_actions_by_cell[cell].append(action)


# --- Joueurs ---

func viewer() -> String:
	return raw.get("viewer", PLAYER_ONE)


func opponent() -> String:
	return other(viewer())


static func other(player: String) -> String:
	return PLAYER_TWO if player == PLAYER_ONE else PLAYER_ONE


static func side_index(player: String) -> int:
	return 0 if player == PLAYER_ONE else 1


## L'arsenal de ce joueur, emplacement par emplacement : `{"attack": {"kind", "rarity"},
## "cooldown_left", "secret"}`, ou `null` pour une carte cachée qu'on ne peut pas voir.
func arsenal(player: String) -> Array:
	return side(player).get("arsenal", [])


## Le coût et la recharge de cette carte (`{"kind", "rarity"}`), lus dans le catalogue de
## la configuration : `{"cost", "cooldown"}`, vide si elle n'y est pas.
func stats_of(attack: Dictionary) -> Dictionary:
	for entry: Variant in config().get("attack_catalog", []):
		if entry is Dictionary and entry.get("attack") == attack:
			return entry.get("stats", {})
	return {}


## Ce qu'a ce joueur : réserve, banque, actions et poses du tour, arsenal…
func side(player: String) -> Dictionary:
	var sides: Array = raw.get("sides", [])
	var index := side_index(player)
	return sides[index] if index < sides.size() else {}


# --- Tour et décision ---

func turn() -> int:
	return raw.get("turn", 0)


func current_player() -> String:
	return raw.get("current_player", PLAYER_ONE)


func config() -> Dictionary:
	return raw.get("config", {})


func end() -> Variant:
	return raw.get("end")


func is_over() -> bool:
	return end() != null


## La décision attendue, par exemple `{"MainAction": {"player": "One"}}`, ou `null`.
func decision() -> Variant:
	return raw.get("decision")


## Le nom de la décision attendue (`MainAction`, `ChooseLauncher`, `ChooseTarget`, `React`,
## `Draft`), vide si aucune.
func decision_kind() -> String:
	return Wire.tag(decision())


## Le joueur qui doit décider, vide si personne. Le draft dit son joueur dans son étape.
func decision_player() -> String:
	var details: Variant = Wire.content(decision())
	if not (details is Dictionary):
		return ""
	if details.has("player"):
		return details["player"]
	var player: Variant = Wire.content(details.get("step"))
	return player if player is String else ""



# --- Coups permis ---

## Les coups permis à ce joueur, à renvoyer tels quels ; vide si ce n'est pas à lui.
func legal_actions() -> Array:
	return raw.get("legal_actions", [])


## Les coups permis qui désignent cette case, dans l'ordre du serveur.
func actions_at(cell: Vector2i) -> Array:
	return _actions_by_cell.get(cell, [])


## Le coup permis de cette sorte (`Place`, `Choose`…) sur cette case, ou `null`.
func action_at(cell: Vector2i, kind: String) -> Variant:
	for action: Variant in actions_at(cell):
		if action_kind(action) == kind:
			return action
	return null


## Le coup permis sans case (`EndTurn`, `Pass`, `FinishAttack`…), ou `null`.
func simple_action(kind: String) -> Variant:
	for action: Variant in legal_actions():
		if action is String and action == kind:
			return action
	return null


## La sorte d'un coup : `"EndTurn"` pour un coup sans donnée, la clé sinon (`"Place"`).
static func action_kind(action: Variant) -> String:
	return Wire.tag(action)


## La case que désigne un coup (`Place`, `Choose`, `StepQueen`, `Shoot`), ou `NO_CELL`. Le
## pas du Lièvre est rangé sous sa case de départ : on touche d'abord le pion.
static func action_cell(action: Variant) -> Vector2i:
	var payload: Variant = Wire.content(action)
	if payload is Dictionary and payload.has("from") and payload["from"] is Dictionary:
		return to_cell(payload["from"])
	if payload is Dictionary and payload.has("row") and payload.has("col"):
		return to_cell(payload)
	return NO_CELL


## La case d'arrivée d'un pas du Lièvre, ou `NO_CELL` pour un autre coup.
static func step_destination(action: Variant) -> Vector2i:
	if action is Dictionary and action.get("HareStep") is Dictionary:
		var to: Variant = action["HareStep"].get("to")
		return to_cell(to) if to is Dictionary else NO_CELL
	return NO_CELL


## Les cases désignées par les coups permis de cette sorte.
func cells_of(kind: String) -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	for cell: Vector2i in _actions_by_cell:
		if action_at(cell, kind) != null:
			cells.append(cell)
	return cells


## Le coup permis qui lance la carte de cet emplacement de l'arsenal, ou `null`.
func use_attack(slot: int) -> Variant:
	for action: Variant in legal_actions():
		if action is Dictionary and action.get("UseAttack") == slot:
			return action
	return null


static func to_cell(position: Dictionary) -> Vector2i:
	return Vector2i(position.get("col", -1), position.get("row", -1))


# --- Plateau ---

## Le nombre de cases par côté de la grille, mur compris.
func grid_size() -> int:
	return raw.get("board", {}).get("size", 0)


## Le contenu d'une case (`{"pion": …, "tomb": …}`), ou `null` pour le mur et hors de la
## grille.
func cell_content(at: Vector2i) -> Variant:
	var size := grid_size()
	if at.x < 0 or at.y < 0 or at.x >= size or at.y >= size:
		return null
	var cells: Array = raw.get("board", {}).get("cells", [])
	var index := at.y * size + at.x
	return cells[index] if index < cells.size() else null


func is_wall(at: Vector2i) -> bool:
	return cell_content(at) == null


## Le pion de cette case (`{"id": 3, "owner": "One"}`), ou `null`.
func pion_at(at: Vector2i) -> Variant:
	var content: Variant = cell_content(at)
	return content.get("pion") if content is Dictionary else null


## La case de ce pion (par identité), ou `NO_CELL` s'il n'est pas sur le plateau.
func cell_of(pion_id: int) -> Vector2i:
	var cells: Array = raw.get("board", {}).get("cells", [])
	var size := grid_size()
	for index in cells.size():
		var content: Variant = cells[index]
		if content is Dictionary and content.get("pion") is Dictionary and content["pion"].get("id") == pion_id:
			@warning_ignore("integer_division")
			return Vector2i(index % size, index / size)
	return NO_CELL


## Le pion mort dont la tombe est sur cette case, ou `null`.
func tomb_at(at: Vector2i) -> Variant:
	var content: Variant = cell_content(at)
	return content.get("tomb") if content is Dictionary else null


## Vrai si la case est dans la bande du mur d'origine (réglage `wall_thickness`) : une
## telle case, si elle n'est pas du mur, a été creusée.
func is_in_wall_band(at: Vector2i) -> bool:
	var band: int = config().get("wall_thickness", 0)
	var size := grid_size()
	return at.x < band or at.y < band or at.x >= size - band or at.y >= size - band


# --- Ce qui dure : effets, jetons, infirmerie, cimetière, Terre brûlée ---

## Les effets continus qui visent ce pion (par identité), dans leur ordre de création :
## `{"id", "kind", "pion", "creator", "source", "play", "end"}`. Leur sorte se lit avec
## `Wire.tag(effect["kind"])` (`Immobilized`, `Pact`…), leur fin avec `Wire.tag(effect["end"])`
## (`Turn`, `Framed`, `Tokens`, `Never`…).
func effects_on(pion_id: int) -> Array[Dictionary]:
	var found: Array[Dictionary] = []
	for effect: Variant in raw.get("effects", []):
		if effect is Dictionary and effect.get("pion") == pion_id:
			found.append(effect)
	return found


## Les piles de jetons du Massacre divin de ce joueur : `{"source", "owner", "reward",
## "tokens"}`, où `reward` vaut `{"Points": n}` ou `"Graveyard"`.
func token_piles(player: String) -> Array[Dictionary]:
	return _owned_by(raw.get("token_piles", []), player, "owner")


## Les pions de ce joueur à l'infirmerie : `{"pion": {"id", "owner"}, "last_turn"}`.
func infirmary(player: String) -> Array[Dictionary]:
	var found: Array[Dictionary] = []
	for patient: Variant in raw.get("infirmary", []):
		if patient is Dictionary and patient.get("pion", {}).get("owner") == player:
			found.append(patient)
	return found


## Les pions de ce joueur au cimetière : `{"id", "owner"}`.
func graveyard(player: String) -> Array[Dictionary]:
	return _owned_by(raw.get("graveyard", []), player, "owner")


## Le dernier tour où ce joueur est sous Terre brûlée, ou -1 s'il ne l'est pas.
func scorched_until(player: String) -> int:
	var last: Variant = side(player).get("scorched_until")
	return last if last is int else -1


static func _owned_by(items: Array, player: String, field: String) -> Array[Dictionary]:
	var found: Array[Dictionary] = []
	for item: Variant in items:
		if item is Dictionary and item.get(field) == player:
			found.append(item)
	return found
