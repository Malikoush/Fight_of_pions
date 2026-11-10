class_name Screen
extends Control
## Ce que tout écran sait faire : recevoir les nouvelles du serveur et de la session par des
## méthodes nommées, que chaque écran ne réécrit que si elles le concernent. La racine
## (`main.gd`) transmet tout à l'écran affiché sans avoir à savoir lequel c'est ; chaque
## écran garde pour lui ce qu'il en fait.

## Ce qu'un écran a fait d'un refus du serveur.
enum Reply {
	## Rien : le refus ne le concerne pas.
	IGNORED,
	## Il l'a traité (un message affiché, un bouton réactivé…).
	HANDLED,
	## Il l'a traité, et sa vue est peut-être en retard : il faut la redemander.
	RESYNC,
}

## La session en cours, donnée par le routeur avant l'affichage.
var session: Session


## Vrai pour un écran qui montre le plateau 3D derrière lui.
func uses_board() -> bool:
	return false


## L'état de la connexion au serveur a changé.
func on_connection(_status: ServerLink.Status) -> void:
	pass


## Une vue de la partie, et ce qui s'est passé depuis la précédente.
func on_view(_view: ViewData, _events: Array) -> void:
	pass


## La réponse à un aperçu (le choix d'une carte), ou `null` : la carte est complète.
func on_preview(_view: ViewData) -> void:
	pass


## Un refus du serveur, par sa catégorie, avec son message en français.
func on_refused(_category: ServerErrors.Category, _message: String) -> Reply:
	return Reply.IGNORED


## La session a changé : l'adversaire est arrivé ou parti, le bot ne joue plus, le
## catalogue, le pseudo ou la collection sont arrivés.
func on_session_changed() -> void:
	pass


## La partie vient de finir et la victoire a fait gagner cette carte (`{"kind", "rarity"}`).
func on_reward(_card: Dictionary) -> void:
	pass
