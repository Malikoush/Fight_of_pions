class_name RulesPage
extends MenuPage
## L'essentiel des règles, en quelques sections courtes (`Labels.RULES`, tirées du carnet de
## règles). Le détail de chaque carte est dans la page Cartes.


func _init() -> void:
	super(Labels.RULES_TITLE)


func _fill() -> void:
	for section: Array in Labels.RULES:
		var column := Ui.vbox(Look.GAP_XS)
		column.add_child(Ui.label(section[0], ThemeBuilder.GOLD_CAPS_LABEL))
		column.add_child(_paragraph(section[1], ThemeBuilder.STRONG_LABEL))
		body.add_child(column)
