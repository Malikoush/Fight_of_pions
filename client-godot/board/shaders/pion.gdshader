// Un pion peint : le palet rond rouge de J1 (bandes blanches sur la tranche, anneaux et
// cœur dorés) ou le palet octogonal blanc de J2 (anneau, losange et fenêtres bleus). Le
// motif est dessiné ici plutôt que plaqué en image : net à tout zoom, et animable.
// Sans éclairage ; une fausse lumière fixe nuance seulement la tranche.
// Les réglages viennent de `Look` (par `BoardMaterials.pion`). Les proportions du motif
// (rayons des anneaux, largeur des fenêtres) reprennent le dessin des SVG fournis
// (pions/pion-j1.svg, pion-j2.svg) : elles font partie du dessin.
shader_type spatial;
render_mode unshaded;

uniform vec4 top_color : source_color;
uniform vec4 side_color : source_color;
uniform vec4 motif_color : source_color;  // pointillés (J1) ou motif bleu (J2)
uniform vec4 core_color : source_color;   // anneau et cœur (J1) ; égal au motif pour J2
uniform vec4 ink : source_color;
uniform float radius;
uniform float half_height;
uniform bool octagon;
uniform float square_turn;    // remet le losange central dans l'axe de la grille
uniform float rim;            // le trait du bord du dessus, en part du rayon
uniform float side_rim;       // le trait haut et bas de la tranche
uniform float stripes;        // les bandes de la tranche (J1)
uniform float dashes;         // les pointillés du dessus (J1)
uniform vec3 light;
uniform float shade_dark;
uniform float shade_light;
uniform float underside;

const float EIGHTH = TAU / 8.0;

varying vec3 local_position;
varying vec3 local_normal;

void vertex() {
	local_position = VERTEX;
	local_normal = NORMAL;
}

// L'angle autour de l'axe du pion, compté depuis +z, comme les sommets du cylindre.
float angle_of(vec2 xz) {
	return atan(xz.x, xz.y);
}

// Pour l'octogone : l'écart à la normale de la face, entre -π/8 et π/8.
float in_face(float angle) {
	return mod(angle, EIGHTH) - EIGHTH * 0.5;
}

vec3 top(vec2 p) {
	float angle = angle_of(p);
	if (!octagon) {
		float r = length(p) / radius;
		if (r > 1.0 - rim) return ink.rgb;
		if (r > 0.80 && r < 0.89 && fract(angle / TAU * dashes) < 0.45) return motif_color.rgb;
		if (r > 0.53 && r < 0.59) return core_color.rgb;
		if (r < 0.215) return core_color.rgb;
		if (r < 0.245) return ink.rgb;
		return top_color.rgb;
	}
	// Distance à l'octogone, 1 sur son bord.
	float d = length(p) * cos(in_face(angle)) / (radius * cos(EIGHTH * 0.5));
	if (d > 1.0 - rim) return ink.rgb;
	if (d > 0.55 && d < 0.63) return motif_color.rgb;
	// Un carré aligné sur la grille : un losange à l'écran.
	vec2 q = mat2(vec2(cos(square_turn), sin(square_turn)), vec2(-sin(square_turn), cos(square_turn))) * p;
	float square = max(abs(q.x), abs(q.y)) / radius;
	if (square < 0.21) return motif_color.rgb;
	if (square < 0.245) return ink.rgb;
	return top_color.rgb;
}

vec3 side(vec3 p, vec3 world_normal) {
	if (abs(p.y) > half_height - side_rim) return ink.rgb;
	float angle = angle_of(p.xz);
	vec3 color = side_color.rgb;
	if (!octagon) {
		if (fract(angle / TAU * stripes) < 0.38) color = motif_color.rgb;
	} else if (abs(in_face(angle)) < 0.13 && abs(p.y) < half_height * 0.5) {
		color = motif_color.rgb;
	}
	float lit = clamp(dot(normalize(world_normal), normalize(light)) * 0.5 + 0.5, 0.0, 1.0);
	return color * mix(shade_dark, shade_light, lit);
}

void fragment() {
	if (local_normal.y > 0.5) {
		ALBEDO = top(local_position.xz);
	} else if (local_normal.y < -0.5) {
		ALBEDO = side_color.rgb * underside;
	} else {
		vec3 radial = vec3(local_position.x, 0.0, local_position.z);
		if (octagon) {
			// Une facette plate par face, pas la normale lissée du cylindre.
			float face = (floor(angle_of(local_position.xz) / EIGHTH) + 0.5) * EIGHTH;
			radial = vec3(sin(face), 0.0, cos(face));
		}
		vec3 world_normal = (MODEL_MATRIX * vec4(radial, 0.0)).xyz;
		ALBEDO = side(local_position, world_normal);
	}
}
