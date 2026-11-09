class_name BoardPreview
extends TextureRect
## L'aperçu du plateau de l'accueil (maquettes) : le vrai plateau 3D, dans son propre monde,
## sur une position de démonstration. C'est une vraie vue du serveur où l'on a posé quelques
## pions (`assets/demo/home_board.json`, fabriquée par `tests/fixtures/make_home_board.py`).
## On ne peut pas y jouer : il ne reçoit aucun toucher.
##
## Il est rendu à la résolution réelle de l'écran, pour rester net, et une seule fois par
## changement de taille : la position ne bouge pas.

const POSITION := "res://assets/demo/home_board.json"

## Sa hauteur en part de sa largeur ; 0 : la hauteur que lui donne son conteneur.
var aspect := 0.0:
	set(value):
		aspect = value
		_fit()

var _viewport: SubViewport
var _board: Board3D


func _ready() -> void:
	expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	stretch_mode = TextureRect.STRETCH_SCALE
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	custom_minimum_size.y = Look.PREVIEW_MIN_HEIGHT
	_viewport = SubViewport.new()
	_viewport.own_world_3d = true
	_viewport.transparent_bg = true
	_viewport.gui_disable_input = true
	_viewport.msaa_3d = get_tree().root.msaa_3d
	_viewport.render_target_update_mode = SubViewport.UPDATE_DISABLED
	add_child(_viewport)
	_board = Board3D.new()
	_board.set_process_unhandled_input(false)
	_viewport.add_child(_board)
	var saved: Variant = Wire.normalize((load(POSITION) as JSON).data)
	if saved is Dictionary:
		_board.show_view(ViewData.new(saved.get("view", {})))
	texture = _viewport.get_texture()
	resized.connect(_fit)
	_fit()


## La fenêtre du plateau prend la taille de l'aperçu, en pixels de l'écran ; le plateau y
## est cadré avec une marge, et rendu une fois à cette taille.
func _fit() -> void:
	if _viewport == null:
		return
	custom_minimum_size.y = maxf(Look.PREVIEW_MIN_HEIGHT, size.x * aspect)
	var scale_factor := get_viewport().get_final_transform().get_scale()
	var pixels := Vector2i((size * scale_factor).round())
	if pixels.x <= 0 or pixels.y <= 0:
		return
	_viewport.size = pixels
	_board.frame(Rect2(Vector2.ZERO, pixels).grow(-Look.PREVIEW_PADDING * scale_factor.y))
	_viewport.render_target_update_mode = SubViewport.UPDATE_ONCE
