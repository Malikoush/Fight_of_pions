# Architecture de Fight of Pions

Dernière mise à jour : 2026-11-04.

Ce document décrit comment le code est organisé et **pourquoi**. Il sert de référence pour toutes les tâches de la [feuille de route](FEUILLE_DE_ROUTE.md). Ce qui n'existe pas encore est marqué **(cible)**.

Les règles du jeu sont dans le GDD (document de design partagé avec Théo, hors dépôt, v0.2) et, pour ce que le moteur implémente réellement, dans le [carnet de règles](REGLES.md). Les noms viennent du [glossaire](GLOSSAIRE.md).

---

## 1. Le but

Un **moteur de règles indépendant**, sur lequel se branchent tous les clients sans jamais en contenir une seule règle :

```
                 ┌────────────┐
   terminal ───▶ │            │
   labo     ───▶ │   moteur   │  ◀── bots (aléatoire, glouton ; ML en cible)
   serveur  ───▶ │ (sans dép.)│  ◀── py_bindings pour l'entraînement (cible)
     ▲           └────────────┘
   Godot (WebSocket)
```

Le moteur ne sait pas qui l'utilise. Un client fait toujours la même chose :

1. lire l'état par les getters de `Game` ;
2. envoyer une `Action` avec `game.apply(action)` ;
3. réagir aux `Event` renvoyés, ou afficher la `GameError`.

## 2. Les principes

Ils viennent de la conception initiale, et certains sont précisés par l'analyse du GDD v0.2.

