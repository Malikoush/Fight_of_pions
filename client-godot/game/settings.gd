class_name Settings
extends RefCounted
## Les réglages du joueur, gardés sur l'appareil (`user://settings.cfg`) : son pseudo, le
## volume de la musique et celui des effets (à zéro, coupés), les vibrations, et son trio
## de cache (les cartes qu'il compte cacher, mises en avant au draft). Lus une fois, écrits
## à chaque changement.

const PATH := "user://settings.cfg"
const SECTION := "reglages"
## Le nombre de cartes du trio de cache.
const TRIO_SIZE := 3

static var player_name := Labels.DEFAULT_PLAYER_NAME
static var music := Look.DEFAULT_MUSIC
static var volume := Look.DEFAULT_VOLUME
static var vibration := true
## Les cartes du trio de cache (`{"kind", "rarity"}`), au plus `TRIO_SIZE`.
static var cache_trio: Array = []
static var _loaded := false


## Lit les réglages gardés (une fois).
static func load_saved() -> void:
	if _loaded:
		return
	_loaded = true
	var file := ConfigFile.new()
	if file.load(PATH) != OK:
		return
	player_name = file.get_value(SECTION, "name", player_name)
	music = file.get_value(SECTION, "music", music)
	volume = file.get_value(SECTION, "volume", volume)
	vibration = file.get_value(SECTION, "vibration", vibration)
	# Un ancien fichier coupait le son par `sound` : coupé, c'est le volume à zéro.
	if file.get_value(SECTION, "sound", true) == false:
		volume = 0.0
	cache_trio = _valid_trio(file.get_value(SECTION, "cache_trio", []))


static func save() -> void:
	var file := ConfigFile.new()
	file.set_value(SECTION, "name", player_name)
	file.set_value(SECTION, "music", music)
	file.set_value(SECTION, "volume", volume)
	file.set_value(SECTION, "vibration", vibration)
	file.set_value(SECTION, "cache_trio", cache_trio)
	file.save(PATH)


## Les cartes lisibles d'un trio gardé (`{"kind", "rarity"}`, des noms), sans doublon, au plus
## `TRIO_SIZE`.
static func _valid_trio(saved: Variant) -> Array:
	var trio: Array = []
	if not (saved is Array):
		return trio
	for card: Variant in saved:
		if card is Dictionary and card.get("kind") is String and card.get("rarity") is String:
			var attack := {"kind": card["kind"], "rarity": card["rarity"]}
			if not trio.has(attack) and trio.size() < TRIO_SIZE:
				trio.append(attack)
	return trio


## Vrai si les effets sonores sont permis (leur volume n'est pas à zéro).
static func effects_on() -> bool:
	return volume > Look.MIN_VOLUME


## Vrai si cette carte (`{"kind", "rarity"}`) est dans le trio de cache.
static func in_trio(attack: Dictionary) -> bool:
	return cache_trio.has(attack)


## Met cette carte dans le trio, ou l'en retire ; rien si le trio est déjà complet. Gardé.
static func toggle_trio(attack: Dictionary) -> void:
	if in_trio(attack):
		cache_trio.erase(attack)
	elif cache_trio.size() < TRIO_SIZE:
		cache_trio.append(attack)
	else:
		return
	save()


## Une brève vibration du téléphone, si le joueur les veut.
static func vibrate(milliseconds: int) -> void:
	if vibration:
		Input.vibrate_handheld(milliseconds)
