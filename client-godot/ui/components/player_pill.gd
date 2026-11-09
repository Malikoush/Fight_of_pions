class_name PlayerPill
extends Button
## La pastille du joueur (maquettes) : son palet et son pseudo (`Settings`, « Joueur » tant
## qu'il ne l'a pas choisi : le choix viendra dans les réglages, en R8). La toucher ouvre
## les réglages.


func _ready() -> void:
	theme_type_variation = ThemeBuilder.PILL_BUTTON
	icon = Ui.chip_texture(ViewData.PLAYER_ONE)
	text = Settings.player_name
	size_flags_vertical = Control.SIZE_SHRINK_CENTER
	focus_mode = Control.FOCUS_NONE
	custom_minimum_size.y = Look.PILL_CHIP.y + get_theme_stylebox(&"normal").get_minimum_size().y
