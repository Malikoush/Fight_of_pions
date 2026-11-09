class_name JoinScreen
extends Screen
## Rejoindre la salle d'un ami (maquettes 1-5, p. 5 et 6) : le code à 5 caractères, une case
## par caractère, tapé au clavier (celui du téléphone s'ouvre tout de suite), et
## « REJOINDRE », allumé quand le code est complet. Un refus du serveur (salle inconnue,
## complète) s'affiche sous les cases, bordées de braise ; le code reste tapé, pour
## corriger une lettre, et le message suivant remplace le précédent. Sa place est gardée
## même vide : le bouton ne saute pas quand il paraît.

signal join_requested(code: String)
signal back_requested

var _waiting := false

@onready var _page: MarginContainer = %Page
@onready var _header: PageHeader = %Header
@onready var _help: Label = %Help
@onready var _code: CodeField = %Code
@onready var _error: RichTextLabel = %Error
@onready var _join: Button = %Join


func _ready() -> void:
	Ui.fit_page(_page, Look.CONTENT_MAX_WIDTH)
	_header.set_title(Labels.JOIN_TITLE)
	_header.back_pressed.connect(back_requested.emit)
	_help.text = Labels.JOIN_HELP
	_join.text = Labels.JOIN
	_join.custom_minimum_size.y = Look.BUTTON_HEIGHT
	_join.pressed.connect(_on_join)
	_code.changed.connect(func(_text: String) -> void:
		_show_error(false)
		_refresh())
	_code.submitted.connect(func(_text: String) -> void: _on_join())
	# Une ligne vide garde la place du message.
	_write_error("")
	_show_error(false)
	_refresh()
	_code.focus()


## Un refus du serveur : son message sous les cases (bordées de braise s'il parle du code :
## salle inconnue, complète), le bouton de nouveau permis, et le focus rendu au code pour
## corriger une lettre.
func on_refused(category: ServerErrors.Category, message: String) -> Reply:
	_waiting = false
	_write_error(Labels.sentence(message))
	_show_error(true)
	if category == ServerErrors.Category.JOIN_REFUSED:
		_code.show_refused()
	_refresh()
	_code.focus()
	return Reply.HANDLED


## Une coupure a pu perdre la demande : le bouton se réactive.
func on_connection(status: ServerLink.Status) -> void:
	if status == ServerLink.Status.RECONNECTING:
		_waiting = false
		_refresh()


func _on_join() -> void:
	if _join.disabled:
		return
	_waiting = true
	_refresh()
	join_requested.emit(_code.code())


## Le message, centré, derrière sa croix ; il passe à la ligne s'il est long. Le texte du
## serveur est ajouté tel quel, jamais lu comme du BBCode.
func _write_error(text: String) -> void:
	_error.clear()
	_error.push_paragraph(HORIZONTAL_ALIGNMENT_CENTER)
	_error.add_image(load(Ui.ICON_DIR + "fermer.svg"), Look.ICON_SIZE, Look.ICON_SIZE, Look.THREAT, INLINE_ALIGNMENT_CENTER)
	_error.add_text(Labels.ICON_SPACE + text)
	_error.pop()


func _show_error(shown: bool) -> void:
	_error.modulate.a = 1.0 if shown else 0.0


func _refresh() -> void:
	_join.disabled = _waiting or not _code.is_complete()
