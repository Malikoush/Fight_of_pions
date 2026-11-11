class_name EdgeGuard
extends Node
## Le geste du système, sur un téléphone : il part du bas de l'écran (sur iPhone, pour
## changer d'application), et le navigateur le transmettait au jeu comme un toucher, qui
## appuyait sur le bouton placé là (Martin, 2026-11-11 : « FERMER LA SALLE », et en partie
## « FINIR LE TOUR » ou « JOUER LA CARTE »). Un toucher qui commence dans cette bande
## (`Look.GESTURE_EDGE_POINTS`) est donc ignoré jusqu'à ce qu'il se lève, avec la souris
## que Godot en déduit. Rien ne bouge à l'écran ; un bouton du bas répond partout ailleurs
## que sur son tout dernier bord. Seulement sur téléphone (navigateur ou application).
##
## Il lit l'entrée avant les écrans (`_input` vient avant les contrôles) : placé dans la
## racine du jeu, il arrête ces touchers pour tout le monde.

## Les doigts posés dans la bande, par numéro de toucher.
var _ignored_fingers := {}
## Vrai tant que la souris déduite d'un de ces touchers est enfoncée.
var _ignoring_mouse := false


func _input(event: InputEvent) -> void:
	var area := get_viewport().get_visible_rect().size
	if blocks(event, area.y, edge_height(area)):
		get_viewport().set_input_as_handled()


## La hauteur de la bande, dans les coordonnées de l'écran du jeu (`area` : sa taille) ;
## zéro hors d'un téléphone.
static func edge_height(area: Vector2) -> float:
	if not on_phone():
		return 0.0
	var window := Vector2(DisplayServer.window_get_size())
	if window.y <= 0.0:
		return 0.0
	# Points → pixels de la fenêtre → coordonnées du jeu (étiré pour tenir la fenêtre).
	return Look.GESTURE_EDGE_POINTS * DisplayServer.screen_get_scale() * area.y / window.y


## Vrai sur un téléphone, dans un navigateur comme en application. Dans un navigateur, un
## écran tactile suffit (un PC tactile perd aussi le dernier bord de ses boutons du bas).
static func on_phone() -> bool:
	if OS.has_feature("mobile") or OS.has_feature("web_ios") or OS.has_feature("web_android"):
		return true
	return OS.has_feature("web") and DisplayServer.is_touchscreen_available()


## Vrai si cet événement est à ignorer : un toucher (ou sa souris) commencé dans la bande
## du bas, sur un écran haut de `height`, la bande haute de `edge` ; puis, jusqu'à ce qu'il
## se lève, ses glissés et son lever.
func blocks(event: InputEvent, height: float, edge: float) -> bool:
	if event is InputEventScreenTouch:
		var touch := event as InputEventScreenTouch
		if touch.pressed:
			if _in_edge(touch.position.y, height, edge):
				_ignored_fingers[touch.index] = true
				return true
			_ignored_fingers.erase(touch.index)
			return false
		return _ignored_fingers.erase(touch.index)
	if event is InputEventScreenDrag:
		return _ignored_fingers.has((event as InputEventScreenDrag).index)
	if event.device != InputEvent.DEVICE_ID_EMULATION:
		return false
	if event is InputEventMouseButton:
		var button := event as InputEventMouseButton
		if button.pressed:
			_ignoring_mouse = _in_edge(button.position.y, height, edge)
			return _ignoring_mouse
		var ignored := _ignoring_mouse
		_ignoring_mouse = false
		return ignored
	return event is InputEventMouseMotion and _ignoring_mouse


static func _in_edge(y: float, height: float, edge: float) -> bool:
	return edge > 0.0 and y >= height - edge
