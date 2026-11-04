class_name BoardTimeline
extends RefCounted
## Le déroulé d'une vue sur le plateau, lu dans ses événements, dans leur ordre : quand
## commence chaque animation, et combien de temps dure le tout. Les événements d'une même
## sorte qui se suivent jouent ensemble (les deux kills de l'Ours) ; une autre sorte attend
## que la précédente ait fini (l'attaque est lancée, puis ses cibles tombent, puis le
## lanceur saute). La tombe d'un pion tué sort du sol à la fin de son kill, sans retarder
## la suite : le moteur l'annonce juste après le kill, et les kills suivants jouent quand
## même ensemble. Les événements qui ne se voient pas (points, recharges…) ne comptent pas.
##
## Rien n'est déduit : le déroulé ne fait que dater ce que les événements racontent.

## Les temps du déroulé, chacun avec sa durée (`_duration`).
enum Beat { NONE, ATTACK, SHOT, KILL, TOMB, MOVE, DIG, DROP }

## Les pions qui bougent (par identité) : `{"delay", "arc"}`, la hauteur de leur saut.
var moves: Dictionary[int, Dictionary] = {}
## Les pions posés et les tombes ajoutées (par identité du pion) : quand.
var drops: Dictionary[int, float] = {}
var tombs: Dictionary[int, float] = {}
## Les pions tués : `{"delay", "cell"}`, la case où ils meurent (ils ont pu y sauter avant).
var kills: Dictionary[int, Dictionary] = {}
## Les éclats à plat sur une case : `{"cell", "color", "delay"}`.
var pulses: Array[Dictionary] = []
## Les tirs de la reine : `{"from", "to", "delay"}`.
var shots: Array[Dictionary] = []
## La durée du tout, en secondes ; 0 s'il n'y a rien à animer.
var length := 0.0

var _beat := Beat.NONE
var _start := 0.0
## La fin de ce qui déborde des groupes (une tombe qui sort après le dernier kill).
var _tail := 0.0


## Le déroulé de ces événements (tels que le serveur les joint à une vue).
static func of(events: Array) -> BoardTimeline:
	var timeline := BoardTimeline.new()
	for event: Variant in events:
		timeline._read(event)
	timeline.length = maxf(timeline._start + _duration(timeline._beat), timeline._tail)
	return timeline


## Le point d'un saut en cloche de `from` à `to`, à cette part du trajet : à mi-chemin, il
## est à `height` au-dessus de la droite (une parabole). Pour les pions et la balle du tir.
static func along_arc(from: Vector3, to: Vector3, height: float, progress: float) -> Vector3:
	return from.lerp(to, progress) + Vector3.UP * height * 4.0 * progress * (1.0 - progress)


func _read(event: Variant) -> void:
	var data := GameEvents.payload(event)
	match GameEvents.kind(event):
		"AttackUsed":
			_pulse(_cell(data, "launcher"), Look.ATTACK_PULSE, _begin(Beat.ATTACK))
		"QueenShot":
			shots.append({"from": _cell(data, "queen"), "to": _cell(data, "target"), "delay": _begin(Beat.SHOT)})
		"PionKilled":
			var at := _begin(Beat.KILL)
			kills[_id(data, "pion")] = {"delay": at, "cell": _cell(data, "at")}
			_pulse(_cell(data, "at"), Look.KILL_PULSE, at)
		"TombAdded":
			var dead := _id(data, "pion")
			if kills.has(dead):
				var rise: float = kills[dead]["delay"] + Look.KILL_TIME
				tombs[dead] = rise
				_tail = maxf(_tail, rise + Look.TOMB_RISE_TIME)
			else:
				tombs[dead] = _begin(Beat.TOMB)
		"PionMoved":
			moves[_id(data, "pion")] = {"delay": _begin(Beat.MOVE), "arc": _arc(_cell(data, "from"), _cell(data, "to"))}
		"PionsSwapped":
			# Les deux sautent ensemble, l'un plus haut que l'autre, pour se croiser sans se toucher.
			var at := _begin(Beat.MOVE)
			var arc := _arc(_cell(data, "first_at"), _cell(data, "second_at"))
			moves[_id(data, "first")] = {"delay": at, "arc": arc}
			moves[_id(data, "second")] = {"delay": at, "arc": arc * Look.SWAP_LOW_ARC}
		"WallDug":
			_pulse(_cell(data, "at"), Look.DIG_PULSE, _begin(Beat.DIG))
		"PionPlaced":
			drops[_id(data, "pion")] = _begin(Beat.DROP)


## Le début de ce temps : celui du groupe en cours s'il est de la même sorte, sinon la fin
## du groupe précédent.
func _begin(beat: Beat) -> float:
	if beat != _beat:
		_start += _duration(_beat)
		_beat = beat
	return _start


func _pulse(cell: Vector2i, color: Color, delay: float) -> void:
	pulses.append({"cell": cell, "color": color, "delay": delay})


## Le temps qu'occupe un groupe avant le suivant (l'éclat d'une attaque, lui, déborde un peu
## sur la suite).
static func _duration(beat: Beat) -> float:
	match beat:
		Beat.ATTACK:
			return Look.ATTACK_BEAT
		Beat.SHOT:
			return Look.SHOT_TIME
		Beat.KILL:
			return Look.KILL_TIME
		Beat.TOMB:
			return Look.TOMB_RISE_TIME
		Beat.MOVE:
			# Le saut, puis l'écrasement à l'arrivée (`Pion3D._travel`).
			return Look.JUMP_TIME + Look.PION_SQUASH_TIME
		Beat.DIG:
			return Look.DIG_BEAT
		Beat.DROP:
			return Look.PION_DROP_TIME + Look.PION_SQUASH_TIME
	return 0.0


## La hauteur d'un saut entre ces cases : plus haut quand il va plus loin, jusqu'à une borne.
static func _arc(from: Vector2i, to: Vector2i) -> float:
	var distance := Vector2(to - from).length()
	return minf(distance * Look.JUMP_ARC_PER_CELL, Look.JUMP_ARC_MAX)


static func _cell(data: Dictionary, field: String) -> Vector2i:
	var position: Variant = data.get(field)
	return ViewData.to_cell(position) if position is Dictionary else ViewData.NO_CELL


static func _id(data: Dictionary, field: String) -> int:
	var pion: Variant = data.get(field)
	return pion.get("id", -1) if pion is Dictionary else -1
