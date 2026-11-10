class_name Sounds
extends Node
## Les sons du jeu, fabriqués dans le code (aucun fichier) : une note courte qui s'éteint,
## de hauteur et de durée propres à chaque moment (`Look.SOUNDS`). Un seul lecteur par
## son ; le réglage « Son » coupe tout, le volume se règle.

## La fréquence d'échantillonnage des sons fabriqués.
const MIX_RATE := 22050

static var _instance: Sounds

var _players := {}


func _ready() -> void:
	_instance = self
	for kind: String in Look.SOUNDS:
		var player := AudioStreamPlayer.new()
		player.stream = _tone(Look.SOUNDS[kind])
		add_child(player)
		_players[kind] = player


## Joue ce son (`place`, `attack`, `kill`, `turn`), si le son est permis.
static func play(kind: String) -> void:
	if _instance == null or not Settings.effects_on() or not _instance._players.has(kind):
		return
	var player: AudioStreamPlayer = _instance._players[kind]
	player.volume_db = linear_to_db(maxf(Settings.volume, Look.MIN_VOLUME))
	player.play()


## Une note : `{"pitch", "length", "glide"}` (Hz, secondes, glissement final en part de la
## hauteur), qui s'éteint en douceur.
static func _tone(note: Dictionary) -> AudioStreamWAV:
	var pitch: float = note["pitch"]
	var length: float = note["length"]
	var glide: float = note["glide"]
	var count := int(MIX_RATE * length)
	var data := PackedByteArray()
	data.resize(count * 2)
	var phase := 0.0
	for index in count:
		var progress := float(index) / count
		phase += TAU * pitch * (1.0 + glide * progress) / MIX_RATE
		var envelope := (1.0 - progress) * (1.0 - progress)
		var sample := int(sin(phase) * envelope * Look.SOUND_LEVEL * 32767.0)
		data.encode_s16(index * 2, sample)
	var stream := AudioStreamWAV.new()
	stream.format = AudioStreamWAV.FORMAT_16_BITS
	stream.mix_rate = MIX_RATE
	stream.data = data
	return stream
