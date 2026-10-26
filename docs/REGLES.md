# Carnet de règles

Ce carnet dit **ce que le moteur implémente réellement**, avec la date de chaque changement. La référence de design reste le GDD (v0.2, octobre 2026). Une règle n'est codée qu'une fois écrite ici.

## Règles implémentées

### Plateau et objectif (2026-10-26)

- Plateau de `board_size × board_size` cases (10 par défaut), toutes jouables.
- Chaque joueur a `pawns_per_player` pions en réserve (15 par défaut).
- Aligner au moins `align_to_win` pions (6 par défaut) en ligne droite (horizontale, verticale ou diagonale) **en posant** gagne immédiatement.

### Tour de jeu (2026-10-26)

- Le joueur 1 commence avec `first_turn_actions` action (1 par défaut). Tous les autres tours, y compris le premier du joueur 2, donnent `action_per_turn` actions (2 par défaut ; renommé `actions_per_turn` en T2).
- Poser un pion coûte 1 action. Finir son tour coûte 0 action.
- Le tour se termine automatiquement quand il ne reste plus d'action.
- En finissant son tour, on garde jusqu'à `max_banked_actions` actions non utilisées (2 par défaut). Elles s'ajoutent au tour suivant, soit jusqu'à 4 actions.

### Refus (2026-10-26)

Une action est refusée, sans rien modifier, si :

- la partie est finie ;
- il ne reste pas assez d'actions ;
- la case est hors du plateau ou occupée ;
- la réserve de pions est vide.

## Pas encore implémenté

- Les attaques et les réactions.
- La règle d'or.
- Le draft et la cache secrète.
- Le match nul.
- L'Effondrement (règle optionnelle, mise de côté par le GDD).

## Questions bloquantes

À trancher par Martin et Théo avant la tâche indiquée. La réponse est notée ici, datée, puis reportée dans le GDD.

| Bloque | Question | Proposition par défaut | Réponse |
|---|---|---|---|
| T4 | Comment finit une partie où plus personne ne peut aligner (réserves vides, attaques inutiles) ? | Match nul après `max_turns` tours (réglable), en attendant l'Effondrement | |
| B1 | Le coût d'une attaque est-il payé quand tous les choix sont faits ? Peut-on annuler gratuitement pendant le ciblage ? | Oui et oui | |
| B1 | Un pion tué retourne dans la réserve de son propriétaire, et laisse-t-il une tombe dès maintenant ? | Retour en réserve oui ; tombe ajoutée avec le Médecin | |
| B2 | « Survivre à un tour adverse » : la même ligne, ou n'importe quelle ligne de 6 ? | N'importe quelle ligne de 6 au début du tour suivant du joueur | |
| B2 | Si une attaque crée une ligne pour **l'adversaire** (échange, séduction), que se passe-t-il ? | Victoire en attente pour cet adversaire | |
| B3 | « Pendant N tours » : les tours de qui ? | N tours du joueur qui subit l'effet | |
| B3 | Un pion immobilisé peut-il lancer une attaque ? Être déplacé ou échangé par une attaque ? | Ne peut ni lancer ni bouger de lui-même ; peut subir | |
| B4 | Le déplacement est-il retiré des actions de base ? | Oui (proposition du GDD) | |
| B4 | Peut-on sauter par-dessus un allié (Saut du loup) ? | Non | |
| B5 | Les pions invoqués (Familia, Meilleurs amis, Sentinelle…) viennent-ils de la réserve ? Réserve vide = impossible ? | Oui, ils viennent de la réserve | |
| B5 | Un pion séduit qui meurt retourne dans la réserve de quel joueur ? | Son camp actuel | |
| B7 | Qu'est-ce que le « mur » du Coup du taureau ? Un trou est-il permanent ? | À définir | |
| B8 | Quand plusieurs effets réagissent au même événement, dans quel ordre ? | D'abord le joueur qui subit, puis l'autre | |
| B10 | Les Légendaires peuvent-elles sortir au draft ? | Non (comme le GDD actuel) | |
| B10 | Comment se déclenche la révélation d'une carte de la cache adverse ? | À définir | |

## Historique

| Date | Changement |
|---|---|
| 2026-10-26 | Carnet créé. Correctif : l'événement de fin de tour annonçait le mauvais joueur. |
