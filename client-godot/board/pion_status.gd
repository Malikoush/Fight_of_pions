class_name PionStatus
extends RefCounted
## Ce qu'on montre d'un pion d'après les effets continus qui le visent (la vue les donne ;
## rien n'est déduit) :
## - un anneau sous lui, pour son statut le plus fort : immobilisé, protégé, puis pacte ;
## - plus petit, s'il est un enfant (Familia : une protection qui dure tant qu'on l'encadre) ;
## - une couronne, s'il est une reine, et le nombre de ses gardes ;
## - ses jetons de pas (Lièvre Épique) ;
## - un badge par sorte d'effet, au-dessus de lui.

## Les statuts qui dessinent un anneau, du plus fort au plus faible : un seul anneau se voit.
const RING_ORDER: Array[String] = ["Immobilized", "Protected", "Pact"]
## Le badge de chaque sorte d'effet (design/elements/effets), dans l'ordre où ils se rangent.
const BADGES := {
	"Immobilized": "badge-immobilise",
	"Protected": "badge-protege",
	"Pact": "badge-pacte",
	"Return": "badge-sablier",
}
## Un enfant porte ce badge au lieu de celui de la protection.
const CHILD_BADGE := "badge-enfant"
const PROTECTED := "Protected"
const CROWN := "Crown"
const GUARD := "Guard"
const STEP_TOKENS := "StepTokens"
const FRAMED := "Framed"
const TOKENS := "Tokens"

## La sorte de l'anneau (une clé de `RING_ORDER`), ou vide.
var ring := ""
var child := false
var crown := false
var guards := 0
var step_tokens := 0
## Les badges à montrer (noms de fichiers de design/elements/effets), chacun une fois.
var badges: Array[String] = []


## L'état de ce pion, d'après ces effets (`ViewData.effects_on`).
static func of(effects: Array[Dictionary]) -> PionStatus:
	var status := PionStatus.new()
	var kinds: Array[String] = []
	for effect in effects:
		var kind := Wire.tag(effect.get("kind"))
		var end: Variant = effect.get("end")
		kinds.append(kind)
		match kind:
			CROWN:
				status.crown = true
			GUARD:
				status.guards += 1
			STEP_TOKENS:
				var left: Variant = Wire.content(end) if Wire.tag(end) == TOKENS else null
				status.step_tokens += left if left is int else 0
			PROTECTED:
				status.child = status.child or Wire.tag(end) == FRAMED
	for kind in RING_ORDER:
		if kinds.has(kind):
			status.ring = kind
			break
	for kind: String in BADGES:
		if kinds.has(kind):
			status.badges.append(CHILD_BADGE if kind == PROTECTED and status.child else BADGES[kind])
	return status


## Vrai si ce pion n'a rien à montrer.
func is_plain() -> bool:
	return ring == "" and not crown and guards == 0 and step_tokens == 0 and badges.is_empty()
