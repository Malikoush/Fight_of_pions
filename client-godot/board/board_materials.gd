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

static var _pions := {}
static var _ghosts := {}


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


## L'ombre douce et ronde d'une pièce.
static func shadow(color: Color) -> ShaderMaterial:
	var material := _flat(color, color)
	material.set_shader_parameter(&"round_shadow", true)
	material.set_shader_parameter(&"softness", Look.PION_SHADOW_SOFTNESS)
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
	material.set_shader_parameter(&"ink", Look.INK)
	return material


static func _material(shader_path: String) -> ShaderMaterial:
	var material := ShaderMaterial.new()
	material.shader = load(shader_path)
	return material
