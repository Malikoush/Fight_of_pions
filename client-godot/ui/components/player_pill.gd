class_name PlayerPill
extends Button
## La pastille du joueur (maquettes) : son palet et son pseudo, celui du compte
## (`Session.pseudo`, « Joueur » tant qu'il n'est pas connu). La toucher ouvre les réglages.


func _ready() -> void:
	theme_type_variation = ThemeBuilder.PILL_BUTTON
	icon = Ui.chip_texture(ViewData.PLAYER_ONE)
	if text == "":
		text = Labels.DEFAULT_PLAYER_NAME
	size_flags_vertical = Control.SIZE_SHRINK_CENTER
	focus_mode = Control.FOCUS_NONE
	custom_minimum_size.y = Look.PILL_CHIP.y + get_theme_stylebox(&"normal").get_minimum_size().y


## Montre ce pseudo (il vient d'arriver, ou de changer).
func show_pseudo(pseudo: String) -> void:
	text = pseudo