1. **Moteur déterministe.** Même configuration, même graine, mêmes actions : même partie, au bit près. Le hasard passe uniquement par le générateur `Rng` du moteur, initialisé avec une graine (`seed`) stockée dans la partie. On n'utilise jamais d'horloge, de `HashMap`/`HashSet` (leur ordre de parcours est aléatoire) ni de flottants dans les règles.
2. **Une partie = configuration + graine + liste d'actions** (`GameRecord`). Rejouer les actions reconstruit la partie. Cela donne gratuitement la sauvegarde, le replay, l'annulation dans le labo et la reproduction exacte d'un bug.
3. **Le jeu piloté par les données.** Tous les réglages chiffrés vivent dans `GameConfig`, validés au démarrage. Plus tard, ils seront chargés depuis un fichier pour équilibrer sans recompiler.
4. **Des tranches verticales.** Une fonctionnalité est finie quand elle marche de bout en bout : moteur, tests, labo. On ne commence pas la suivante avant.
5. **Des invariants vérifiés automatiquement.** Le principal : *un miroir construit uniquement à partir des `Event` (plateau, réserves, actions restantes et gardées, recharges des attaques, joueur courant, fin, pions posés ce tour et poses restantes) est toujours égal à l'état du moteur*. C'est la garantie qu'un client qui suit les événements ne se désynchronise jamais, sans recalculer aucune règle. Il est vérifié après chaque action de centaines de parties aléatoires.
6. **Abstraire à la deuxième occurrence, pas avant.** La modularité vient des bonnes abstractions de base (les effets primitifs, le modèle de décision, les données), pas d'options prévues pour des besoins imaginaires.
7. **Le serveur** (`serveur/`, E2) fait autorité sur les parties en ligne : il garde chaque partie entière et n'envoie à chaque joueur que sa vue ; le client (Godot) ne contient aucune règle. Protocole : [PROTOCOLE.md](PROTOCOLE.md).
   - Découpage : `room` (une partie et ses places, sans réseau, testée sans WebSocket), `lobby` (les salles, la diffusion des vues, la fermeture des salles abandonnées), `connection` (une WebSocket), `protocol` (les messages), `limits` (les réglages d'hébergement), `cards` (le catalogue des cartes à afficher).
   - **Le catalogue des cartes** (`cards`, décision du 2026-11-04) : le nom affiché, le résumé, les règles et l'effet de chaque rareté, envoyés au client sur demande (`cards`). Les textes sont des données, dans `serveur/data/cartes.json`, compris dans le serveur à la compilation : on les relit et on les corrige sans toucher au code. Ils sont tirés des fiches attaques, corrigées par le carnet. Les noms viennent du moteur (`Display`, les mêmes que dans le labo), le coût et la recharge de la configuration de chaque partie : rien n'est écrit deux fois. Des tests vérifient que chaque carte jouable d'une variante a son texte, et qu'aucun texte ne décrit une carte injouable.
   - **Le secret** : chaque joueur reçoit `Game::view` et des événements passés par `Event::seen_by`. L'aperçu (`Room::preview`) s'arrête au coup qui finit les choix d'une carte : la suite (fenêtre de réaction, carte jouée) dépend de la cache adverse.
   - **Les bots ne bloquent pas la salle** : une recherche (`BotSearch`) part d'une copie de la partie et du hasard des bots, tourne dans `spawn_blocking` hors du verrou, et attend un des cœurs réservés (`Semaphore`). Le coup trouvé (`BotMove`) n'est joué que si la partie n'a pas changé entre-temps. Une seule tâche par salle fait jouer ses bots (`Table::start_bots`), relancée après chaque message d'un humain. Un bot sans coup valable (aucun coup, coup refusé, panique) joue le premier coup permis par le moteur (`Room::play_bot`) : la partie ne reste jamais figée sur un bug de bot. Si même ce coup est impossible, les joueurs sont prévenus et la partie attend.
   - **Verrous** : `std::sync::Mutex`, jamais tenus pendant un `await` ni pendant une recherche. Le hall est verrouillé avant une table, jamais l'inverse.
   - **Le hasard du serveur** (codes, clés, graines) vient du hachage aléatoire de la bibliothèque standard : le serveur n'a pas à être déterministe, la partie l'est (graine et liste d'actions, `GameRecord`).
8. **La vue joueur** (`game/view.rs`). Ce qu'un joueur voit de la partie, en JSON avec l'option `serde` : `Game::view(viewer)` rend une `PlayerView` (plateau entier mur compris, ce que chaque joueur a, effets, infirmerie, cimetière, cartes tirées du draft, fin) avec la décision attendue et, si c'est à lui, ses coups permis : un client (Godot, via le serveur) n'a besoin d'aucune règle. Le secret n'est décidé qu'à trois endroits (le miroir d'événements des parties au hasard est aussi tenu par chaque joueur, sur ses seuls événements passés par `seen_by`) : `Game::arsenal_seen_by` (une carte secrète adverse est `None`), `Game::visible_draft_pool` (les cartes tirées, cachées pendant le choix des caches) et `Event::seen_by` (la carte mise en cache par l'adversaire devient `Event::CardCached`, sans la carte : rien n'est retiré, un client qui suit ses seuls événements reste à jour). `Game::with_secrets_guessed` permet à un bot de chercher sans lire la cache adverse. Ajouter une information visible, c'est ajouter un champ à la vue.
9. **Les règles avant le code.** Une attaque n'est codée que lorsque sa règle est écrite et datée dans le [carnet de règles](REGLES.md). Les attaques « EN REFONTE » dans le GDD attendent.

## 3. Le workspace

| Membre | Rôle | Dépendances |
|---|---|---|
| `moteur/` (paquet `fight_of_pions`) | Toutes les règles. Bibliothèque. | Aucune ; feature `serde` optionnelle (sauvegarde JSON d'un `GameRecord`) |
| `labo/` | Client de test eframe/egui, en deux écrans. **Partie** : panneau, damier cliquable avec cases jouables, journal, graine, annuler, coups au hasard, copier/charger une partie, chaque joueur humain ou bot, configuration de la prochaine partie réglable. **Arène** : simuler des parties entre bots sur une configuration réglable, et en revoir une. | `moteur` (avec `serde`), `bots`, `eframe`, `serde_json` |
| `terminal/` | Client texte. | `moteur` |
| `bots/` | Bots (aléatoire, glouton) et arène de parties automatiques. Bibliothèque ; l'arène est l'exemple `arene`. | `moteur` |
| `serveur/` | Serveur WebSocket qui fait autorité sur les parties en ligne (principe 7). | `moteur`, `bots`, `axum`, `tokio` |
| `client-godot/` | Le jeu : client Godot 4.7 (GDScript), branché sur le serveur. Hors du workspace Cargo. Voir « Le client Godot ». | Godot 4.7 (rendu Compatibilité) |
| `py_bindings/` (cible) | Environnement Python pour l'entraînement ML. | `moteur`, `pyo3` |
| `godot_bridge/` (cible) | Pont vers Godot. | `moteur`, `godot` |

Commandes :

- `cargo test` : tests du moteur, du terminal et des bots, sans jamais compiler eframe ;
- `cargo run` : le terminal ;
- `cargo labo` : le labo ;
- `cargo arene` (ou `cargo arene 2000`) : l'arène des bots, en release.

## 4. Le moteur

### Organisation

```
moteur/src/
  lib.rs        API publique, lints stricts (pas d'unwrap/expect/panic hors tests)
  config.rs     GameConfig et sa validation
  error.rs      GameError : toutes les raisons de refus
  common/       Player, Position, Direction
  board/        Board, Cell, Pion, PionId
  game/         Game (state.rs), déroulement des attaques (attack_flow.rs), ce qui dure
                (lasting.rs : ActiveEffect, EffectKind, Patient) et son horloge (clock.rs),
                tuer et ses suites (kill.rs), les jetons du Massacre divin et le
                cimetière (tokens.rs : TokenPile, TokenReward), les retours d'Interchange
                (returns.rs), les pas du Lièvre (hare.rs), la reine (queen.rs),
                ArsenalSlot, Action, Event, Decision, GameEnd, GameRecord
  rules/        règles pures qui lisent le plateau (alignement)
  rng.rs        Rng : générateur déterministe maison (SplitMix64), sans dépendance
  attacks/      attaques : données (Attack, AttackKind, Rarity, AttackStats), trait AttackRules,
                Effect, portées partagées (reach.rs), et une famille par fichier
                (ours.rs, beaute_fatale.rs, saut_du_loup.rs, reine.rs, attila.rs,
                medecin.rs, interchange.rs, lievre.rs, meilleurs_amis.rs, familia.rs)
```

### Le contrat d'une action : valider, puis exécuter

`Game::apply` appelle `validate` (lit l'état, ne modifie rien), puis `execute` (modifie l'état, ne vérifie plus et ne peut plus échouer). **Une action refusée ne modifie jamais la partie.** Le moteur ne plante jamais sur une entrée de client : toute erreur devient une `GameError`.

### Chaque changement d'état est annoncé

Tout ce qui change pour un observateur (un pion posé, tué, déplacé, un effet continu qui commence ou s'arrête, un pion qui entre à l'infirmerie ou en sort, des actions dépensées ou gardées, un tour qui se termine…) produit un `Event`. Le labo, Godot et les bots ne lisent que ces événements pour animer. C'est vérifié par l'invariant du miroir (principe 5).

### Le modèle de décision

Les attaques demandent plusieurs choix (carte, pion lanceur, cible, sauts enchaînés). Les réactions se jouent pendant le tour adverse, et le draft est une suite de choix. Au lieu d'une `Action` géante qui contiendrait tout, **le moteur expose à tout moment qui doit choisir quoi** :

- `pending_decision()` : la décision attendue et le joueur concerné. Aujourd'hui `Draft` (avant la partie, B9a), `MainAction`, `React` (la fenêtre de réaction, B8a), `ChooseLauncher` et `ChooseTarget`.
- `legal_actions()` : la liste exacte des actions acceptées maintenant, dans un ordre fixe. Elle est construite en passant chaque candidate par la validation elle-même : elle ne peut pas contredire `apply`. Un test le vérifie sur des parties aléatoires. Les candidates sont seulement celles qui peuvent répondre à la décision attendue (pendant une attaque : ses pions pour le lanceur, puis les cibles proposées par la famille), pour ne pas valider tout le plateau à chaque coup.

Chaque choix est une petite `Action` (`Place(pos)`, `UseAttack(slot)`, `Choose(pos)`, `ChooseEnemyTokens`, `FinishAttack`, `StepQueen(pos)`, `Shoot(pos)`, `HareStep { from, to }`, `Pass`, `Draft(attack)`, `EndTurn`). Il n'y a **pas d'action « annuler »** : une attaque commencée va au bout, et le moteur ne propose jamais un choix qui mène à une impasse. Les clients font la sélection sur une **copie** de la partie et n'envoient la suite qu'une fois complète : annuler reste gratuit pour le joueur, et aucun bot ne peut boucler. Avantages :

- le labo surligne les cases jouables sans connaître aucune règle ;
- les bots jouent en piochant dans `legal_actions()` ;
- pour le ML, l'espace d'actions est petit et fixe : environ 100 cases plus quelques boutons ;
- le replay reste une simple liste d'actions.

### Les attaques

- **Identité par données** : une `Attack` est une famille (`AttackKind`) dans une rareté (`Rarity`). Le coût et la recharge viennent du catalogue `attack_catalog` de `GameConfig`, jamais du code de la famille. Le coût n'est pas lié à la rareté. Les attaques de départ de chaque joueur (`starting_attacks`) sont aussi dans la configuration, en attendant le draft.
- **Le déroulement est commun à toutes les familles** (`game/attack_flow.rs`) : validation de la carte (recharge, coût, au moins une attaque complète), du lanceur (ni posé ce tour, ni immobilisé) et de chaque cible, puis résolution d'un bloc (coût payé, carte en recharge, effets appliqués). L'attaque est jouée dès qu'il n'y a plus rien à choisir, ou plus tôt avec `FinishAttack` quand elle est déjà complète. Une famille ne fournit que ses raretés jouables, son nombre de cibles (de `min` à `max`, `AttackRules::target_count`), ses cibles possibles et ses effets ; une attaque n'est **complète** que si son nombre de cibles est permis et qu'elle a au moins un effet (`attacks/rules.rs`) : le moteur ne propose jamais de coup sans effet.
- **Ce qu'une famille lit** passe par `AttackContext` : le plateau, les effets en jeu, et des lectures communes (`is_enemy`, `has`, `can_kill`, `can_act`), pour que toutes les familles lisent les statuts et les lanceurs de la même façon. Un lanceur prêt se définit à un seul endroit (`can_act`), partagé par la partie et par les familles qui font agir plusieurs pions. Les portées partagées (collé au lanceur, regard, ligne droite) sont dans `attacks/reach.rs`.
- **Ignorer le lanceur** : une variante dont les cibles sont n'importe où le déclare (`AttackRules::ignores_launcher`) ; savoir si elle a une cible ne demande alors qu'un lanceur prêt, au lieu de tous les essayer. Sans cela, une carte sans cible (le Médecin sans tombe) coûtait deux fois le temps du moteur. Un test parcourt des parties au hasard et vérifie chaque déclaration.
- **La reine** (`game/queen.rs`) : son pas et le tir de ses gardes sont des actions à part (`StepQueen`, `Shoot`), pas des cartes. Le bouclier d'un garde agit dans `kill` : un kill renvoie `Hit::Killed`, `Hit::Shielded` ou `Hit::Missed`, et le résolveur n'applique plus rien à un pion sauvé (`Effect::single_target`).
- **Les attaques à trajet** (sauts enchaînés, pas du Lièvre) rejouent les choix déjà faits sur une copie du plateau (`Walk` dans leur fichier) : c'est là qu'elles voient où en est le pion, qui est déjà mort ou survolé. Les effets s'appliquent ensuite dans l'ordre ; chaque case désigne le pion qui s'y trouve à ce moment-là. Une famille peut refuser de s'arrêter à un choix intermédiaire (`AttackRules::can_stop`).
- **Une famille par fichier** dans `attacks/`. Chaque famille implémente un trait interne sans état (les cibles légales, et les effets à produire), appelé via `AttackKind::rules()`. `Game` ne stocke que des données simples : pas de `Box<dyn>`, ce qui garde `Game` clonable, comparable et hachable ; la sauvegarde passe par `GameRecord`.
- **Les effets primitifs** (`Effect`) sont le cœur DRY. Une famille *décrit* ses effets. **Un résolveur central** les applique, émet les `Event` et déclenche les conséquences : infirmerie, tombe, vengeance de Meilleurs amis, fenêtres de réaction. Les attaques changent souvent, les primitives rarement. Catalogue établi le 2026-10-30 à partir des [fiches attaques](FICHES_ATTAQUES.md) ; chaque primitive arrive avec la première tâche qui en a besoin, et son nom de code est fixé à ce moment-là :

  | Primitive | Ce qu'elle fait | Cartes | Tâche |
  |---|---|---|---|
  | Tuer (`Effect::Kill`) | Le pion quitte le plateau, ses effets s'arrêtent, il part à l'infirmerie (ou au cimetière) et laisse une tombe ; un pion protégé ne peut pas être choisi comme cible | Ours, Saut du loup, Beauté fatale, Coup du taureau, tir de la reine, retour d'Interchange, Médecin, vengeance | B1 ; infirmerie en B3 |
  | Séduire (`Effect::Seduce`) | Tuer, puis poser un pion de la réserve sur la même case, seulement si le kill a eu lieu (pas après une esquive ni un garde) | Beauté fatale Épique et Légendaire | B5a ; B8c (esquive) |
  | Donner un statut (`Effect::Apply`) | Immobilisé ou protégé, pour N tours | Ours Commune et Légendaire, Beauté fatale, Garde royal, Saut du loup Commune, Médecin Épique, Familia, Sentinelle Rare | B3 |
  | Retirer des statuts (`Effect::RemoveEnemyStatuses`) | Ceux qu'a placés l'adversaire, sur un pion | Attila Commune | B3 |
  | Déplacer (`Effect::Move`) | Le pion change de case ; c'est la famille qui calcule le trajet (pas, saut, poussée, charge, téléportation) | Saut du loup, Lièvre, Coup du taureau, Interchange Commune, reine, Feintaille, Hibou | B4 |
  | Échanger (`Effect::Swap`) | Deux pions échangent leurs cases d'un coup | Interchange | B4 |
  | Poser (`Effect::Place`), ressusciter (`Effect::Resurrect`) | Un pion de la réserve, ou de l'infirmerie (le Médecin, sur sa tombe), arrive sur une case et compte comme posé ce tour | Pose, Beauté fatale, Reine, Familia, Meilleurs amis, Médecin, Sentinelle, Queue du lézard | B5a (réserve) ; B5b (infirmerie) |
  | Créer un effet continu | Couronne, gardes (`Effect::Crown`, B5c), pacte (`Effect::Bind`, B5a), enfant (`Effect::MakeChild`, B5d), retour programmé et jetons de pas (`Effect::Attach`, B6c) | Reine, Lièvre Épique, Meilleurs amis, Interchange Commune, Familia, Attila Légendaire | B3 à B6 |
  | Terre brûlée (`Effect::ScorchEarth`) | Pendant son tour suivant, l'adversaire ne peut jouer aucun coup dont un effet crée un effet continu (`Effect::creates_lasting`) : `is_complete` le refuse, via `AttackContext::may_create_effects` | Attila Légendaire | B6d |
  | Creuser (`Effect::Dig`) | Une case de mur devient une case normale, pour toujours ; le lanceur y entre ensuite par un `Effect::Move` | Coup du taureau | B7 |
  | Annuler un effet (`Effect::CancelEnemyEffects`, `Effect::CancelEnemyTokens`) | Les effets continus adverses entiers qui touchent un pion ; toutes les piles de jetons adverses | Attila Rare à Légendaire | B6a (Rare, Épique) ; B6b (jetons) ; B6d (Légendaire) |
  | Poser des jetons (`Effect::AddTokens`) | Une pile devant le joueur ; ses prochains kills usent un jeton de chaque pile : points rendus tout de suite, ou cimetière (`game/tokens.rs`, appelé par `kill`) | Massacre divin | B6b |

  - **Pas de « changer de camp »** : séduire (Beauté fatale), c'est tuer puis poser un des siens sur la même case (`Effect::Seduce`, une seule primitive pour que la pose dépende du kill). Ressusciter sur une tombe occupée (Médecin), c'est tuer l'occupant puis poser depuis l'infirmerie. Le retour d'Interchange, c'est tuer l'occupant puis déplacer.
  - **Un effet continu** (`ActiveEffect`) retient qui l'a créé, la carte qui l'a créé, ce sur quoi il porte, et sa fin. Depuis B3 : il porte sur un pion (par son identité), et sa fin est une durée (`last_turn`, le numéro du dernier tour où il agit). Depuis B5a : un effet peut n'avoir aucune durée (le pacte, `last_turn() == None`) ; il ne compte alors pas comme « programmé » pour la partie figée. Depuis B6a : le **jeu de carte** qui l'a créé (`play`, un `PlayId` donné à chaque attaque résolue) ; tous les effets d'un même jeu forment l'« effet entier » qu'Attila Rare et Épique annulent (`Effect::CancelEnemyEffects`). Depuis B6c, sa fin est un seul type, `EffectEnd` : une durée (`Turn`), le début du prochain tour de son créateur (`NextTurn`, le retour d'Interchange, joué par `returns.rs` avant la règle d'or), une condition (`Framed`), des jetons (`Tokens`, le Lièvre Épique, dépensés par `hare.rs`, annoncés par `Event::EffectChanged`), ou rien (`Never`). Les jetons du Massacre divin (B6b) ne sont **pas** des effets continus : ils sont posés devant un joueur, pas sur un pion, et ne finissent ni par une durée ni par une condition du plateau, mais quand ses kills les usent. Ce sont des piles à part (`TokenPile`, `game/tokens.rs`), avec leur jeu de carte (`play`) : chaque pile reste un effet entier pour Attila. Depuis B5d : une condition, la protection d'un enfant de la Familia, qui retient les deux pions qui l'encadrent (`framed_by`) ; la partie la retire à la fin de chaque action où ils ne l'encadrent plus (`end_unframed_children`, appelé par `after_action`). Un statut est un effet continu : un pion est immobilisé tant qu'au moins un effet l'immobilise, donc pas de règle de cumul.
  - **Tuer et ses conséquences** (`game/kill.rs`) : un kill arrête les effets du pion, use un jeton de chaque pile du Massacre divin de l'adversaire du pion tué (`use_tokens`), l'envoie au cimetière, à l'infirmerie ou dans la réserve, et note son pacte ; à la fin de l'attaque, `avenge` joue les vengeances, en chaîne. La tombe est posée au départ pour l'infirmerie (`clock.rs`, `send_to_infirmary`). Une conséquence de plus (Queue du lézard) se branchera ici. La défaite par manque de pions se vérifie après chaque action (`after_action`, `lack_of_pions`).
  - **Une seule horloge** (`game/clock.rs`) : une durée se compte en tours d'un joueur (l'adversaire de celui qui a joué l'effet ; pour l'infirmerie, le propriétaire du pion), sans le tour en cours, et s'éteint à la fin du dernier (carnet, 2026-10-30). Le dernier tour est calculé une fois, à la création : rien ne décompte à chaque tour, et la fin d'un tour arrête simplement ce dont c'était le dernier tour.
  - **Déclencheurs** auxquels réagissent les effets continus et les réactions : pion tué, pion posé, pion déplacé, pion visé (avant la résolution), attaque terminée, début de tour, fin de tour.
  - **Portées** (où sont les cibles), chaque famille dit la sienne, sans portée par défaut : collé au lanceur, regard (en ligne droite jusqu'à N cases, sans pion entre), ligne de tir (le premier pion rencontré), n'importe où. **Trajets** (comment un pion bouge) : pas, saut, charge.
  - **Le mur** (B7) est dans le plateau, pas dans les cases : `Board` connaît la grille entière (`size()`, mur compris) et ses cases de mur, qui ne sont pas sur le plateau (`contains`, `get`, `neighbor`, `positions` les ignorent). Toutes les portées, les trajets et les alignements s'arrêtent donc au mur sans le savoir ; seul le Coup du taureau le regarde (`Board::is_wall`) pour le creuser.
  - **Un déplacement impossible arrête l'attaque** (B7) : si un `Effect::Move` ne peut pas se faire (une reine sauvée par un garde occupe encore sa case), le résolveur ne joue pas la suite des effets. C'est ce qui arrête une charge devant la reine.
- **Les nombres d'une famille** (portée, nombre de cibles, durée) sont des constantes nommées dans son fichier. Ils passeront dans la configuration quand l'équilibrage par bots en aura besoin (principe 6).

### Les pions et les cases

- Chaque pion (`Pion`) a une **identité stable `PionId`**, donnée dans l'ordre des poses à partir de 0. Un effet en cours vise un pion, pas une position, car un pion peut être attiré, échangé ou téléporté. Seul le moteur crée des pions ; les clients les lisent dans `Event::PionPlaced` ou sur le plateau.
- Une case (`Cell`) est une **structure en couches** : un pion optionnel et, depuis B5b, une tombe optionnelle  ; le mur n'est pas une couche (voir « Le mur », plus haut). Ce n'est pas un enum exclusif : une tombe n'empêche pas un pion d'être sur la même case. Le plateau ne s'écrit que **pion par pion** (`place_pion`, `remove_pion`, `move_pion`, `swap`) : une tombe reste sur sa case quand les pions bougent.

### La victoire et la fin de partie

- Une ligne de `align_to_win` pions **faite en posant** gagne tout de suite, sans la règle d'or.
- **Règle d'or** (`golden_rule`, en option) : une ligne ne gagne qu'au début du tour de son propriétaire. Le contrôle « une ligne au début de ton tour gagne » existe dans tous les cas : il couvre les lignes faites par une carte, sans avoir à suivre l'origine d'une ligne. Quand la partie s'arrête (figée, limite de tours), les lignes en attente sont vérifiées comme si les tours continuaient (`stop_game`).
- **Fin garantie** : toute partie se termine (`GameEnd`), soit par alignement, soit **figée** (plus personne ne pourra jamais agir), soit par la **limite de tours** `max_turns`, filet de sécurité pour les bots.
- Trois choses sont séparées pour rester modifiables : le **déclencheur** (`EndReason`, utile aux statistiques), le **résultat** (`Outcome`, calculé à un seul endroit : nul pour l'instant, départage possible plus tard) et la **limite** (réglage).
- « Figée » veut dire *plus personne ne peut agir et rien de programmé ne peut changer la situation*. Depuis B3, un effet continu en jeu ou un pion à l'infirmerie compte comme « programmé » ; la vérification se fait aussi à la fin d'un tour où quelque chose a expiré. Un effet sans fin dans le temps (pacte, couronne, gardes, enfant) ne compte pas : seule une action peut le défaire. Une future mécanique anti-stagnation (Effondrement) devra être prise en compte dans ce test.

## 5. Bots et machine learning

Les bots de base et l'arène existent depuis le 2026-10-27. Ils ont été avancés avant B2 pour mesurer chaque réglage au lieu de le deviner. Le machine learning reste prévu après la v1.0.

### Les bots, des clients comme les autres

- **Aucune règle recopiée** : un bot choisit parmi `legal_actions()`. Pour savoir ce que donne une attaque, il la joue jusqu'au bout sur une **copie** de la partie (`AttackPlan`) ; pour connaître les actions du prochain tour adverse, il finit son tour sur une copie. Une nouvelle famille d'attaques, ou un changement du début de tour, est donc pris en compte sans rien changer dans `bots/`. Aucune lecture de règle de légalité : les poses restantes viennent du moteur (`Game::placements_left`), pour le tour en cours ou, sur une copie, pour le suivant. Seule hypothèse, pour jouer et non pour décider de la légalité : `Windows` suppose qu'une ligne gagnante est faite de `align_to_win` cases consécutives sur un axe. Une règle d'or différente ne cassera pas un bot, il jouera seulement moins bien.
- **Sans état** : tout vient de la partie, et le hasard du générateur que l'arène passe au bot, initialisé avec la graine de la partie. Une partie entre bots se rejoue à l'identique, et le même appel de l'arène donne toujours le même bilan.
- **Lecture en fenêtres** (`Windows`) : la géométrie vient du moteur (`Direction::AXES`, `Board::neighbor`) et la longueur de `align_to_win`. Aucune taille n'est écrite en dur.
- **L'arène** (`bots::run`) joue N parties entre deux bots et rend un `Report` ; `run_pairs` fait de même pour plusieurs paires en parallèle, et `record_game` rend une partie précise sous forme de `GameRecord`. Une action refusée par le moteur abandonne la partie et apparaît dans le bilan (« refus ») : c'est toujours un bug.
- **Une partie de l'arène = configuration + graine + deux bots.** `start_game` initialise le moteur et le hasard des bots avec la même graine ; le labo commence ses parties par lui, et revoit donc exactement les parties de l'arène. Un test du labo le vérifie, y compris après une annulation.
- **Les réglages prêts à comparer** vivent dans `bots::presets` : les bots disponibles (`BotKind`), les paires de l'arène (`STANDARD_PAIRS`) et les variantes (`variants()`). L'exemple `arene` et le labo lisent les mêmes listes : une variante ajoutée là apparaît partout. Une règle qui n'est pas encore un réglage (départage, fin sur passes répétées…) doit d'abord devenir une option de la configuration pour être mesurée.

### Préparé pour la suite

Le moteur est conçu dès maintenant pour que les bots soient simples et rapides :

- `Game` est `Clone` et petit : un bot de recherche peut copier l'état pour essayer un coup ;
- tout est déterministe, et une partie se rejoue depuis son `GameRecord` ;
- `legal_actions()` donne directement l'espace d'actions ;
- `PlayerView` donne l'observation d'un joueur, sans triche possible sur l'information cachée ;
- le membre `bots/` contiendra l'encodage observation → tenseur et action ↔ indice, pour ne pas mettre de code ML dans le moteur ;
- la mesure `cargo run --release -p fight_of_pions --example random_games` (parties aléatoires par seconde, notée dans la feuille de route) sert de référence de performance. On n'optimise rien sans elle.

## 6. Les clients

Règles communes à tous les clients :

- aucune règle de jeu : la légalité vient de `legal_actions()` ou d'une `GameError` ;
- un seul endroit appelle `game.apply` (dans le labo : `partie.rs`) ;
- l'affichage du journal vient des `Event`, pas d'une relecture de l'état ;
- une seule conversion position ↔ écran (dans le labo : `plateau.rs`) ;
- aucun nombre en dur : la taille vient de `GameConfig`, l'apparence d'un fichier de thème.

### Le labo

Le labo réunit tout ce qu'on veut voir à l'écran, en deux écrans, sans contenir ni règle ni logique de bot :

- **Partie** : jouer, seul ou contre un bot, ou regarder deux bots. Chaque joueur est contrôlé par un humain ou un bot (`BotKind`) ; les bots jouent seuls (lecture automatique, attente réglable) ou coup par coup. Un coup de bot passe par `partie.rs`, comme un clic. Au tour d'un bot, les boutons de jeu sont désactivés (et `app.rs` refuse quand même tout geste de jeu). Face à un bot, « Annuler le coup » remonte jusqu'au dernier coup d'un humain, et `partie.rs` remet le hasard des bots tel qu'il était : ils rejouent les mêmes coups. La section « Prochaine partie » règle la configuration de la prochaine partie (voir le réglage de partie, plus bas). Le panneau défile en entier. Les bots jouent dans le fil de l'interface : sur un très grand plateau, la fenêtre répond mal pendant la lecture automatique.
- **Arène** : régler la configuration simulée, choisir des paires et un nombre de parties, lancer `bots::run_pairs` dans un fil à part (la fenêtre ne se fige pas) et lire les bilans, affichés comme par `cargo arene`. « Revoir » ouvre une partie précise dans l'écran Partie, avec sa graine et ses bots, pour la regarder se jouer.
- **Réglage de partie** (`reglage.rs`) : un seul éditeur de configuration, utilisé par les deux écrans (la prochaine partie, la configuration simulée). On part d'une variante ou de la partie en cours, puis on retouche chaque réglage ; le labo ne vérifie rien lui-même, il affiche le verdict de `GameConfig::validate` et ne lance pas une configuration incohérente.
- Le labo tourne en debug, mais le moteur et les bots sont optimisés dans le profil dev (`Cargo.toml`) : l'écran Arène simule à la vitesse de `cargo arene`. Chaque bilan affiche ses fourchettes à 95 % : deux bilans dont les fourchettes se chevauchent largement ne permettent pas de conclure.

### Le client Godot

Le jeu lui-même, dans `client-godot/` (Godot 4.7, GDScript typé, rendu Compatibilité pour le web et le mobile). Il joue en ligne par le serveur ([PROTOCOLE.md](PROTOCOLE.md)) et ne contient **aucune règle** : il affiche la vue que le serveur envoie et ne propose que les coups de `legal_actions`, renvoyés tels quels. Découpage en étapes : feuille de route, E3. Noms du code en anglais (sauf « Pion »), commentaires et texte affiché en français.

```
client-godot/
  main.gd / main.tscn   la racine : le plateau 3D, la couche d'interface, l'écran affiché ;
                        elle enchaîne les écrans selon ce que répond le serveur
  net/server_link.gd    (autoload ServerLink) la WebSocket, la place gardée et sa reprise
  net/wire.gd           le format du fil (JSON, nombres entiers remis en entiers, valeurs
                        étiquetées `{"Place": …}` lues par `tag` et `content`)
  game/view_data.gd     la vue du serveur, avec des lectures typées et les coups rangés par case
  game/game_events.gd   la lecture des événements joints à une vue (pour animer)
  game/card_catalog.gd  le catalogue des cartes reçu du serveur (noms et textes)
  game/choice_session.gd  le choix d'une carte en cours, par aperçus du serveur
  game/game_journal.gd  le journal des coups, raconté à partir des événements
  board/                le plateau 3D : grid_geometry.gd (la seule conversion case ↔ monde),
                        board_3d.gd (cases, caméra, toucher), pion_3d.gd, tomb_3d.gd,
                        crown_3d.gd, pion_status.gd (ce que les effets donnent à un pion),
                        board_timeline.gd (le déroulé d'une vue, lu dans ses événements),
                        badge_layer.gd (les badges, en 2D par-dessus), board_materials.gd,
                        shaders/
  ui/                   ui.gd (les briques), components/ (panneau de joueur, losanges,
                        cartes, arsenal, choix, menu de case, journal, annonce d'une
                        carte adverse, cache révélée, fenêtre de réaction),
                        screens/ (démarrage, accueil, partie)
  theme/look.gd         l'apparence : couleurs, tailles, marges et durées (comme theme.rs)
  theme/theme_builder.gd  le thème Godot, construit à partir de `Look`
  text/labels.gd        tout le texte affiché
  assets/               polices (licence OFL jointe) et SVG copiés de design/elements
```

- **Aucune règle.** La légalité vient de `legal_actions` : `ViewData` range les coups par case dans l'ordre du serveur, et l'écran de partie, seul endroit qui transforme un geste en coup, prend le coup tel quel dans cette liste. Ce qui est affiché vient de la vue (réserve, points, banque, poses), jamais d'un calcul.
- **Un seul chemin vers le serveur** : `ServerLink`. Il garde la place (code, joueur, clé) dans `user://session.cfg` et la reprend toute seule après une coupure ou au lancement suivant (`rejoin`). Une salle par connexion : quitter sa salle ouvre une connexion neuve (`leave_room`). Les demandes faites hors connexion sont perdues, sauf créer et rejoindre : la vue reçue au retour fait foi.
- **Le plateau est en vraie 3D** (décision du 2026-11-04), filmé de face et en plongée par une caméra orthographique (essai du 2026-11-04 : le plateau droit plutôt qu'en losange, à la demande de Martin ; les maquettes le montraient en isométrique 2:1). Les cases sont des pavés, dessinés en un appel par sorte (`MultiMeshInstance3D`) ; J1 est un cylindre et J2 un prisme à 8 faces, la même primitive : la forme distingue les joueurs. Tout est peint sans éclairage, par des shaders qui donnent à chaque face sa couleur et à chaque arête son trait, comme les éléments fournis : net à tout zoom, léger sur téléphone. Un modèle plus riche pourra remplacer le corps d'un pion sans toucher au reste.
- **Lisible sur un téléphone.** Du mur, on ne voit que les cases qui touchent le plateau. La caméra cadre les cases jouables, plus une demi-case, dans le rectangle que l'écran laisse au plateau, marges de la page comprises. Face au plateau, une case occupe deux fois plus de place à l'écran qu'en losange, et les lignes se lisent comme sur un damier. Parmi deux plongées (45° et 55°), la caméra prend celle qui donne les plus grandes cases : en portrait, où la largeur manque et la hauteur abonde, c'est 55° (environ 38 × 31 points par case). Les palets gardent leur relief ; les cases, vues de face, en montrent moins.
- **Portrait d'abord, paysage sur PC.** En portrait : l'adversaire en haut, le tour, le plateau, sa barre, puis le bouton d'action sur toute la largeur, sous le pouce. En paysage : le plateau à gauche, une colonne à droite. Les mêmes éléments changent seulement de boîtes.
- **Un système visuel unique.** Les couleurs et les tailles sont dans `look.gd` (`Look`), et nulle part ailleurs ; `ThemeBuilder` en fait le thème, et les composants choisissent un style par son nom de variante. Les boutons d'action sont dessinés (`StyleBoxHex`), pas étirés depuis une image, pour rester nets à toute taille. L'interface et le plateau sont construits dans le code, pour que tout parte de `Look` ; les scènes `.tscn` se limitent à la racine.
- **Des retours sans attendre le serveur, sans règle.** Ce qui est montré avant la réponse vient seulement de la liste des coups permis :
  - le palet fantôme au survol ;
  - le palet qui flotte au-dessus de la case touchée ;
  - le refus d'une case où rien n'est permis : le pion tremble, ou un voile terne couvre la case vide.

  La vue qui arrive fait toujours foi. Une vue sans événement, comme à la reprise d'une partie, s'affiche sans animation.
- **Ce qui se passe s'anime** (E3d2), d'après les seuls événements joints à la vue :
  - le déroulé (`BoardTimeline`) date chaque animation dans l'ordre des événements : éclat d'or de l'attaque lancée, tir de la reine (une balle en cloche), pions tués (ils gonflent, puis se ratatinent en s'élevant, dans un anneau rouge), tombes qui sortent du sol, sauts en arc et échanges, mur creusé, poses. Des événements de même sorte qui se suivent jouent ensemble ; une autre sorte attend la fin de la précédente. Les statuts changent à la fin du déroulé ;
  - l'écran de partie fait passer les vues par une file : chacune attend que la précédente ait fini de s'animer, car le bot joue sans pause. Tant que des vues attendent, aucun geste n'est pris : la vue affichée est périmée ;
  - une carte envoyée coup par coup ne montre que sa dernière vue, avec les événements de toutes.
- **Le tour adverse** (E3e) : une carte lancée par l'adversaire est annoncée en haut de l'écran (`AttackBanner`) ; sa cache, quand il la lance, est révélée en grand jusqu'à un toucher (`RevealOverlay`, la file des vues attend) ; quand on peut réagir (décision `React`), la fenêtre de réaction (`ReactionPanel`) remplace l'arsenal : le dernier coup adverse, ses réactions, « PASSER » ou « JOUER … » ; pendant que l'adversaire décide, une pastille le dit (« Le bot réfléchit… »). Tout vient des événements et des coups permis.
- **Le journal des coups** (E3d3) raconte la partie à partir des seuls événements (`GameJournal`) : qui pose, qui lance quelle carte et depuis où, les pions tués et déplacés, les statuts posés, le mur creusé, les jetons, Terre brûlée. Les cases s'écrivent comme le moteur les nomme (« C5 », sur toute la grille, mur compris) et les cartes avec les noms du catalogue. En paysage, il occupe une colonne à gauche du plateau ; en portrait, un bouton à côté du tour l'ouvre en page. Il commence avec la connexion : à la reprise d'une partie, le serveur n'envoie pas les coups passés.
- **Ce qui dure se voit** (E3d1), tout lu dans la vue :
  - sur un pion, d'après les effets continus qui le visent (`PionStatus`) : un anneau à ses pieds pour son statut le plus fort (immobilisé : chaîne grise ; protégé : or ; pacte : tirets magenta), la taille d'un enfant (Familia), une couronne en 3D pour une reine ;
  - au-dessus de lui, ses badges (`BadgeLayer`) : un par sorte d'effet (immobilisé, protégé ou enfant, pacte, retour d'Interchange), puis un losange chiffré pour les gardes d'une reine et les jetons de pas du Lièvre. Les badges sont en 2D, dans un calque entre le plateau et l'interface : nets, et d'une taille qui suit celle des cases à l'écran, entre deux bornes ;
  - dans le panneau de chaque joueur, une ligne par chose qui dure : Terre brûlée (jusqu'à quel tour, et un bord de braise en pointillés quand il attend), gardes de sa reine, piles de jetons du Massacre divin, pions à l'infirmerie et au cimetière.
- **Jouer une carte sans règle** (protocole, « Le déroulé »). Toucher une carte vive (son `UseAttack` est permis) ouvre un choix (`ChoiceSession`) : la liste des coups choisis, et l'aperçu que le serveur rend pour elle. Les cases proposées sont celles des coups `Choose` de cet aperçu ; « JOUER LA CARTE » et « VISER LES JETONS » ne s'allument que si l'aperçu propose `FinishAttack` ou `ChooseEnemyTokens`. Quand l'aperçu répond `null`, la carte part coup par coup ; annuler, c'est oublier la liste. Un toucher pendant un choix s'aimante à la case proposée la plus proche : viser une petite case au doigt est difficile, et seules ces cases comptent alors.
- **Une case, plusieurs coups.** Les coups permis sont rangés par case (le pas du Lièvre sous sa case de départ). Une case où seule la pose est permise pose tout de suite ; sinon, un petit menu propose chaque sorte de coup, et le pas du Lièvre demande ensuite sa case d'arrivée.
- **Les écrans** : démarrage, accueil (bot, créer une salle, rejoindre), rejoindre (le code), draft, partie. Une vue dont la décision est un choix de draft s'affiche sur l'écran du draft (`DraftScreen`), construit sur les coups permis et le format de la partie ; la partie commence quand le draft finit.
- **Pendant le développement**, le bot est le bot aléatoire, qui ne charge pas la machine ; la variante est celle du jeu en ligne, choisie par le serveur (draft compris, depuis E3f).
- **L'outil d'éditeur** `addons/godot_mcp_toolkit/` (greffon tiers, avec son autoload `MCPRuntimeServer`) permet à Claude de piloter l'éditeur et le jeu lancé depuis l'éditeur (captures, touchers simulés). Il ne fait pas partie du jeu : le code du client ne s'en sert jamais, et son serveur ne s'ouvre que lancé depuis l'éditeur (`OS.has_feature("editor")`), jamais dans un export. Il sera exclu des exports avec E3h.
