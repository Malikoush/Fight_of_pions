// Une marque posée à plat sur le plateau : un voile, avec un bord (case survolée), ou une
// ombre douce et ronde (sous un pion).
shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_never, shadows_disabled;

uniform vec4 fill_color : source_color;
uniform vec4 border_color : source_color;
uniform float border;
uniform bool round_shadow;
uniform float softness;   // où commence le fondu de l'ombre, en part du rayon

void fragment() {
	vec2 from_center = UV - vec2(0.5);
	if (round_shadow) {
		float r = length(from_center) * 2.0;
		ALBEDO = fill_color.rgb;
		ALPHA = fill_color.a * (1.0 - smoothstep(softness, 1.0, r));
	} else {
		vec2 d = vec2(0.5) - abs(from_center);
		vec4 color = min(d.x, d.y) < border ? border_color : fill_color;
		ALBEDO = color.rgb;
		ALPHA = color.a;
	}
}
