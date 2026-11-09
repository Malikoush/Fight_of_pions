class_name DraftSteps
extends HBoxContainer
## La barre des étapes du draft (maquettes) : un losange numéroté par phase, dans l'ordre du
## format (bans, picks, cache), relié au suivant par un trait. Une phase faite a son losange
## bordé d'or et le trait qui la suit d'or ; celle en cours, son losange d'or plein ; les
## suivantes sont ternes.

enum State { DONE, CURRENT, NEXT }


func _ready() -> void:
	theme_type_variation = ThemeBuilder.ROW_S
	mouse_filter = Control.MOUSE_FILTER_IGNORE


## Montre ces phases (leurs noms affichés), celle de rang `current` en cours.
func show_phases(names: Array[String], current: int) -> void:
	Ui.clear(self)
	for index in names.size():
		if index > 0:
			add_child(StepLine.new(index <= current))
		var state := State.DONE if index < current else (State.CURRENT if index == current else State.NEXT)
		var step := HBoxContainer.new()
		step.theme_type_variation = ThemeBuilder.ROW_XS
		step.mouse_filter = Control.MOUSE_FILTER_IGNORE
		step.add_child(StepDiamond.new(index + 1, state))
		var label := Ui.label(names[index], ThemeBuilder.CAPS_LABEL if state == State.NEXT else ThemeBuilder.PILL_LABEL)
		label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		step.add_child(label)
		add_child(step)


## Le losange numéroté d'une phase.
class StepDiamond:
	extends Control

	var _state: State

	func _init(number: int, state: State) -> void:
		_state = state
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		custom_minimum_size = Vector2.ONE * Look.STEP_DIAMOND
		var label := Ui.centered_label(str(number), ThemeBuilder.BADGE_NUMBER_LABEL)
		label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		label.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
		var colors := {State.DONE: Look.GOLD, State.CURRENT: Look.INK, State.NEXT: Look.MUTED}
		label.add_theme_color_override(&"font_color", colors[state])
		add_child(label)

	func _draw() -> void:
		var path := DiamondStrip.outline(size / 2.0, minf(size.x, size.y) / 2.0 - Look.DIAMOND_OUTLINE)
		# Le contour répète son premier point pour se fermer : le losange plein n'en prend que 4.
		draw_colored_polygon(path.slice(0, 4), Look.GOLD if _state == State.CURRENT else Look.FOREST_DEEP)
		draw_polyline(path, Look.MUTED if _state == State.NEXT else Look.GOLD, Look.DIAMOND_OUTLINE, true)


## Le trait entre deux phases : d'or quand la phase d'avant est faite.
class StepLine:
	extends Control

	var _done := false

	func _init(done: bool) -> void:
		_done = done
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		size_flags_horizontal = Control.SIZE_EXPAND_FILL
		custom_minimum_size.y = Look.STEP_DIAMOND

	func _draw() -> void:
		var middle := size.y / 2.0
		draw_line(Vector2(0.0, middle), Vector2(size.x, middle), Look.GOLD if _done else Look.FOREST_RAISED, Look.STEP_LINE, true)
