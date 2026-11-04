class_name BoardMaterials
extends RefCounted
## Les matériaux et les mesures des pièces du plateau, construits à partir de l'apparence
## (`Look`) et partagés : tous les pions d'un joueur utilisent le même matériau, ce qui
## garde le rendu léger.

const PRISM_SHADER := "res://board/shaders/prism.gdshader"
const PION_SHADER := "res://board/shaders/pion.gdshader"
const OUTLINE_SHADER := "res://board/shaders/outline.gdshader"
const FLAT_SHADER := "res://board/shaders/flat_mark.gdshader"

static var _pions := {}


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
	if _pions.has(player):
		return _pions[player]
	var octagon := is_octagon(player)
	var radius := pion_radius(player)
	var material := _material(PION_SHADER)
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
	material.next_pass = outline(Vector3(radius, Look.PION_HEIGHT / 2.0, radius))
	_pions[player] = material
	return material


## Le contour d'un objet convexe dont `half_extent` est la demi-taille.
static func outline(half_extent: Vector3) -> ShaderMaterial:
	var material := _material(OUTLINE_SHADER)
	var thickness := Vector3.ONE * Look.PION_OUTLINE
	material.set_shader_parameter(&"ink", Look.INK)
	material.set_shader_parameter(&"grow", (half_extent + thickness) / half_extent)
	return material


## Une marque à plat : un voile et son bord, ou une ombre ronde.
static func flat(fill: Color, border: Color, round_shadow: bool) -> ShaderMaterial:
	var material := _material(FLAT_SHADER)
	material.set_shader_parameter(&"fill_color", fill)
	material.set_shader_parameter(&"border_color", border)
	material.set_shader_parameter(&"border", Look.MARK_BORDER_WIDTH)
	material.set_shader_parameter(&"round_shadow", round_shadow)
	material.set_shader_parameter(&"softness", Look.PION_SHADOW_SOFTNESS)
	return material


static func _material(shader_path: String) -> ShaderMaterial:
	var material := ShaderMaterial.new()
	material.shader = load(shader_path)
	return material
