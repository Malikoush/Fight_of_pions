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


## Le joueur qui doit décider, vide si personne. Le draft dit son joueur dans son étape.
func decision_player() -> String:
	var pending: Variant = decision()
	if not (pending is Dictionary) or pending.is_empty():
		return ""
	var details: Variant = pending.values()[0]
	if not (details is Dictionary):
		return ""
	if details.has("player"):
		return details["player"]
	var step: Variant = details.get("step")
	if step is Dictionary and not step.is_empty():
		return step.values()[0]
	return ""



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
	if action is String:
		return action
	if action is Dictionary and not action.is_empty():
		return action.keys()[0]
	return ""


## La case que désigne un coup qui en désigne une seule (`Place`, `Choose`, `StepQueen`,
## `Shoot`), ou `NO_CELL`. Le pas du Lièvre, qui en désigne deux, viendra avec les cartes.
static func action_cell(action: Variant) -> Vector2i:
	if not (action is Dictionary) or action.is_empty():
		return NO_CELL
	var payload: Variant = action.values()[0]
	if payload is Dictionary and payload.has("row") and payload.has("col"):
		return to_cell(payload)
	return NO_CELL


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
