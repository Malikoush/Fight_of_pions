# Fiches attaques validées

Fiches de Martin du 29 octobre 2026, copiées telles quelles (mise en forme seulement). Elles remplacent le GDD v0.2 pour les attaques et les réactions.

Ce sont des **fiches de design** : le moteur n'en code une règle qu'une fois écrite dans le [carnet de règles](REGLES.md), qui prime en cas de désaccord. Les précisions et corrections décidées ensuite y sont datées (à partir du 2026-10-30).

## Règles de base

La base retenue sépare les pions et les points d'action, et un pion tué passe 2 tours à l'infirmerie avant de revenir. Toutes les fiches ci-dessous sont équilibrées pour ces règles.

| Règle | Valeur retenue |
|---|---|
| Plateau | 10 × 10, aligner 6 pions |
| Pions | 15 par joueur |
| Pose | 1 par tour, facultative, non cumulable |
| Points d'action | 1 par tour |
| Banque | 3 points maximum, sert aussi à payer les réactions |
| Pion tué | 2 tours à l'infirmerie, puis retour dans la réserve de son propriétaire |
| Coût d'un kill | 2 points |
| Lancer une attaque | Depuis un pion posé avant ce tour (mal du débarquement), au contact par défaut (retiré le 2026-10-30 : chaque carte dit sa portée, voir le carnet) |
| Règle d'or | Toute ligne de 6, faite par pose ou par attaque, doit survivre au tour adverse |
| Départ | J2 commence avec 1 point en banque (la Pièce) |
| Réserve vide | On saute la pose, on garde son point d'action |
| Fin de partie | Alignement, ou adversaire réduit à moins de 6 pions au total (plateau, réserve et infirmerie) ; limite de tours de sécurité avec départage |

## Règles communes des attaques

Ces règles s'appliquent à toutes les cartes, sauf si la carte dit le contraire.

**Principe de simplicité** : chaque carte doit rester simple à expliquer. Entre deux versions équilibrées, on garde la plus simple, même si elle est un peu moins spectaculaire.

