# Protocole du serveur

Ce que doit savoir un client (Godot, sur le web, le PC ou le mobile) pour jouer en ligne. Le client **ne connaît aucune règle** : le serveur lui envoie à chaque changement ce qu'il voit de la partie, la décision attendue et la liste exacte de ses coups permis ; il en renvoie un.

## Connexion

- Lancer le serveur en local : `cargo serveur` (port 3000, ou la variable `PORT`).
- WebSocket : `ws://localhost:3000/ws` en local ; `wss://…/ws` une fois en ligne (obligatoire depuis une page en `https`).
- Un message JSON par trame texte, dans les deux sens.
- `GET /health` répond `ok` (pour l'hébergeur).

## Messages du client

Chaque message a un champ `type`.

| Message | Exemple | Effet |
|---|---|---|
| Créer une salle | `{"type":"create","opponent":"mcts"}` | Le client est joueur 1. `opponent` : `"human"` (un ami rejoint avec le code), `"mcts"` (le bot du serveur) ou `"random"`. `variant` (facultatif) : le nom d'une variante du labo ; par défaut « règles actuelles avec draft ». |
| Rejoindre | `{"type":"join","code":"K7QXM"}` | Le client prend la place libre (joueur 2). Majuscules ou minuscules. |
| Reprendre sa place | `{"type":"rejoin","code":"K7QXM","player":"One","token":"…"}` | Après une déconnexion, avec le jeton reçu en s'asseyant. |
| Jouer | `{"type":"play","action":…}` | Un coup de `legal_actions` de sa dernière vue, **recopié tel quel**. |
| Aperçu | `{"type":"preview","actions":[{"UseAttack":0},{"Choose":{"row":4,"col":4}}]}` | Essaie ces coups sur une copie, sans rien jouer : la réponse `preview` donne les choix suivants. |

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
| Place | `{"type":"seated","code":"K7QXM","player":"One","token":"…"}` | Après `create`, `join` ou `rejoin`. Garder `code` (à donner à l'ami) et `token` (secret, pour revenir). |
| Vue | `{"type":"state","view":{…},"events":[…]}` | Juste après s'être assis, puis à chaque coup de n'importe qui. |
| Aperçu | `{"type":"preview","view":{…}}` | Réponse à un `preview` : la vue après ces coups. Rien n'est joué, l'adversaire ne voit rien. |
| Erreur | `{"type":"error","message":"ce n'est pas à toi de jouer"}` | Demande refusée, rien n'a changé. Le message est en français, à afficher. |

### La vue (`view`)

Tout ce qu'il faut afficher. Le client redessine tout à partir d'elle.

| Champ | Contenu |
|---|---|
| `viewer` | `"One"` ou `"Two"` : qui regarde |
| `decision` | Ce qui est attendu et de qui, par exemple `{"MainAction":{"player":"One"}}`, `{"React":{"player":"Two"}}`, `{"ChooseTarget":{"player":"One","attack":…,"launcher":…}}`, `{"Draft":{"step":{"Pick":"Two"}}}` ; `null` si la partie est finie |
| `legal_actions` | Les coups permis **à ce client**, vide si ce n'est pas à lui |
| `board` | `size` (côté de la grille, mur compris) et `cells`, rangée par rangée (indice `row * size + col`) : `null` pour le mur, sinon `{"pion":{"id":0,"owner":"One"} ou null,"tomb":… ou null}` |
| `sides` | Pour J1 puis J2 : `reserve`, `banked_actions`, `actions_left`, `placements_left`, `queen`, `scorched_until`, `arsenal` (une entrée par carte : `{"attack":{"kind":…,"rarity":…},"cooldown_left":0,"secret":false}`, ou `null` pour la carte en cache de l'adversaire, pas encore jouée) |
| `effects` | Les statuts et effets en jeu (immobilisé, protégé, gardes de la reine…), avec le pion visé |
| `token_piles`, `infirmary`, `graveyard` | Jetons du Massacre divin, pions à l'infirmerie, pions au cimetière |
| `draft_pool` | Pendant le draft : les cartes tirées qui restent |
| `turn`, `current_player`, `end` | Le tour, à qui c'est le tour, la fin (gagnant et raison) |
| `config` | Les réglages : taille, mur, catalogue et coûts des cartes |

### Les événements (`events`)

Ce qui s'est passé depuis la vue précédente, pour animer et tenir un journal : `{"PionPlaced":{…}}`, `{"PionKilled":{…}}`, `{"PionMoved":{…}}`, `{"AttackUsed":{…}}`, `{"CardRevealed":{…}}`… La carte que l'adversaire met en cache n'y apparaît jamais. L'état à afficher est toujours celui de la vue : les événements ne servent qu'à l'animation.

## Le déroulé d'une partie

1. Le client se connecte et envoie `create` (ou `join` avec un code).
2. Il reçoit `seated`, puis une première vue.
3. À chaque vue : si `legal_actions` n'est pas vide, c'est à lui. Il affiche les choix et envoie le coup choisi. Sinon, il attend la vue suivante (l'adversaire ou le bot joue).
4. Une carte se joue en plusieurs coups (`UseAttack`, puis des `Choose`…). Pour qu'on puisse l'annuler, le client garde ses choix en cours et demande à chaque étape un `preview` avec la liste des choix déjà faits : la réponse donne les choix suivants (`legal_actions`). Quand la carte est complète (la vue de l'aperçu n'attend plus de choix de carte), il envoie les coups un par un avec `play`. Annuler, c'est simplement oublier la liste : rien n'a été joué.
5. Quand `end` n'est plus `null`, la partie est finie.

En cas de coupure, le client rouvre une WebSocket et envoie `rejoin` : il retrouve sa place et la vue actuelle.

## Limites actuelles

- Les salles vivent en mémoire : un redémarrage du serveur les perd. Chaque partie est gardée comme liste d'actions, ce qui permettra de les sauvegarder et de les recharger.
- Une salle n'est jamais fermée ; il faudra les nettoyer après un temps d'inactivité.
- Pas de comptes : le jeton tient lieu d'identité.
