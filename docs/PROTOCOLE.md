# Protocole du serveur

Ce que doit savoir un client (Godot, sur le web, le PC ou le mobile) pour jouer en ligne. Le client **ne connaît aucune règle** : le serveur lui envoie à chaque changement ce qu'il voit de la partie, la décision attendue et la liste exacte de ses coups permis ; il en renvoie un.

## Connexion

- Lancer le serveur en local : `cargo serveur` (port 3000, ou la variable `PORT`).
- WebSocket : `ws://127.0.0.1:3000/ws` en local ; `wss://…/ws` une fois en ligne (obligatoire depuis une page en `https`).
- Un message JSON par trame texte, dans les deux sens.
- Le serveur n'écoute qu'en IPv4 : un client local vise `127.0.0.1`, car `localhost` peut désigner l'IPv6 (`::1`) sous Windows.
- Une vue complète (configuration et catalogue compris) peut approcher la mémoire de réception par défaut de Godot (64 Kio) : le client Godot en prend 1 Mio.
- Les coups reçus se renvoient tels quels. Un client dont le JSON lit les nombres en flottants (Godot) doit les réécrire en entiers : le serveur refuse `{"row":3.0}`.
- `GET /health` répond `ok` (pour l'hébergeur).

## Messages du client

Chaque message a un champ `type`.

| Message | Exemple | Effet |
|---|---|---|
| Créer une salle | `{"type":"create","opponent":"mcts"}` | Le client est joueur 1. `opponent` : `"human"` (un ami rejoint avec le code), `"mcts"` (le bot du serveur) ou `"random"`. `variant` (facultatif) : le nom d'une variante du labo ; par défaut « règles actuelles avec draft ». |
| Rejoindre | `{"type":"join","code":"K7QXM"}` | Le client prend la place libre (joueur 2). Majuscules ou minuscules. |
| Reprendre sa place | `{"type":"rejoin","code":"K7QXM","player":"One","key":"…"}` | Après une déconnexion, avec la clé reçue en s'asseyant. |
| Jouer | `{"type":"play","action":…}` | Un coup de `legal_actions` de sa dernière vue, **recopié tel quel**. |
| Se remettre à jour | `{"type":"sync"}` | Redemande la vue actuelle (`state`, avec `events` vide). À envoyer si le client pense avoir manqué une vue. |
| Catalogue des cartes | `{"type":"cards"}` | Demande le nom et les textes de toutes les cartes (réponse `cards`). Possible à tout moment, assis ou non. |
| Aperçu | `{"type":"preview","actions":[{"UseAttack":0},{"Choose":{"row":4,"col":4}}]}` | Essaie les choix d'une carte sur une copie, sans rien jouer : la réponse `preview` donne les choix suivants. L'aperçu s'arrête au coup qui finit ces choix (voir « Le déroulé »). |

Les coups (`action`) ont la forme du moteur. Le client n'a pas à les construire : il les reprend dans `legal_actions`. Pour les afficher :

| Coup | JSON | Sens |
|---|---|---|
| Poser | `{"Place":{"row":3,"col":4}}` | Poser un pion sur cette case |
| Lancer une carte | `{"UseAttack":0}` | La carte de cet emplacement de son arsenal (réaction comprise) |
| Choisir une case | `{"Choose":{"row":4,"col":4}}` | Le lanceur, puis chaque cible de la carte en cours |
| Viser les jetons | `"ChooseEnemyTokens"` | Attila Rare et Épique |
| Jouer la carte | `"FinishAttack"` | Avec les cibles déjà choisies (« jusqu'à 2 ») |
| Pas de la reine | `{"StepQueen":{"row":2,"col":2}}` | Geste gratuit |
| Tir d'un garde | `{"Shoot":{"row":5,"col":2}}` | |
| Pas du Lièvre | `{"HareStep":{"from":{"row":1,"col":1},"to":{"row":1,"col":2}}}` | Geste gratuit |
| Draft | `{"Draft":{"kind":"CoupDuTaureau","rarity":"Rare"}}` | Mettre en cache, bannir ou prendre cette carte, selon l'étape |
| Ne pas réagir | `"Pass"` | |
| Finir son tour | `"EndTurn"` | |

`row` et `col` partent de 0, en haut à gauche de la grille **mur compris**.

## Messages du serveur

| Message | Exemple | Quand |
|---|---|---|
| Place | `{"type":"seated","code":"K7QXM","player":"One","key":"…"}` | Après `create`, `join` ou `rejoin`. Garder `code` (à donner à l'ami) et `key` (secrète, pour revenir). |
| Vue | `{"type":"state","view":{…},"events":[…]}` | Juste après s'être assis, puis à chaque coup de n'importe qui. |
| Aperçu | `{"type":"preview","view":{…}}` ou `{"type":"preview","view":null}` | Réponse à un `preview` : la vue après ces coups, ou `null` si le dernier coup finit les choix de la carte (la suite est prête à être jouée). Rien n'est joué, l'adversaire ne voit rien. |
| Catalogue | `{"type":"cards","cards":[{"kind":"Ours","name":"Ours","reaction":false,"summary":"…","rules":["…"],"rarities":[{"rarity":"Rare","name":"Rare","effect":"…"}]}]}` | Réponse à `cards` : toutes les familles, dans l'ordre des fiches attaques, avec leurs raretés jouables. Les textes viennent de `serveur/data/cartes.json`, les noms du moteur. Le coût et la recharge sont dans la configuration de la partie (`config.attack_catalog` de la vue). |
| Adversaire | `{"type":"opponent","connected":true}` | L'adversaire humain vient de se brancher sur sa place (`true`) ou de s'en débrancher (`false`). Aussi juste après sa première vue, à celui qui s'assoit, si son adversaire est déjà là. Jamais pour un bot. Sert à la salle d'attente : le créateur d'une salle lance la partie quand son ami est là. |
| Erreur | `{"type":"error","kind":"not_your_turn","message":"ce n'est pas à toi de jouer"}` | Demande refusée, rien n'a changé. `message` est en français, à afficher ; `kind` est stable, pour réagir sans lire le texte (voir « Les erreurs »). Arrive aussi sans demande, `bot_failed`, si le bot du serveur ne peut plus jouer du tout : la partie attend, et le prochain message du client relance le bot. |

### La vue (`view`)

Tout ce qu'il faut afficher. Le client redessine tout à partir d'elle.

| Champ | Contenu |
|---|---|
| `viewer` | `"One"` ou `"Two"` : qui regarde |
| `decision` | Ce qui est attendu et de qui, par exemple `{"MainAction":{"player":"One"}}`, `{"React":{"player":"Two"}}`, `{"ChooseTarget":{"player":"One","attack":…,"launcher":…}}`, `{"Draft":{"step":{"Pick":"Two"}}}` ; `null` si la partie est finie |
| `legal_actions` | Les coups permis **à ce client**, vide si ce n'est pas à lui |
| `board` | `size` (côté de la grille, mur compris) et `cells`, rangée par rangée (indice `row * size + col`) : `null` pour le mur, sinon `{"pion":{"id":0,"owner":"One"} ou null,"tomb":… ou null}` |
| `sides` | Pour J1 puis J2 : `reserve`, `banked_actions`, `actions_left`, `placements_left`, `queen`, `scorched_until`, `arsenal` (une entrée par carte : `{"attack":{"kind":…,"rarity":…},"cooldown_left":0,"secret":false}`, ou `null` pour la carte en cache de l'adversaire, pas encore lancée) |
| `effects` | Les statuts et effets en jeu (immobilisé, protégé, gardes de la reine…), avec le pion visé |
| `token_piles`, `infirmary`, `graveyard` | Jetons du Massacre divin, pions à l'infirmerie, pions au cimetière |
| `draft_pool` | Pendant le draft : les cartes tirées qui restent, à partir du premier ban ou pick (vide pendant le choix des caches) |
| `turn`, `current_player`, `end` | Le tour, à qui c'est le tour, la fin (gagnant et raison) |
| `winning_line` | Les cases de la ligne gagnante (`[{"row":…,"col":…}]`), si la partie est gagnée par alignement ; vide sinon |
| `config` | Les réglages : taille, mur, catalogue et coûts des cartes |

### Les événements (`events`)

Ce qui s'est passé depuis la vue précédente, pour animer et tenir un journal : `{"PionPlaced":{…}}`, `{"PionKilled":{…}}`, `{"PionMoved":{…}}`, `{"AttackUsed":{…}}`, `{"CardRevealed":{…}}`… La carte que l'adversaire met en cache n'y est jamais nommée : elle arrive comme `{"CardCached":{"player":…}}`, puis `CardRevealed` (avec la carte) quand il la lance. L'état à afficher est toujours celui de la vue : les événements ne servent qu'à l'animation.

## Le déroulé d'une partie

1. Le client se connecte et envoie `create` (ou `join` avec un code).
2. Il reçoit `seated`, puis une première vue.
3. À chaque vue : si `legal_actions` n'est pas vide, c'est à lui. Il affiche les choix et envoie le coup choisi. Sinon, il attend la vue suivante (l'adversaire ou le bot joue).
4. Une carte se joue en plusieurs coups (`UseAttack`, puis des `Choose`…). Pour qu'on puisse l'annuler, le client garde ses choix en cours et demande à chaque étape un `preview` avec la liste des choix déjà faits : la réponse donne les choix suivants (`legal_actions`). Quand la réponse est `"view":null`, la carte est complète : il envoie les coups un par un avec `play`. Annuler, c'est simplement oublier la liste : rien n'a été joué. Tout coup autre que `UseAttack` (poser, pas de la reine ou du Lièvre, tir, passer, finir son tour, le draft) s'envoie directement avec `play` ; un `preview` d'un seul de ces coups répond `null`, ce qui revient au même.
   - Pourquoi l'aperçu s'arrête là : ce qui suit une carte jouée (une Feintaille adverse qui attend sa réponse, une Sentinelle qui répond à une pose) dépend de la cache de l'adversaire. Le montrer sans rien jouer la dévoilerait. Un aperçu qui continue après ce coup est refusé.
5. Quand `end` n'est plus `null`, la partie est finie.

En cas de coupure, le client rouvre une WebSocket et envoie `rejoin` : il retrouve sa place et la vue actuelle. L'ancienne connexion, si elle vit encore, ne reçoit plus rien et ses demandes sont refusées (`seat_taken`).

### Les erreurs (`kind`)

| `kind` | Sens | Que faire |
|---|---|---|
| `bad_message` | Le message n'est pas du JSON du protocole | Un bug du client |
| `unknown_room` | Aucune salle n'a ce code (ou elle a été fermée) | Revenir au menu |
| `room_full` | Les deux places sont prises | Revenir au menu |
| `bad_key` | La clé ne correspond pas à cette place | Revenir au menu |
| `not_seated`, `already_seated` | Demande hors d'une salle, ou deuxième salle sur la même connexion | Un bug du client |
| `unknown_variant` | Aucune variante n'a ce nom | Un bug du client |
| `not_your_turn` | Ce n'est pas à ce joueur de décider | Envoyer `sync` |
| `seat_taken` | Une autre connexion a repris cette place | Fermer cette connexion |
| `preview_past_the_card` | Un aperçu continue après la fin des choix de la carte | Un bug du client |
| `server_full` | Le serveur a son nombre maximal de salles | Réessayer plus tard |
| `bot_failed` | Le bot du serveur ne peut plus jouer | Afficher le message ; la partie attend |
| `refused_move` | Le moteur refuse ce coup | Envoyer `sync` |
| `cards_unavailable` | Le fichier des textes des cartes est mal écrit (un bug du serveur) | Afficher les cartes sans texte |

## Limites actuelles

- Les salles vivent en mémoire : un redémarrage du serveur les perd. Chaque partie est gardée comme liste d'actions, ce qui permettra de les sauvegarder et de les recharger.
- Une salle est fermée quand plus aucun humain n'y est connecté, tout de suite si la partie est finie, sinon après 30 minutes : on ne peut plus y revenir. Le serveur ouvre au plus 1000 salles (`le serveur est complet`).
- Un message du client fait au plus 64 Kio.
- Un client qui ne lit pas assez vite perd des vues : chaque vue étant complète, la suivante le remet à jour, et `sync` redemande la vue actuelle (seul le journal des événements a un trou).
- Un bot du serveur sans coup valable (un bug) joue à la place le premier coup permis par le moteur, et le serveur le note dans son journal : la partie ne reste pas figée.
- À faire pour la mise en ligne (E4) : une limite de salles par adresse (sinon on remplit le serveur en ouvrant des salles), et une salle « contre un humain » dont le créateur reste connecté ne ferme jamais.
- Les bots cherchent en parallèle sur les cœurs de la machine moins un ; au-delà, leurs coups attendent leur tour.
- Ces limites sont dans `serveur/src/limits.rs`.
- Pas de comptes : la clé de place tient lieu d'identité.
