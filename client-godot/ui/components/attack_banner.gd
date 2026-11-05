class_name AttackBanner
extends PanelContainer
## L'annonce d'une carte que l'adversaire vient de jouer, en haut de l'écran (maquettes) : la
## carte, « ATTAQUE » ou « RÉACTION », qui la lance, et depuis quelle case. Elle reste un
## instant, puis s'efface d'elle-même ; elle ne prend aucun toucher.

var _card: CardView
var _tag: Label
var _title: Label
var _detail: Label
var _show: Tween


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL_THREAT
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false
	var row := Ui.hbox(Look.GAP_L)
	_card = CardView.new(Look.ANNOUNCE_CARD_WIDTH)
	_card.focus_mode = Control.FOCUS_NONE
	_card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(_card)
	var text := Ui.vbox(Look.GAP_XS)
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text.alignment = BoxContainer.ALIGNMENT_CENTER
	_tag = Ui.label("", ThemeBuilder.THREAT_LABEL)
	text.add_child(_tag)
	_title = Ui.label("", ThemeBuilder.HEADING_LABEL)
	_title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	text.add_child(_title)
	_detail = Ui.label("", ThemeBuilder.MUTED_LABEL)
	text.add_child(_detail)
	row.add_child(text)
	add_child(row)


## Annonce cette carte (`{"kind", "rarity"}`), lancée par ce joueur depuis cette case.
func announce(attack: Dictionary, who: String, launcher: String, catalog: CardCatalog, cost: int) -> void:
	_card.show_attack(attack, catalog, cost)
	var reaction := catalog != null and catalog.is_reaction(attack.get("kind", ""))
	_tag.text = Labels.REACTION_TAG if reaction else Labels.ATTACK_TAG
	_title.text = Labels.ANNOUNCE_TITLE % [who, CardCatalog.card_name(catalog, attack)]
	_detail.text = Labels.ANNOUNCE_FROM % launcher
	if _show != null:
		_show.kill()
	visible = true
	modulate.a = 0.0
	_show = create_tween()
	_show.tween_property(self, "modulate:a", 1.0, Look.ANNOUNCE_FADE)
	_show.tween_interval(Look.ANNOUNCE_HOLD)
	_show.tween_property(self, "modulate:a", 0.0, Look.ANNOUNCE_FADE)
	_show.tween_callback(hide)
