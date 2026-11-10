class_name Settings
extends RefCounted
## Les réglages de l'appareil (`user://settings.cfg`) : le volume de la musique et celui
## des effets (à zéro, coupés), les vibrations, et le bot choisi à l'accueil. Lus une fois,
## écrits à chaque changement.
## Le pseudo et le trio de cache, eux, sont gardés par le serveur, avec le compte
## (`Session.pseudo`, `Session.collection`) ; un ancien fichier qui les gardait ici est lu
## sans eux.

const PATH := "user://settings.cfg"
const SECTION := "reglages"
## Les bots que le serveur propose (`opponent` de `create`), le premier par défaut.
const BOT_OPPONENTS: Array[String] = ["random", "mcts"]

static var music := Look.DEFAULT_MUSIC
static var volume := Look.DEFAULT_VOLUME
static var vibration := true
## Le bot à affronter depuis l'accueil, un de `BOT_OPPONENTS`.
static var bot_opponent := BOT_OPPONENTS[0]
static var _loaded := false


## Lit les réglages gardés (une fois).
static func load_saved() -> void:
	if _loaded:
		return
	_loaded = true
	var file := ConfigFile.new()
	if file.load(PATH) != OK:
		return
	music = file.get_value(SECTION, "music", music)
	volume = file.get_value(SECTION, "volume", volume)
	vibration = file.get_value(SECTION, "vibration", vibration)
	var bot: Variant = file.get_value(SECTION, "bot", bot_opponent)
	if bot in BOT_OPPONENTS:
		bot_opponent = bot
	# Un ancien fichier coupait le son par `sound` : coupé, c'est le volume à zéro.
	if file.get_value(SECTION, "sound", true) == false:
		volume = 0.0


static func save() -> void:
	var file := ConfigFile.new()
	file.set_value(SECTION, "music", music)
	file.set_value(SECTION, "volume", volume)
	file.set_value(SECTION, "vibration", vibration)
	file.set_value(SECTION, "bot", bot_opponent)
	file.save(PATH)


## Vrai si les effets sonores sont permis (leur volume n'est pas à zéro).
static func effects_on() -> bool:
	return volume > Look.MIN_VOLUME


## Une brève vibration du téléphone, si le joueur les veut.
static func vibrate(milliseconds: int) -> void:
	if vibration:
		Input.vibrate_handheld(milliseconds)
