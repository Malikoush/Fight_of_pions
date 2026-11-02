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

Une configuration où la victoire semble impossible (moins de pions que l'alignement demandé) est **acceptée** : elle sert aux tests, et les invocations ne la rendent pas jouable (2026-10-30 : un pion invoqué vient de la réserve).

### Tour de jeu (2026-11-02)

Depuis le 2026-11-02, les règles cibles (décidées le 2026-10-28, voir plus bas) sont les **règles actuelles** : c'est la configuration par défaut du moteur (`GameConfig::default()`), du labo et du terminal.

- **Pose** : au plus 1 par tour (`max_placements_per_turn`), facultative et gratuite (`place_cost` = 0) ; elle ne s'accumule pas.
- **Points d'action** : 1 par tour (`actions_per_turn`), y compris le tout premier (`first_turn_actions`) ; jusqu'à 3 gardés d'un tour à l'autre (`max_banked_actions`), donc jusqu'à 4 dans un tour. Ils servent aux cartes. Finir son tour est gratuit.
- **La Pièce** : J2 commence avec 1 point en banque (`starting_banked_actions`).
- **Mal du débarquement** (`summoning_sickness`), **règle d'or** (`golden_rule`) et **infirmerie de 2 tours** (`infirmary_turns`) : activés.
- Le tour se termine automatiquement quand il ne reste ni action, ni pose, ni pas de reine.

### Premières règles (2026-10-26, remplacées le 2026-11-02)

Gardées au cas où, sous le nom `GameConfig::first_rules()` (variante « premières règles » du labo). Les tests de règles du moteur et du labo partent de cette configuration, car leurs scénarios comptent ses actions.

- Le joueur 1 commence avec `first_turn_actions` action (1). Tous les autres tours, y compris le premier du joueur 2, donnent `actions_per_turn` actions (2).
- Poser un pion coûte `place_cost` actions (1), sans limite de poses par tour : les actions sont partagées entre poser et attaquer.
- Le tour se termine automatiquement quand il ne reste plus d'action, ni de pose possible (2026-10-28, voir « Économie réglable »).
- En finissant son tour, on garde jusqu'à `max_banked_actions` actions non utilisées (2). Elles s'ajoutent au tour suivant, soit jusqu'à 4 actions.
- Ni banque de départ, ni mal du débarquement, ni règle d'or, ni infirmerie.

### Économie réglable (2026-10-28)

Ces réglages ont permis de coder les règles cibles à côté des premières règles. Les valeurs entre parenthèses sont celles des premières règles ; les règles actuelles sont données plus haut.

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
- (2026-10-30) on veut jouer l'attaque en cours (`FinishAttack`) avant d'avoir choisi son lanceur, ou sans avoir assez de cibles ;
- (2026-10-31) la reine ne peut pas faire ce pas (`QueenCannotStep`) ou ce tir (`QueenCannotShoot`) : pas de reine ou de garde, déjà fait ce tour, posée ce tour, immobilisée, case ou cible non permise.

### Attaques (2026-10-27)

- Chaque joueur commence avec des attaques (son **arsenal**). En attendant le draft, elles sont fixées par la configuration (`starting_attacks`).
- Le **coût** et la **recharge** de chaque attaque viennent du catalogue de la configuration. Par défaut (2026-10-30), les coûts des fiches : Commune 1 action, Rare 2, Épique 3, Légendaire 4 ; recharge d'un tour pour toutes, c'est-à-dire une fois par tour.
- Lancer une attaque se fait en plusieurs choix : la carte, puis le **pion lanceur** (un de ses pions sur le plateau), puis les **cibles**. Une attaque n'est proposée que si au moins une attaque complète est possible, et chaque choix proposé mène à une attaque complète.
- (2026-10-30) Une carte dit **combien de cibles** choisir : un nombre exact, ou « jusqu'à N ». Une attaque sans cible (Grognement, Garde royal) est jouée dès le choix du lanceur. Une attaque est jouée dès qu'il n'y a plus rien à choisir : nombre maximal de cibles atteint, ou plus aucune cible possible. Avant le maximum, le joueur peut la jouer avec les cibles déjà choisies (`Action::FinishAttack`), si elle en a assez.
- (2026-10-30) Une attaque **sans effet** n'est jamais proposée : un Grognement sans ennemi collé au lanceur, un Attila sans statut adverse à retirer…
- Une attaque commencée va **toujours au bout** ; pendant les choix, aucune autre action n'est possible. L'annulation se fait dans l'interface, avant d'envoyer les choix au moteur.
- Le **coût est payé** et la carte part **en recharge** quand l'attaque est jouée. Elle redevient prête au début du tour de son propriétaire, une fois sa recharge écoulée.
- Un pion **tué** quitte le plateau ; ses effets continus s'arrêtent. Il retourne dans la réserve de son propriétaire, tout de suite ou après l'infirmerie (voir plus bas) ; à l'infirmerie, il laisse sa tombe, et le Médecin peut le ramener sur le plateau (B5b).
- Une attaque qui dépense les dernières actions du tour termine le tour, comme une pose, s'il ne reste aucune pose possible (2026-10-28).

### Statuts et durées (2026-10-30)

Les règles communes des attaques décidées le 2026-10-30 (voir « Règles communes des attaques », plus bas), codées :

- Un **effet continu** est ce qu'une carte laisse en jeu, posé sur un pion qu'il suit par son identité : deux statuts (ci-dessous), et le pacte de Meilleurs amis (sans durée, B5a). Les statuts :
  - **immobilisé** : le pion ne peut pas lancer d'attaque (refus : « aucun lanceur prêt ») ;
  - **protégé** : le pion ne peut pas être choisi comme cible d'un kill ; il peut encore être immobilisé.
- Chaque effet retient le joueur qui l'a créé et la carte qui l'a créé. Un pion est immobilisé tant qu'au moins un effet l'immobilise : deux immobilisations se superposent, sans règle de cumul.
- **Durée** : une durée de N tours se compte en tours de l'adversaire de celui qui a joué l'effet ; le tour pendant lequel l'effet est créé ne compte pas ; l'effet s'arrête **à la fin** du N-ième de ces tours. Le moteur retient le numéro de ce dernier tour (`last_turn`).
- Les effets d'un pion tué s'arrêtent avec lui.
- Chaque début et chaque fin d'effet sont annoncés aux clients.

### Infirmerie (2026-10-30)

- Un pion tué passe `infirmary_turns` tours à l'infirmerie (2 dans les règles actuelles ; 0 dans les premières règles : retour immédiat dans la réserve). La durée suit la même horloge que les effets, mais se compte toujours en tours du **propriétaire** du pion (décidé le 2026-10-30), même tué pendant son propre tour (une vengeance, codé en B5a) : le tour en cours ne compte pas. Il revient dans la réserve de son propriétaire à la fin du dernier de ces tours, et peut être reposé au tour suivant.
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
- (2026-10-30) **Épique** : séduit 1 pion adverse dans le regard. **Légendaire** : séduit 1 pion adverse n'importe où ; le nouveau pion est protégé jusqu'au prochain tour du joueur (1 tour de l'adversaire).
- **Séduction** : le pion visé est tué (c'est un kill), puis un pion de ta réserve est posé à sa place. Ce n'est pas un changement de camp (carnet, 2026-10-30). Impossible si ta réserve est vide ou si la cible est protégée. Le pion posé compte comme posé ce tour et n'utilise pas la pose normale.

### Reine : le Garde royal (2026-10-30)

- **Commune** : le lanceur est protégé pendant 2 tours. Comme toute carte qui agit sur un de tes pions, elle s'applique au lanceur, qui doit donc être posé avant ce tour et non immobilisé. Pas de cible à choisir.
- Un joueur qui l'a dans son arsenal peut toujours agir tant qu'il a un pion sur le plateau (un pion posé ce tour ou immobilisé pourra la lancer plus tard) : une telle partie ne se fige jamais, la limite de tours la termine.
- (2026-10-31) **Rare** : pose une reine. **Épique** : une reine avec 1 garde. **Légendaire** : une reine avec 2 gardes.

### La reine (2026-10-31)

- **Pose** : un pion de ta réserve, sur une case libre, n'importe où, en plus de ta pose normale, avec une couronne. Elle compte pour l'alignement. Impossible si ta réserve est vide ou si tu as déjà une reine : **une seule reine à la fois** (rejouer la carte redevient possible quand elle est morte).
- **Pas gratuit** (`Action::StepQueen`) : 1 case dans les 8 directions vers une case libre, une fois par tour, à n'importe quel moment du tour entre deux actions (pas pendant le choix d'une attaque), sans coût ; ce n'est pas une attaque. Pas le tour de sa pose, ni quand elle est immobilisée. Le tour ne se termine pas tout seul tant que ce pas reste possible. Une ligne faite par ce pas gagne au début du tour suivant de son propriétaire.
- **Gardes** : des jetons sur la reine, qui n'occupent aucune case et ne comptent pas pour l'alignement. Chacun sert une fois :
  - **bouclier** : il sauve la reine de la prochaine attaque qui la **tuerait** (kill, séduction, tir, vengeance…). Le garde est dépensé, la reine survit, et l'attaque ne lui fait plus rien d'autre (une séduction Légendaire ne la protège pas, par exemple) ; les autres pions sont touchés normalement. Une immobilisation, un échange ou un survol sans kill ne consomment pas de garde ;
  - **tir** (`Action::Shoot`) : une attaque, une fois par tour, qui coûte `guard_shot_cost` (1 par défaut) et un garde. Il tue le premier pion rencontré en ligne droite depuis la reine (8 directions, portée illimitée), seulement s'il est adverse et non protégé. Pas le tour de sa pose, ni quand elle est immobilisée. C'est un kill, dont l'assassin est la reine.
- **Mort de la reine** : elle part à l'infirmerie comme un pion normal, sans couronne, et ses gardes disparaissent. Le Médecin la ramène comme un pion normal.
- La couronne et les gardes la suivent si elle est déplacée ou échangée. Attila Commune ne les retire pas (ce ne sont pas des statuts) ; Attila Rare (B6) le fera.

### Attila Commune (2026-10-30)

- **Commune** : sur tout le plateau, retire d'un pion tous les **statuts** placés par l'adversaire (immobilisé, protégé ; pas un pacte), quelle que soit la carte qui les a posés. Le pion peut être à toi (le libérer) ou à l'adversaire (lui ôter sa protection). Un pion qui ne porte que tes propres effets ne peut pas être visé.
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
- **Sauts enchaînés** : chaque saut part de la case d'arrivée du précédent ; on peut s'arrêter avant le maximum, et repasser par une case déjà visitée, y compris sa case de départ (même sans avoir survolé de pion, décidé le 2026-10-30), mais jamais survoler deux fois le même pion.
- Les choix : le lanceur, puis chaque case d'arrivée.

### Lièvre (2026-10-30)

- Un **pas** va vers une case libre collée (8 directions) ; on peut tourner en chemin, jamais passer par-dessus un pion.
- **Commune** : 1 de tes pions avance de jusqu'à 2 pas. **Rare** : jusqu'à 2 pions, de jusqu'à 2 pas chacun. **Légendaire** : jusqu'à 3 pions, de jusqu'à 3 pas chacun.
- Chaque pion qui avance finit sur une autre case que sa case de départ.
- Le premier pion est le lanceur ; les suivants sont des lanceurs aussi (posés avant ce tour, non immobilisés), différents, et bougent l'un après l'autre : le deuxième voit la nouvelle place du premier.
- Les choix : le lanceur, ses pas, puis le pion suivant et ses pas… Un pas se choisit sur une case libre, un pion suivant sur un de tes pions : les deux ne se confondent jamais.
- L'Épique (jetons) viendra avec les jetons (B6).

### Interchange (2026-10-30)

- **Rare** : le lanceur échange sa case avec un pion adverse à 1 ou 2 cases en ligne droite, même derrière un autre pion.
- **Épique** : avec n'importe quel pion adverse.
- Le pion adverse peut être protégé ou immobilisé.
- La Commune (téléportation avec retour) et la Légendaire viendront avec les retours programmés (B6).

### Une carte qui pose (2026-10-30)

- Le pion vient de la réserve du joueur, sur une case libre, n'importe où (« une carte qui pose agit comme une pose »). Il reçoit une nouvelle identité et compte comme posé ce tour (mal du débarquement), mais n'utilise pas la pose normale du tour et ne coûte rien de plus que la carte.
- Une ligne faite par une carte qui pose suit la règle des lignes faites par une carte : elle gagne au début du tour de son propriétaire.

### Kill et vengeance (2026-10-30)

- Est un **kill** tout pion envoyé à l'infirmerie ou au cimetière : par une attaque, par une séduction, par une vengeance. L'**assassin** est le pion qui a lancé l'attaque, même s'il a bougé.
- L'infirmerie se compte en tours du **propriétaire** du pion tué. Un pion tué pendant le tour de son propriétaire (par une vengeance) ne compte pas ce tour-là.

### Tombes et Médecin (2026-10-30)

- **Tombe** : un pion tué qui part à l'infirmerie laisse une tombe sur la case où il est mort. Elle disparaît quand il sort de l'infirmerie (retour dans la réserve, ou résurrection). Une tombe ne bloque pas la case, et reste sur sa case quand un pion y arrive, en part, bouge ou est échangé. Une case porte au plus une tombe : la plus récente remplace l'autre, et le premier pion reste à l'infirmerie sans pouvoir être ressuscité. Sans infirmerie, pas de tombe.
- **Médecin** : il ressuscite un de tes pions de l'infirmerie, sur sa propre tombe (jamais sur une tombe adverse), n'importe où. Le pion revenu reçoit une nouvelle identité, compte comme posé ce tour, n'utilise pas ta pose normale et ne vient pas de ta réserve.
  - **Commune** : 1 pion, sur une tombe libre. **Rare** : jusqu'à 2 pions, sur des tombes libres.
  - **Épique** : 1 pion, même sur une tombe occupée par un pion adverse, qui est tué (un kill, dont l'assassin est le lanceur du Médecin) ; le pion revenu est protégé 2 tours.
  - **Légendaire** : tous tes pions qui peuvent revenir, même sur des tombes occupées par un pion adverse ; il en faut au moins un. Pas de choix.
  - Une tombe occupée par ton propre pion, ou par un pion adverse protégé, ne permet pas de revenir : l'Épique ne peut pas la viser, la Légendaire saute ce pion.
- Le Médecin ne sert qu'avec l'infirmerie (règles actuelles).

### Meilleurs amis (2026-10-30)

- **Épique** (version unique) : pose 2 pions de ta réserve n'importe où, liés par un **pacte**. Impossible avec moins de 2 pions en réserve. Rejouer la carte crée un autre pacte, indépendant.
- **Vengeance** : à la fin d'une attaque, si un seul des deux amis a été tué par cette attaque, l'autre tue l'assassin, même s'il est immobilisé. Si les deux sont morts pendant la même attaque, personne ne se venge. Si l'assassin est protégé ou déjà mort, la vengeance échoue.
- Après la vengeance (réussie ou non), le pacte prend fin : l'ami survivant redevient un pion normal.
- Une vengeance est un kill : si l'assassin était lui-même lié par un pacte, son ami le venge à son tour. La chaîne s'arrête, puisque chaque vengeance met fin à un pacte.
- Le pacte est un effet continu sans durée : il ne compte pas comme « programmé » pour la partie figée. Attila Commune ne le retire pas (ce n'est pas un statut).

### Familia (2026-11-02)

- **Enfant** : un de tes pions, encadré de part et d'autre, en ligne droite (8 directions), par deux membres de sa famille. Il est **protégé** comme avec le Garde royal (jamais choisi pour un kill, il survit au survol ; il peut encore être immobilisé, déplacé ou échangé), sans durée, tant que les **deux mêmes membres** l'encadrent. Attila Commune lui retire cette protection (c'est un statut).
- La protection tombe **pour de bon** dès que ces deux pions ne l'encadrent plus : l'un d'eux est tué, déplacé ou échangé, ou l'enfant lui-même a bougé. Si les pions se remettent en place, l'enfant n'est pas protégé à nouveau. La vérification se fait à la fin des effets d'une attaque, **avant les vengeances** (puis avant chaque vague de vengeances), et à la fin de chaque action (attaque, pas ou tir de la reine). Pendant l'attaque elle-même, la protection est lue au début : un loup qui survole un parent puis l'enfant ne tue pas l'enfant (questions B5d).
- **Le lanceur est l'enfant** : posé avant ce tour, non immobilisé. Un enfant ne le redevient pas tant qu'il l'est, même dans une autre ligne ; il peut servir de parent à une autre famille. Une fois sa protection tombée (ou retirée par Attila), il peut redevenir enfant.
- **Commune** : le lanceur, déjà encadré par deux de tes pions (n'importe lesquels, même posés ce tour ou immobilisés), devient enfant. Le joueur choisit la ligne en cliquant un des deux pions qui encadrent.
- **Rare** : le joueur choisit une case libre collée au lanceur, dont la case opposée porte un de tes pions. Un parent y est posé ; le lanceur devient enfant de ces deux pions.
- **Épique** : le joueur choisit une case libre collée au lanceur, dont la case opposée est libre aussi. Deux parents y sont posés ; le lanceur devient enfant.
- **Légendaire** : le joueur choisit le second enfant, un de tes pions collé au lanceur, qui doit pouvoir agir aussi (posé avant ce tour, non immobilisé) et ne pas être déjà enfant. Les deux parents se posent tout seuls aux deux bouts, qui doivent être libres. Chaque enfant est encadré par un parent et par son frère : tuer un parent ne fait tomber que la protection de l'enfant voisin.
- Les parents viennent de la réserve et comptent comme posés ce tour, sans utiliser la pose normale. La carte pose ses parents ou rien : la Rare demande 1 pion en réserve, l'Épique et la Légendaire 2.
- Une famille ne programme rien : la protection d'un enfant ne compte pas pour la partie figée.
- Attila Rare et Épique (annuler tous les enfants d'une Familia, qui forment un seul effet) viendront avec B6.

### Fin de partie (2026-10-26)

Une partie se termine dans l'un de ces trois cas, vérifiés dans cet ordre :

1. **Alignement** : une ligne d'au moins `align_to_win` pions fait gagner son propriétaire :
   - sans la règle d'or (premières règles), dès qu'elle est faite en posant ;
   - **au début de son tour**, si elle y est encore, dans tous les cas (2026-10-29). C'est ce qui fait gagner avec la règle d'or (`golden_rule`), et ce qui couvre les lignes faites par une carte, y compris une carte de l'adversaire. Ce contrôle passe avant le gain des actions et les recharges : le joueur qui gagne ainsi ne reçoit pas ses actions.
2. **Partie figée** : plus aucun joueur ne pourra jamais agir autrement qu'en finissant son tour, et rien de programmé ne peut changer la situation : aucun effet continu à durée (un pacte, sans durée, ne compte pas), personne à l'infirmerie (2026-10-30). Elle est vérifiée après chaque action, et à la fin d'un tour où un effet s'est arrêté ou un pion est sorti de l'infirmerie. Un joueur peut encore agir s'il peut poser (un pion en réserve et une case libre) ou lancer une attaque qui a une cible, même si elle est en recharge ou qu'il doit économiser pour la payer (2026-10-27). Résultat : **match nul**.
   Une réserve vide ne fige donc plus la partie tant qu'un contact entre ennemis permet d'attaquer. Le risque principal devient la partie sans fin, traité par la limite de tours et l'anti-stagnation.
3. **Limite de tours** : à la fin du tour numéro `max_turns` (200 par défaut). Chaque tour d'un joueur compte : 200 tours font 100 tours par joueur. C'est un filet de sécurité pour les bots ; il n'est pas censé arriver dans une vraie partie. Résultat : **match nul**.

Quand la partie s'arrête ainsi (figée ou limite de tours), une ligne qui attendait le début du tour de son propriétaire **gagne quand même** (2026-10-29) : on fait comme si les tours continuaient, donc l'adversaire du dernier joueur est vérifié d'abord, puis le dernier joueur. Sinon, nul.

Le moteur retient la raison de la fin, en plus du résultat. Le résultat hors alignement est pour l'instant toujours le nul ; un départage pourra le remplacer sans toucher au reste.

## Pas encore implémenté

- Les autres attaques que l'Ours, la Beauté fatale, le Saut du loup, la Reine, Attila Commune, le Médecin, l'Interchange Rare et Épique, le Lièvre Commune, Rare et Légendaire, et Meilleurs amis ; et les réactions.
- La défaite par manque de pions (avec le cimetière, B6).
- Le draft et la cache secrète.
- La mécanique anti-stagnation (Effondrement et/ou boost d'actions), qui fera de vraies fins de partie au lieu de nuls. À trancher par playtests puis bots (voir la feuille de route).

### Règles cibles (décidées le 2026-10-28, par défaut depuis le 2026-11-02)

Codées avec les réglages ci-dessus : 1 pose gratuite par tour, 1 action par tour, 3 actions gardées au plus, la Pièce, le mal du débarquement, la règle d'or (`golden_rule`, 2026-10-29) et l'infirmerie de 2 tours (`infirmary_turns`, 2026-10-30). Ce sont les règles actuelles depuis le 2026-11-02 (voir « Tour de jeu »). Reste à coder : la défaite par manque de pions (B6, avec le cimetière) et le départage, en attente.

Elles remplacent les premières règles (actions partagées entre poser et attaquer). Comparée à une économie « à la pioche » (pions sacrifiés en points d'action, pions tués recrutés, fatigue), écartée : le jeu n'est pas un Connect6, ce sont les cartes (attaque, mouvement, invocation, pose…) qui font avancer le plateau.

- **Pose** : au plus 1 par tour, facultative ; elle ne s'accumule pas et ne coûte aucun point d'action.
- **Points d'action** : 1 par tour, gardés en banque jusqu'à 3 ; le surplus est perdu. Ils servent aux cartes. La banque est visible.
- **Attaques** : ~~le lanceur doit être au contact de sa cible (les 8 cases autour)~~ ; depuis le 2026-10-30, chaque carte dit sa portée, sans portée par défaut. Les cartes gardent leur recharge. L'Ours Rare reste à 2 points au moins : à 1, chaque joueur pourrait annuler la pose adverse à chaque tour (le pion tué revient à son propriétaire, qui le repose).
- **Mal du débarquement** : un pion **posé ou invoqué** ce tour ne peut pas lancer d'attaque. Un pion déplacé garde son ancienneté. ~~Les futures cartes avec le mot-clé « Charge » ignoreront cette règle.~~ Le mot-clé « Charge » n'existe plus dans les fiches du 2026-10-29 (question ouverte plus bas).
- **Pion tué** : ~~il retourne dans la réserve de son propriétaire~~. Depuis les fiches du 2026-10-29 : il passe **2 tours à l'infirmerie**, puis retourne dans la réserve de son propriétaire (pas de vol de pion). L'infirmerie suit l'horloge des durées (voir « Règles communes des attaques »). Codé en B3 (`infirmary_turns` = 2 dans les règles actuelles).
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
| B4 | Lièvre Rare et Légendaire : « 2 (3) de tes pions » : exactement, ou jusqu'à ? | Jusqu'à : on peut jouer l'attaque après un seul pion, comme pour les pas | **2026-10-30** : jusqu'à. |
| B4 | Saut du loup Épique ou Légendaire : le loup peut-il finir sur sa case de départ sans avoir survolé de pion (aller et retour dans le vide) ? Ce n'est pas « sans effet » au sens décidé (deux déplacements), mais rien ne change, pour 3 ou 4 actions. Le Lièvre, lui, ne peut pas finir sur sa case de départ. | Non, comme le Lièvre | **2026-10-30** : oui, le loup peut revenir sur sa case de départ, même sans avoir survolé de pion. |
| B7 | Coup du taureau Rare : le pion adverse ne peut faire qu'une partie de son recul de 2 cases | Écrasé | **2026-10-30** : un pion non protégé est écrasé (infirmerie) s'il ne peut pas reculer des 2 cases. Un pion **protégé** recule aussi loin que possible et n'est jamais tué ; s'il ne peut pas reculer d'au moins une case, la charge est impossible dans cette direction. |
| B7 | Coup du taureau : que choisit le joueur ? | La direction seulement | **2026-10-30** : la direction seulement ; la charge va aussi loin que sa rareté le permet. Le bord ne l'arrête pas : **à toutes les raretés, le taureau creuse le mur** (fiche). Seuls ses propres pions et un pion adverse protégé (Épique, Légendaire) l'arrêtent. |
| B7 | Le mur : jusqu'où le plateau peut-il grandir ? (La fiche propose un plateau 14 × 14, soit 2 rangées de mur autour du 10 × 10.) Au-delà, la charge s'arrête-t-elle ? | Épaisseur du mur réglable (2 par défaut) ; au-delà, la limite est infranchissable et arrête la charge | |
| B7 | Les noms des cases avec le mur : le 10 × 10 garde-t-il « A1 » à « J10 » (les cases creusées ont alors des noms nouveaux), ou le plateau entier, mur compris, est-il nommé de A1 à N14 (le 10 × 10 devient C3 à L12) ? | Le plateau entier est nommé (A1 à N14) : un seul système de noms, sans coordonnées négatives | |
| B7 | Le mur arrête-t-il le regard (Beauté fatale), l'Interchange Rare et le saut du loup (sauter par-dessus du mur vers une case creusée) ? | Oui : le mur n'est pas une case du plateau tant qu'il n'est pas creusé | |
| B5 | Les pions invoqués (Familia, Meilleurs amis, Sentinelle…) viennent-ils de la réserve ? Réserve vide = impossible ? | Oui, ils viennent de la réserve | **2026-10-29** (fiches) : oui ; réserve vide, l'invocation pose ce qu'elle peut. Seul le Médecin ressuscite depuis l'infirmerie. |
| B5 | Un pion séduit qui meurt retourne dans la réserve de quel joueur ? | Son camp actuel | **2026-10-29** (fiches) : la question disparaît. Personne ne change de camp : séduire, c'est tuer le pion adverse puis poser un des siens à sa place. |
| B5 | Qu'est-ce qu'un kill (pour le Massacre divin, la Queue du lézard, la vengeance et les tombes) ? | Tout pion envoyé à l'infirmerie ou au cimetière : séduction, charme fatal, écrasement, tir de la reine, retour d'Interchange, résurrection sur une tombe occupée, vengeance | **2026-10-30** : oui. La séduction reste « tuer puis poser » (fiche), donc un kill : plus simple et déjà équilibrée (3,5) ; un vrai changement de camp pourra être essayé après les playtests. L'**assassin** est le lanceur de l'attaque, même s'il a bougé. |
| B5 | Deux tombes sur une même case (un de tes pions meurt où un autre est déjà mort) ? Le Médecin Épique ou Légendaire sur une tombe occupée par **ton** pion ? | La plus récente remplace l'autre ; résurrection impossible sur ton propre pion | **2026-10-30** : une case porte au plus une tombe, la plus récente remplace l'autre (le premier pion reste à l'infirmerie sans pouvoir être ressuscité). Résurrection impossible sur une tombe occupée par ton propre pion. |
| B5b | Médecin Épique ou Légendaire sur une tombe occupée par un pion adverse **protégé** | L'Épique ne peut pas viser cette tombe ; la Légendaire saute ce pion, qui reste à l'infirmerie | **2026-10-30** : oui. |
| B5b | Médecin Légendaire (« obligatoirement tous ») quand certains pions ne peuvent pas revenir (tombe occupée par ton pion ou par un pion protégé) | Il ressuscite tous ceux qu'il peut ; il en faut au moins un pour jouer la carte | **2026-10-30** : oui. |
| B5b | Une tombe reste-t-elle sur sa case quand un pion bouge ou est échangé ? Sans infirmerie, le Médecin est-il jouable ? | La tombe reste sur sa case ; sans infirmerie (0 tour), pas de tombe, donc pas de Médecin | **2026-10-30** : oui pour les deux : le Médecin ne sert qu'avec l'infirmerie. |
| B5 | Rejouer une carte dont l'effet est encore en jeu : 2 reines, 2 pactes, 2 piles de jetons Massacre qui comptent pour le même kill ? | À trancher | **2026-10-30**, pour les pactes : permis, chaque pacte est indépendant. **2026-10-31**, pour la reine : une seule à la fois. Reste à trancher pour le Massacre divin (B6). |
| B5 | Meilleurs amis : quand la vengeance a-t-elle lieu ? (Un Saut du loup peut tuer un ami au premier saut et l'autre au deuxième.) | À la fin de l'attaque | **2026-10-30** : à la fin de l'attaque. Si un seul des deux amis est mort pendant l'attaque, l'autre tue l'assassin ; si les deux sont morts, personne ne se venge. |
| B5 | Vengeance en chaîne : si l'assassin est lui-même lié par un pacte, son ami le venge-t-il ? | Oui | **2026-10-30** : oui, la vengeance est un kill comme un autre. La chaîne s'arrête forcément : chaque pacte prend fin après sa vengeance. |
| B5 | Meilleurs amis avec un seul pion en réserve (la règle commune dit qu'une invocation « pose ce qu'elle peut ») | Carte impossible | **2026-10-30** : impossible, un pacte demande deux amis. |
| B5 | Le pas gratuit de la reine (puis le jeton du Lièvre Épique) est-il une attaque ? | Non : un « geste gratuit », sans coût, une fois par tour ; il déclenche quand même le Hibou | **2026-10-31** : non, ce n'est pas une attaque. 1 case dans les 8 directions vers une case libre, une fois par tour, à n'importe quel moment du tour, gratuit ; à partir du tour suivant la pose de la reine, jamais si elle est immobilisée. Il déclenchera le Hibou (B8). |
| B5c | Contre quoi le bouclier d'un garde protège-t-il la reine ? | Seulement contre ce qui la tuerait | **2026-10-31** : seulement contre ce qui la tuerait (kill, séduction, tir, vengeance…). Le garde est dépensé, la reine survit, et l'attaque ne lui fait rien d'autre ; les autres pions sont touchés normalement. Une immobilisation, un échange ou un survol sans kill ne consomment pas de garde. |
| B5c | Combien de tirs de garde par tour ? | Un au plus | **2026-10-31** : un tir par tour au plus, comme une carte. Il coûte 1 point (réglable), demande un garde, et tue le premier pion rencontré en ligne droite (8 directions, portée illimitée), seulement s'il est adverse et non protégé. Impossible si la reine a été posée ce tour ou est immobilisée (le tir est une attaque). C'est un kill, dont l'assassin est la reine. |
| B5c | Saut du loup sur une reine gardée : le garde la sauve, elle reste sur sa case. Le loup peut-il la survoler une seconde fois pendant ses sauts enchaînés ? (Aujourd'hui, le chemin rejoué la traite comme morte ; sans conséquence, car un loup ne peut jamais repasser au-dessus d'une case qu'il a survolée.) | Non, comme tout pion : jamais deux fois le même pion | |
| B5c | Plusieurs reines en même temps ? | Une seule à la fois | **2026-10-31** : une seule reine à la fois par joueur ; la Reine Rare, Épique ou Légendaire n'est pas jouable tant que ta reine est en vie. |
| B5d | Familia : « en ligne droite », quelles directions ? | Les 8 directions (4 lignes), comme l'alignement | **2026-11-02** : oui. |
| B5d | Familia Commune : qui choisit la ligne, quand le lanceur est encadré dans plusieurs ? Les pions qui encadrent doivent-ils pouvoir agir ? | Le joueur, par un des deux pions qui encadrent ; n'importe lesquels de tes pions | **2026-11-02** : oui. |
| B5d | Un enfant peut-il le redevenir dans une autre ligne ? | Non : une seule famille à la fois ; il peut servir de parent | **2026-11-02** : oui. |
| B5d | La protection tombée revient-elle si la famille se reforme ? | Non : elle tombe pour de bon | **2026-11-02** : oui. |
| B5d | Familia Épique (ou Légendaire) avec un seul pion en réserve ? | Impossible : la carte pose ses parents ou rien | **2026-11-02** : oui. La Rare demande 1 pion. |
| B5d | Familia Légendaire : comment se joue-t-elle ? | Lanceur, puis second enfant collé ; parents posés tout seuls aux deux bouts libres ; les deux enfants sont des lanceurs | **2026-11-02** : oui. |
| B5d | Familia Rare : comment se choisit le parent ? | Sa case libre, collée au lanceur, en face d'un de tes pions, qui entre dans la famille | **2026-11-02** : oui. |
| B5d | Pendant une même attaque, un enfant dont un parent vient d'être tué (par exemple, un Saut du loup Épique qui survole le parent puis l'enfant) est-il encore protégé ? | Oui : une attaque est calculée sur la situation du début ; la famille se défait à la fin des effets de l'attaque. Codé ainsi, en attendant la réponse. | |
| B5d | Un enfant qui sort de sa famille pendant son attaque (un saut du loup) et tue un ami d'un pacte : est-il encore protégé quand l'autre ami le venge ? | Non : les familles se défont à la fin des effets de l'attaque, avant les vengeances. Codé ainsi, en attendant la réponse. | |
| B6 | Le retour d'Interchange Commune est-il un effet continu (Attila peut l'annuler) ? La fiche dit « elle ne peut pas faire gagner », or le retour peut compléter une ligne de 5 avant la vérification. | Oui, c'est un effet continu ; le retour qui complète une ligne gagne (l'adversaire a eu un tour pour casser les 5 autres pions) : corriger la phrase de la fiche | |
| B6 | Terre brûlée : « les cartes adverses s'appliquent sans leur partie continue » rend Interchange Commune plus forte (téléportation sans retour) | « Pendant Terre brûlée, l'adversaire ne peut jouer aucune carte qui crée un effet continu » | |
| B6 | Garder le mot-clé Inébranlable, qui ne sert qu'à une carte ? | Le retirer (principe de simplicité) | |
| B6 | Un pion tué par une carte de son propriétaire (le retour d'Interchange Commune tue « tout pion qui occupe » la case) : l'infirmerie se compte-t-elle en tours de son propriétaire, ou de l'adversaire de celui qui a joué la carte ? Aujourd'hui c'est pareil : on ne tue que des pions adverses. | En tours de son propriétaire | **2026-10-30** : en tours de son **propriétaire**, toujours. Codé en B5a : un pion tué par la vengeance pendant le tour de son propriétaire compte les tours suivants de son propriétaire (testé). |
| B6 | Un ami tué par une carte de son propre camp (le retour d'Interchange Commune) : son ami venge-t-il sa mort sur son propre lanceur, ou sur lui-même s'il est le lanceur ? Aujourd'hui, on ne tue que des pions adverses, et le moteur tuerait le lanceur allié. | Pas de vengeance si l'assassin est du même camp ; le pacte prend fin quand même | |
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
| 2026-10-30 | B4 : déplacer et échanger (`PionMoved`, `PionsSwapped`) ; Saut du loup (Commune à Légendaire, sauts enchaînés, kill immédiat), Lièvre Commune, Rare et Légendaire (jusqu'à N pas et jusqu'à N pions, confirmé par Martin), Interchange Rare (en ligne droite, même derrière un pion) et Épique. Le Coup du taureau passe en B7 avec le mur. |
| 2026-10-30 | B5a : une carte qui pose ; kill et assassin définis ; l'infirmerie se compte en tours du propriétaire, même tué pendant son tour ; séduction (Beauté fatale Épique et Légendaire = tuer + poser) ; Meilleurs amis (pacte sans durée, vengeance à la fin de l'attaque, en chaîne) ; Attila Commune ne retire que les statuts. |
| 2026-10-30 | B5b : tombes (une par case, la plus récente remplace l'autre, elles restent sur leur case quand les pions bougent) ; le Médecin (Commune à Légendaire) ressuscite depuis l'infirmerie, sur sa propre tombe. Sans infirmerie, ni tombe ni Médecin. |
| 2026-10-31 | B5c : la reine (Reine Rare à Légendaire) : pose n'importe où, une seule à la fois ; pas gratuit une fois par tour (le tour l'attend) ; gardes qui la sauvent d'un kill ou tirent (un tir par tour, `guard_shot_cost`). |