| Règle | Proposition | Statut |
|---|---|---|
| Pions invoqués | Ils viennent de ta réserve et comptent comme posés ce tour ; réserve vide = l'invocation pose ce qu'elle peut. Seul le Médecin ressuscite depuis l'infirmerie | Validée |
| Lanceur | Un de tes pions posé avant ce tour, non immobilisé | Validée |
| Portée | Contact par défaut (les 8 cases autour du lanceur), sauf si la carte précise une autre portée (regard à 2 cases, 2 cases en ligne, n'importe où…) | Validée, puis remplacée le 2026-10-30 : pas de portée par défaut, chaque carte dit la sienne (voir le carnet) |
| Cibles | Les attaques ne visent que les pions adverses | Validée (précisée le 2026-10-30, voir le carnet) |
| Statuts | Immobilisé : ne peut ni lancer d'attaque, ni être déplacé par son propriétaire (l'adversaire peut encore le déplacer). Protégé : ne peut pas être tué, mais peut être déplacé ou échangé | Validée |
| Déplacement | Tout mouvement d'un pion : saut, pas, poussée, charge, échange, téléportation, déplacement de la reine | Validée |
| Effet continu | Tout ce qu'une carte laisse en jeu après avoir été jouée : statuts (immobilisé, protégé, enfants), gardes de la Reine, jetons (Massacre divin, Lièvre Épique), pacte de Meilleurs amis. Les pions, l'infirmerie, les tombes et les cases creusées ne sont pas des effets | Validée |
| Kill en Commune | Pas de kill sans condition en Commune (exemples : charme fatal de la Beauté fatale, retour de la téléportation d'Interchange) | Validée |
| Cartes de zone | Elles ne tuent pas tes propres pions | Validée |
| Durées | Une durée de N tours se compte en tours de l'adversaire de celui qui a joué l'effet (immobilisé, protégé, blessure) | Validée |
| Inébranlable | Mot-clé : un effet Inébranlable ne peut être annulé que par Terre brûlée (Attila Légendaire). Pour l'instant, seul Massacre divin Légendaire l'a | Validée |
| Ligne formée par une réaction | Une ligne de 6 complétée pendant le tour adverse (Sentinelle, Queue du lézard, Hibou…) doit survivre au tour adverse suivant, complet, pour gagner | Validée |

## Échelle de valeur

Une variante est juste si sa valeur est à 0,5 près de son coût, avec une prime de +0,5 pour une Épique et +1 pour une Légendaire. Chaque rareté ajoute environ 1 point de valeur, pas un effet entier.

| Effet | Valeur (points) |
|---|---|
| Poser un pion en plus | 1 |
| Déplacer un pion d'1 ou 2 cases | 1 |
| Déplacer un pion n'importe où | 1,5 |
| Échanger deux pions | 2 à 2,5 |
| Immobiliser 1 tour | 0,5 |
| Protéger 1 tour | 0,5 |
| Tuer | 2 |
| Gagner 1 point | 1 |

## Suivi des fiches

| # | Carte | Axes | Difficulté | Statut |
|---|---|---|---|---|
| 01 | Beauté fatale | Crowd control, Invocation | 1 variante à changer (L) | Validée |
| 02 | Saut du loup | Crowd control, Mouvement, Kill | Réglable par chiffres | Validée |
| 03 | Reine | Spécial | À refondre | Validée |
| 04 | Attila | Crowd control, Spécial | À refondre (nouvelle version proposée) | Validée |
| 05 | Médecin | Spécial, Invocation | À refondre | Validée |
| 06 | Interchange | Mouvement | Réglable par chiffres | Validée |
| 07 | Coup du taureau | Spécial, Kill | À refondre | Validée |
| 08 | Massacre divin | Buff | Réglable par chiffres | Validée |
| 09 | Ours | Crowd control, Kill | Réglable par chiffres | Validée |
| 10 | Lièvre | Mouvement | Réglable par chiffres | Validée |
| 11 | Familia | Invocation, Spécial | Réglable par chiffres | Validée |
| 12 | Meilleurs amis | Kill, Invocation | Réglable par chiffres | Validée |
| R1 | Feintaille | Réaction | Réglable par chiffres | Validée |
| R2 | Sentinelle | Réaction | 1 variante à changer (R) | Validée |
| R3 | Queue du lézard | Réaction | Réglable par chiffres | Validée |
| R4 | Perroquet | Réaction | Réglable par chiffres | Validée |
| R5 | Hibou | Réaction | Réglable par chiffres | Validée |

## Les attaques

### 01 · Beauté fatale

Crowd control, Invocation. Le charme fige les pions adverses, puis les fait changer de camp.

**Règles de la carte**

- **Regard** : la carte vise un pion adverse en ligne droite (horizontale, verticale ou diagonale), jusqu'à 2 cases, sans pion entre les deux. Seule la Légendaire agit n'importe où.
- **Charme fatal** : si la Commune ou la Rare vise un pion déjà immobilisé (par n'importe quelle carte), il part à l'infirmerie. Un pion protégé est seulement immobilisé à nouveau.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Immobilise 1 pion adverse pendant 2 tours | 1,5 |
| Rare | 2 | Immobilise 2 pions adverses pendant 2 tours | 2,5 |
| Épique | 3 | Séduit 1 pion adverse : il part à l'infirmerie, tu poses un de tes pions à sa place | 3,5 |
| Légendaire | 4 | Séduit 1 pion adverse n'importe où sur le plateau ; ton nouveau pion est protégé jusqu'à ton prochain tour | 4,5 à 5 |

**Précisions**

- Un pion immobilisé ne peut ni lancer d'attaque ni être déplacé par son propriétaire pendant ses 2 prochains tours. L'adversaire, lui, peut encore le déplacer.
- La séduction est impossible si ta réserve est vide ou si la cible est protégée. Le pion posé s'ajoute à ta pose normale et compte comme posé ce tour (il ne peut pas attaquer tout de suite).
- Combos repérés : Commune puis Commune au tour suivant (un kill pour 2 points en 2 tours) ; Saut du loup Commune puis Beauté fatale Commune dans le même tour (un kill et un déplacement pour 2 points).
- Changements par rapport au GDD : Commune 1 → 2 tours ; Rare « attire » remplacée par « immobilise 2 pions » ; Épique précisée (remplacement) ; Légendaire « séduit 2 pions adjacents » remplacée par « séduit 1 pion n'importe où, protégé jusqu'à ton prochain tour » ; regard et charme fatal ajoutés.

### 02 · Saut du loup

Crowd control, Mouvement, Kill. Un saut comme aux dames : le lanceur bondit de 2 cases et frappe le pion adverse qu'il survole.

**Règles de la carte**

- **Saut** : le lanceur bondit de 2 cases en ligne droite (8 directions, diagonales comprises) et atterrit sur une case libre. Il passe par-dessus n'importe quelle case : vide, un de tes pions ou un pion adverse.
- **Sauts enchaînés** : chaque saut part de la case d'arrivée du précédent. On peut s'arrêter avant le maximum, et un même pion ne peut pas être survolé deux fois.
- **Saut meurtrier** : le pion adverse survolé part à l'infirmerie. Un pion protégé peut être survolé, mais il n'est pas tué.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | 1 saut ; un pion adverse survolé est immobilisé 2 tours | 1 à 1,5 |
| Rare | 2 | 1 saut meurtrier | 2,5 |
| Épique | 3 | Jusqu'à 2 sauts meurtriers | environ 4 |
| Légendaire | 4 | Jusqu'à 3 sauts meurtriers | 5,5 à 6 |

**Précisions**

- À surveiller en test : l'Épique et la Légendaire sont à la limite haute, freinées par la difficulté à enchaîner les sauts. Si elles dominent, on interdit les sauts dans le vide ou on limite le nombre de kills par saut (exemple : 3 sauts dont 2 kills).
- Si un saut complète une de tes lignes, la règle d'or s'applique.
- Combo repéré : Saut du loup Commune puis Beauté fatale Commune dans le même tour (un kill et un déplacement pour 2 points).
- Sauts dans le vide et par-dessus ses propres pions autorisés ; immobilisation fixée à 2 tours.

### 03 · Reine

Spécial. Une reine posée en plus, qui se déplace seule et que ses gardes protègent.

**Règles de la carte**

- **Reine** : un pion pris dans ta réserve, posé sur une case libre en plus de ta pose normale, avec un jeton couronne. Elle compte pour l'alignement.
- **Déplacement gratuit** : 1 case par tour dans les 8 directions, à partir du tour suivant sa pose. Une reine immobilisée ne bouge pas.
- **Gardes** : des jetons posés sur la reine, qui n'occupent aucune case. Ce ne sont pas des pions : ils ne viennent pas de ta réserve et ne comptent pas pour l'alignement. Chaque garde sert une seule fois : en **bouclier**, il annule automatiquement la prochaine attaque qui vise la reine ; en **tir**, pour 1 point, il part en ligne droite depuis la reine (portée illimitée) et tue le premier pion rencontré, s'il est adverse. Le garde est retiré et la case du pion tué reste libre.
- **Mort de la reine** : elle part à l'infirmerie comme un pion normal, sans sa couronne, et ses gardes disparaissent.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Garde royal : un de tes pions est protégé pendant 2 tours | 1 |
| Rare | 2 | Pose une reine | 2 à 2,5 |
| Épique | 3 | Pose une reine avec 1 garde | 3 à 3,5 |
| Légendaire | 4 | Pose une reine avec 2 gardes | 4 à 4,5 |

**Précisions**

- Impossible de poser une reine si ta réserve est vide.
- Le tir est une attaque : une reine posée ce tour ne peut pas tirer, et on ne peut pas tirer si le premier pion rencontré est à toi. Un pion protégé touché par un tir n'est pas tué.
- Si le déplacement de la reine complète ta ligne, la règle d'or s'applique.
- Contres : la Beauté fatale peut séduire la reine (sauf si un garde la protège), et Attila peut lui retirer tous ses gardes d'un coup.
- À surveiller en test : la reine en fin de partie, seul pion qui bouge gratuitement quand les réserves sont vides.
- Changements par rapport au GDD : les gardes deviennent des jetons à usage unique (bouclier ou tir pour 1 point), sans gravitation ni rappel ; invocation et déplacement gratuit conservés.

### 04 · Attila

Crowd control, Spécial. Le contre des effets en cours, sur tout le plateau. Là où passe Attila, l'herbe ne repousse pas.

**Règles de la carte**

- **Portée** : tout le plateau.
- **Effets visés** : les effets continus placés par l'adversaire, toujours annulés en entier. Un effet, c'est tout ce qu'une carte a laissé en jeu : une immobilisation ou une protection, toutes les immobilisations d'un Rugissement, tous les gardes restants d'une Reine, tous les jetons d'un Massacre divin, les déplacements en cours du Lièvre, le pacte de Meilleurs amis. Seule Terre brûlée peut annuler un effet Inébranlable.
- **Non touchés** : l'infirmerie, les tombes, les trous du Coup du taureau.
- **Instantanée** : elle efface les effets au moment où elle est jouée. Seule la Légendaire empêche d'en créer de nouveaux.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Retire les statuts placés par l'adversaire sur 1 pion (immobilisé, protégé), quelle que soit la carte qui les a posés | 1 |
| Rare | 2 | Annule 1 effet adverse entier, sauf un effet Inébranlable | 2 |
| Épique | 3 | Annule 2 effets adverses entiers, sauf les effets Inébranlables | 3 |
| Légendaire | 4 | Terre brûlée : annule 2 effets adverses entiers, même Inébranlables ; jusqu'à ton prochain tour, l'adversaire ne peut créer aucun nouvel effet continu | 4 à 4,5 |

**Précisions**

- Pendant Terre brûlée, les cartes adverses s'appliquent sans leur partie continue : un Saut du loup Commune saute sans immobiliser, une Reine arrive sans gardes, un Garde royal ne protège rien.
- Combo repéré : Commune puis Ours Rare (3 points) pour tuer un pion protégé.
- Coup typique de la Légendaire : jouer Terre brûlée pour que l'adversaire ne puisse ni protéger ses pions ni immobiliser les tiens pendant la fenêtre et facilement aligner ensuite.
- Changements par rapport au GDD : la zone 5 × 5 devient tout le plateau, avec un nombre d'effets annulés par rareté ; Légendaire Terre brûlée ajoutée.

### 05 · Médecin

Spécial, Invocation. Le Médecin ramène tes pions tombés, là où ils sont tombés.

**Règles de la carte**

- **Tombe** : quand un de tes pions est tué, il laisse une tombe sur sa case. Elle disparaît quand ce pion quitte l'infirmerie. Une tombe ne bloque pas la case.
- **Résurrection** : un pion de ton infirmerie revient sur sa propre tombe, jamais sur une tombe adverse. Cette pose s'ajoute à ta pose normale ; le pion redevient un pion normal et compte comme posé ce tour.
- **Tombe occupée** : impossible d'y ressusciter, sauf avec l'Épique et la Légendaire. Dans ce cas, le pion adverse qui s'y trouve est tué et ton pion prend sa place.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Ressuscite 1 pion sur sa tombe, si elle est libre | 1 à 1,5 |
| Rare | 2 | Ressuscite jusqu'à 2 pions, sur des tombes libres | 1,5 à 2,5 |
| Épique | 3 | Ressuscite 1 pion, même sur une tombe occupée ; il est protégé pendant 2 tours | 2 à 4 |
| Légendaire | 4 | Ressuscite tous tes pions de l'infirmerie, même sur des tombes occupées | 4 à 6 |

**Précisions**

- Avec la Commune, la Rare et l'Épique, tu choisis les tombes. La Légendaire ressuscite obligatoirement tous tes pions.
- C'est une carte de réponse : elle ne sert à rien tant que ton infirmerie est vide.
- Dilemme pour l'attaquant : après un kill, poser sur la case libérée empêche la résurrection, mais lui coûte sa pose du tour.
- La protection de l'Épique couvre la fenêtre de la règle d'or : un pion ressuscité pour réparer ta ligne ne peut pas être retué tout de suite.
- Changements par rapport au GDD : les zombies deviennent des pions normaux ; la résurrection se fait depuis l'infirmerie ; le sacrifice est retiré ; la tombe dure aussi longtemps que la blessure ; la puissance monte par l'effet plutôt que par le nombre.

### 06 · Interchange

Mouvement. La carte de l'échange : prendre la place d'un pion adverse en lui laissant la tienne.

**Règles de la carte**

- **Échange** : ton pion et le pion adverse visé échangent leurs cases. Ça marche sur un pion protégé, puisque protégé veut dire « ne peut pas être tué », pas « ne peut pas bouger ».
- **Immobilisé** : tu ne peux pas déplacer un de tes pions immobilisés, mais tu peux échanger un pion adverse immobilisé.
- Si un échange ou une téléportation complète ta ligne, la règle d'or s'applique.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Téléporte un de tes pions sur une case vide ; au début de ton tour suivant, il revient à sa case d'origine, et tout pion qui l'occupe part à l'infirmerie | 1 à 1,5 |
| Rare | 2 | Échange un de tes pions avec un pion adverse à 2 cases maximum, en ligne droite | 2 |
| Épique | 3 | Échange un de tes pions avec n'importe quel pion adverse | 3 à 3,5 |
| Légendaire | 4 | 2 de tes pions échangent leur place avec 2 pions adverses collés entre eux, n'importe où | 4 à 4,5 |

**Précisions**

- Commune : le retour se fait avant la vérification de victoire, donc elle ne peut pas faire gagner. Elle bloque une ligne adverse pendant un tour, n'importe où, et sa case de retour est « réservée », quel que soit le camp du pion qui s'y installe.
- Légendaire : tes deux pions arrivent au milieu de la formation adverse, entourés de ses pions, donc faciles à attaquer. C'est surtout une carte de défense, qui peut faire gagner si ta ligne passe par ces deux cases.
- À surveiller en test : l'Épique, qui est la façon la moins chère de compléter une ligne n'importe où.
- Changements par rapport au GDD : la Rare passe d'un pion voisin à 2 cases de portée ; la Légendaire passe de « 2 échanges n'importe où » à un double échange sur 2 pions adverses collés ; le moment du retour de la Commune est précisé.

Autres pistes pour la Légendaire, si le double échange ne convainc pas en test :

- **Téléportation de groupe** : téléporte jusqu'à 3 de tes pions sur des cases vides, n'importe où (environ 4,5). Pur repositionnement, peut faire gagner.
- **Échange protégé** : échange un de tes pions avec n'importe quel pion adverse, et ton pion est protégé 2 tours (4 à 4,5). Proche de la Légendaire de la Beauté fatale.
- **Échange piégé** : échange n'importe où ; au début de ton tour suivant, ton pion revient et le pion adverse qui avait pris sa place part à l'infirmerie (environ 3,5). Un peu juste pour une Légendaire.
- **Supprimer la Légendaire**, en suivant le principe du GDD « pas forcément 4 raretés » : l'Épique devient le sommet de la carte.

### 07 · Coup du taureau

Spécial, Kill. Une charge en ligne droite, assez puissante pour percer le mur du plateau et l'agrandir.

**Règles de la carte**

- **Charge** : le lanceur fonce en ligne droite (8 directions) sur un nombre de cases donné, et termine sa course sur la dernière case atteinte. Sans pion adverse sur son chemin, il se déplace simplement.
- **Le mur** : si le lanceur atteint le bord avant la fin de sa course, il creuse le mur et continue dans la case creusée, qui devient une case normale pour toujours. Le mur ne cède que sous le lanceur : un pion poussé contre le bord est bloqué.
- **Obstacles** : la charge s'arrête devant tes propres pions. Un pion adverse protégé peut être poussé (Commune, Rare) s'il a la place de reculer ; sinon, et toujours en Épique et Légendaire, il arrête la charge.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Charge d'1 case : le pion adverse qui s'y trouve recule d'1 case et le lanceur prend sa place ; s'il ne peut pas reculer, la charge est impossible | 1,25 |
| Rare | 2 | Charge d'1 case : le pion adverse recule de 2 cases et le lanceur prend sa place ; s'il est bloqué avant, il est écrasé et part à l'infirmerie | 2 à 2,25 |
| Épique | 3 | Charge de 2 cases : chaque pion adverse sur le passage part à l'infirmerie | 3 à 3,5 |
| Légendaire | 4 | Charge de 3 cases : chaque pion adverse sur le passage part à l'infirmerie | 4,5 à 5 |

**Précisions**

- Charger dans l'axe d'une ligne adverse, depuis son bout, écrase ou tue ses pions ; charger de côté les pousse hors de la ligne.
- Le lanceur quitte sa case : il peut défaire une de tes lignes, et il finit souvent exposé au milieu des pions adverses.
- Dans le code : un plateau plus grand que 10 × 10 (par exemple 14 × 14), dont les cases extérieures sont du mur ; creuser transforme une case de mur en case normale.
- À surveiller en test : la Légendaire, qui peut tuer 3 pions d'une ligne adverse d'un coup.
- La poussée devient le rôle du taureau : elle sera retirée de l'Ours, dont la Commune aura besoin d'un nouvel effet.
- Changements par rapport au GDD : la Commune et la Rare deviennent des charges qui poussent ou écrasent ; la charge creuse le mur à toutes les raretés ; la récompense trouvée dans le mur (pion ou point d'action) est retirée pour l'instant.

### 08 · Massacre divin

Buff. Tes kills rapportent : d'abord des points d'action, puis des morts qui ne reviennent jamais.

**Règles de la carte**

- **Prochains kills** : l'effet s'applique à tes prochains kills, tous tours confondus, jusqu'à épuisement. Sur un vrai plateau, on le suit avec des jetons posés sur la carte.
- **Kill** : n'importe quel pion adverse que tu envoies à l'infirmerie, avec n'importe quelle carte.
- **Points rendus** : utilisables tout de suite. En fin de tour, ta banque redescend à son maximum (3).
- **Cimetière** : un pion au cimetière ne revient jamais. Il ne laisse pas de tombe, donc le Médecin ne peut pas le ressusciter.
- **Défaite** : un joueur qui a moins de 6 pions au total (plateau, réserve et infirmerie) ne peut plus aligner, donc il a perdu.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Ton prochain kill te rend 2 points | 1 |
| Rare | 2 | Tes 4 prochains kills te rendent 1 point chacun | 2 |
| Épique | 3 | Tes 2 prochains kills envoient le pion au cimetière au lieu de l'infirmerie | 3 |
| Légendaire | 4 | Tes 3 prochains kills envoient le pion au cimetière au lieu de l'infirmerie. Inébranlable | 4,5 |

**Précisions**

- La Rare ne rend qu'1 point par kill, pour qu'un kill ne devienne jamais gratuit : sinon, elle permettrait trois kills dans le même tour.
- La carte reste posée devant toi, face visible, avec ses jetons, tant qu'il en reste. Pour Attila, tous ses jetons forment un seul effet. La Légendaire est Inébranlable : seule Terre brûlée peut l'annuler.
- À surveiller en test : la carte récompense celui qui tue déjà, donc elle peut accélérer l'effet boule de neige.
- Changements par rapport au GDD : les durées deviennent des « prochains kills » ; la Commune rend 2 points ; la Rare rend 1 point sur 4 kills ; l'Épique et la Légendaire envoient au cimetière ; l'ancienne Épique sur la banque est retirée.

### 09 · Ours

Crowd control, Kill. Le corps à corps : l'ours grogne, frappe et rugit sur les 8 cases autour de lui.

**Règles de la carte**

- **Portée** : uniquement les pions collés au lanceur, sur les 8 cases autour de lui.
- Un pion protégé ne peut pas être tué.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Grognement : les pions adverses collés au lanceur sont immobilisés pendant 1 tour | 1 |
| Rare | 2 | Tue 1 pion adverse collé au lanceur | 2 |
| Épique | 3 | Tue jusqu'à 2 pions adverses collés au lanceur | 3,5 |
| Légendaire | 4 | Rugissement : tue jusqu'à 2 pions adverses collés au lanceur, et les autres pions adverses collés à lui sont immobilisés pendant 2 tours | 4 à 5 |

**Précisions**

- La Rare est l'étalon du jeu : elle fixe la valeur d'un kill à 2 points.
- Le Grognement est la seule immobilisation d'1 tour : il fige toute la zone, donc il doit être plus court.
- L'Épique tue « jusqu'à 2 » voisins, pas exactement 2.
- La Légendaire brille en pleine mêlée : avec seulement 2 voisins adverses, elle ne fait guère mieux que l'Épique.
- Combo repéré : Grognement ou Rugissement, puis Beauté fatale Commune pour achever un pion figé (dans le même tour pour le Grognement).
- Changements par rapport au GDD : la poussée est retirée et passe au Coup du taureau ; la Commune devient le Grognement ; la Légendaire « tue 3 et repousse les autres » devient le Rugissement.

### 10 · Lièvre

Mouvement. La carte de mouvement pur : déplacer tes pions vite, dans toutes les directions.

**Règles de la carte**

- **Pas** : le pion se déplace d'1 case vers une case libre voisine, dans les 8 directions. Avancer de plusieurs pas permet de tourner en chemin (les « L » du GDD).
- **Chemin libre** : chaque pas doit arriver sur une case libre. Le Lièvre ne passe pas par-dessus les pions, contrairement au Saut du loup.
- Un pion immobilisé ne peut pas bouger, et si un déplacement complète ta ligne, la règle d'or s'applique.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | 1 de tes pions avance de 2 pas | 1 |
| Rare | 2 | 2 de tes pions avancent de 2 pas chacun | 2 |
| Épique | 3 | 3 jetons sur 1 de tes pions : à chacun de tes tours, il peut en dépenser 1 pour avancer de 2 pas | 2,5 à 3 |
| Légendaire | 4 | 3 de tes pions avancent de 3 pas chacun, en une seule fois | 4 |

**Précisions**

- Épique : un seul jeton par tour. Ses jetons forment un seul effet pour Attila, et si le pion est immobilisé, ils attendent qu'il soit libéré.
- Légendaire : jouée en une seule fois, elle ne laisse aucun effet en jeu, donc Attila ne peut rien contre elle.
- Le Lièvre devient la carte clé quand ta réserve est vide : déplacer est alors la seule façon de construire.
- Changements par rapport au GDD : l'Épique passe de « gratuitement pendant 3 tours » à 3 jetons ; la Légendaire passe de 2 tours à une seule fois.

### 11 · Familia

Invocation, Spécial. Des familles de pions en ligne, où les enfants encadrés par leurs parents sont protégés.

**Règles de la carte**

- **Parents** : des pions pris dans ta réserve, posés en ligne droite de part et d'autre d'un ou plusieurs de tes pions. Ils s'ajoutent à ta pose normale et comptent comme posés ce tour.
- **Enfant** : un de tes pions encadré en ligne droite par deux membres de sa famille (ses parents ou son frère). Il est protégé tant qu'il reste encadré des deux côtés par sa famille, dans la direction de la famille ; tes autres pions ne comptent pas. Si un membre voisin est tué ou déplacé, la protection tombe. Avec la Commune, les deux pions qui encadrent l'enfant deviennent sa famille.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Un de tes pions déjà encadré par deux des tiens devient enfant | 1 à 1,5 |
| Rare | 2 | Pose 1 parent pour finir d'encadrer un de tes pions, qui devient enfant | 2 |
| Épique | 3 | Pose 2 parents de part et d'autre d'un de tes pions, qui devient enfant | 3,5 |
| Légendaire | 4 | Pose 2 parents de part et d'autre de 2 de tes pions collés en ligne : les 2 du milieu deviennent enfants | 4,5 |

**Précisions**

- Impossible de poser des parents si ta réserve est vide.
- Contres : tuer un parent fait tomber la protection de l'enfant voisin (dans une famille de 4, l'autre enfant reste encadré par son frère et l'autre parent, jusqu'à ce que l'un d'eux tombe) ; Attila Commune retire la protection d'un enfant ; Attila Rare ou Épique annule d'un coup tous les enfants d'une Familia, puisqu'ils forment un seul effet ; un enfant peut toujours être déplacé ou échangé.
- Attila ne touche jamais les parents, qui sont des pions normaux.
- Changements par rapport au GDD : aucune variante ne pose plus de 2 pions ; la Rare finit d'encadrer un pion au lieu de poser 2 parents sans enfant ; l'Épique « 3 pions alignés » et la Légendaire « famille de 4 avec 2 enfants » sont réduites pour ne pas recréer les gros tours de poses.

### 12 · Meilleurs amis

Kill, Invocation. Deux pions liés par un pacte : si l'un est tué, l'autre le venge.

**Règles de la carte**

- **Amis** : 2 pions pris dans ta réserve, posés n'importe où, en plus de ta pose normale. Ils comptent comme posés ce tour.
- **Vengeance** : si un des amis est tué, l'autre tue aussitôt l'assassin, c'est-à-dire le pion qui a lancé l'attaque. Elle est automatique : ce n'est pas une réaction, et elle a lieu même si l'ami survivant est immobilisé.
- **Fin du pacte** : après la vengeance, le survivant redevient un pion normal.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Épique | 3 | Pose 2 pions n'importe où, liés par le pacte | 3,5 |

**Précisions**

- Si les deux amis meurent en même temps, personne ne se venge. Si l'assassin est protégé, la vengeance échoue.
- Le pacte est un effet d'Épique : Attila Rare ou Épique peut l'annuler.
- À surveiller en test : 2 poses n'importe où peuvent transformer une ligne de 4 en ligne de 6 d'un coup.
- Version unique, comme prévu dans le GDD (« pas forcément 4 raretés »).
- Changements par rapport au GDD : le cas « kill qui ne vient pas d'un pion » est retiré, puisque toutes les attaques partent d'un pion.

## Les réactions

Les réactions se jouent pendant le tour adverse, payées avec la banque, une seule par tour.

### R1 · Feintaille

Réaction. Déclencheur : une attaque adverse vise un de tes pions.

**Règles de la carte**

- **Esquive** : juste avant que l'attaque ne s'applique, ton pion visé se déplace d'1 case vers une case libre voisine. L'attaque ne le touche pas ; si elle visait plusieurs pions, les autres sont touchés normalement.
- Impossible si ton pion est immobilisé, ou s'il n'a aucune case libre autour de lui.
- L'adversaire a dépensé ses points : son attaque est perdue pour ce pion.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Esquive, contre une attaque de 2 points ou moins | 1 à 1,5 |
| Rare | 2 | Esquive, contre n'importe quelle attaque | 2 |

**Précisions**

- Esquiver déplace ton pion : si l'attaque visait un pion de ta ligne, la ligne est cassée quand même. La Feintaille sauve le pion, pas la position.
- La limite se lit sur le coût de l'attaque, pas sur sa rareté, pour rester valable si le coût est un jour découplé de la rareté.
- Changements par rapport au GDD : la Rare ne redirige plus le coup sur un ennemi voisin ; elle esquive toutes les attaques, Épiques et Légendaires comprises.

### R2 · Sentinelle

Réaction. Déclencheur : l'adversaire pose un pion.

**Règles de la carte**

- **Pose** : le pion vient de ta réserve (impossible si elle est vide). C'est une pose en plus, pendant le tour adverse.
- **Déclencheur** : n'importe quelle pose adverse, sa pose normale comme les pions posés par une carte (Familia, Reine, Meilleurs amis…).
- On ne réagit pas à une réaction : ta pose de Sentinelle ne déclenche pas la sienne.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Tu poses un pion collé au sien | 1,5 |
| Rare | 2 | Tu poses un pion collé au sien, et son pion est immobilisé 2 tours | 2,5 |

**Précisions**

- C'est la réaction de blocage : quand il prolonge sa ligne, tu poses juste au bout. C'est aussi la principale raison de garder des points en banque.
- La Rare protège ton bloqueur (son pion immobilisé ne peut pas l'attaquer) et prépare le charme fatal de la Beauté fatale.
- Changements par rapport au GDD : la Rare ne pose plus « où tu veux » ; elle pose collé et immobilise le pion adverse.

### R3 · Queue du lézard

Réaction. Déclencheur : un de tes pions meurt. Comme le lézard qui perd sa queue, tu repousses tout de suite à côté.

**Règles de la carte**

- **Mourir** : partir à l'infirmerie ou au cimetière, quelle que soit l'attaque.
- **Pose** : les pions viennent de ta réserve (impossible si elle est vide), sur des cases libres collées à la case du pion mort, mais jamais sur cette case elle-même.
- **Plusieurs morts d'un coup** : une seule réaction par tour, donc tu choisis autour de quel pion mort tu poses.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Tu poses 1 pion à côté de ton pion mort | 1,5 |
| Rare | 2 | Tu poses 2 pions à côté de ton pion mort | 2,5 |

**Précisions**

- Pas sur la case du mort : la réaction a lieu juste après le kill, avant que l'attaquant puisse poser sur la case libérée. Elle n'annule donc pas son « tuer puis poser ».
- Elle se combine avec le Médecin : la tombe reste libre, donc tu peux encore y ressusciter ton pion plus tard.
- Changements par rapport au GDD : précisions seulement (pas sur la case du mort, cas de plusieurs morts).

### R4 · Perroquet

Réaction. Déclencheur : l'adversaire joue une attaque. Tu la rejoues aussitôt, comme un perroquet.

**Règles de la carte**

- **Le moment** : une fois son attaque terminée, tu rejoues tout de suite la même attaque, dans la même rareté, pendant son tour.
- **Comme si c'était ta carte** : tu choisis ton lanceur (un de tes pions posé avant ce tour et non immobilisé) et tes cibles, en respectant la portée de la carte. Les pions qu'elle pose viennent de ta réserve, et les effets qu'elle crée sont à toi.
- Seulement les attaques, pas les réactions, puisqu'on ne réagit pas à une réaction.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | Rejoue une attaque adverse de 1 point | 1 à 1,25 |
| Rare | 2 | Rejoue une attaque adverse de 2 points ou moins | 2 |

**Précisions**

- Équilibrée par construction : tu paies à peu près ce que vaut l'attaque copiée. Son bonus, c'est d'agir pendant son tour et de te donner un effet que tu n'as peut-être pas dans ton deck.
- La limite se lit sur le coût de l'attaque, comme pour la Feintaille.
- Changements par rapport au GDD : limites exprimées en coût au lieu de la rareté, et précisions sur le moment et le lanceur.

### R5 · Hibou

Réaction. Déclencheur : l'adversaire déplace un pion. Le hibou voit tout bouger, et il répond aussitôt.

**Règles de la carte**

- **Déclencheur** : n'importe quel déplacement causé par l'adversaire, d'un de ses pions ou d'un des tiens : saut, poussée, échange, téléportation, charge, Lièvre, déplacement gratuit de sa reine.
- **Mouvement** : en pas, comme le Lièvre (1 case vers une case libre voisine, dans les 8 directions). Un pion immobilisé ne peut pas bouger.

| Rareté | Coût | Effet | Valeur estimée |
|---|---|---|---|
| Commune | 1 | 1 de tes pions avance d'1 pas | 1 |
| Rare | 2 | 2 de tes pions avancent d'1 pas chacun | 2 |

**Précisions**

- Comme il n'y a plus de déplacement de base, le Hibou ne réagit qu'aux cartes, mais elles sont nombreuses.
- Une ligne complétée par le Hibou doit survivre au tour adverse suivant, complet (règle commune des réactions).
- Changements par rapport au GDD : la Rare déplace 2 pions d'1 pas au lieu d'1 pion de 2 cases.
