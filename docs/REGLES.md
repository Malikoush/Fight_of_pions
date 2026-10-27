# Carnet de règles

Ce carnet dit **ce que le moteur implémente réellement**, avec la date de chaque changement. La référence de design reste le GDD (v0.2, octobre 2026). Une règle n'est codée qu'une fois écrite ici.

## Règles implémentées

### Plateau et objectif (2026-10-26)

- Plateau de `board_size × board_size` cases (10 par défaut), toutes jouables.
- Chaque joueur a `pions_per_player` pions en réserve (15 par défaut).
- Aligner au moins `align_to_win` pions (6 par défaut) en ligne droite (horizontale, verticale ou diagonale) **en posant** gagne immédiatement.

### Configuration (2026-10-26)

Une partie refuse de démarrer si ses réglages sont incohérents :

- la taille du plateau doit être comprise entre 2 et 26 (une colonne par lettre, de A à Z) ;
- l'alignement doit être compris entre 2 et la taille du plateau ;
- chaque joueur doit avoir au moins un pion ;
- chaque tour, y compris le premier, doit donner au moins une action ;
- poser un pion doit coûter au moins une action, sinon toute la réserve partirait en un seul tour ;
- poser un pion doit être possible en économisant : `place_cost` ≤ `actions_per_turn` + `max_banked_actions` ;
- la limite de tours `max_turns` doit valoir au moins 1 ;
- (2026-10-27) chaque attaque du catalogue doit être jouable par le moteur, n'y figurer qu'une fois, coûter au moins une action et au plus ce qu'on peut avoir en économisant, et avoir une recharge d'au moins 1 tour ;
- (2026-10-27) chaque attaque de départ d'un joueur doit être dans le catalogue.

