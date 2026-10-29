# Architecture de Fight of Pions

Dernière mise à jour : 2026-10-29.

Ce document décrit comment le code est organisé et **pourquoi**. Il sert de référence pour toutes les tâches de la [feuille de route](FEUILLE_DE_ROUTE.md). Ce qui n'existe pas encore est marqué **(cible)**.

Les règles du jeu sont dans le GDD (document de design partagé avec Théo, hors dépôt, v0.2) et, pour ce que le moteur implémente réellement, dans le [carnet de règles](REGLES.md). Les noms viennent du [glossaire](GLOSSAIRE.md).

---

## 1. Le but

Un **moteur de règles indépendant**, sur lequel se branchent tous les clients sans jamais en contenir une seule règle :

```
                 ┌────────────┐
   terminal ───▶ │            │
   labo     ───▶ │   moteur   │  ◀── bots (aléatoire, glouton ; ML en cible)
   Godot    ───▶ │ (sans dép.)│  ◀── py_bindings pour l'entraînement (cible)
   (cible)       └────────────┘
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
7. **La vue joueur** (`PlayerView`, cible). Dès qu'il y a de l'information cachée (cache secrète), le moteur fournit une vue restreinte par joueur. Les bots et le jeu en ligne ne voient que celle-ci.
8. **Les règles avant le code.** Une attaque n'est codée que lorsque sa règle est écrite et datée dans le [carnet de règles](REGLES.md). Les attaques « EN REFONTE » dans le GDD attendent.

## 3. Le workspace

| Membre | Rôle | Dépendances |
|---|---|---|
| `moteur/` (paquet `fight_of_pions`) | Toutes les règles. Bibliothèque. | Aucune ; feature `serde` optionnelle (sauvegarde JSON d'un `GameRecord`) |
| `labo/` | Client de test eframe/egui, en deux écrans. **Partie** : panneau, damier cliquable avec cases jouables, journal, graine, annuler, coups au hasard, copier/charger une partie, chaque joueur humain ou bot, configuration de la prochaine partie réglable. **Arène** : simuler des parties entre bots sur une configuration réglable, et en revoir une. | `moteur` (avec `serde`), `bots`, `eframe`, `serde_json` |
| `terminal/` | Client texte. | `moteur` |
| `bots/` | Bots (aléatoire, glouton) et arène de parties automatiques. Bibliothèque ; l'arène est l'exemple `arene`. | `moteur` |
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
  game/         Game (state.rs), déroulement des attaques (attack_flow.rs), ArsenalSlot,
                Action, Event, Decision, GameEnd, GameRecord
  rules/        règles pures qui lisent le plateau (alignement)
  rng.rs        Rng : générateur déterministe maison (SplitMix64), sans dépendance
  attacks/      attaques : données (Attack, AttackKind, Rarity, AttackStats), trait AttackRules,
                Effect, et une famille par fichier (ours.rs)
```

### Le contrat d'une action : valider, puis exécuter

`Game::apply` appelle `validate` (lit l'état, ne modifie rien), puis `execute` (modifie l'état, ne vérifie plus et ne peut plus échouer). **Une action refusée ne modifie jamais la partie.** Le moteur ne plante jamais sur une entrée de client : toute erreur devient une `GameError`.

### Chaque changement d'état est annoncé

Tout ce qui change pour un observateur (un pion posé, tué, déplacé, qui change de camp, des actions dépensées ou gardées, un tour qui se termine…) produit un `Event`. Le labo, Godot et les bots ne lisent que ces événements pour animer. C'est vérifié par l'invariant du miroir (principe 5).

### Le modèle de décision

Les attaques demandent plusieurs choix (carte, pion lanceur, cible, sauts enchaînés). Les réactions se jouent pendant le tour adverse, et le draft est une suite de choix. Au lieu d'une `Action` géante qui contiendrait tout, **le moteur expose à tout moment qui doit choisir quoi** :

- `pending_decision()` : la décision attendue et le joueur concerné. Aujourd'hui `MainAction`, `ChooseLauncher` et `ChooseTarget` ; viendront la fenêtre de réaction et le draft.
- `legal_actions()` : la liste exacte des actions acceptées maintenant, dans un ordre fixe. Elle est construite en passant chaque candidate par la validation elle-même : elle ne peut pas contredire `apply`. Un test le vérifie sur des parties aléatoires.

