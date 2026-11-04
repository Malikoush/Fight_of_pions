// Le contour noir d'un objet convexe (les pions) : une copie un peu plus grande, dont on ne
// voit que l'intérieur, derrière l'objet. Les échelles viennent du code, d'après la taille
// de l'objet, pour que le trait ait la même épaisseur partout.
shader_type spatial;
render_mode unshaded, cull_front;

uniform vec4 ink : source_color;
uniform vec3 grow;

void vertex() {
	VERTEX *= grow;
}

void fragment() {
	ALBEDO = ink.rgb;
}
