class_name ProgressRunner
extends Panel
## Une barre de chargement sans fin (maquettes) : une piste arrondie où court de l'or, tant
## qu'on attend sans savoir combien de temps (la connexion au serveur, une reconnexion).

var _runner: Panel
var _loop: Tween


func _ready() -> void:
	theme_type_variation = ThemeBuilder.PROGRESS_TRACK
	clip_contents = true
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	custom_minimum_size = Look.PROGRESS_SIZE
	var runner := Panel.new()
	runner.theme_type_variation = ThemeBuilder.PROGRESS_FILL
	runner.mouse_filter = Control.MOUSE_FILTER_IGNORE
	runner.size = Vector2(Look.PROGRESS_SIZE.x * Look.PROGRESS_RUNNER_SHARE, Look.PROGRESS_SIZE.y)
	add_child(runner)
	_runner = runner
	visibility_changed.connect(_run)
	_run()


## L'or court tant que la barre est visible, et s'arrête sinon.
func _run() -> void:
	if _loop != null:
		_loop.kill()
		_loop = null
	if not is_visible_in_tree():
		return
	_loop = create_tween().set_loops()
	_loop.tween_property(_runner, "position:x", Look.PROGRESS_SIZE.x, Look.PROGRESS_LOOP).from(-_runner.size.x)
