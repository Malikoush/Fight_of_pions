class_name ServerErrors
extends RefCounted
## Les refus du serveur, classés par ce qu'ils demandent au client (docs/PROTOCOLE.md,
## « Les erreurs »). C'est le seul endroit qui connaît les `kind` du protocole : le reste du
## client ne raisonne que par catégorie.

enum Category {
	## La place gardée n'existe plus (salle fermée, par son créateur ou faute de joueurs,
	## mauvaise clé, salle pleine à la reprise) : revenir à l'accueil.
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
	## Le pseudo demandé est refusé (ou n'a pas pu être gardé) : le dire sous le champ.
	PSEUDO_REFUSED,
	## Le trio de cache demandé est refusé : la collection du client est sans doute en retard,
	## la redemander.
	TRIO_REFUSED,
	## Le code de récupération tapé n'ouvre aucun compte : le dire sous le champ.
	RECOVERY_REFUSED,
	## Un autre refus (un bug du client, le plus souvent) : rien de particulier à faire.
	OTHER,
}

const _CATEGORIES := {
	"unknown_room": Category.SEAT_LOST,
	"bad_key": Category.SEAT_LOST,
	"room_full": Category.SEAT_LOST,
	"room_closed": Category.SEAT_LOST,
	"seat_taken": Category.SEAT_TAKEN,
	"not_your_turn": Category.STALE_VIEW,
	"refused_move": Category.STALE_VIEW,
	"bot_failed": Category.BOT_DOWN,
	"server_full": Category.SERVER_FULL,
	"bad_pseudo": Category.PSEUDO_REFUSED,
	"not_owned": Category.TRIO_REFUSED,
	"bad_cache_trio": Category.TRIO_REFUSED,
	"bad_recovery_code": Category.RECOVERY_REFUSED,
}
## Les catégories des refus qui parlent d'une salle : pendant qu'on la quitte, ils ne
## comptent plus (`ServerLink`). Les autres (le compte, la base…) comptent toujours.
const ROOM_CATEGORIES := [Category.SEAT_LOST, Category.SEAT_TAKEN, Category.STALE_VIEW, Category.BOT_DOWN]
## Le refus d'un jeton de compte que le serveur ne connaît pas : le lien le traite lui-même
## (il crée un compte neuf), sans catégorie.
const UNKNOWN_ACCOUNT := "unknown_account"
## Le refus d'un code de récupération : le lien remet l'appareil sur son compte, puis
## l'annonce (`RECOVERY_REFUSED`).
const BAD_RECOVERY_CODE := "bad_recovery_code"

## Les refus qui, pendant qu'une de ces demandes attend sa réponse, ne parlent que d'elle
## (par le `type` de la demande) : un code tapé qui ne mène nulle part n'est pas une place
## perdue ; une base indisponible refuse le pseudo ou le trio demandé.
const _REQUEST_REFUSALS := {
	"join": {"unknown_room": Category.JOIN_REFUSED, "room_full": Category.JOIN_REFUSED},
	"set_pseudo": {"storage": Category.PSEUDO_REFUSED},
	"set_cache_trio": {"storage": Category.TRIO_REFUSED},
}


## Vrai si ce refus parle d'une salle (sa place, son tour, son bot).
static func is_about_room(kind: String) -> bool:
	return category(kind) in ROOM_CATEGORIES


## La catégorie de ce refus. `pending` : le `type` de la demande qui attend sa réponse
## (`join`, `set_pseudo`, `set_cache_trio`), ou vide.
static func category(kind: String, pending: String = "") -> Category:
	var during: Dictionary = _REQUEST_REFUSALS.get(pending, {})
	return during.get(kind, _CATEGORIES.get(kind, Category.OTHER))
