// La couronne d'une reine (effets/couronne.svg) : une bande d'or dentelée, plus sombre à
// l'intérieur, bordée d'un trait en haut et en bas. Sans éclairage ; la même fausse
// lumière que les palets nuance l'extérieur. Les réglages viennent de `Look` (par
// `BoardMaterials.crown`).
shader_type spatial;
render_mode unshaded, cull_disabled;

uniform vec4 gold : source_color;
uniform vec4 inside : source_color;
uniform vec4 ink : source_color;
uniform float ink_width;
uniform vec3 light;
uniform float shade_dark;
uniform float shade_light;

varying vec3 world_normal;

void vertex() {
	world_normal = (MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz;
}

void fragment() {
	// UV.y va de 0 (bas) à 1 (haut de la bande) ; UV2.x est la hauteur de la bande ici.
	float from_bottom = UV.y * UV2.x;
	float to_top = (1.0 - UV.y) * UV2.x;
	if (from_bottom < ink_width || to_top < ink_width) {
		ALBEDO = ink.rgb;
	} else if (!FRONT_FACING) {
		ALBEDO = inside.rgb;
	} else {
		float lit = clamp(dot(normalize(world_normal), normalize(light)) * 0.5 + 0.5, 0.0, 1.0);
		ALBEDO = gold.rgb * mix(shade_dark, shade_light, lit);
	}
}
