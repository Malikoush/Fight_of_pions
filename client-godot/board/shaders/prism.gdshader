// Un pavé peint comme les éléments fournis : le dessus, la tranche gauche et la tranche
// droite ont chacun leur couleur, et un trait marque les arêtes. Sans éclairage : le rendu
// reste celui de l'illustration, net, et ne coûte presque rien. Sert aux cases, au mur et
// aux tombes. Les réglages viennent de `Look` (par `BoardMaterials.prism`).
//
// La caméra regarde depuis +x +z : la tranche +z est à gauche de l'écran, la +x à droite.
shader_type spatial;
render_mode unshaded;

uniform vec4 top_color : source_color;
uniform vec4 side_a_color : source_color; // tranche gauche (+z)
uniform vec4 side_b_color : source_color; // tranche droite (+x)
uniform vec4 edge_color : source_color;
uniform vec3 half_size;
uniform float edge;

varying vec3 local_position;
varying vec3 local_normal;

void vertex() {
	local_position = VERTEX;
	local_normal = NORMAL;
}

void fragment() {
	vec3 color;
	float to_edge;
	vec3 d = half_size - abs(local_position);
	if (local_normal.y > 0.5) {
		color = top_color.rgb;
		to_edge = min(d.x, d.z);
	} else if (abs(local_normal.z) > 0.5) {
		color = side_a_color.rgb;
		to_edge = min(d.x, d.y);
	} else if (abs(local_normal.x) > 0.5) {
		color = side_b_color.rgb;
		to_edge = min(d.z, d.y);
	} else {
		color = side_b_color.rgb;
		to_edge = 1.0;
	}
	ALBEDO = to_edge < edge ? edge_color.rgb : color;
}
