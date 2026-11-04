// Une marque posée à plat sur le plateau : un voile avec son bord (case survolée), un bord
// en pointillés (dernier coup adverse), ou une ombre douce et ronde (sous un pion). Les
// réglages viennent de `Look` (par `BoardMaterials.flat`).
shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_never, shadows_disabled;

uniform vec4 fill_color : source_color;
uniform vec4 border_color : source_color;
uniform float border;
uniform bool round_shadow;
uniform float softness;   // où commence le fondu de l'ombre, en part du rayon
uniform float dashes;     // 0 : bord plein ; sinon, le nombre de traits sur le tour

void fragment() {
	vec2 from_center = UV - vec2(0.5);
	if (round_shadow) {
		float r = length(from_center) * 2.0;
		ALBEDO = fill_color.rgb;
		ALPHA = fill_color.a * (1.0 - smoothstep(softness, 1.0, r));
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
