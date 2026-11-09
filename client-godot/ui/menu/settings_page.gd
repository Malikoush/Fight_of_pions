class_name SettingsPage
extends MenuPage
## Les réglages utiles, gardés sur l'appareil (`Settings`) : le son et son volume, les
## vibrations du téléphone.


func _init() -> void:
	super(Labels.SETTINGS_TITLE)


func _fill() -> void:
	var sound := _switch(Labels.SETTING_SOUND, Settings.sound)
	var volume := HSlider.new()
	volume.min_value = 0.0
	volume.max_value = 1.0
	volume.step = Look.VOLUME_STEP
	volume.value = Settings.volume
	volume.editable = Settings.sound
	volume.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sound.toggled.connect(func(on: bool) -> void:
		Settings.sound = on
		volume.editable = on
		Settings.save()
		Sounds.play("place"))
	volume.drag_ended.connect(func(_changed: bool) -> void:
		Settings.volume = volume.value
		Settings.save()
		Sounds.play("place"))
	body.add_child(sound)
	var row := Ui.hbox(Look.GAP_M)
	row.add_child(Ui.label(Labels.SETTING_VOLUME, ThemeBuilder.STRONG_LABEL))
	row.add_child(volume)
	body.add_child(row)
	var vibration := _switch(Labels.SETTING_VIBRATION, Settings.vibration)
	vibration.toggled.connect(func(on: bool) -> void:
		Settings.vibration = on
		Settings.save()
		Settings.vibrate(Look.TURN_VIBRATION_MS))
	body.add_child(vibration)
	body.add_child(_paragraph(Labels.SETTINGS_NOTE))


func _switch(text: String, on: bool) -> CheckButton:
	var button := CheckButton.new()
	button.text = text
	button.button_pressed = on
	return button
