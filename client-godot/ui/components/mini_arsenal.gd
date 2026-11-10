class_name MiniArsenal
extends HBoxContainer
## L'arsenal d'un joueur en petit (maquettes) : ses cartes (la cache adverse en dos) ; au
## draft, puis une place en pointillés pour chaque carte qu'il lui reste à choisir, nommée
## par sa sorte (« PICK », « CACHE ») si on le demande. Dans la partie, le panneau de
## l'adversaire montre ainsi ses cartes.


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
		add_child(CardView.placeholder(width, Labels.DRAFT_SLOTS.get(kind, "") if named else ""))
