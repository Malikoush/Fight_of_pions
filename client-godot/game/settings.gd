class_name Settings
extends RefCounted
## Les réglages du joueur, gardés sur l'appareil (`user://settings.cfg`) : son pseudo, le
## son, son volume, les vibrations. Lus une fois, écrits à chaque changement.

const PATH := "user://settings.cfg"
const SECTION := "reglages"

static var player_name := Labels.DEFAULT_PLAYER_NAME
static var sound := true
static var volume := Look.DEFAULT_VOLUME
static var vibration := true
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
	sound = file.get_value(SECTION, "sound", sound)
	volume = file.get_value(SECTION, "volume", volume)
	vibration = file.get_value(SECTION, "vibration", vibration)


static func save() -> void:
	var file := ConfigFile.new()
	file.set_value(SECTION, "name", player_name)
	file.set_value(SECTION, "sound", sound)
	file.set_value(SECTION, "volume", volume)
	file.set_value(SECTION, "vibration", vibration)
	file.save(PATH)


## Une brève vibration du téléphone, si le joueur les veut.
static func vibrate(milliseconds: int) -> void:
	if vibration:
		Input.vibrate_handheld(milliseconds)
