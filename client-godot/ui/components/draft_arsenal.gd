class_name DraftArsenal
extends HBoxContainer
## L'arsenal d'un joueur pendant le draft (maquettes) : ses cartes en petit (la cache
## adverse en dos), puis une place en pointillés pour chaque carte qu'il lui reste à
## choisir, nommée par sa sorte (« PICK », « CACHE ») si on le demande.


func _ready() -> void:
	theme_type_variation = ThemeBuilder.ROW_XS
	mouse_filter = Control.MOUSE_FILTER_IGNORE


## Les cartes de `player` dans cette vue, de cette largeur, puis une place par choix à venir
## (`remaining` : leurs sortes, dans l'ordre), nommée si `named`.
func show_arsenal(view: ViewData, player: String, catalog: CardCatalog, remaining: Array[String], width: float, named: bool) -> void:
	Ui.clear(self)
	for index in view.arsenal(player).size():
		var card := CardView.new(width)
		card.focus_mode = Control.FOCUS_NONE
		card.mouse_filter = Control.MOUSE_FILTER_IGNORE
		card.show_slot(view, player, index, catalog, true)
		add_child(card)
	for kind in remaining:
		var place := PanelContainer.new()
		place.theme_type_variation = ThemeBuilder.WAITING_PANEL
		place.custom_minimum_size = CardView.footprint(width)
		place.mouse_filter = Control.MOUSE_FILTER_IGNORE
		if named:
			var label := Ui.centered_label(Labels.DRAFT_SLOTS.get(kind, ""), ThemeBuilder.SLOT_LABEL)
			label.size_flags_vertical = Control.SIZE_SHRINK_END
			place.add_child(label)
		add_child(place)