Une configuration où la victoire semble impossible (moins de pions que l'alignement demandé) est **acceptée** : elle sert aux tests, et les invocations pourraient la rendre jouable (à revoir en B5).

### Tour de jeu (2026-10-26)

- Le joueur 1 commence avec `first_turn_actions` action (1 par défaut). Tous les autres tours, y compris le premier du joueur 2, donnent `actions_per_turn` actions (2 par défaut).
- Poser un pion coûte `place_cost` actions (1 par défaut). Finir son tour est gratuit.
- Le tour se termine automatiquement quand il ne reste plus d'action.
- En finissant son tour, on garde jusqu'à `max_banked_actions` actions non utilisées (2 par défaut). Elles s'ajoutent au tour suivant, soit jusqu'à 4 actions.

### Refus (2026-10-26, complété le 2026-10-27)

Une action est refusée, sans rien modifier, si :

- la partie est finie ;
- elle ne répond pas à la décision attendue (par exemple poser un pion pendant le choix d'une cible) ;
- il ne reste pas assez d'actions ;
- la case est hors du plateau ou occupée ;
- la réserve de pions est vide ;
- l'attaque n'existe pas dans l'arsenal, est en recharge, ou n'a aucune cible possible ;
- le lanceur ou la cible choisie n'est pas permis.

### Attaques (2026-10-27)

- Chaque joueur commence avec des attaques (son **arsenal**). En attendant le draft, elles sont fixées par la configuration (`starting_attacks`).
- Le **coût** et la **recharge** de chaque attaque viennent du catalogue de la configuration. Par défaut : Ours Rare 2 actions, Ours Épique 3 actions, recharge d'un tour, c'est-à-dire une fois par tour.
- Lancer une attaque se fait en plusieurs choix : la carte, puis le **pion lanceur** (un de ses pions sur le plateau), puis chaque **cible**. Une attaque n'est proposée que si au moins une attaque complète est possible, et chaque choix proposé mène à une attaque complète.
- Une attaque commencée va **toujours au bout** ; pendant les choix, aucune autre action n'est possible. L'annulation se fait dans l'interface, avant d'envoyer les choix au moteur.
- Le **coût est payé** et la carte part **en recharge** au dernier choix, quand l'attaque est résolue. Elle redevient prête au début du tour de son propriétaire, une fois sa recharge écoulée.
- Un pion **tué** quitte le plateau et retourne dans la réserve de son propriétaire. Il ne laisse pas de tombe (pas avant le Médecin).
- Une attaque qui dépense les dernières actions du tour termine le tour, comme une pose.

### Ours (2026-10-27)

Attaque de corps à corps sur les 8 cases autour du lanceur, diagonales comprises.

- **Rare** : élimine 1 ennemi adjacent au lanceur.
- **Épique** : élimine 2 ennemis adjacents au même lanceur, deux pions différents.
- La Commune et la Légendaire (qui repoussent) ne sont pas encore jouables (B4).

### Fin de partie (2026-10-26)

Une partie se termine dans l'un de ces trois cas, vérifiés dans cet ordre :

1. **Alignement** : le joueur qui aligne en posant gagne immédiatement.
2. **Partie figée** : plus aucun joueur ne pourra jamais agir autrement qu'en finissant son tour, et rien de programmé ne peut changer la situation. Un joueur peut encore agir s'il peut poser (un pion en réserve et une case libre) ou lancer une attaque qui a une cible, même si elle est en recharge ou qu'il doit économiser pour la payer (2026-10-27). Résultat : **match nul**.
   Une réserve vide ne fige donc plus la partie tant qu'un contact entre ennemis permet d'attaquer. Le risque principal devient la partie sans fin, traité par la limite de tours et l'anti-stagnation.
3. **Limite de tours** : à la fin du tour numéro `max_turns` (200 par défaut). Chaque tour d'un joueur compte : 200 tours font 100 tours par joueur. C'est un filet de sécurité pour les bots ; il n'est pas censé arriver dans une vraie partie. Résultat : **match nul**.

Le moteur retient la raison de la fin, en plus du résultat. Le résultat hors alignement est pour l'instant toujours le nul ; un départage pourra le remplacer sans toucher au reste.

## Pas encore implémenté

- Les autres attaques que l'Ours Rare et Épique, et les réactions.
- La règle d'or (aucune attaque actuelle ne peut créer une ligne : tuer n'en complète jamais).
- Le draft et la cache secrète.
- La mécanique anti-stagnation (Effondrement et/ou boost d'actions), qui fera de vraies fins de partie au lieu de nuls. À trancher par playtests puis bots (voir la feuille de route).

## Questions bloquantes

À trancher par Martin et Théo avant la tâche indiquée. La réponse est notée ici, datée, puis reportée dans le GDD.

| Bloque | Question | Proposition par défaut | Réponse |
|---|---|---|---|
| T4 | Comment finit une partie où plus personne ne peut aligner (réserves vides, attaques inutiles) ? | Match nul après `max_turns` tours (réglable), en attendant l'Effondrement | **2026-10-26** : partie figée → nul ; limite de 200 tours → nul (filet de sécurité). L'anti-stagnation (Effondrement, boost) est une tâche de design à part. |
| T4 | Une configuration valide peut rendre la partie impossible à finir : pose impossible (`place_cost` > `actions_per_turn` + `max_banked_actions`) ou victoire impossible (`pions_per_player` < `align_to_win`). Faut-il la refuser ? | Oui : refuser ces configurations dans `validate` | **2026-10-26** : mixte. On refuse la pose impossible ; on accepte la victoire impossible (tests, invocations). |
| B1 | Le coût d'une attaque est-il payé quand tous les choix sont faits ? Peut-on annuler gratuitement pendant le ciblage ? | Oui et oui | **2026-10-27** : oui. Le coût est payé et la carte part en recharge au dernier choix. L'annulation est gratuite mais se fait dans l'interface (sélection sur une copie de la partie) : le moteur n'a pas d'action « annuler », pour qu'aucun bot ne puisse boucler. Le moteur ne propose jamais un choix qui mène à une impasse. |
| B1 | Un pion tué retourne dans la réserve de son propriétaire, et laisse-t-il une tombe dès maintenant ? | Retour en réserve oui ; tombe ajoutée avec le Médecin | **2026-10-27** : retour en réserve, comme le dit le GDD. Pas de tombe avant B9 (Médecin), quand sa règle sera figée. |
| B2 | « Survivre à un tour adverse » : la même ligne, ou n'importe quelle ligne de 6 ? | N'importe quelle ligne de 6 au début du tour suivant du joueur | |
| B2 | Si une attaque crée une ligne pour **l'adversaire** (échange, séduction), que se passe-t-il ? | Victoire en attente pour cet adversaire | |
| B2 | Si la partie s'arrête (figée ou limite de tours) alors qu'une victoire est en attente, que se passe-t-il ? | La victoire en attente l'emporte | |
| Anti-stagnation | Effondrement, boost d'actions par tour, ou les deux ? Règle de répétition (même position 3 fois) ? Nul ou départage quand la partie s'arrête sans alignement ? | À mesurer avec les bots | |
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
| 2026-10-26 | T2 : bornes de configuration vérifiées ; coût de la pose réglable (`place_cost`) ; `action_per_turn` renommé `actions_per_turn`. Aucune règle de jeu modifiée. |
| 2026-10-26 | T3 : chaque pion reçoit une identité stable à sa pose ; `pawns_per_player` renommé `pions_per_player`. Aucune règle de jeu modifiée. |
| 2026-10-26 | T4 : fin de partie par partie figée ou limite de tours (nul) ; configuration où la pose est impossible refusée. |
| 2026-10-26 | T5 : chaque partie a une graine choisie à sa création ; une partie se rejoue depuis sa configuration, sa graine et ses actions. Aucune règle de jeu modifiée. |
| 2026-10-27 | T5 : les compteurs d'actions (restantes et gardées) sont annoncés par un événement à chaque changement. Aucune règle de jeu modifiée. |
| 2026-10-27 | T6 : le labo affiche la graine, copie et charge une partie en JSON, annule un coup, joue au hasard et montre les cases jouables. Aucune règle de jeu modifiée. |
| 2026-10-27 | B1 : attaques (arsenal, coût, recharge, choix du lanceur et des cibles, résolution au dernier choix, kill avec retour en réserve) et Ours Rare et Épique. « Partie figée » tient compte des attaques. |
| 2026-10-27 | Arène et bots (membre `bots/`, `cargo arene`) : des bots jouent des milliers de parties pour mesurer l'effet d'un réglage. Moteur inchangé. Aucune règle de jeu modifiée. |
| 2026-10-27 | Labo : chaque joueur humain ou bot, écran Arène pour simuler des parties et en revoir une. Moteur inchangé. Aucune règle de jeu modifiée. |
