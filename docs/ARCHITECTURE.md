# Architecture de Fight of Pions

Dernière mise à jour : 2026-10-26.

Ce document décrit comment le code est organisé et **pourquoi**. Il sert de référence pour toutes les tâches de la [feuille de route](FEUILLE_DE_ROUTE.md). Ce qui n'existe pas encore est marqué **(cible)**.

Les règles du jeu sont dans le GDD (document de design partagé avec Théo, hors dépôt, v0.2) et, pour ce que le moteur implémente réellement, dans le [carnet de règles](REGLES.md). Les noms viennent du [glossaire](GLOSSAIRE.md).

---

## 1. Le but

Un **moteur de règles indépendant**, sur lequel se branchent tous les clients sans jamais en contenir une seule règle :

```
                 ┌────────────┐
   terminal ───▶ │            │
   labo     ───▶ │   moteur   │  ◀── bots (aléatoire, glouton, ML)   (cible)
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
2. **Une partie = configuration + graine + liste d'actions** (`GameRecord`, cible). Rejouer les actions reconstruit la partie. Cela donne gratuitement la sauvegarde, le replay, l'annulation dans le labo et la reproduction exacte d'un bug.
3. **Le jeu piloté par les données.** Tous les réglages chiffrés vivent dans `GameConfig`, validés au démarrage. Plus tard, ils seront chargés depuis un fichier pour équilibrer sans recompiler.
4. **Des tranches verticales.** Une fonctionnalité est finie quand elle marche de bout en bout : moteur, tests, labo. On ne commence pas la suivante avant.
5. **Des invariants vérifiés automatiquement.** Le principal : *un miroir du plateau, construit uniquement à partir des `Event`, est toujours égal à `game.board()`*. C'est la garantie qu'un client qui suit les événements ne se désynchronise jamais. Il est vérifié après chaque action de centaines de parties aléatoires.
6. **Abstraire à la deuxième occurrence, pas avant.** La modularité vient des bonnes abstractions de base (les effets primitifs, le modèle de décision, les données), pas d'options prévues pour des besoins imaginaires.
7. **La vue joueur** (`PlayerView`, cible). Dès qu'il y a de l'information cachée (cache secrète), le moteur fournit une vue restreinte par joueur. Les bots et le jeu en ligne ne voient que celle-ci.
8. **Les règles avant le code.** Une attaque n'est codée que lorsque sa règle est écrite et datée dans le [carnet de règles](REGLES.md). Les attaques « EN REFONTE » dans le GDD attendent.

## 3. Le workspace

| Membre | Rôle | Dépendances |
|---|---|---|
| `moteur/` (paquet `fight_of_pions`) | Toutes les règles. Bibliothèque. | Aucune (`serde` optionnel, cible) |
| `labo/` | Client de test eframe/egui : panneau, damier cliquable, journal. | `moteur`, `eframe` |
| `terminal/` (cible, aujourd'hui `moteur/src/main.rs`) | Client texte. | `moteur` |
| `bots/` (cible) | Bots et arène de parties automatiques. | `moteur` |
| `py_bindings/` (cible) | Environnement Python pour l'entraînement ML. | `moteur`, `pyo3` |
| `godot_bridge/` (cible) | Pont vers Godot. | `moteur`, `godot` |

Commandes :

- `cargo test` : tests du moteur, sans jamais compiler eframe ;
- `cargo run` : le terminal ;
- `cargo labo` : le labo.

## 4. Le moteur

### Organisation

```
moteur/src/
  lib.rs        API publique, lints stricts (pas d'unwrap/expect/panic hors tests)
  config.rs     GameConfig et sa validation
  error.rs      GameError : toutes les raisons de refus
  common/       Player, Position, Direction
  board/        Board, Cell
  game/         Game, Action, Event
  rules/        règles pures qui lisent le plateau (alignement)
  rng.rs        (cible) générateur déterministe maison, sans dépendance
  attacks/      (cible) attaques : une famille par fichier
```

### Le contrat d'une action : valider, puis exécuter

`Game::apply` appelle `validate` (lit l'état, ne modifie rien), puis `execute` (modifie l'état, ne vérifie plus et ne peut plus échouer). **Une action refusée ne modifie jamais la partie.** Le moteur ne plante jamais sur une entrée de client : toute erreur devient une `GameError`.

### Chaque changement d'état est annoncé

Tout ce qui change pour un observateur (un pion posé, tué, déplacé, qui change de camp, un tour qui se termine…) produit un `Event`. Le labo, Godot et les bots ne lisent que ces événements pour animer. C'est vérifié par l'invariant du miroir (principe 5).

### Le modèle de décision (cible, tâche T4)

Les attaques demandent plusieurs choix (carte, pion lanceur, cible, sauts enchaînés). Les réactions se jouent pendant le tour adverse, et le draft est une suite de choix. Au lieu d'une `Action` géante qui contiendrait tout, **le moteur expose à tout moment qui doit choisir quoi** :

- `pending_decision()` : la décision attendue (action principale, choix du lanceur, choix d'une cible, fenêtre de réaction, choix de draft…) et le joueur concerné ;
- `legal_actions()` : la liste exacte des actions acceptées maintenant.

Chaque choix est une petite `Action` (`Place(pos)`, `UseAttack(slot)`, `Choose(pos)`, `Pass`, `EndTurn`…). Avantages :

- le labo surligne les cases jouables sans connaître aucune règle ;
- les bots jouent en piochant dans `legal_actions()` ;
- pour le ML, l'espace d'actions est petit et fixe : environ 100 cases plus quelques boutons ;
- le replay reste une simple liste d'actions.

### Les attaques (cible, phase B)

- **Identité par données** : `AttackKind` (la famille) + `Rarity`. Le coût et la recharge viennent de `AttackStats` dans le `AttackCatalog` de `GameConfig`, jamais du code de la famille. Le coût n'est pas lié à la rareté.
- **Une famille par fichier** dans `attacks/`. Chaque famille implémente un trait interne sans état (les cibles légales, et les effets à produire), appelé via `AttackKind::rules()`. `Game` ne stocke que des données simples : pas de `Box<dyn>`, ce qui garde `Game` clonable, comparable, hachable et sérialisable.
- **Les effets primitifs** (`Effect`) sont le cœur DRY : tuer, déplacer, échanger, poser, changer de camp, ajouter un effet en cours, creuser un trou, gagner des actions. Une famille *décrit* ses effets. **Un résolveur central** les applique, émet les `Event` et déclenche les conséquences : retour en réserve, tombe, vengeance de Meilleurs amis, fenêtres de réaction. Les attaques changent souvent, les primitives rarement.
- **Les nombres d'une famille** (portée, nombre de cibles, durée) sont des constantes nommées dans son fichier. Ils passeront dans la configuration quand l'équilibrage par bots en aura besoin (principe 6).

### Les pions et les cases (cible, tâche T3)

- Chaque pion a une **identité stable `PawnId`**. Un effet en cours vise un pion, pas une position, car un pion peut être attiré, échangé ou téléporté.
- Une case est une **structure en couches** (un pion optionnel, puis le terrain : trou, tombe… ajoutés quand une attaque en a besoin). Ce n'est pas un enum exclusif : une tombe n'empêche pas un pion d'être sur la même case.

### La victoire et la fin de partie

- Une ligne de `align_to_win` pions **faite en posant** gagne tout de suite.
- **Règle d'or (cible, B2)** : une ligne obtenue grâce à une attaque donne une victoire en attente, qui doit survivre à un tour adverse.
- **Fin garantie (cible, T4)** : une issue `Draw` avec une limite de tours dans la configuration. Sans elle, des bots peuvent jouer à l'infini.

## 5. Bots et machine learning (préparé, construit après la v1.0)

Le moteur est conçu dès maintenant pour qu'ils soient simples et rapides :

- `Game` est `Clone` et petit : un bot de recherche peut copier l'état pour essayer un coup ;
- tout est déterministe, et une partie se rejoue depuis son `GameRecord` ;
- `legal_actions()` donne directement l'espace d'actions ;
- `PlayerView` donne l'observation d'un joueur, sans triche possible sur l'information cachée ;
- le membre `bots/` contiendra l'encodage observation → tenseur et action ↔ indice, pour ne pas mettre de code ML dans le moteur ;
- la mesure du nombre de parties aléatoires par seconde (T5) sert de référence de performance. On n'optimise rien sans elle.

## 6. Les clients

Règles communes à tous les clients :

- aucune règle de jeu : la légalité vient de `legal_actions()` ou d'une `GameError` ;
- un seul endroit appelle `game.apply` (dans le labo : `partie.rs`) ;
- l'affichage du journal vient des `Event`, pas d'une relecture de l'état ;
- une seule conversion position ↔ écran (dans le labo : `plateau.rs`) ;
- aucun nombre en dur : la taille vient de `GameConfig`, l'apparence d'un fichier de thème.
