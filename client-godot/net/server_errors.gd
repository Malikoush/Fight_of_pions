class_name ServerErrors
extends RefCounted
## Les refus du serveur, classés par ce qu'ils demandent au client (docs/PROTOCOLE.md,
## « Les erreurs »). C'est le seul endroit qui connaît les `kind` du protocole : le reste du
## client ne raisonne que par catégorie.

enum Category {
	## La place gardée n'existe plus (salle fermée, mauvaise clé, salle pleine à la reprise) :
	## revenir à l'accueil.
	SEAT_LOST,
	## Une autre connexion a repris la place : ne plus se reconnecter, proposer de la reprendre.
	SEAT_TAKEN,
	## La vue du client est en retard sur la partie : la redemander.
	STALE_VIEW,
	## Le code tapé pour rejoindre une salle ne mène nulle part.
	JOIN_REFUSED,
	## Le bot du serveur ne joue plus : la partie attend ; un message le relance.
	BOT_DOWN,
	## Le serveur n'accepte plus de nouvelle salle.
	SERVER_FULL,
	## Un autre refus (un bug du client, le plus souvent) : rien de particulier à faire.
	OTHER,
}

const _CATEGORIES := {
	"unknown_room": Category.SEAT_LOST,
	"bad_key": Category.SEAT_LOST,
	"room_full": Category.SEAT_LOST,
	"seat_taken": Category.SEAT_TAKEN,
	"not_your_turn": Category.STALE_VIEW,
	"refused_move": Category.STALE_VIEW,
	"bot_failed": Category.BOT_DOWN,
	"server_full": Category.SERVER_FULL,
}
## Les refus d'un code tapé à la main : les mêmes `kind` que la place perdue, mais pendant
## une demande `join`, ils disent seulement que ce code ne va pas.
const _JOIN_KINDS := ["unknown_room", "room_full"]


## La catégorie de ce refus. `joining` : la demande en cours était un `join`.
static func category(kind: String, joining: bool = false) -> Category:
	if joining and kind in _JOIN_KINDS:
		return Category.JOIN_REFUSED
	return _CATEGORIES.get(kind, Category.OTHER)
