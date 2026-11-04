// Le palet fantôme : le même palet peint (pion.gdshaderinc), transparent. Il montre où
// tombera un pion (survol, coup envoyé en attente de réponse).
shader_type spatial;
render_mode unshaded, depth_draw_always;

#define GHOST
#include "res://board/shaders/pion.gdshaderinc"
