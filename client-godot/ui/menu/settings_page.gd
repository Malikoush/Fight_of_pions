class_name SettingsPage
extends MenuPage
## Les réglages (maquettes 12-17, p. 13), par sections : le profil (le pseudo du compte,
## gardé par le serveur), le son (le volume de la musique et celui des effets, à zéro
## coupés) et les vibrations du téléphone, gardés sur l'appareil (`Settings`), et
## « À propos ». Pas d'adresse du serveur (choix de Martin) ; pas encore de musique : son
## réglage est prêt.

## Le joueur valide ce pseudo (Entrée, ou en quittant le champ) : à demander au serveur,
## qui le vérifie. Sa réponse revient par `show_pseudo`, ou son refus par `show_refusal`.
signal pseudo_submitted(pseudo: String)

var _pseudo_field: LineEdit
var _pseudo_error: Label
## Le pseudo du compte, tel que le serveur l'a donné.
var _pseudo := ""
## Le dernier pseudo demandé, tant qu'on n'y a rien retapé : Entrée puis la sortie du champ
## ne le demandent qu'une fois.
var _asked := ""


func _init(pseudo: String) -> void:
	super(Labels.SETTINGS_TITLE)
	_pseudo = pseudo


func _fill() -> void:
	var profile := _section(Labels.SETTINGS_PROFILE)
	profile.add_child(Ui.label(Labels.SETTING_NAME, ThemeBuilder.STRONG_LABEL))
	_pseudo_field = LineEdit.new()
	_pseudo_field.theme_type_variation = ThemeBuilder.TEXT_INPUT
	_pseudo_field.max_length = Look.PLAYER_NAME_MAX
	_pseudo_field.text = _pseudo
	_pseudo_field.text_submitted.connect(func(_text: String) -> void: _submit_pseudo())
	_pseudo_field.focus_exited.connect(_submit_pseudo)
	_pseudo_field.text_changed.connect(func(_text: String) -> void:
		_asked = ""
		_pseudo_error.visible = false)
	profile.add_child(_pseudo_field)
	_pseudo_error = _paragraph("", ThemeBuilder.THREAT_LABEL)
	_pseudo_error.visible = false
	profile.add_child(_pseudo_error)
	profile.add_child(_paragraph(Labels.SETTING_NAME_NOTE, ThemeBuilder.CAPTION_LABEL))

	var sound := _section(Labels.SETTINGS_SOUND)
	sound.add_child(_volume_row(Labels.SETTING_MUSIC, Settings.music, func(value: float) -> void:
		Settings.music = value))
	var effects := _volume_row(Labels.SETTING_EFFECTS, Settings.volume, func(value: float) -> void:
		Settings.volume = value)
	# Au lâcher du curseur des effets, un son pour l'entendre.
	(effects.get_child(1) as HSlider).drag_ended.connect(func(_moved: bool) -> void: Sounds.play("place"))
	sound.add_child(effects)
	var vibration := CheckButton.new()
	vibration.text = Labels.SETTING_VIBRATION
	vibration.button_pressed = Settings.vibration
	vibration.toggled.connect(func(on: bool) -> void:
		Settings.vibration = on
		Settings.save()
		Settings.vibrate(Look.TURN_VIBRATION_MS))
	sound.add_child(vibration)

	var about := _section(Labels.SETTINGS_ABOUT)
	about.add_child(_paragraph(Labels.ABOUT_TEXT, ThemeBuilder.STRONG_LABEL))
	body.add_child(_paragraph(Labels.SETTINGS_NOTE, ThemeBuilder.CAPTION_LABEL))


## Le pseudo du compte a changé (ou vient d'arriver) : le champ le reprend, sauf pendant
## qu'on y tape.
func show_pseudo(pseudo: String) -> void:
	_pseudo = pseudo
	if _pseudo_field != null and not _pseudo_field.has_focus():
		_pseudo_field.text = pseudo
		_pseudo_error.visible = false


## Le serveur refuse le pseudo demandé : son message, sous le champ.
func show_refusal(message: String) -> void:
	_pseudo_error.text = Labels.sentence(message)
	_pseudo_error.visible = true


## Demande le pseudo tapé, s'il a changé (sans espace autour, comme le serveur le garde).
func _submit_pseudo() -> void:
	var typed := _pseudo_field.text.strip_edges()
	if typed != _pseudo and typed != _asked:
		_asked = typed
		pseudo_submitted.emit(typed)


## Une section : un panneau, son nom en petites capitales d'or ; rend sa colonne.
func _section(title: String) -> VBoxContainer:
	var panel := PanelContainer.new()
	panel.theme_type_variation = ThemeBuilder.PANEL
	var column := Ui.vbox(Look.GAP_S)
	column.add_child(Ui.label(title, ThemeBuilder.TAG_LABEL))
	panel.add_child(column)
	body.add_child(panel)
	return column


## Une ligne de volume : son nom, le curseur, la valeur en pour cent. `changed` reçoit la
## valeur (de 0 à 1) à chaque changement, à la souris comme au clavier ; elle est gardée au
## lâcher du curseur, ou quand il perd le focus.
func _volume_row(title: String, value: float, changed: Callable) -> HBoxContainer:
	var row := Ui.hbox(Look.GAP_M)
	var label := Ui.label(title, ThemeBuilder.STRONG_LABEL)
	label.custom_minimum_size.x = Look.SETTING_LABEL_WIDTH
	row.add_child(label)
	var slider := HSlider.new()
	slider.min_value = 0.0
	slider.max_value = 1.0
	slider.step = Look.VOLUME_STEP
	slider.value = value
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(slider)
	var shown := Ui.label(Labels.PERCENT % roundi(value * Look.PERCENT_SCALE), ThemeBuilder.PILL_LABEL)
	shown.custom_minimum_size.x = Look.SETTING_VALUE_WIDTH
	shown.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	row.add_child(shown)
	slider.value_changed.connect(func(now: float) -> void:
		shown.text = Labels.PERCENT % roundi(now * Look.PERCENT_SCALE)
		changed.call(now))
	slider.drag_ended.connect(func(_moved: bool) -> void: Settings.save())
	slider.focus_exited.connect(Settings.save)
	return row
