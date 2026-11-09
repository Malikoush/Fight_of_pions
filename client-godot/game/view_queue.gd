class_name ViewQueue
extends Node
## La file des vues de l'écran de partie. Chaque vue attend que la précédente ait fini de
## s'animer : le bot joue sans pause, et l'on doit voir chacun de ses coups. Tant que des
## vues attendent, celle qui est affichée est périmée (`is_behind`) : aucun geste n'est pris.
##
## Quand une carte part coup par coup (`expect_sequence`), ses vues intermédiaires (le
## lanceur choisi, les cibles) ne sont pas montrées : seulement la dernière, avec les
## événements de toutes.

## Montre une vue et ses événements, et rend la durée de son animation (secondes).
var show: Callable = func(_view: ViewData, _events: Array) -> float: return 0.0
## Vrai tant que la file doit attendre avant la vue suivante (la cache révélée attend un
## toucher) ; `resume` la relance ensuite.
var blocked: Callable = func() -> bool: return false

var _waiting: Array[Array] = []
var _playing: Tween
## Les vues d'une carte qui part, encore attendues, et les événements des vues déjà reçues.
var _sequence_left := 0
var _carried: Array = []


## Une vue reçue du serveur : montrée dès que les précédentes ont fini de s'animer.
func push(view: ViewData, events: Array) -> void:
	_carried.append_array(events)
	if _sequence_left > 0:
		_sequence_left -= 1
		if _sequence_left > 0:
			return
	_waiting.append([view, _carried])
	_carried = []
	resume()


## Vrai si des vues attendent leur tour.
func is_behind() -> bool:
	return not _waiting.is_empty()


## Une carte part en `count` coups : seule la vue du dernier sera montrée.
func expect_sequence(count: int) -> void:
	_sequence_left = count


## Vrai pendant qu'une carte part, avant sa dernière vue.
func is_sending() -> bool:
	return _sequence_left > 0


## La carte qui partait n'aura pas toutes ses vues (refus, coupure) : la suite est oubliée.
func cancel_sequence() -> void:
	_sequence_left = 0
	_carried = []


## Reprend la file, si aucune animation n'est en cours.
func resume() -> void:
	if _playing == null or not _playing.is_running():
		_play_next()


func _play_next() -> void:
	while not _waiting.is_empty():
		if blocked.call():
			return
		var next: Array = _waiting.pop_front()
		var duration: float = show.call(next[0], next[1])
		if duration > 0.0:
			_playing = create_tween()
			_playing.tween_interval(duration)
			_playing.tween_callback(_play_next)
			return
