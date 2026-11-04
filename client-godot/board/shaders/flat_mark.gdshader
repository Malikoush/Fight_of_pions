// Une marque posée à plat sur le plateau : un voile avec son bord (case survolée, case
// choisie), un bord en pointillés (dernier coup adverse), un petit losange au centre (case
// proposée), ou une ombre douce et ronde (sous un pion). Les réglages viennent de `Look`
// (par `BoardMaterials`).
shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_never, shadows_disabled;

uniform vec4 fill_color : source_color;
uniform vec4 border_color : source_color;
uniform float border;
uniform bool round_shadow;
uniform float softness;   // où commence le fondu de l'ombre, en part du rayon
uniform float dashes;     // 0 : bord plein ; sinon, le nombre de traits sur le tour
uniform float dot;        // 0 : pas de losange ; sinon, sa demi-diagonale (part de la case)
uniform float ring;       // 0 : pas d'anneau ; sinon, son rayon (part de la case), autour d'un pion
uniform float ring_width;
uniform vec4 ink : source_color;

void fragment() {
	vec2 from_center = UV - vec2(0.5);
	float diamond = abs(from_center.x) + abs(from_center.y);
	if (round_shadow) {
		float r = length(from_center) * 2.0;
		ALBEDO = fill_color.rgb;
		ALPHA = fill_color.a * (1.0 - smoothstep(softness, 1.0, r));
	} else if (ring > 0.0 && abs(length(from_center) - ring) < ring_width) {
		// L'anneau d'or, visible autour d'un pion posé sur la case (effets/anneau-lanceur.svg).
		ALBEDO = border_color.rgb;
		ALPHA = 1.0;
	} else if (dot > 0.0 && diamond < dot) {
		// Le losange et son trait, comme sur marques/permise.svg.
		ALBEDO = diamond > dot * 0.78 ? ink.rgb : border_color.rgb;
		ALPHA = 1.0;
	} else {
		vec2 d = vec2(0.5) - abs(from_center);
		bool on_border = min(d.x, d.y) < border;
		if (on_border && dashes > 0.0) {
			// Un trait sur deux, mesurés à l'angle autour du centre.
			float turn = atan(from_center.y, from_center.x) / TAU;
			on_border = fract(turn * dashes) < 0.5;
		}
		vec4 color = on_border ? border_color : fill_color;
		ALBEDO = color.rgb;
		ALPHA = color.a;
	}
}
