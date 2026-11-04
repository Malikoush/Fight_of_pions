class_name BoardMaterials
extends RefCounted
## Les matériaux et les mesures des pièces du plateau, construits à partir de l'apparence
## (`Look`) et partagés : tous les pions d'un joueur utilisent le même matériau, ce qui
## garde le rendu léger.

const PRISM_SHADER := "res://board/shaders/prism.gdshader"
const PION_SHADER := "res://board/shaders/pion.gdshader"
const GHOST_SHADER := "res://board/shaders/pion_ghost.gdshader"
const OUTLINE_SHADER := "res://board/shaders/outline.gdshader"
const FLAT_SHADER := "res://board/shaders/flat_mark.gdshader"
const CROWN_SHADER := "res://board/shaders/crown.gdshader"
## L'ordre de dessin de ce qui est posé à plat au même niveau : l'ombre d'un pion, puis
## l'anneau de son statut, puis les marques des cases (ordre 0), toujours lisibles.
const SHADOW_PRIORITY := -2
const RING_PRIORITY := -1

static var _pions := {}
static var _ghosts := {}
static var _rings := {}
static var _pulses := {}
static var _solids := {}
static var _crown: ShaderMaterial
static var _shot: SphereMesh


## Vrai si les pions de ce joueur sont des palets octogonaux (J2) plutôt que ronds.
static func is_octagon(player: String) -> bool:
	return player == ViewData.PLAYER_TWO


## Le rayon des pions de ce joueur.
static func pion_radius(player: String) -> float:
	return Look.PION_RADIUS * (Look.PION_OCTAGON_SCALE if is_octagon(player) else 1.0)


## Un pavé de cette taille, avec ses trois couleurs et la couleur de ses arêtes.
static func prism(size: Vector3, top: Color, side_a: Color, side_b: Color, edge: Color) -> ShaderMaterial:
	var material := _material(PRISM_SHADER)
	material.set_shader_parameter(&"top_color", top)
	material.set_shader_parameter(&"side_a_color", side_a)
	material.set_shader_parameter(&"side_b_color", side_b)
	material.set_shader_parameter(&"edge_color", edge)
	material.set_shader_parameter(&"half_size", size / 2.0)
	material.set_shader_parameter(&"edge", Look.TILE_EDGE)
	return material


## Le matériau des pions de ce joueur, contour compris.
static func pion(player: String) -> ShaderMaterial:
	if not _pions.has(player):
		var material := _painted_pion(PION_SHADER, player)
		var radius := pion_radius(player)
		material.next_pass = outline(Vector3(radius, Look.PION_HEIGHT / 2.0, radius))
		_pions[player] = material
	return _pions[player]


## Le matériau du palet fantôme de ce joueur : le même dessin, transparent, sans contour.
static func ghost(player: String) -> ShaderMaterial:
	if not _ghosts.has(player):
		var material := _painted_pion(GHOST_SHADER, player)
		material.set_shader_parameter(&"ghost_alpha", Look.GHOST_ALPHA)
		_ghosts[player] = material
	return _ghosts[player]


## Le palet peint de ce joueur, avec ce shader (posé ou fantôme).
static func _painted_pion(shader_path: String, player: String) -> ShaderMaterial:
	var octagon := is_octagon(player)
	var radius := pion_radius(player)
	var material := _material(shader_path)
	material.set_shader_parameter(&"top_color", Look.J2_TOP if octagon else Look.J1_TOP)
	material.set_shader_parameter(&"side_color", Look.J2_SIDE if octagon else Look.J1_SIDE)
	material.set_shader_parameter(&"motif_color", Look.J2_MOTIF if octagon else Look.J1_STRIPE)
	material.set_shader_parameter(&"core_color", Look.J2_MOTIF if octagon else Look.J1_CORE)
	material.set_shader_parameter(&"ink", Look.INK)
	material.set_shader_parameter(&"radius", radius)
	material.set_shader_parameter(&"half_height", Look.PION_HEIGHT / 2.0)
	material.set_shader_parameter(&"octagon", octagon)
	material.set_shader_parameter(&"square_turn", -Look.PION_OCTAGON_TURN if octagon else 0.0)
	material.set_shader_parameter(&"rim", Look.PION_RIM)
	material.set_shader_parameter(&"side_rim", Look.PION_SIDE_RIM)
	material.set_shader_parameter(&"stripes", Look.PION_STRIPES)
	material.set_shader_parameter(&"dashes", Look.PION_DASHES)
	material.set_shader_parameter(&"light", Look.PION_LIGHT)
	material.set_shader_parameter(&"shade_dark", Look.PION_SHADE_DARK)
	material.set_shader_parameter(&"shade_light", Look.PION_SHADE_LIGHT)
	material.set_shader_parameter(&"underside", Look.PION_UNDERSIDE)
	return material


## Le contour d'un objet convexe dont `half_extent` est la demi-taille.
static func outline(half_extent: Vector3) -> ShaderMaterial:
	var material := _material(OUTLINE_SHADER)
	var thickness := Vector3.ONE * Look.PION_OUTLINE
	material.set_shader_parameter(&"ink", Look.INK)
	material.set_shader_parameter(&"grow", (half_extent + thickness) / half_extent)
	return material


## Une marque de case : un voile et son bord plein.
static func mark(fill: Color, border: Color) -> ShaderMaterial:
	return _flat(fill, border)


