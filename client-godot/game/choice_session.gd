class_name ChoiceSession
extends RefCounted
## Le choix d'une carte en cours (docs/PROTOCOLE.md, « Le déroulé ») : la liste des coups
## déjà choisis, et l'aperçu que le serveur rend pour eux (les choix suivants). Rien n'est
## joué tant que la carte n'est pas complète : annuler, c'est oublier cette liste.
##
## Chaque choix est pris dans les coups permis de l'aperçu précédent, tel quel. Quand le
## serveur répond qu'il n'y a plus rien à choisir (aperçu `null`), la liste est prête à
## être jouée, coup par coup.

## Un nouvel aperçu est arrivé : les choix suivants sont dans `choices`.
signal changed
## La carte est complète : ces coups sont à jouer, dans l'ordre.
signal complete(actions: Array)

## L'emplacement de la carte dans l'arsenal.
var slot: int
## Les coups choisis, à commencer par celui qui lance la carte.
var actions: Array = []
## L'aperçu après ces coups (la décision attendue et les choix suivants), ou `null` en
## attendant la réponse du serveur.
var choices: ViewData
## Les cibles choisies (les choix faits quand la carte attendait une cible).
var targets := 0
## Les aperçus demandés et pas encore reçus. Si le choix est abandonné avant, leurs
## réponses arriveront quand même, et devront être ignorées (`GameScreen`).
var pending := 0


func _init(card_slot: int, use_attack: Variant) -> void:
	slot = card_slot
	choose(use_attack)


## Ajoute ce choix (un coup permis de l'aperçu) et demande l'aperçu suivant.
func choose(action: Variant) -> void:
	if choices != null and choices.decision_kind() == "ChooseTarget":
		targets += 1
	actions.append(action)
	choices = null
	pending += 1
	ServerLink.preview(actions)


## La réponse du serveur à l'aperçu demandé.
func on_preview(view: ViewData) -> void:
	pending -= 1
	if view == null:
		complete.emit(actions)
		return
	choices = view
	changed.emit()


## Vrai tant que la réponse à l'aperçu n'est pas arrivée.
func is_waiting() -> bool:
	return choices == null


## Les cases déjà choisies (le lanceur, puis les cibles), dans l'ordre.
func chosen_cells() -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	for action: Variant in actions:
		var cell := ViewData.action_cell(action)
		if cell != ViewData.NO_CELL:
			cells.append(cell)
	return cells
