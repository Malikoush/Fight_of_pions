// Une marque posée à plat sur le plateau : un voile avec son bord (case survolée, case
// choisie), un bord en pointillés (dernier coup adverse), un petit losange au centre et un
// anneau (case proposée), un anneau seul (statut d'un pion), ou une ombre douce et ronde
// (sous un pion). Les réglages viennent de `Look` (par `BoardMaterials`).
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
uniform float ring_core;  // la part du milieu de l'anneau, peinte ; le reste est son bord
uniform vec4 ring_edge_color : source_color;
uniform vec4 ring_dash_color : source_color;
uniform float ring_dashes; // 0 : anneau plein ; sinon, ses traits sur le tour, au milieu
uniform vec4 ink : source_color;

void fragment() {
	vec2 from_center = UV - vec2(0.5);
	float diamond = abs(from_center.x) + abs(from_center.y);
	if (round_shadow) {
		float r = length(from_center) * 2.0;
		ALBEDO = fill_color.rgb;
		ALPHA = fill_color.a * (1.0 - smoothstep(softness, 1.0, r));
	} else if (ring > 0.0 && abs(length(from_center) - ring) < ring_width) {
		// L'anneau, visible autour d'un pion posé sur la case (effets/anneau-*.svg) : son
		// milieu, éventuellement coupé de traits, et son bord.
		float across = abs(length(from_center) - ring) / ring_width;
		vec4 color = across > ring_core ? ring_edge_color : border_color;
		if (across <= ring_core && ring_dashes > 0.0) {
			float turn = atan(from_center.y, from_center.x) / TAU;
			color = fract(turn * ring_dashes) < 0.5 ? ring_dash_color : color;
		}
		ALBEDO = color.rgb;
		ALPHA = color.a;
	} else if (dot > 0.0 && diamond < dot) {
		// Le losange et son trait, comme sur marques/permise.svg.
		ALBEDO = diamond > dot * 0.78 ? ink.rgb : border_color.rgb;
		ALPHA = 1.0;
	} else {
		vec2 d = vec2(0.5) - abs(from_center);
		// Sans bord (largeur nulle), rien : au ras du carré, l'anticrénelage lit des points
		// juste hors de la marque, qui passeraient sinon pour son bord.
		bool on_border = border > 0.0 && min(d.x, d.y) < border;
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