## Un bord en pointillés, sans voile (le dernier coup adverse).
static func dashed_mark(border: Color, dashes: float) -> ShaderMaterial:
	var material := _flat(Color(border, 0.0), border)
	material.set_shader_parameter(&"dashes", dashes)
	return material


## Un voile léger, un petit losange au centre et un anneau, sans bord (une case proposée) :
## le losange marque une case vide, l'anneau un pion, qui cache le losange.
static func dotted_mark(fill: Color, dot: Color, dot_size: float) -> ShaderMaterial:
	var material := _flat(fill, dot)
	material.set_shader_parameter(&"border", 0.0)
	material.set_shader_parameter(&"dot", dot_size)
	material.set_shader_parameter(&"ring", Look.CANDIDATE_RING)
	material.set_shader_parameter(&"ring_width", Look.CANDIDATE_RING_WIDTH)
	return material


## L'anneau d'un statut sous un pion (`PionStatus.RING_ORDER`), partagé par tous les pions
## qui l'ont : son milieu, son bord, et ses traits (`Look.STATUS_RINGS`).
static func status_ring(kind: String) -> ShaderMaterial:
	if not _rings.has(kind):
		var look: Dictionary = Look.STATUS_RINGS[kind]
		var material := _flat(Color.TRANSPARENT, look["band"])
		material.set_shader_parameter(&"border", 0.0)
		material.set_shader_parameter(&"ring", Look.STATUS_RING)
		material.set_shader_parameter(&"ring_width", Look.STATUS_RING_WIDTH)
		material.set_shader_parameter(&"ring_core", Look.STATUS_RING_CORE)
		material.set_shader_parameter(&"ring_edge_color", look["edge"])
		material.set_shader_parameter(&"ring_dash_color", look["dash"])
		material.set_shader_parameter(&"ring_dashes", look["dashes"])
		material.render_priority = RING_PRIORITY
		_rings[kind] = material
	return _rings[kind]


## L'éclat d'un événement sur une case : un anneau seul, de cette couleur, sans voile.
static func pulse(color: Color) -> ShaderMaterial:
	if not _pulses.has(color):
		var material := _flat(Color.TRANSPARENT, color)
		material.set_shader_parameter(&"border", 0.0)
		material.set_shader_parameter(&"ring", Look.PULSE_RING)
		material.set_shader_parameter(&"ring_width", Look.PULSE_RING_WIDTH)
		_pulses[color] = material
	return _pulses[color]


## Une matière d'une seule couleur, sans éclairage (la pierre d'une couronne, la balle d'un
## tir).
static func solid(color: Color) -> StandardMaterial3D:
	if not _solids.has(color):
		var material := StandardMaterial3D.new()
		material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		material.albedo_color = color
		_solids[color] = material
	return _solids[color]


## La balle d'or que tire un garde de la reine.
static func shot_mesh() -> SphereMesh:
	if _shot == null:
		_shot = SphereMesh.new()
		_shot.radius = Look.SHOT_RADIUS
		_shot.height = Look.SHOT_RADIUS * 2.0
		_shot.material = solid(Look.GOLD)
	return _shot


## L'or des couronnes.
static func crown() -> ShaderMaterial:
	if _crown == null:
		_crown = _material(CROWN_SHADER)
		_crown.set_shader_parameter(&"gold", Look.GOLD)
		_crown.set_shader_parameter(&"inside", Look.GOLD_DEEP)
		_crown.set_shader_parameter(&"ink", Look.INK)
		_crown.set_shader_parameter(&"ink_width", Look.CROWN_INK)
		_crown.set_shader_parameter(&"light", Look.PION_LIGHT)
		_crown.set_shader_parameter(&"shade_dark", Look.PION_SHADE_DARK)
		_crown.set_shader_parameter(&"shade_light", Look.PION_SHADE_LIGHT)
	return _crown


## L'ombre douce et ronde d'une pièce.
static func shadow(color: Color) -> ShaderMaterial:
	var material := _flat(color, color)
	material.set_shader_parameter(&"round_shadow", true)
	material.set_shader_parameter(&"softness", Look.PION_SHADOW_SOFTNESS)
	material.render_priority = SHADOW_PRIORITY
	return material


static func _flat(fill: Color, border: Color) -> ShaderMaterial:
	var material := _material(FLAT_SHADER)
	material.set_shader_parameter(&"fill_color", fill)
	material.set_shader_parameter(&"border_color", border)
	material.set_shader_parameter(&"border", Look.MARK_BORDER_WIDTH)
	material.set_shader_parameter(&"round_shadow", false)
	material.set_shader_parameter(&"softness", 0.0)
	material.set_shader_parameter(&"dashes", 0.0)
	material.set_shader_parameter(&"dot", 0.0)
	material.set_shader_parameter(&"ring", 0.0)
	material.set_shader_parameter(&"ring_width", 0.0)
	material.set_shader_parameter(&"ring_core", 1.0)
	material.set_shader_parameter(&"ring_edge_color", Color.TRANSPARENT)
	material.set_shader_parameter(&"ring_dash_color", Color.TRANSPARENT)
	material.set_shader_parameter(&"ring_dashes", 0.0)
	material.set_shader_parameter(&"ink", Look.INK)
	return material


static func _material(shader_path: String) -> ShaderMaterial:
	var material := ShaderMaterial.new()
	material.shader = load(shader_path)
	return material
