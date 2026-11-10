class_name SettingsPage
extends MenuPage
## Les réglages (maquettes 12-17, p. 13), gardés sur l'appareil (`Settings`), par sections :
## le profil (le pseudo), le son (le volume de la musique et celui des effets, à zéro
## coupés), les vibrations du téléphone, et « À propos ». Pas d'adresse du serveur (choix
## de Martin) ; pas encore de musique : son réglage est prêt.

## Le pseudo a changé (la pastille du joueur le reprend).
signal player_name_changed


func _init() -> void:
	super(Labels.SETTINGS_TITLE)


func _fill() -> void:
	var profile := _section(Labels.SETTINGS_PROFILE)
	profile.add_child(Ui.label(Labels.SETTING_NAME, ThemeBuilder.STRONG_LABEL))
	var name_field := LineEdit.new()
	name_field.theme_type_variation = ThemeBuilder.TEXT_INPUT
	name_field.max_length = Look.PLAYER_NAME_MAX
	name_field.text = Settings.player_name
	name_field.text_changed.connect(func(text: String) -> void:
		var cleaned := text.strip_edges()
		Settings.player_name = cleaned if cleaned != "" else Labels.DEFAULT_PLAYER_NAME
		Settings.save()
		player_name_changed.emit())
	profile.add_child(name_field)
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
