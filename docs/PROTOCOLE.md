# Protocole du serveur

Ce que doit savoir un client (Godot, sur le web, le PC ou le mobile) pour jouer en ligne. Le client **ne connaît aucune règle** : le serveur lui envoie à chaque changement ce qu'il voit de la partie, la décision attendue et la liste exacte de ses coups permis ; il en renvoie un.

## Connexion

- Lancer le serveur en local : `cargo serveur` (port 3000, ou la variable `PORT`).
- WebSocket : `ws://127.0.0.1:3000/ws` en local ; `wss://…/ws` une fois en ligne (obligatoire depuis une page en `https`).
- Un message JSON par trame texte, dans les deux sens.
- Le serveur n'écoute qu'en IPv4 : un client local vise `127.0.0.1`, car `localhost` peut désigner l'IPv6 (`::1`) sous Windows.
- Une vue complète (configuration et catalogue compris) peut approcher la mémoire de réception par défaut de Godot (64 Kio) : le client Godot en prend 1 Mio.
- Les coups reçus se renvoient tels quels. Un client dont le JSON lit les nombres en flottants (Godot) doit les réécrire en entiers : le serveur refuse `{"row":3.0}`.
- `GET /health` répond `ok` (pour l'hébergeur) ; `serveur --health` le vérifie depuis la machine (le contrôle de santé du conteneur).
- **Une connexion morte ne reste pas ouverte.** Le serveur envoie un ping toutes les 20 secondes (le navigateur, ou le `WebSocketPeer` de Godot, y répond tout seul) et ferme une connexion dont rien n'est arrivé depuis 60 secondes. Un client qui ne lit plus ses messages pendant 10 secondes est abandonné ; un client qui envoie plus de 200 trames en 10 secondes (messages, pongs compris) est fermé (`too_many_messages`), et une connexion qui ne s'est pas connectée à un compte en 30 secondes aussi. Au-delà de 2000 connexions ouvertes, la WebSocket est refusée (HTTP 503).
- **Arrêt propre** (Ctrl+C, ou SIGTERM de `docker stop`) : le serveur n'accepte plus personne, ferme les connexions (le client se reconnectera au serveur suivant) et attend au plus 10 secondes les parties en cours d'enregistrement. Les parties pas finies sont perdues : elles ne vivent qu'en mémoire.
- **Le client web est servi par le serveur**, sur `/`, depuis le dossier de la variable `WEB_DIR` (par défaut `client-godot/export/web`, `/web` dans le conteneur), compressé en gzip (un fichier déjà compressé à côté, `index.wasm.gz`, est envoyé tel quel : voir `outils/precompresser_web.py`) et revalidé à chaque visite (`Cache-Control: no-cache`) ; sans `index.html` dans ce dossier, le serveur ne sert que la WebSocket. La page et le jeu ont ainsi la même adresse : dans un navigateur, le client Godot déduit celle de la WebSocket de la page (`https://hôte:port/` → `wss://hôte:port/ws`, `ServerLink.server_url`) ; hors navigateur, il vise `ws://127.0.0.1:3000/ws`.
- **Un compte d'abord** : créer ou rejoindre une salle demande d'être connecté à son compte. La première fois, le client envoie `register` et garde le jeton reçu (`account.token`) ; ensuite, à chaque connexion (et après chaque coupure), il envoie `login` avec ce jeton, avant ses autres demandes. `cards` n'en a pas besoin. Le serveur crée au plus 60 comptes par minute, tous clients confondus (`too_many_registrations`).
- **Un compte, une salle** : un compte n'est assis que dans une salle à la fois tant que sa partie n'est pas finie (`already_seated`). Se connecter à son compte **le rassoit dans sa salle en cours** : `account` dit `"resumed":true`, puis arrivent `seated`, sa vue et la présence de l'adversaire, puis `collection`. Pour en ouvrir une autre, il faut d'abord quitter la sienne (`leave`).
- La base des comptes est le fichier de `DATABASE_URL` (par défaut `sqlite://fight_of_pions.db`, dans le dossier où le serveur est lancé), créé au premier lancement.

## Messages du client

Chaque message a un champ `type`.

| Message | Exemple | Effet |
|---|---|---|
| Créer un compte | `{"type":"register","pseudo":"Martin"}` | Crée un compte invité et s'y connecte. `pseudo` (facultatif) : de 3 à 16 caractères, lettres, chiffres, espaces, `-`, `_`, `.` ; sans pseudo, « Pion » et quatre chiffres. Réponses : `account` **avec son jeton**, puis `collection`. |
| Se connecter | `{"type":"login","token":"…"}` | Avec le jeton reçu à la création. Réponses : `account` (sans jeton), puis, si le compte est assis dans une salle encore ouverte, `seated` et sa vue (il y est rassis), puis `collection`. |
| Changer de pseudo | `{"type":"set_pseudo","pseudo":"Martin"}` | Réponse : `account`. |
| Sa collection | `{"type":"collection"}` | Redemande ses cartes et son trio (réponse `collection`). |
| Ranger son trio de cache | `{"type":"set_cache_trio","cards":[{"kind":"Ours","rarity":"Common"}]}` | De 0 à 3 cartes différentes de sa collection, dans l'ordre voulu. Au draft, la cache se choisit dans le trio ; si le trio est vide, le serveur tire 3 cartes de la collection au hasard, à chaque partie. Compte pour les salles créées ou rejointes ensuite. Réponse : `collection`. |
| Créer une salle | `{"type":"create","opponent":"mcts"}` | Le client s'assoit à une place tirée au sort (J1 commence le draft et la partie). Contre un bot, la partie est lancée tout de suite ; contre un ami, elle attend `launch`. `opponent` : `"human"` (un ami rejoint avec le code), `"mcts"` (le bot du serveur) ou `"random"`. `variant` (facultatif) : seule « règles actuelles avec draft », celle par défaut, est acceptée (`unknown_variant` sinon) ; les autres variantes servent au labo. |
| Rejoindre | `{"type":"join","code":"K7QXM"}` | Le client prend la place laissée libre par le créateur. Majuscules ou minuscules. |
| Lancer la partie | `{"type":"launch"}` | Seul le créateur d'une salle entre amis, une fois son ami assis et branché (`opponent_missing` sinon). Les deux joueurs reçoivent `launched` ; avant, `play` et `preview` sont refusés (`not_launched`). Une partie déjà lancée le reste. |
| Quitter sa salle | `{"type":"leave"}` | Partie lancée et pas finie : **perdue par abandon**, l'adversaire gagne. Avant le lancement : la place de l'invité redevient libre ; le créateur, lui, ferme la salle (son invité reçoit `room_closed`). Partie finie : on la quitte, simplement. Réponse : `left`, même sans salle. Ensuite, la même connexion peut créer ou rejoindre une autre salle. |
| Reprendre sa place | `{"type":"rejoin","code":"K7QXM","player":"One","key":"…"}` | Rarement utile : `login` rassoit déjà. Avec la clé reçue en s'asseyant, depuis le compte assis à cette place. |
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
| Place | `{"type":"seated","code":"K7QXM","player":"One","key":"…","launched":false}` | Après `create`, `join`, `rejoin`, ou `login` quand le compte est rassis dans sa salle. Garder `code` (à donner à l'ami) et `key` (secrète, pour revenir). `launched` : la partie est-elle lancée ? Sinon, attendre `launched` (salle d'attente). |
| Partie lancée | `{"type":"launched"}` | Le créateur de la salle l'a lancée : aux deux joueurs. On peut jouer. |
| Vue | `{"type":"state","view":{…},"events":[…]}` | Juste après s'être assis, puis à chaque coup de n'importe qui. |
| Aperçu | `{"type":"preview","view":{…}}` ou `{"type":"preview","view":null}` | Réponse à un `preview` : la vue après ces coups, ou `null` si le dernier coup finit les choix de la carte (la suite est prête à être jouée). Rien n'est joué, l'adversaire ne voit rien. |
| Catalogue | `{"type":"cards","cards":[{"kind":"Ours","name":"Ours","reaction":false,"summary":"…","rules":["…"],"rarities":[{"rarity":"Rare","name":"Rare","effect":"…"}]}]}` | Réponse à `cards` : toutes les familles, dans l'ordre des fiches attaques, avec leurs raretés jouables. Les textes viennent de `serveur/data/cartes.json`, les noms du moteur. Le coût et la recharge sont dans la configuration de la partie (`config.attack_catalog` de la vue). |
| Adversaire | `{"type":"opponent","connected":true,"pseudo":"Bob"}` | L'adversaire humain vient de se brancher sur sa place (`true`) ou de s'en débrancher (`false`, aussi quand il quitte la salle) ; `pseudo` : le sien. Aussi juste après sa première vue, à celui qui s'assoit, dès qu'un humain occupe l'autre place, là ou non. Jamais pour un bot. Sert à la salle d'attente (le créateur lance la partie quand son ami est là) et à nommer l'adversaire. |
| Départ | `{"type":"left"}` | Réponse à `leave` : le client n'est plus dans aucune salle. |
| Compte | `{"type":"account","id":12,"pseudo":"Martin","token":"…","resumed":false}` | Après `register`, `login` ou `set_pseudo`. `token` n'est là qu'après `register` : **le garder**, le serveur ne le redonnera jamais (il n'en garde que l'empreinte). `resumed` : vrai si `login` vient de rasseoir le compte dans sa salle (`seated` suit) ; faux sinon. Un client qui croyait avoir une place et reçoit `false` sait qu'elle n'existe plus. |
| Collection | `{"type":"collection","cards":[{"kind":"Ours","rarity":"Common"}],"cache_trio":[]}` | Après `register`, `login`, `collection` ou `set_cache_trio` : les cartes que possède le compte (triées par famille puis rareté) et son trio de cache. Un compte neuf reçoit le paquet de départ (`serveur/data/depart.json`). |
| Récompense | `{"type":"reward","card":{"kind":"Ours","rarity":"Epic"}}` | La partie vient de finir et ce joueur a gagné cette carte, déjà rangée dans sa collection. Seulement au gagnant d'un autre compte ou du MCTS (rien contre le bot aléatoire, contre soi-même, pour un nul, ni pour une victoire par abandon), et s'il lui manque encore une carte du catalogue de la partie. Arrive juste après la dernière vue ; un gagnant déconnecté la trouvera dans sa collection. |
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
| `draft_history` | Pendant le draft : les choix déjà faits, dans l'ordre des étapes (`[{"step": {"Ban": "Two"}, "attack": {"kind": "Ours", "rarity": "Common"}}, …]`) ; la cache de l'adversaire y est `"attack": null`. Le rang de l'étape en cours est leur nombre. Vide hors du draft |
| `turn`, `current_player`, `end` | Le tour, à qui c'est le tour, la fin (gagnant et raison) |
| `winning_line` | Les cases de la ligne gagnante (`[{"row":…,"col":…}]`), si la partie est gagnée par alignement ; vide sinon |
| `config` | Les réglages : taille, mur, catalogue et coûts des cartes |

### Les événements (`events`)

Ce qui s'est passé depuis la vue précédente, pour animer et tenir un journal : `{"PionPlaced":{…}}`, `{"PionKilled":{…}}`, `{"PionMoved":{…}}`, `{"AttackUsed":{…}}`, `{"CardRevealed":{…}}`… La carte que l'adversaire met en cache n'y est jamais nommée : elle arrive comme `{"CardCached":{"player":…}}`, puis `CardRevealed` (avec la carte) quand il la lance. L'état à afficher est toujours celui de la vue : les événements ne servent qu'à l'animation.

## Le déroulé d'une partie

1. Le client se connecte et envoie `create` (ou `join` avec un code).
2. Il reçoit `seated`, puis une première vue. Entre amis, la partie attend son lancement : le créateur envoie `launch` quand son ami est là, et les deux reçoivent `launched` ; jusque-là, le client montre la salle d'attente.
3. À chaque vue : si `legal_actions` n'est pas vide, c'est à lui. Il affiche les choix et envoie le coup choisi. Sinon, il attend la vue suivante (l'adversaire ou le bot joue).
4. Une carte se joue en plusieurs coups (`UseAttack`, puis des `Choose`…). Pour qu'on puisse l'annuler, le client garde ses choix en cours et demande à chaque étape un `preview` avec la liste des choix déjà faits : la réponse donne les choix suivants (`legal_actions`). Quand la réponse est `"view":null`, la carte est complète : il envoie les coups un par un avec `play`. Annuler, c'est simplement oublier la liste : rien n'a été joué. Tout coup autre que `UseAttack` (poser, pas de la reine ou du Lièvre, tir, passer, finir son tour, le draft) s'envoie directement avec `play` ; un `preview` d'un seul de ces coups répond `null`, ce qui revient au même.
   - Pourquoi l'aperçu s'arrête là : ce qui suit une carte jouée (une Feintaille adverse qui attend sa réponse, une Sentinelle qui répond à une pose) dépend de la cache de l'adversaire. Le montrer sans rien jouer la dévoilerait. Un aperçu qui continue après ce coup est refusé.
5. Quand `end` n'est plus `null`, la partie est finie. Le client envoie `leave` pour revenir au menu.

En cas de coupure, le client rouvre une WebSocket et envoie `login` : le serveur le rassoit à sa place, avec la vue actuelle. L'ancienne connexion, si elle vit encore, reçoit `seat_taken`, ne reçoit plus rien, et ses demandes sont refusées. **Un joueur débranché plus de 3 minutes d'une partie lancée la perd par abandon** (le serveur balaie ses salles toutes les 30 secondes) ; avant le lancement, personne ne perd.

### Les erreurs (`kind`)

| `kind` | Sens | Que faire |
|---|---|---|
| `bad_message` | Le message n'est pas du JSON du protocole | Un bug du client |
| `unknown_room` | Aucune salle n'a ce code (ou elle a été fermée) | Revenir au menu |
| `room_full` | Les deux places sont prises | Revenir au menu |
| `bad_key` | La clé ne correspond pas à cette place | Revenir au menu |
| `not_seated` | Demande hors d'une salle | Un bug du client |
| `already_seated` | Le client, ou son compte, est déjà assis dans une partie pas finie | Envoyer `leave` d'abord |
| `room_closed` | Le créateur de la salle l'a quittée avant de la lancer | Revenir au menu |
| `not_launched` | Un coup ou un aperçu avant le lancement de la partie | Attendre `launched` |
| `not_host`, `opponent_missing` | `launch` par l'ami (seul le créateur lance), ou avant que l'ami soit assis | Un bug du client |
| `unknown_variant` | Aucune variante n'a ce nom | Un bug du client |
| `not_your_turn` | Ce n'est pas à ce joueur de décider | Envoyer `sync` |
| `seat_taken` | Une autre connexion a repris cette place | Fermer cette connexion |
| `preview_past_the_card` | Un aperçu continue après la fin des choix de la carte | Un bug du client |
| `server_full` | Le serveur a son nombre maximal de salles | Réessayer plus tard |
| `bot_failed` | Le bot du serveur ne peut plus jouer | Afficher le message ; la partie attend |
| `refused_move` | Le moteur refuse ce coup | Envoyer `sync` |
| `cards_unavailable` | Le fichier des textes des cartes est mal écrit (un bug du serveur) | Afficher les cartes sans texte |
| `not_logged_in` | `create`, `join`, `collection`, `set_pseudo` ou `set_cache_trio` sans compte | Envoyer `login` (ou `register`) |
| `already_logged_in` | `register` ou `login` sur une connexion déjà connectée | Un bug du client |
| `unknown_account` | Aucun compte n'a ce jeton | Oublier le jeton et proposer de créer un compte |
| `bad_pseudo` | Pseudo refusé (longueur, signe) | Afficher le message |
| `not_owned` | Le trio contient une carte que le compte ne possède pas | Redemander `collection` |
| `bad_cache_trio` | Plus de 3 cartes, ou une carte deux fois | Un bug du client |
| `nothing_to_cache` | Aucune carte de la collection n'est au catalogue de cette variante | Choisir une autre variante |
| `storage` | La base n'a pas pu lire ou écrire (détail dans la sortie du serveur) | Réessayer plus tard |
| `starter_unavailable` | Le fichier du paquet de départ est mal écrit (un bug du serveur) | Réessayer plus tard |
| `too_many_registrations` | Le serveur a créé trop de comptes cette minute | Réessayer dans une minute |
| `too_many_messages` | Le client envoie trop de messages : sa connexion est fermée | Un bug du client |

## Limites actuelles

- Les salles vivent en mémoire : un redémarrage du serveur les perd. Chaque partie est gardée comme liste d'actions, ce qui permettra de les sauvegarder et de les recharger.
- Une salle est fermée quand plus aucun humain n'y est connecté, tout de suite si la partie est finie, sinon après 30 minutes (une salle entre amis pas encore lancée ; une partie lancée est perdue par abandon bien avant) : on ne peut plus y revenir. Le serveur ouvre au plus 1000 salles (`le serveur est complet`).
- Un message du client fait au plus 64 Kio.
- Un client qui ne lit pas assez vite perd des vues : chaque vue étant complète, la suivante le remet à jour, et `sync` redemande la vue actuelle (seul le journal des événements a un trou).
- Un bot du serveur sans coup valable (un bug) joue à la place le premier coup permis par le moteur, et le serveur le note dans son journal : la partie ne reste pas figée.
- Un compte n'ouvre qu'une salle à la fois ; mais on peut créer beaucoup de comptes (au plus 60 par minute). Pas encore de limite par adresse IP. Une salle « contre un humain » dont le créateur reste connecté ne ferme jamais.
- Les bots cherchent en parallèle sur les cœurs de la machine moins un ; au-delà, leurs coups attendent leur tour.
- Ces limites sont dans `serveur/src/limits.rs`.
- Les comptes sont des comptes invités : pas d'e-mail ni de mot de passe. Un client qui perd son jeton (données du navigateur effacées, autre appareil) perd son compte. Deux comptes peuvent avoir le même pseudo.
- C'est le compte qui donne le droit de reprendre sa place ; la clé de place ne sert plus qu'à `rejoin`, avec le même compte.
- Chaque partie finie est enregistrée en base (table `games` : joueurs, variante, graine, coups, fin), qu'elle rapporte une carte ou non, abandon compris (la fin dit `Forfeit`). Si la base refuse, l'enregistrement est retenté deux fois, à 2 secondes d'écart. Une salle fermée avant la fin de sa partie (son créateur est parti avant de la lancer) n'est pas enregistrée.
