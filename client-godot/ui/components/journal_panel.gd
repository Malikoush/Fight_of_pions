class_name JournalPanel
extends PanelContainer
## Le journal des coups (`GameJournal`), les plus récents en haut : pour chaque ligne, son
## tour, le palet du joueur concerné, et ce qui s'est passé. Une ligne d'action nomme son
## joueur (« Toi » en or, l'adversaire en crème) ; une conséquence est plus discrète.

var _rows: VBoxContainer
var _empty: Label
var _scroll: ScrollContainer
## Le nom affiché de chaque joueur, et celui qui regarde.
var _names := {}
var _viewer := ""


func _init() -> void:
	theme_type_variation = ThemeBuilder.PANEL
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var column := Ui.vbox(Look.GAP_S)
	add_child(column)

	var header := Ui.hbox(Look.GAP_S)
	header.add_child(Ui.label(Labels.JOURNAL_TITLE, ThemeBuilder.TAG_LABEL))
	header.add_child(Ui.spacer())
	header.add_child(Ui.label(Labels.JOURNAL_ORDER, ThemeBuilder.JOURNAL_TURN_LABEL))
	column.add_child(header)

	_empty = Ui.label(Labels.JOURNAL_EMPTY, ThemeBuilder.JOURNAL_TURN_LABEL)
	_empty.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_empty)

	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	column.add_child(_scroll)
	_rows = Ui.vbox(Look.GAP_S)
	_rows.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_rows)


## Les noms à afficher (`{"One": "Toi", "Two": "Bot"}`) et le joueur qui regarde.
func set_names(viewer: String, names: Dictionary) -> void:
	_viewer = viewer
	_names = names


## Ajoute ces lignes (de la plus ancienne à la plus récente) en haut du journal.
func add_entries(entries: Array[Dictionary]) -> void:
	for entry in entries:
		var row := _row(entry)
		_rows.add_child(row)
		_rows.move_child(row, 0)
	while _rows.get_child_count() > Look.JOURNAL_SHOWN:
		var oldest := _rows.get_child(_rows.get_child_count() - 1)
		_rows.remove_child(oldest)
		oldest.queue_free()
	# Le défilement ne bouge pas : en haut, on voit arriver les nouvelles lignes ; plus bas,
	# on continue de relire les anciennes.
	_empty.visible = _rows.get_child_count() == 0


func _row(entry: Dictionary) -> HBoxContainer:
	var row := Ui.hbox(Look.GAP_S)
	var turn := Ui.label(Labels.JOURNAL_TURN % entry.get("turn", 0), ThemeBuilder.JOURNAL_TURN_LABEL)
	turn.custom_minimum_size.x = Look.JOURNAL_TURN_WIDTH
	row.add_child(turn)
	var player: String = entry.get("player", "")
	row.add_child(Ui.chip(player, Look.JOURNAL_CHIP) if player != "" else Ui.gap(Look.JOURNAL_CHIP))
	var line := RichTextLabel.new()
	line.theme_type_variation = ThemeBuilder.JOURNAL_LINE
	line.bbcode_enabled = true
	line.fit_content = true
	line.scroll_active = false
	line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	line.mouse_filter = Control.MOUSE_FILTER_IGNORE
	line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	line.text = _line_text(entry, player)
	row.add_child(line)
	return row


## Le texte d'une ligne : le nom coloré de son joueur puis l'action, ou la conséquence en
## plus discret. Un nom de carte (catalogue) ou de joueur (les pseudos viendront) ne doit
## pas passer pour une balise : ses crochets sont échappés.
func _line_text(entry: Dictionary, player: String) -> String:
	var text := _plain(entry.get("text", ""))
	if not entry.get("acts", false):
		return "[color=#%s]%s[/color]" % [Look.MUTED.to_html(false), text]
	var color := Look.GOLD if player == _viewer else Look.CREAM
	var who := _plain(_names.get(player, ""))
	return "[color=#%s][b]%s[/b][/color] %s" % [color.to_html(false), who, text]


## Ce texte, lu tel quel par le texte riche : ses crochets ne sont pas des balises.
static func _plain(text: Variant) -> String:
	return str(text).replace("[", "[lb]")
