# Carnet de règles

Ce carnet dit **ce que le moteur implémente réellement**, avec la date de chaque changement. La référence de design est le GDD (v0.2, octobre 2026), remplacé pour les attaques et les réactions par les [fiches attaques validées](FICHES_ATTAQUES.md) du 2026-10-29. Une règle n'est codée qu'une fois écrite ici ; en cas de désaccord, ce carnet prime.

## Règles implémentées

### Plateau et objectif (2026-10-26)

- Plateau de `board_size × board_size` cases (10 par défaut), toutes jouables.
- Chaque joueur a `pions_per_player` pions en réserve (15 par défaut).
- Aligner au moins `align_to_win` pions (6 par défaut) en ligne droite (horizontale, verticale ou diagonale) **en posant** gagne immédiatement, sans la règle d'or ; avec elle, la ligne gagne au début du tour suivant de son propriétaire (voir « Fin de partie »).

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
- Le tour se termine automatiquement quand il ne reste plus d'action, ni de pose possible (2026-10-28, voir « Économie réglable »).
- En finissant son tour, on garde jusqu'à `max_banked_actions` actions non utilisées (2 par défaut). Elles s'ajoutent au tour suivant, soit jusqu'à 4 actions.

### Économie réglable (2026-10-28)

Quatre réglages permettent de jouer les règles cibles (voir plus bas). Par défaut, ils ne changent rien aux règles ci-dessus.

- **Poses par tour** (`max_placements_per_turn`, sans limite par défaut) : on pose au plus ce nombre de pions dans un tour ; les poses non faites ne s'accumulent pas. Une pose de trop est refusée (« plus aucune pose permise ce tour »).
- **Pose gratuite** : `place_cost` peut valoir 0, seulement si les poses sont limitées par tour (sinon toute la réserve partirait en un tour). Une pose gratuite ne change pas les compteurs d'actions.
- **Banque de départ** (`starting_banked_actions`, 0 et 0 par défaut), au plus `max_banked_actions` : J1 reçoit la sienne avec les actions de son premier tour ; J2 trouve la sienne en banque. La Pièce : 0 pour J1, 1 pour J2.
- **Mal du débarquement** (`summoning_sickness`, désactivé par défaut) : un pion posé ce tour ne peut pas être choisi comme lanceur, et une attaque dont le seul lanceur possible vient d'arriver n'est pas proposée (refus : « aucun lanceur prêt »). Pour la partie figée, il compte quand même : il pourra lancer au tour suivant.
- **Poses restantes** (`Game::placements_left`) : autant que les actions en paient, que les poses permises ce tour, que la réserve et que les cases libres. Le tour se termine automatiquement quand il ne reste ni action ni pose (plateau plein compris).
- Une configuration est refusée si la pose est gratuite sans limite par tour, si la limite de poses vaut 0, ou si une banque de départ dépasse `max_banked_actions`.
- Une partie copiée avant ces réglages se charge toujours : les champs absents prennent leur valeur par défaut.

### Refus (2026-10-26, complété le 2026-10-27 et le 2026-10-30)

Une action est refusée, sans rien modifier, si :

