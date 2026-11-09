class_name BootScreen
extends Screen
## L'écran de démarrage (maquettes) : une rangée de cases avec un palet de chaque joueur, le
## nom du jeu en grand, et l'état de la connexion au serveur, avec une barre qui court tant
## que la connexion se cherche ; après quelques essais ratés, il dit que le serveur ne
## répond pas et propose de réessayer.

## Après autant d'essais ratés, l'écran dit que le serveur ne répond pas.
const FAILURES_BEFORE_WARNING := 3

signal retry_requested

var _loop: Tween

@onready var _page: MarginContainer = %Page
@onready var _top_space: Control = %TopSpace
@onready var _art: ElementArt = %Art
@onready var _title: Label = %Title
@onready var _track: Panel = %Track
@onready var _runner: Panel = %Runner
@onready var _status: Label = %Status
@onready var _retry: Button = %Retry


func _ready() -> void:
	Ui.set_page_margins(_page)
	_top_space.size_flags_stretch_ratio = Look.BOOT_TOP_SHARE
	_title.text = Labels.GAME_NAME_STACKED
	_status.text = Labels.CONNECTING
	_retry.text = Labels.RETRY
	_retry.custom_minimum_size.y = Look.BUTTON_HEIGHT
	_retry.pressed.connect(retry_requested.emit)
	_track.custom_minimum_size = Look.PROGRESS_SIZE
	_runner.size = Vector2(Look.PROGRESS_SIZE.x * Look.PROGRESS_RUNNER_SHARE, Look.PROGRESS_SIZE.y)
	_draw_art()
	_loop = create_tween().set_loops()
	_loop.tween_property(_runner, "position:x", Look.PROGRESS_SIZE.x, Look.PROGRESS_LOOP).from(-_runner.size.x)


func on_connection(status: ServerLink.Status) -> void:
	if status == ServerLink.Status.ONLINE:
		return
	var lost := ServerLink.failed_attempts >= FAILURES_BEFORE_WARNING
	_status.text = Labels.UNREACHABLE if lost else Labels.CONNECTING
	_retry.visible = lost


## Le dessin : des cases des éléments fournis, côte à côte pointe contre pointe, et un palet
## de chaque joueur posé à leurs jonctions.
func _draw_art() -> void:
	var anchor := ElementArt.first_anchor(Look.BOOT_ART_SCALE)
	var step := Look.ELEMENT_TILE_WIDTH * Look.BOOT_ART_SCALE
	for index in Look.BOOT_TILES:
		_art.add_tile(anchor + Vector2(step * index, 0.0), Look.BOOT_ART_SCALE)
	var players := [ViewData.PLAYER_ONE, ViewData.PLAYER_TWO]
	for index in mini(players.size(), Look.BOOT_TILES - 1):
		_art.add_pion(players[index], anchor + Vector2(step * (index + 0.5), 0.0), Look.BOOT_ART_SCALE * Look.BOOT_PION_SCALE)
