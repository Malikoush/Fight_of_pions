class_name ChoicePanel
extends PanelContainer
## Le panneau du choix d'une carte, à la place de la barre de jeu (maquettes) : la carte,
## l'étape (lanceur, puis cibles), ce qu'il faut choisir, l'effet de la carte, et les
## boutons « ANNULER », « VISER LES JETONS » et « JOUER LA CARTE ». Les boutons ne
## s'allument que si le serveur propose le coup correspondant.

signal cancel_requested
signal finish_requested
signal tokens_requested

var _card: CardView
var _step: Label
var _title: Label
var _effect: Label
var _count: Label
var _tokens: Button
var _finish: Button


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL_ACTIVE
	var column := Ui.vbox(Look.GAP_M)
	var top := Ui.hbox(Look.GAP_L)
	_card = CardView.new()
	_card.focus_mode = Control.FOCUS_NONE
	_card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	top.add_child(_card)
	var text := Ui.vbox(Look.GAP_XS)
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_step = Ui.label("", ThemeBuilder.CAPS_LABEL)
	text.add_child(_step)
	_title = Ui.label("", ThemeBuilder.NAME_LABEL)
	_title.autowrap_mode = TextServer.AUTOWRAP_WORD
	text.add_child(_title)
	_effect = Ui.label("", ThemeBuilder.MUTED_LABEL)
	_effect.autowrap_mode = TextServer.AUTOWRAP_WORD
	text.add_child(_effect)
	_count = Ui.label("", ThemeBuilder.STRONG_LABEL)
	text.add_child(_count)
	top.add_child(text)
	column.add_child(top)

	var buttons := Ui.hbox(Look.GAP_M)
	var cancel := Ui.button(Labels.CANCEL, ThemeBuilder.SECONDARY_BUTTON)
	cancel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cancel.pressed.connect(cancel_requested.emit)
	buttons.add_child(cancel)
	_tokens = Ui.button(Labels.AIM_TOKENS, ThemeBuilder.SECONDARY_BUTTON)
	_tokens.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_tokens.pressed.connect(tokens_requested.emit)
	buttons.add_child(_tokens)
	_finish = Ui.button(Labels.PLAY_CARD, ThemeBuilder.PRIMARY_BUTTON)
	_finish.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_finish.pressed.connect(finish_requested.emit)
	buttons.add_child(_finish)
	column.add_child(buttons)
	add_child(column)


## Montre le choix en cours : la carte (`attack`) et l'aperçu qui donne les choix suivants
## (`null` en attendant la réponse).
func show_choice(session: ChoiceSession, attack: Dictionary, catalog: CardCatalog, cost: int) -> void:
	var kind: String = attack.get("kind", "")
	var rarity: String = attack.get("rarity", "")
	var reaction := catalog != null and catalog.is_reaction(kind)
	var card_name := catalog.family_name(kind) if catalog != null else kind
	_card.show_card(attack, card_name, cost, reaction, 0, false, true)
	_effect.text = catalog.effect(kind, rarity) if catalog != null else ""
	_effect.visible = _effect.text != ""

	var choices := session.choices
	var deciding := "" if choices == null else choices.decision_kind()
	_step.text = Labels.STEP_LAUNCHER if deciding == "ChooseLauncher" else Labels.STEP_TARGETS
	match deciding:
		"ChooseLauncher":
			_title.text = Labels.CHOOSE_LAUNCHER
		"ChooseTarget":
			_title.text = Labels.CHOOSE_TARGET
		_:
			_title.text = Labels.READY_TO_PLAY
	_count.text = Labels.TARGETS_CHOSEN % session.targets
	_count.visible = deciding == "ChooseTarget"
	_finish.disabled = choices == null or choices.simple_action("FinishAttack") == null
	_tokens.visible = choices != null and choices.simple_action("ChooseEnemyTokens") != null