- la partie est finie ;
- elle ne répond pas à la décision attendue (par exemple poser un pion pendant le choix d'une cible) ;
- il ne reste pas assez d'actions ;
- la case est hors du plateau ou occupée ;
- la réserve de pions est vide ;
- (2026-10-28) le joueur a déjà posé autant de pions que permis ce tour ;
- l'attaque n'existe pas dans l'arsenal, est en recharge, ou n'a aucune cible possible ;
- (2026-10-28, complété le 2026-10-30) ses seuls lanceurs possibles ont été posés ce tour (avec le mal du débarquement) ou sont immobilisés ;
- le lanceur ou la cible choisie n'est pas permis ;
- (2026-10-30) on veut jouer l'attaque en cours (`FinishAttack`) avant d'avoir choisi son lanceur, ou sans avoir assez de cibles.

### Attaques (2026-10-27)

- Chaque joueur commence avec des attaques (son **arsenal**). En attendant le draft, elles sont fixées par la configuration (`starting_attacks`).
- Le **coût** et la **recharge** de chaque attaque viennent du catalogue de la configuration. Par défaut (2026-10-30), les coûts des fiches : Commune 1 action, Rare 2, Épique 3, Légendaire 4 ; recharge d'un tour pour toutes, c'est-à-dire une fois par tour.
- Lancer une attaque se fait en plusieurs choix : la carte, puis le **pion lanceur** (un de ses pions sur le plateau), puis les **cibles**. Une attaque n'est proposée que si au moins une attaque complète est possible, et chaque choix proposé mène à une attaque complète.
- (2026-10-30) Une carte dit **combien de cibles** choisir : un nombre exact, ou « jusqu'à N ». Une attaque sans cible (Grognement, Garde royal) est jouée dès le choix du lanceur. Une attaque est jouée dès qu'il n'y a plus rien à choisir : nombre maximal de cibles atteint, ou plus aucune cible possible. Avant le maximum, le joueur peut la jouer avec les cibles déjà choisies (`Action::FinishAttack`), si elle en a assez.
- (2026-10-30) Une attaque **sans effet** n'est jamais proposée : un Grognement sans ennemi collé au lanceur, un Attila sans statut adverse à retirer…
- Une attaque commencée va **toujours au bout** ; pendant les choix, aucune autre action n'est possible. L'annulation se fait dans l'interface, avant d'envoyer les choix au moteur.
- Le **coût est payé** et la carte part **en recharge** quand l'attaque est jouée. Elle redevient prête au début du tour de son propriétaire, une fois sa recharge écoulée.
- Un pion **tué** quitte le plateau ; ses effets continus s'arrêtent. Il retourne dans la réserve de son propriétaire, tout de suite ou après l'infirmerie (voir plus bas). Il ne laisse pas de tombe (pas avant le Médecin).
- Une attaque qui dépense les dernières actions du tour termine le tour, comme une pose, s'il ne reste aucune pose possible (2026-10-28).

### Statuts et durées (2026-10-30)

Les règles communes des attaques décidées le 2026-10-30 (voir « Règles communes des attaques », plus bas), codées :

- Un **effet continu** est ce qu'une carte laisse en jeu : pour l'instant, deux statuts, posés sur un pion (qu'ils suivent par son identité) :
  - **immobilisé** : le pion ne peut pas lancer d'attaque (refus : « aucun lanceur prêt ») ;
  - **protégé** : le pion ne peut pas être choisi comme cible d'un kill ; il peut encore être immobilisé.
- Chaque effet retient le joueur qui l'a créé et la carte qui l'a créé. Un pion est immobilisé tant qu'au moins un effet l'immobilise : deux immobilisations se superposent, sans règle de cumul.
- **Durée** : une durée de N tours se compte en tours de l'adversaire de celui qui a joué l'effet ; le tour pendant lequel l'effet est créé ne compte pas ; l'effet s'arrête **à la fin** du N-ième de ces tours. Le moteur retient le numéro de ce dernier tour (`last_turn`).
- Les effets d'un pion tué s'arrêtent avec lui.
- Chaque début et chaque fin d'effet sont annoncés aux clients.

### Infirmerie (2026-10-30)

- Un pion tué passe `infirmary_turns` tours à l'infirmerie (2 dans les règles cibles, 0 par défaut : retour immédiat dans la réserve, comme avant). La durée suit la même horloge que les effets : elle se compte en tours de l'adversaire du joueur qui l'a tué, c'est-à-dire aujourd'hui de son propriétaire (on ne tue que des pions adverses) ; il revient dans la réserve de son propriétaire à la fin du dernier de ces tours, et peut être reposé au tour suivant. Décidé le 2026-10-30 : l'infirmerie se compte toujours en tours du propriétaire du pion, même tué par sa propre carte (codé en B6, avec la première carte qui le permet).
- L'infirmerie n'est pas un effet continu : aucune carte ne l'annule.

### Ours (2026-10-27, complété le 2026-10-30)

Le corps à corps, sur les 8 cases collées au lanceur, diagonales comprises.

- **Commune, le Grognement** (2026-10-30) : les pions adverses collés au lanceur sont immobilisés pendant 1 tour. Pas de cible à choisir ; il faut au moins un ennemi collé.
- **Rare** : tue 1 pion adverse collé au lanceur.
- **Épique** : tue **jusqu'à 2** pions adverses collés au même lanceur, deux pions différents (2026-10-30 ; avant : exactement 2).
- **Légendaire, le Rugissement** (2026-10-30) : tue jusqu'à 2 pions adverses collés au lanceur (aucun, si l'on veut) ; les autres pions adverses collés à lui sont immobilisés pendant 2 tours.
- Un pion protégé n'est jamais proposé pour être tué ; le Grognement et le Rugissement l'immobilisent comme les autres.

### Beauté fatale (2026-10-30)

Le **regard** : un pion adverse en ligne droite (8 directions), à 1 ou 2 cases, sans pion entre les deux.

- **Commune** : immobilise 1 pion adverse pendant 2 tours.
- **Rare** : immobilise 2 pions adverses différents pendant 2 tours.
- **Charme fatal** : un pion visé déjà immobilisé (par n'importe quelle carte) est tué ; s'il est protégé, il est seulement immobilisé à nouveau.
- L'Épique et la Légendaire (séduction) viendront avec la pose (B5).

### Reine : le Garde royal (2026-10-30)

- **Commune** : le lanceur est protégé pendant 2 tours. Comme toute carte qui agit sur un de tes pions, elle s'applique au lanceur, qui doit donc être posé avant ce tour et non immobilisé. Pas de cible à choisir.
- Un joueur qui l'a dans son arsenal peut toujours agir tant qu'il a un pion sur le plateau (un pion posé ce tour ou immobilisé pourra la lancer plus tard) : une telle partie ne se fige jamais, la limite de tours la termine.
- La Rare, l'Épique et la Légendaire (la reine) viendront avec la pose (B5).

### Attila Commune (2026-10-30)

- **Commune** : sur tout le plateau, retire d'un pion tous les effets placés par l'adversaire (immobilisé, protégé), quelle que soit la carte qui les a posés. Le pion peut être à toi (le libérer) ou à l'adversaire (lui ôter sa protection). Un pion qui ne porte que tes propres effets ne peut pas être visé.
- La Rare, l'Épique et la Légendaire (effets entiers, Terre brûlée) viendront avec l'annulation (B6).

### Mouvement (2026-10-30)

- Un pion **déplacé** ou **échangé** garde son identité, ses effets continus et son ancienneté (mal du débarquement). Chaque déplacement et chaque échange est annoncé.
- Une ligne faite en bougeant (pour l'un ou l'autre joueur) suit la règle des lignes faites par une carte : elle gagne au début du tour de son propriétaire si elle y est encore, avec ou sans la règle d'or.
- Un pion immobilisé ne peut pas lancer, donc pas bouger par une carte de son propriétaire ; l'adversaire peut encore l'échanger.

### Saut du loup (2026-10-30)

- Un **saut** fait toujours 2 cases en ligne droite (8 directions), même dans le vide, et atterrit sur une case libre, par-dessus n'importe quoi : du vide, un de tes pions, un pion adverse.
- **Commune** : 1 saut ; un pion adverse survolé est immobilisé 2 tours.
- **Rare** : 1 saut meurtrier. **Épique** : jusqu'à 2. **Légendaire** : jusqu'à 3.
- **Saut meurtrier** : le pion adverse survolé est tué tout de suite ; sa case est libre pour la suite. Un pion protégé peut être survolé, mais il n'est pas tué.
- **Sauts enchaînés** : chaque saut part de la case d'arrivée du précédent ; on peut s'arrêter avant le maximum, et repasser par une case déjà visitée, mais jamais survoler deux fois le même pion.
- Les choix : le lanceur, puis chaque case d'arrivée.

### Lièvre (2026-10-30)

- Un **pas** va vers une case libre collée (8 directions) ; on peut tourner en chemin, jamais passer par-dessus un pion.
- **Commune** : 1 de tes pions avance de jusqu'à 2 pas. **Rare** : jusqu'à 2 pions, de jusqu'à 2 pas chacun. **Légendaire** : jusqu'à 3 pions, de jusqu'à 3 pas chacun. « Jusqu'à 2 (3) pions » est une hypothèse, en attendant la réponse à la question ouverte (voir les questions bloquantes) ; « jusqu'à 2 (3) pas » est décidé.
- Chaque pion qui avance finit sur une autre case que sa case de départ.
- Le premier pion est le lanceur ; les suivants sont des lanceurs aussi (posés avant ce tour, non immobilisés), différents, et bougent l'un après l'autre : le deuxième voit la nouvelle place du premier.
- Les choix : le lanceur, ses pas, puis le pion suivant et ses pas… Un pas se choisit sur une case libre, un pion suivant sur un de tes pions : les deux ne se confondent jamais.
- L'Épique (jetons) viendra avec les jetons (B6).

### Interchange (2026-10-30)

- **Rare** : le lanceur échange sa case avec un pion adverse à 1 ou 2 cases en ligne droite, même derrière un autre pion.
- **Épique** : avec n'importe quel pion adverse.
- Le pion adverse peut être protégé ou immobilisé.
- La Commune (téléportation avec retour) et la Légendaire viendront avec les retours programmés (B6).

### Fin de partie (2026-10-26)

Une partie se termine dans l'un de ces trois cas, vérifiés dans cet ordre :

1. **Alignement** : une ligne d'au moins `align_to_win` pions fait gagner son propriétaire :
   - sans la règle d'or (par défaut), dès qu'elle est faite en posant ;
   - **au début de son tour**, si elle y est encore, dans tous les cas (2026-10-29). C'est ce qui fait gagner avec la règle d'or (`golden_rule`), et ce qui couvre les lignes faites par une carte, y compris une carte de l'adversaire. Ce contrôle passe avant le gain des actions et les recharges : le joueur qui gagne ainsi ne reçoit pas ses actions.
2. **Partie figée** : plus aucun joueur ne pourra jamais agir autrement qu'en finissant son tour, et rien de programmé ne peut changer la situation : aucun effet continu en jeu, personne à l'infirmerie (2026-10-30). Elle est vérifiée après chaque action, et à la fin d'un tour où un effet s'est arrêté ou un pion est sorti de l'infirmerie. Un joueur peut encore agir s'il peut poser (un pion en réserve et une case libre) ou lancer une attaque qui a une cible, même si elle est en recharge ou qu'il doit économiser pour la payer (2026-10-27). Résultat : **match nul**.
   Une réserve vide ne fige donc plus la partie tant qu'un contact entre ennemis permet d'attaquer. Le risque principal devient la partie sans fin, traité par la limite de tours et l'anti-stagnation.
3. **Limite de tours** : à la fin du tour numéro `max_turns` (200 par défaut). Chaque tour d'un joueur compte : 200 tours font 100 tours par joueur. C'est un filet de sécurité pour les bots ; il n'est pas censé arriver dans une vraie partie. Résultat : **match nul**.

Quand la partie s'arrête ainsi (figée ou limite de tours), une ligne qui attendait le début du tour de son propriétaire **gagne quand même** (2026-10-29) : on fait comme si les tours continuaient, donc l'adversaire du dernier joueur est vérifié d'abord, puis le dernier joueur. Sinon, nul.

Le moteur retient la raison de la fin, en plus du résultat. Le résultat hors alignement est pour l'instant toujours le nul ; un départage pourra le remplacer sans toucher au reste.

## Pas encore implémenté

- Les autres attaques que l'Ours, la Beauté fatale Commune et Rare, le Saut du loup, le Garde royal, Attila Commune, l'Interchange Rare et Épique, et le Lièvre Commune, Rare et Légendaire ; et les réactions.
- La défaite par manque de pions (avec le cimetière, B6).
- Le draft et la cache secrète.
- La mécanique anti-stagnation (Effondrement et/ou boost d'actions), qui fera de vraies fins de partie au lieu de nuls. À trancher par playtests puis bots (voir la feuille de route).

### Règles cibles (décidées le 2026-10-28, codées en option sauf le départage)

Codées avec les réglages ci-dessus (variante « règles cibles », `bots::target_rules()`) : 1 pose gratuite par tour, 1 action par tour, 3 actions gardées au plus, la Pièce, le mal du débarquement, la règle d'or (`golden_rule`, 2026-10-29) et l'infirmerie de 2 tours (`infirmary_turns`, 2026-10-30). Reste à coder : la défaite par manque de pions (B6, avec le cimetière) et le départage, en attente.

Elles remplaceront le tour de jeu actuel (actions partagées entre poser et attaquer). Comparée à une économie « à la pioche » (pions sacrifiés en points d'action, pions tués recrutés, fatigue), écartée : le jeu n'est pas un Connect6, ce sont les cartes (attaque, mouvement, invocation, pose…) qui font avancer le plateau.

- **Pose** : au plus 1 par tour, facultative ; elle ne s'accumule pas et ne coûte aucun point d'action.
- **Points d'action** : 1 par tour, gardés en banque jusqu'à 3 ; le surplus est perdu. Ils servent aux cartes. La banque est visible.
- **Attaques** : ~~le lanceur doit être au contact de sa cible (les 8 cases autour)~~ ; depuis le 2026-10-30, chaque carte dit sa portée, sans portée par défaut. Les cartes gardent leur recharge. L'Ours Rare reste à 2 points au moins : à 1, chaque joueur pourrait annuler la pose adverse à chaque tour (le pion tué revient à son propriétaire, qui le repose).
- **Mal du débarquement** : un pion **posé ou invoqué** ce tour ne peut pas lancer d'attaque. Un pion déplacé garde son ancienneté. ~~Les futures cartes avec le mot-clé « Charge » ignoreront cette règle.~~ Le mot-clé « Charge » n'existe plus dans les fiches du 2026-10-29 (question ouverte plus bas).
- **Pion tué** : ~~il retourne dans la réserve de son propriétaire~~. Depuis les fiches du 2026-10-29 : il passe **2 tours à l'infirmerie**, puis retourne dans la réserve de son propriétaire (pas de vol de pion). L'infirmerie suit l'horloge des durées (voir « Règles communes des attaques »). Codé en B3 (`infirmary_turns` = 2 dans la variante « règles cibles »).
- **Défaite par manque de pions** (fiches du 2026-10-29) : un joueur qui a moins de `align_to_win` pions au total (plateau, réserve et infirmerie) ne peut plus aligner, donc il a perdu. Seul le cimetière (Massacre divin) fait baisser ce total.
- **Règle d'or** : une ligne de 6 ou plus, faite par une pose ou par une carte, ne gagne pas tout de suite. **Si, au début de ton tour, tu as une ligne de 6 ou plus, tu gagnes** : l'adversaire a eu un tour complet pour la casser. N'importe quelle ligne compte, pas seulement celle d'origine.
- **Début de tour**, dans cet ordre : 1) la règle d'or, 2) le gain du point d'action, 3) les recharges.
- **Premier joueur** : J2 commence avec 1 point en banque (la Pièce).
- **Tous ses pions sur le plateau** : on ne peut plus poser, il reste les cartes. C'est au joueur de gérer (à revoir plus tard si besoin).
- **Limite de tours** : la partie s'arrête sur une égalité. Le départage (critère et tour) est en attente.
- Une ligne créée pour un joueur par une carte de l'adversaire (échange, séduction) compte, comme toute ligne (2026-10-29).

### Règles communes des attaques (décidées le 2026-10-30, codées en B3 pour ce qui concerne les cartes de B3)

Elles complètent les règles communes des [fiches](FICHES_ATTAQUES.md#règles-communes-des-attaques), après l'analyse de game design du 2026-10-30.

- **Cibles** : les effets **nuisibles** (tuer, immobiliser, déplacer de force) ne touchent que les pions **adverses** ; les effets **favorables** (protéger, déplacer, faire un enfant…) ne touchent que **tes** pions. Remplace « les attaques ne visent que les pions adverses », que contredisaient le Garde royal, le Lièvre, l'Interchange, la Familia et le Médecin.
- **Le lanceur est le pion qui agit** : une carte qui agit sur « un de tes pions » s'applique **au lanceur lui-même**, comme un sort qu'on lui lance. Le Lièvre fait avancer le lanceur, l'Interchange téléporte ou échange le lanceur, le Garde royal protège le lanceur, la Familia Commune fait du lanceur un enfant. Conséquences :
  - les conditions du lanceur s'appliquent au pion qui agit : posé avant ce tour (mal du débarquement), non immobilisé. Un pion posé ce tour ne peut donc être ni déplacé, ni protégé par ses propres cartes ;
  - « un pion immobilisé ne peut pas être déplacé par son propriétaire » découle de cette règle : ce n'est plus une règle à part ;
  - une carte qui agit sur plusieurs de tes pions (Lièvre Rare et Légendaire, Interchange Légendaire, Familia Légendaire) a **plusieurs lanceurs** : chacun remplit les conditions du lanceur, sans avoir à être collé aux autres.
- **Pas de portée par défaut** : chaque carte dit sa portée (collé au lanceur pour l'Ours, regard pour la Beauté fatale, n'importe où pour Meilleurs amis…). Remplace « contact par défaut » des fiches, qui ne servait à aucune carte.
- **Une carte qui pose agit comme une pose** : sur n'importe quelle case libre (sa tombe pour le Médecin), sans portée. Elle s'ajoute à la pose normale du tour, sans l'utiliser.
- **Fin d'une durée** : une durée de N tours se compte en tours de l'adversaire de celui qui a joué l'effet. L'effet s'éteint **à la fin** du N-ième de ces tours, et le tour pendant lequel l'effet est créé ne compte jamais. C'est la seule lecture où tous les combos des fiches marchent :
  - le Grognement (1 tour) fige la victime pendant son tour suivant ; il est fini à ton tour suivant ;
  - la Beauté fatale Commune (2 tours) tient pendant le tour de la victime, ton tour et son tour suivant. Une seconde Commune à ton tour suivant déclenche donc le charme fatal ;
  - la Sentinelle Rare, jouée pendant le tour de la victime : ce tour ne compte pas.
- **Infirmerie** : même horloge. Un pion tué revient dans la réserve à la fin du 2ᵉ tour de son propriétaire, donc il peut être reposé à son 3ᵉ tour.
- **Protégé** : un pion protégé ne peut pas être **choisi** comme cible d'un kill. Atteint autrement (zone, survol, passage d'une charge), il survit. Le moteur ne propose ainsi jamais un kill sans effet.
- **Recharge** : toutes les cartes ont une recharge d'un tour de base, donc chacune se joue au plus une fois par tour. C'est déjà le réglage par défaut du catalogue.
- **Banque** : 3 points **gardés** d'un tour à l'autre, donc jusqu'à 4 dans un tour avec celui du tour. C'est ce que supposent les Légendaires à 4 points (réponse à la question ouverte du 2026-10-28).

Plan de mesure : les règles actuelles et les règles cibles sont deux configurations, comparées par l'arène. Avec les seuls Ours, les règles cibles sont mesurées dans leur pire cas (aucune carte pour construire) ; remesurées après B2 (règle d'or, 2026-10-29), et à remesurer après B4 (mouvement) puis B5 (invocations). Résultats : voir la feuille de route.

## Questions bloquantes

À trancher par Martin et Théo avant la tâche indiquée. La réponse est notée ici, datée, puis reportée dans le GDD.

| Bloque | Question | Proposition par défaut | Réponse |
|---|---|---|---|
| T4 | Comment finit une partie où plus personne ne peut aligner (réserves vides, attaques inutiles) ? | Match nul après `max_turns` tours (réglable), en attendant l'Effondrement | **2026-10-26** : partie figée → nul ; limite de 200 tours → nul (filet de sécurité). L'anti-stagnation (Effondrement, boost) est une tâche de design à part. |
| T4 | Une configuration valide peut rendre la partie impossible à finir : pose impossible (`place_cost` > `actions_per_turn` + `max_banked_actions`) ou victoire impossible (`pions_per_player` < `align_to_win`). Faut-il la refuser ? | Oui : refuser ces configurations dans `validate` | **2026-10-26** : mixte. On refuse la pose impossible ; on accepte la victoire impossible (tests, invocations). |
| B1 | Le coût d'une attaque est-il payé quand tous les choix sont faits ? Peut-on annuler gratuitement pendant le ciblage ? | Oui et oui | **2026-10-27** : oui. Le coût est payé et la carte part en recharge au dernier choix. L'annulation est gratuite mais se fait dans l'interface (sélection sur une copie de la partie) : le moteur n'a pas d'action « annuler », pour qu'aucun bot ne puisse boucler. Le moteur ne propose jamais un choix qui mène à une impasse. |
| B1 | Un pion tué retourne dans la réserve de son propriétaire, et laisse-t-il une tombe dès maintenant ? | Retour en réserve oui ; tombe ajoutée avec le Médecin | **2026-10-27** : retour en réserve, comme le dit le GDD. Pas de tombe avant B9 (Médecin), quand sa règle sera figée. **2026-10-29** (fiches) : remplacé par l'infirmerie (2 tours) ; les tombes viennent avec le Médecin, en B5 depuis le découpage du 2026-10-30. |
| B2 | « Survivre à un tour adverse » : la même ligne, ou n'importe quelle ligne de 6 ? | N'importe quelle ligne de 6 au début du tour suivant du joueur | **2026-10-28** : n'importe quelle ligne de 6 ou plus au début de son tour gagne. |
| B2 | Si une attaque crée une ligne pour **l'adversaire** (échange, séduction), que se passe-t-il ? | Elle compte : si elle est toujours là au début de son tour, cet adversaire gagne (celui qui a joué la carte avait son tour pour l'éviter ou la casser). Découle de la règle d'or du 2026-10-28, sans exception. | **2026-10-29** : elle compte, sans exception. |
| Statistiques | Une victoire obtenue quand la partie s'arrête (ligne en attente) rend la même raison, « alignement », qu'une victoire en jeu : faut-il les distinguer, pour mesurer combien de parties se jouent ainsi ? | Non pour l'instant | |
| B2 | Si la partie s'arrête (figée ou limite de tours) alors qu'une victoire est en attente, que se passe-t-il ? | La victoire en attente l'emporte | **2026-10-29** : la ligne gagne. On fait comme si les tours continuaient : l'adversaire du dernier joueur est vérifié d'abord, puis le dernier joueur ; sinon, nul. |
| Anti-stagnation | Effondrement, boost d'actions par tour, ou les deux ? Règle de répétition (même position 3 fois) ? Nul ou départage quand la partie s'arrête sans alignement ? | À mesurer avec les bots | **2026-10-28** (en partie) : avec les règles cibles, la limite de tours donne une égalité ; le départage est en attente. |
| Règles cibles | « Banque de 3 » : 3 actions gardées d'un tour à l'autre (donc jusqu'à 4 dans un tour, avec celle du tour), ou 3 au total ? | 3 gardées : c'est ce qui est codé (`max_banked_actions` = 3). Pour 3 au total, régler 2 gardées. | **2026-10-30** : 3 gardées, ce que supposent les Légendaires à 4 points des fiches. |
| Règles cibles | Un pion déplacé par une carte redevient-il « nouveau » (mal du débarquement) ? | Non : seuls la pose et l'invocation | **2026-10-28** : seuls un pion posé ou invoqué ce tour ne peut pas attaquer. |
| Règles cibles | Dans quel ordre se déroule le début du tour ? | Règle d'or, gain du point d'action, recharges | **2026-10-28** : oui. |
| Règles cibles | Un joueur dont tous les pions sont sur le plateau ne peut plus poser : faut-il une règle ? | Non pour l'instant | **2026-10-28** : non, il lui reste ses cartes ; à revoir plus tard. |
| B3 | « Pendant N tours » : les tours de qui ? | N tours du joueur qui subit l'effet | **2026-10-29** (fiches) : les tours de l'adversaire de celui qui a joué l'effet. **2026-10-30** : l'effet s'éteint à la fin du N-ième de ces tours ; le tour de sa création ne compte pas ; l'infirmerie suit la même horloge. |
| B3 | Un pion immobilisé peut-il lancer une attaque ? Être déplacé ou échangé par une attaque ? | Ne peut ni lancer ni bouger de lui-même ; peut subir | **2026-10-29** (fiches) : il ne peut ni lancer, ni être déplacé par son propriétaire ; l'adversaire peut encore le déplacer ou l'échanger. |
| B3 | Une carte qui agit sur « un de tes pions » (Lièvre, Garde royal, Interchange Commune) : lequel peut-on choisir ? N'importe lequel de tes pions, ou le lanceur et ses 8 voisins (la portée par défaut) ? | Le lanceur ou un de tes pions collé à lui : la portée par défaut s'applique, sans exception | **2026-10-30** : le lanceur lui-même, qui doit donc être posé avant ce tour et non immobilisé ; une carte qui agit sur plusieurs de tes pions a plusieurs lanceurs (voir « Règles communes des attaques »). |
| B4 | Le déplacement est-il retiré des actions de base ? | Oui (proposition du GDD) | **2026-10-29** (fiches, Hibou) : oui, il n'y a plus de déplacement de base. |
| B4 | Peut-on sauter par-dessus un allié (Saut du loup) ? | Non | **2026-10-29** (fiches) : oui, et dans le vide aussi. |
| B4 | « Viser » : un pion survolé, traversé par une charge ou pris dans une zone est-il visé (bouclier du garde, Feintaille) ? | Oui : visé = choisi comme cible ou atteint par l'attaque, de quelque façon que ce soit | |
| B4 | La Beauté fatale (regard, 2 cases sans pion entre) et l'Interchange Rare (2 cases en ligne droite) ont presque la même portée : une seule portée « Regard » pour les deux ? | Oui, sans pion entre | **2026-10-30** : non. L'Interchange Rare vise n'importe quel pion adverse à 1 ou 2 cases en ligne droite, **même derrière un autre pion**. Deux portées distinctes. |
| B4 | Saut du loup, sauts enchaînés : le pion survolé meurt-il tout de suite, ou à la fin de tous les sauts ? | Tout de suite | **2026-10-30** : tout de suite ; sa case est libre pour la suite. Un saut fait **toujours 2 cases**, même dans le vide. Le loup peut repasser par une case déjà visitée ; seul interdit : survoler deux fois le même pion. |
| B4 | Lièvre : « avance de 2 pas », exactement 2 ou jusqu'à 2 ? | Jusqu'à 2, sans finir sur sa case de départ | **2026-10-30** : jusqu'à 2 pas (3 pour la Légendaire), et le pion finit sur une autre case que sa case de départ. |
| B4 | Lièvre Rare et Légendaire : plusieurs pions | Des pions différents, l'un après l'autre ; chacun est un lanceur | **2026-10-30** : oui. Le deuxième pion voit la nouvelle place du premier. |
| B4 | Lièvre Rare et Légendaire : « 2 (3) de tes pions » : exactement, ou jusqu'à ? | Jusqu'à : on peut jouer l'attaque après un seul pion, comme pour les pas | |
| B4 | Saut du loup Épique ou Légendaire : le loup peut-il finir sur sa case de départ sans avoir survolé de pion (aller et retour dans le vide) ? Ce n'est pas « sans effet » au sens décidé (deux déplacements), mais rien ne change, pour 3 ou 4 actions. Le Lièvre, lui, ne peut pas finir sur sa case de départ. | Non, comme le Lièvre | |
| B7 | Coup du taureau Rare : le pion adverse ne peut faire qu'une partie de son recul de 2 cases | Écrasé | **2026-10-30** : un pion non protégé est écrasé (infirmerie) s'il ne peut pas reculer des 2 cases. Un pion **protégé** recule aussi loin que possible et n'est jamais tué ; s'il ne peut pas reculer d'au moins une case, la charge est impossible dans cette direction. |
| B7 | Coup du taureau : que choisit le joueur ? | La direction seulement | **2026-10-30** : la direction seulement ; la charge va aussi loin que sa rareté le permet. Le bord ne l'arrête pas : **à toutes les raretés, le taureau creuse le mur** (fiche). Seuls ses propres pions et un pion adverse protégé (Épique, Légendaire) l'arrêtent. |
| B7 | Le mur : jusqu'où le plateau peut-il grandir ? (La fiche propose un plateau 14 × 14, soit 2 rangées de mur autour du 10 × 10.) Au-delà, la charge s'arrête-t-elle ? | Épaisseur du mur réglable (2 par défaut) ; au-delà, la limite est infranchissable et arrête la charge | |
| B7 | Les noms des cases avec le mur : le 10 × 10 garde-t-il « A1 » à « J10 » (les cases creusées ont alors des noms nouveaux), ou le plateau entier, mur compris, est-il nommé de A1 à N14 (le 10 × 10 devient C3 à L12) ? | Le plateau entier est nommé (A1 à N14) : un seul système de noms, sans coordonnées négatives | |
| B7 | Le mur arrête-t-il le regard (Beauté fatale), l'Interchange Rare et le saut du loup (sauter par-dessus du mur vers une case creusée) ? | Oui : le mur n'est pas une case du plateau tant qu'il n'est pas creusé | |
| B5 | Les pions invoqués (Familia, Meilleurs amis, Sentinelle…) viennent-ils de la réserve ? Réserve vide = impossible ? | Oui, ils viennent de la réserve | **2026-10-29** (fiches) : oui ; réserve vide, l'invocation pose ce qu'elle peut. Seul le Médecin ressuscite depuis l'infirmerie. |
| B5 | Un pion séduit qui meurt retourne dans la réserve de quel joueur ? | Son camp actuel | **2026-10-29** (fiches) : la question disparaît. Personne ne change de camp : séduire, c'est tuer le pion adverse puis poser un des siens à sa place. |
| B5 | Qu'est-ce qu'un kill (pour le Massacre divin, la Queue du lézard, la vengeance et les tombes) ? | Tout pion envoyé à l'infirmerie ou au cimetière : séduction, charme fatal, écrasement, tir de la reine, retour d'Interchange, résurrection sur une tombe occupée, vengeance | |
| B5 | Deux tombes sur une même case (un de tes pions meurt où un autre est déjà mort) ? Le Médecin Épique ou Légendaire sur une tombe occupée par **ton** pion ? | La plus récente remplace l'autre ; résurrection impossible sur ton propre pion | |
| B5 | Rejouer une carte dont l'effet est encore en jeu : 2 reines, 2 pactes, 2 piles de jetons Massacre qui comptent pour le même kill ? | À trancher | |
| B5 | Le pas gratuit de la reine (puis le jeton du Lièvre Épique) est-il une attaque ? | Non : un « geste gratuit », sans coût, une fois par tour ; il déclenche quand même le Hibou | |
| B6 | Le retour d'Interchange Commune est-il un effet continu (Attila peut l'annuler) ? La fiche dit « elle ne peut pas faire gagner », or le retour peut compléter une ligne de 5 avant la vérification. | Oui, c'est un effet continu ; le retour qui complète une ligne gagne (l'adversaire a eu un tour pour casser les 5 autres pions) : corriger la phrase de la fiche | |
| B6 | Terre brûlée : « les cartes adverses s'appliquent sans leur partie continue » rend Interchange Commune plus forte (téléportation sans retour) | « Pendant Terre brûlée, l'adversaire ne peut jouer aucune carte qui crée un effet continu » | |
| B6 | Garder le mot-clé Inébranlable, qui ne sert qu'à une carte ? | Le retirer (principe de simplicité) | |
| B6 | Un pion tué par une carte de son propriétaire (le retour d'Interchange Commune tue « tout pion qui occupe » la case) : l'infirmerie se compte-t-elle en tours de son propriétaire, ou de l'adversaire de celui qui a joué la carte ? Aujourd'hui c'est pareil : on ne tue que des pions adverses. | En tours de son propriétaire | **2026-10-30** : en tours de son **propriétaire**, toujours. Le moteur compte aujourd'hui en tours de l'adversaire du tueur, ce qui revient au même tant qu'on ne tue que des pions adverses ; le cas d'un pion tué par une carte de son propriétaire sera codé et testé avec la première carte qui le permet (B6). |
| Statuts | Une attaque « sans effet » n'est jamais proposée : mais une immobilisation qui se superpose à une autre plus longue (Grognement juste après un Rugissement), ou une seconde protection sur un pion déjà protégé, compte-t-elle comme un effet ? Elle ne change rien en jeu. | Oui, elle compte (c'est ce que fait le moteur depuis B3) : « sans effet » veut dire « aucune primitive », pas « rien ne change ». Plus simple à expliquer et à calculer. | **2026-10-30** : oui, elle compte. |
| B6 | Interchange Légendaire : double échange, ou une des autres pistes de la fiche ? | Échange protégé (fait seulement de briques existantes : échanger + protéger) | |
| B7 | Qu'est-ce que le « mur » du Coup du taureau ? Un trou est-il permanent ? | À définir | **2026-10-29** (fiches) : le lanceur creuse le bord, la case creusée devient normale pour toujours (plateau 14 × 14 dont le tour est du mur). Proposé le 2026-10-30 : coder d'abord la charge qui s'arrête au bord (B4), le mur ensuite s'il est gardé (B7). **2026-10-30** : le mur est gardé, à toutes les raretés ; tout le Coup du taureau passe donc en B7, avec le mur. |
| B8 | Quand plusieurs effets réagissent au même événement, dans quel ordre ? | D'abord le joueur qui subit, puis l'autre | |
| B8 | Avec le mal du débarquement, un pion posé à son dernier tour peut-il **réagir** pendant le tour adverse ? (Aujourd'hui, l'ancienneté se compte depuis le début du tour en cours : il n'est déjà plus « nouveau » au tour adverse.) | Oui | |
| B8 | Le Perroquet peut-il copier le tir d'un garde (« une attaque de 1 point ») ? | Non : il ne copie que des cartes | |
| Règles cibles | Le mot-clé « Charge » (ignorer le mal du débarquement), prévu le 2026-10-28, a disparu des fiches : abandonné ? | Oui, abandonné | |
| B9 | Les Légendaires peuvent-elles sortir au draft ? | Non (comme le GDD actuel) | |
| B9 | Comment se déclenche la révélation d'une carte de la cache adverse ? | À définir | |

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
| 2026-10-28 | Économie réglable : poses limitées par tour (et alors gratuites si on veut), banque de départ (la Pièce), mal du débarquement. Par défaut, les règles ne changent pas ; la variante « règles cibles » les utilise. Le tour se termine automatiquement quand il ne reste ni action ni pose. |
| 2026-10-28 | Règles cibles décidées (pose et points d'action séparés, mal du débarquement, règle d'or « une ligne au début de ton tour gagne », la Pièce, limite de tours → égalité). Codée en option le même jour (ligne ci-dessus), sauf la règle d'or et le départage. |
| 2026-10-29 | Les poses restantes sont annoncées par un événement à chaque changement, comme les actions. Aucune règle de jeu modifiée. |
| 2026-10-29 | B2 : la règle d'or, en option (`golden_rule`, activée dans les règles cibles). Une ligne présente au début de son tour gagne, dans tous les cas ; une ligne en attente gagne aussi quand la partie s'arrête (figée ou limite de tours). Par défaut, une ligne faite en posant gagne toujours aussitôt. |
| 2026-10-30 | Fiches attaques validées du 2026-10-29 copiées dans [FICHES_ATTAQUES.md](FICHES_ATTAQUES.md). Règles cibles mises à jour (infirmerie, défaite par manque de pions) ; règles communes des attaques décidées (cibles, le lanceur est le pion qui agit, pas de portée par défaut, une carte qui pose agit comme une pose, fin d'une durée, protégé, recharge, banque). Pas encore codé. |
| 2026-10-30 | B3 : statuts immobilisé et protégé, durées comptées en tours de l'adversaire, infirmerie en option (`infirmary_turns`, 2 dans les règles cibles), nombre de cibles « jusqu'à N » (`Action::FinishAttack`), attaques sans effet jamais proposées ; Ours Commune et Légendaire, Ours Épique « jusqu'à 2 », Beauté fatale Commune et Rare, Garde royal, Attila Commune ; coûts des fiches par défaut. Une partie n'est pas figée tant qu'un effet ou l'infirmerie est en cours. |
| 2026-10-30 | B4 : déplacer et échanger (`PionMoved`, `PionsSwapped`) ; Saut du loup (Commune à Légendaire, sauts enchaînés, kill immédiat), Lièvre Commune, Rare et Légendaire (jusqu'à N pas ; jusqu'à N pions en attendant la réponse à la question ouverte), Interchange Rare (en ligne droite, même derrière un pion) et Épique. Le Coup du taureau passe en B7 avec le mur. |