Chaque choix est une petite `Action` (`Place(pos)`, `UseAttack(slot)`, `Choose(pos)`, `EndTurn`, plus tard `Pass`…). Il n'y a **pas d'action « annuler »** : une attaque commencée va au bout, et le moteur ne propose jamais un choix qui mène à une impasse. Les clients font la sélection sur une **copie** de la partie et n'envoient la suite qu'une fois complète : annuler reste gratuit pour le joueur, et aucun bot ne peut boucler. Avantages :

- le labo surligne les cases jouables sans connaître aucune règle ;
- les bots jouent en piochant dans `legal_actions()` ;
- pour le ML, l'espace d'actions est petit et fixe : environ 100 cases plus quelques boutons ;
- le replay reste une simple liste d'actions.

### Les attaques

- **Identité par données** : une `Attack` est une famille (`AttackKind`) dans une rareté (`Rarity`). Le coût et la recharge viennent du catalogue `attack_catalog` de `GameConfig`, jamais du code de la famille. Le coût n'est pas lié à la rareté. Les attaques de départ de chaque joueur (`starting_attacks`) sont aussi dans la configuration, en attendant le draft.
- **Le déroulement est commun à toutes les familles** (`game/attack_flow.rs`) : validation de la carte (recharge, coût, au moins une attaque complète), du lanceur et de chaque cible, puis résolution d'un bloc au dernier choix (coût payé, carte en recharge, effets appliqués). Une famille ne fournit que ses raretés jouables, son nombre de cibles, ses cibles possibles et ses effets.
- **Une famille par fichier** dans `attacks/`. Chaque famille implémente un trait interne sans état (les cibles légales, et les effets à produire), appelé via `AttackKind::rules()`. `Game` ne stocke que des données simples : pas de `Box<dyn>`, ce qui garde `Game` clonable, comparable et hachable ; la sauvegarde passe par `GameRecord`.
- **Les effets primitifs** (`Effect`) sont le cœur DRY : tuer, déplacer, échanger, poser, changer de camp, ajouter un effet en cours, creuser un trou, gagner des actions. Une famille *décrit* ses effets. **Un résolveur central** les applique, émet les `Event` et déclenche les conséquences : retour en réserve, tombe, vengeance de Meilleurs amis, fenêtres de réaction. Les attaques changent souvent, les primitives rarement.
- **Les nombres d'une famille** (portée, nombre de cibles, durée) sont des constantes nommées dans son fichier. Ils passeront dans la configuration quand l'équilibrage par bots en aura besoin (principe 6).

### Les pions et les cases

- Chaque pion (`Pion`) a une **identité stable `PionId`**, donnée dans l'ordre des poses à partir de 0. Un effet en cours vise un pion, pas une position, car un pion peut être attiré, échangé ou téléporté. Seul le moteur crée des pions ; les clients les lisent dans `Event::PionPlaced` ou sur le plateau.
- Une case (`Cell`) est une **structure en couches** : aujourd'hui un pion optionnel ; le terrain (trou, tombe…) s'ajoutera quand une attaque en aura besoin. Ce n'est pas un enum exclusif : une tombe n'empêche pas un pion d'être sur la même case.

### La victoire et la fin de partie

- Une ligne de `align_to_win` pions **faite en posant** gagne tout de suite, sans la règle d'or.
- **Règle d'or** (`golden_rule`, en option) : une ligne ne gagne qu'au début du tour de son propriétaire. Le contrôle « une ligne au début de ton tour gagne » existe dans tous les cas : il couvre les lignes faites par une carte, sans avoir à suivre l'origine d'une ligne. Quand la partie s'arrête (figée, limite de tours), les lignes en attente sont vérifiées comme si les tours continuaient (`stop_game`).
- **Fin garantie** : toute partie se termine (`GameEnd`), soit par alignement, soit **figée** (plus personne ne pourra jamais agir), soit par la **limite de tours** `max_turns`, filet de sécurité pour les bots.
- Trois choses sont séparées pour rester modifiables : le **déclencheur** (`EndReason`, utile aux statistiques), le **résultat** (`Outcome`, calculé à un seul endroit : nul pour l'instant, départage possible plus tard) et la **limite** (réglage).
- « Figée » veut dire *plus personne ne peut agir et rien de programmé ne peut changer la situation*. Une future mécanique anti-stagnation (Effondrement) devra être prise en compte dans ce test.

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
