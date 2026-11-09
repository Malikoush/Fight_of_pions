class_name CardsPage
extends MenuPage
## Les cartes du jeu, telles que le catalogue du serveur les décrit : pour chaque famille,
## son nom (et « RÉACTION »), ce qu'elle fait, ses règles, puis l'effet de chaque rareté.

var _catalog: CardCatalog


func _init(catalog: CardCatalog) -> void:
	super(Labels.CARDS_TITLE)
	_catalog = catalog


func _fill() -> void:
	if _catalog == null:
		body.add_child(_paragraph(Labels.CARDS_LOADING))
		return
	for family in _catalog.families():
		var panel := PanelContainer.new()
		panel.theme_type_variation = ThemeBuilder.PANEL
		var column := Ui.vbox(Look.GAP_S)
		var title := Ui.hbox(Look.GAP_S)
		title.add_child(Ui.label(family.get("name", ""), ThemeBuilder.NAME_LABEL))
		if family.get("reaction", false):
			title.add_child(Ui.label(Labels.REACTION_TAG, ThemeBuilder.THREAT_LABEL))
		column.add_child(title)
		column.add_child(_paragraph(family.get("summary", ""), ThemeBuilder.STRONG_LABEL))
		for rule: Variant in family.get("rules", []):
			column.add_child(_paragraph(str(rule)))
		for card: Variant in family.get("rarities", []):
			if card is Dictionary:
				column.add_child(Ui.label(str(card.get("name", "")).to_upper(), ThemeBuilder.GOLD_CAPS_LABEL))
				column.add_child(_paragraph(card.get("effect", "")))
		panel.add_child(column)
		body.add_child(panel)
