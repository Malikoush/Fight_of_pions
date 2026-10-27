# Architecture de Fight of Pions

Dernière mise à jour : 2026-10-27.

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
2. **Une partie = configuration + graine + liste d'actions** (`GameRecord`). Rejouer les actions reconstruit la partie. Cela donne gratuitement la sauvegarde, le replay, l'annulation dans le labo et la reproduction exacte d'un bug.
3. **Le jeu piloté par les données.** Tous les réglages chiffrés vivent dans `GameConfig`, validés au démarrage. Plus tard, ils seront chargés depuis un fichier pour équilibrer sans recompiler.
4. **Des tranches verticales.** Une fonctionnalité est finie quand elle marche de bout en bout : moteur, tests, labo. On ne commence pas la suivante avant.
5. **Des invariants vérifiés automatiquement.** Le principal : *un miroir construit uniquement à partir des `Event` (plateau, réserves, actions restantes et gardées, recharges des attaques, joueur courant, fin) est toujours égal à l'état du moteur*. C'est la garantie qu'un client qui suit les événements ne se désynchronise jamais, sans recalculer aucune règle. Il est vérifié après chaque action de centaines de parties aléatoires.
6. **Abstraire à la deuxième occurrence, pas avant.** La modularité vient des bonnes abstractions de base (les effets primitifs, le modèle de décision, les données), pas d'options prévues pour des besoins imaginaires.
7. **La vue joueur** (`PlayerView`, cible). Dès qu'il y a de l'information cachée (cache secrète), le moteur fournit une vue restreinte par joueur. Les bots et le jeu en ligne ne voient que celle-ci.
8. **Les règles avant le code.** Une attaque n'est codée que lorsque sa règle est écrite et datée dans le [carnet de règles](REGLES.md). Les attaques « EN REFONTE » dans le GDD attendent.

## 3. Le workspace

| Membre | Rôle | Dépendances |
|---|---|---|
| `moteur/` (paquet `fight_of_pions`) | Toutes les règles. Bibliothèque. | Aucune ; feature `serde` optionnelle (sauvegarde JSON d'un `GameRecord`) |
| `labo/` | Client de test eframe/egui : panneau, damier cliquable avec cases jouables, journal, graine, annuler, coups au hasard, copier/charger une partie. | `moteur` (avec `serde`), `eframe`, `serde_json` |
| `terminal/` | Client texte. | `moteur` |
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

- Une ligne de `align_to_win` pions **faite en posant** gagne tout de suite.
- **Règle d'or (cible, B2)** : une ligne obtenue grâce à une attaque donne une victoire en attente, qui doit survivre à un tour adverse.
- **Fin garantie** : toute partie se termine (`GameEnd`), soit par alignement, soit **figée** (plus personne ne pourra jamais agir), soit par la **limite de tours** `max_turns`, filet de sécurité pour les bots.
- Trois choses sont séparées pour rester modifiables : le **déclencheur** (`EndReason`, utile aux statistiques), le **résultat** (`Outcome`, calculé à un seul endroit : nul pour l'instant, départage possible plus tard) et la **limite** (réglage).
- « Figée » veut dire *plus personne ne peut agir et rien de programmé ne peut changer la situation*. Une future mécanique anti-stagnation (Effondrement) devra être prise en compte dans ce test.

## 5. Bots et machine learning (préparé, construit après la v1.0)

Le moteur est conçu dès maintenant pour qu'ils soient simples et rapides :

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
