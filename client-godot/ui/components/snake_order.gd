class_name SnakeOrder
extends HBoxContainer
## L'ordre des picks en serpent (maquettes) : un palet par pick, avec le nom court de qui le
## fait (« TOI », « BOT »). Les picks passés sont en retrait ; celui en cours est bordé d'or.


func _ready() -> void:
	theme_type_variation = ThemeBuilder.ROW_XS
	mouse_filter = Control.MOUSE_FILTER_IGNORE


## Montre ces picks (le joueur de chacun, dans l'ordre), celui de rang `current` en cours
## (-1 : aucun). `viewer` est « TOI » ; l'autre joueur porte `opponent_tag`.
func show_order(players: Array[String], current: int, viewer: String, opponent_tag: String) -> void:
	Ui.clear(self)
	for index in players.size():
		var column := VBoxContainer.new()
		column.theme_type_variation = ThemeBuilder.COLUMN_XS
		column.mouse_filter = Control.MOUSE_FILTER_IGNORE
		var slot := PanelContainer.new()
		slot.theme_type_variation = ThemeBuilder.SNAKE_SLOT_CURRENT if index == current else ThemeBuilder.SNAKE_SLOT
		slot.mouse_filter = Control.MOUSE_FILTER_IGNORE
		slot.add_child(Ui.chip(players[index], Look.SNAKE_CHIP))
		column.add_child(slot)
		var who := Labels.TAG_YOU if players[index] == viewer else opponent_tag
		column.add_child(Ui.centered_label(who, ThemeBuilder.TAG_LABEL if index == current else ThemeBuilder.TAG_PLAIN_LABEL))
		if index < current:
			column.modulate.a = Look.SNAKE_PAST_ALPHA
		add_child(column)
