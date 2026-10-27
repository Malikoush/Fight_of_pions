# Feuille de route

Dernière mise à jour : 2026-10-27.

## Comment on avance

Une tâche à la fois, **en vase clos**. Une tâche est finie quand :

1. le moteur est codé, avec ses tests de règles ;
2. le labo permet de l'essayer (si elle a un effet visible) ;
3. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace` passent ;
4. l'agent **gardien-qualite** a relu le diff sans point BLOQUANT ;
5. la doc est à jour : [carnet de règles](REGLES.md), [glossaire](GLOSSAIRE.md), ce fichier ;
6. Martin a validé dans le labo ;
7. Martin la commite (Claude propose le message).

On ne commence jamais la suivante avant. Une tâche bloquée par une question de règle attend sa réponse dans [REGLES.md](REGLES.md#questions-bloquantes).

## Fait

- [x] **Phase 1 et 2** : moteur de base (poser, tours, actions en réserve, alignement), terminal.
- [x] **T0 · Workspace et labo** : `moteur/` sans dépendance, labo eframe, tests d'intégration réactivés, correctif de `TurnEnded`.
- [x] **T1 · Socle qualité** : cette doc, `CLAUDE.md`, l'agent gardien-qualite, les lints (pas d'`unwrap`/`panic` dans le moteur, API documentée).

## Phase A · Fondations (aucune nouvelle règle de jeu)

- [x] **T2 · Hygiène du moteur** (points relevés par la première relecture du gardien)
  - **Plus aucune panique possible sur une entrée de client :**
    - l'affichage d'une `Position` au-delà de Z déborde (`b'A' + col as u8`) : il faut un affichage sûr pour toute position, avec un test ;
    - `Position::step` doit utiliser une addition vérifiée ;
    - activer le lint `clippy::cast_possible_truncation` dans le moteur.
  - **Configuration :**
    - `GameConfig::validate` complet : taille entre 2 et 26 (noms « A1 » à « Z26 »), alignement entre 2 et la taille, nombres d'actions et de pions ≥ 1, avec un test par cas refusé ;
    - `Board::new` ne doit plus permettre un plateau invalide (`pub(crate)` ou création vérifiée) ;
    - renommer `action_per_turn` en `actions_per_turn` ;
    - le coût de la pose (`1` en dur dans `Action::cost`) devient une constante nommée ou un réglage ;
    - un getter `Game::config()`, pour que les clients affichent les réglages sans les recopier.
  - **Exécution :**
    - rendre `execute` infaillible : plus de `?` après une modification d'état ;
    - supprimer la branche morte `turn == 1` de `start_turn`.
  - **Code prévu pour des règles pas encore tranchées** (principe 6), à réintroduire en B1/B4 : `ensure_owned`, `GameError::NotYourPion`, `GameError::IllegalMove`, `Position::isadjacent`.
  - **Noms conformes au glossaire :**
    - module `alignement` → `alignment` ;
    - variables `ecart_l` et `ecart_c` ;
    - tests nommés d'après la règle qu'ils protègent ;
    - fonctions `pub` d'un module privé passées en `pub(crate)`.
  - **Tests manquants :**
    - actions en réserve plafonnées à 2, donc 4 actions au plus ;
    - `EndTurn` coûte 0 ;
    - refus `OutOfBounds` exact.
  - Corriger le français des messages de `GameError`.
  - Ajouter `Hash`, et `PartialEq`/`Eq` (y compris sur `Game`).
  - **Terminal** : le sortir dans un membre `terminal/`, ce qui fait de `moteur` une pure bibliothèque ; lire le nombre d'actions gardées dans la configuration ; ne plus dupliquer le nom des colonnes.
  - **Labo** : l'arrondi des cases (`0.0` dans `plateau.rs`) passe dans `theme.rs`.
- [x] **T3 · Identité des pions**
  - `Pion { id: PionId, owner }`, `Cell` en structure (`pion: Option<Pion>`).
  - Compteur d'identifiants dans `Game`, déterministe.
  - `Event::PionPlaced` porte le pion, donc son identifiant.
  - `rules/alignment` est inchangé grâce à `board.owner()`.
  - Vocabulaire harmonisé : « Pion » partout dans le code (`pions_per_player`, `PionPlaced`…).
- [x] **T4 · Modèle de décision et fin de partie**
  - `Decision`, `pending_decision()` et `legal_actions()` (construite à partir de la validation).
  - `GameEnd { Outcome, EndReason }`, `Event::GameEnded` ; partie figée et `max_turns` (200) → nul.
  - Configuration : pose impossible refusée (`PlacementUnreachable`), limite de tours ≥ 1.
  - Tests :
    - toute action listée est acceptée, une action non listée est refusée, sur 100 parties aléatoires ;
    - toute partie se termine ;
    - plateau plein, réserves vides et limite de tours donnent un nul ; l'alignement passe avant.
- [x] **T5 · Déterminisme, replay et invariants**
  - `Rng` (SplitMix64 maison, testé sur les valeurs de référence de l'implémentation C d'origine), graine dans `Game` : `Game::new(config, seed)`.
  - `GameRecord` (hors de `Game`, pour garder l'égalité de position et des copies légères) et `replay()`, avec `ReplayError`.
  - Tests sur des centaines de parties aléatoires :
    - le miroir d'événements (plateau, réserves, actions restantes et gardées, joueur courant, fin) est égal au moteur après chaque action ;
    - même graine + mêmes actions = mêmes événements et même état ;
    - un replay est identique à l'original.
  - `Event::ActionsChanged` : les compteurs d'actions sont annoncés à chaque changement, et jamais quand rien ne change (décision du 2026-10-27, option c), pour qu'aucun client ne recalcule la règle des actions gardées.
  - Mesure : exemple `random_games`, résultat noté plus bas.
  - La feature `serde` est reportée à T6, la première tâche qui en a besoin (principe 6) : le moteur reste sans aucune dépendance.
- [x] **T6 · Labo outillé** (toujours sans aucune règle dans le labo)
  - Affichage de la graine.
  - Feature `serde` optionnelle du moteur (sur `GameRecord`, `GameConfig`, `Action`, `Position`), activée par le labo seulement : `cargo test` à la racine et le moteur seul restent sans dépendance.
  - « Copier la partie » (JSON dans le presse-papiers) / « Charger la partie » (zone où coller le JSON), pour reproduire un bug exactement. Le format JSON n'est pas figé avant la v1.0 : une partie copiée peut ne plus se charger après un changement du moteur.
  - Annuler un coup : `partie.rs` rejoue l'enregistrement sans sa dernière action.
  - Jouer N coups au hasard, avec le `Rng` du moteur initialisé par la graine de la partie.
  - Un point sur chaque case jouable, lue dans `legal_actions()`.
  - Un JSON avec un champ inconnu est refusé au lieu d'être ignoré en silence (`deny_unknown_fields`).
  - Tests du labo (`partie.rs`, graine fixe donc reproductibles) : export puis import, chargement, annulation, coups refusés non enregistrés, coups au hasard jusqu'à la fin ; un chargement refusé (texte illisible, action refusée, configuration invalide, champ inconnu) ne change ni la partie ni l'enregistrement.

**Phase A terminée** : le moteur est déterministe, rejouable, testé par propriétés, et le labo permet de tout essayer à la main.

## Phase B · Attaques

De la plus stable à la plus floue. Aucune famille « EN REFONTE » avant que sa règle soit figée dans le carnet.

- [x] **B1 · Squelette des attaques + Ours R/É** (tuer un adjacent)
  - Données : `AttackKind`, `Rarity`, `Attack` (une carte), `AttackStats` (coût, recharge) ; catalogue `attack_catalog` et arsenaux de départ `starting_attacks` dans `GameConfig` (qui n'est donc plus `Copy`), validés : variante jouable, pas de doublon, coût ≥ 1 et atteignable, recharge ≥ 1, arsenal pris dans le catalogue.
  - Règles : `attacks/`, une famille par fichier (`ours.rs`), trait interne `AttackRules` sans état ; `can_complete` générique garantit **aucune impasse**.
  - Choix : `Action::UseAttack(slot)`, puis `Choose` pour le lanceur et la ou les cibles ; décisions `ChooseLauncher` et `ChooseTarget`. **Pas d'action « annuler » dans le moteur** (décision du 2026-10-27). Les choix intermédiaires ne produisent aucun événement.
  - Résolution d'un bloc au dernier choix (`game/attack_flow.rs`) : coût payé, carte en recharge, effets appliqués par le résolveur central (`Effect::Kill`), retour en réserve. Pas de tombe (B9).
  - Événements : `AttackUsed`, `PionKilled { pion, at, by }`, `ReserveChanged` (maintenant aussi à chaque pose), `CooldownChanged`. Le miroir les suit tous.
  - « Partie figée » : un joueur qui a une attaque avec une cible, même en recharge ou à payer en économisant, peut encore agir.
  - `legal_actions` ne teste que les candidates qui répondent à la décision attendue (sans cela, les performances étaient divisées par deux).
  - `PionId` : saturer demanderait plus de 4 milliards de poses dans une même partie ; avec la limite de tours par défaut, une partie en fait au plus ~800.
  - Labo : section « Attaques » (Lancer, coût, recharge), sélection sur une **copie** (`Partie::apercu`) avec anneaux sur les cases choisissables et le lanceur, la suite complète jouée d'un bloc (`Demande::Jouer`) ; « Annuler l'attaque » gratuit ; « Arsenal de la prochaine partie » réglable par joueur ; coups au hasard et annulation qui ne laissent jamais une attaque à moitié.
  - Terminal : commandes `a 1` et `c C5`, arsenal affiché ; chaque joueur commence avec toutes les attaques du catalogue (`GameConfig::with_full_arsenals`, partagé avec le labo et la mesure).
  - Tests : 11 scénarios de l'Ours, validation du catalogue, et les tests de propriété (legal_actions = apply, aucune impasse, miroir, replay, partie figée) étendus à des configurations avec attaques.
- [ ] **B2 · Règle d'or** : victoire en attente, avant la première attaque qui peut créer une ligne.

> **À prévoir pour les familles suivantes** (relecture de B1, 2026-10-27). Rien n'oblige à réécrire le socle, mais :
> - **Attaques sans lanceur** : `Decision::ChooseTarget` et `Event::AttackUsed` exigent un lanceur. Si une famille sans lanceur est confirmée par le carnet, ces types publics devront devenir optionnels (impact labo, terminal, Godot).
> - **Nombre de cibles variable** (sauts enchaînés du Saut du loup, arrêt facultatif) : remplacer `target_count` par un `is_complete(contexte, choisies)` commun à `can_complete` et à la résolution, et peut-être une action pour s'arrêter.
> - **Effets en chaîne** (déplacer puis tuer, échanger) : viser les `PionId` plutôt que les positions dans `Effect`.
> - **Familles qui posent des pions** : élargir `AttackContext` (réserves, effets en cours).
> - **Réactions (B8)** : ajouter une phase « fenêtre de réaction » à `PendingAttack` ; aujourd'hui seul le joueur courant choisit.
> - **Coût de `can_complete`** : recherche en profondeur sans mémoire ; à mesurer avec `random_games` dès les sauts enchaînés (B4).
- [ ] **B3 · Effets en cours** : `ActiveEffect` et ses échéances ; Beauté fatale C et Saut du loup C (immobiliser).
- [ ] **B4 · Mouvement** : Beauté fatale R, Ours C, Saut du loup R/É/L (sauts enchaînés), Lièvre, Interchange.
- [ ] **B5 · Invocation et changement de camp** : Beauté fatale É/L, Familia, Meilleurs amis (vengeance).
- [ ] **B6 · Économie** : Massacre divin.
- [ ] **B7 · Terrain** : Coup du taureau (trous).
- [ ] **Design · Anti-stagnation** : Effondrement et/ou boost d'actions par tour ; règle de répétition (même position 3 fois → fin, rangée ici le 2026-10-27) ; nul ou départage. Plusieurs variantes en réglages, essayées au labo, puis comparées par les bots (taux de nul, durée, avantage du premier joueur). « Partie figée » doit tenir compte des effondrements à venir.
- [ ] **B8 · Réactions** : fenêtres de réaction dans le modèle de décision (avant ou après la résolution) ; Feintaille, Sentinelle, Queue du lézard, Perroquet, Hibou.
- [ ] **B9 · Refontes figées** : Reine, Attila, Médecin (tombes et zombies).
- [ ] **B10 · Draft et cache secrète** : tirage avec la graine, bans et picks, `PlayerView`, arsenal rempli par le draft. `Game` devra alors garder l'état courant du `Rng`, pas seulement la graine : une copie de partie prise en cours de route doit continuer la même suite de tirages.
- [ ] **Jalon v1.0 « version propre »** : tag Git, toute la doc à jour.

## Phase C · Après la v1.0 (préparée, pas commencée)

- [ ] Configuration et catalogue chargés depuis un fichier (équilibrer sans recompiler).
- [ ] Membre `bots/` : bot aléatoire, bot glouton, arène de N parties avec statistiques par carte.
- [ ] Encodage ML (observation → tenseur, action ↔ indice) et membre `py_bindings/` (pyo3).
- [ ] Membre `godot_bridge/`.

## Mesures

| Date | Mesure | Valeur |
|---|---|---|
| 2026-10-26 | `cargo test` du moteur | ~2 s |
| 2026-10-26 | Relance du labo après une petite modification | ~2 s |
| 2026-10-26 | Parties complètes jouées au hasard, release (`random_games`, 20 000 parties de ~30 actions, ~1,3 s) | ~16 000 parties/s, ~480 000 actions/s (varie de ±20 % selon la charge de la machine) |
| 2026-10-27 | `random_games` après B1, sans attaque | ~15 000 parties/s ; fins : 100 % figée (le hasard n'aligne jamais 6) |
| 2026-10-27 | `random_games` après B1, chaque joueur avec l'Ours R et É | ~630 parties/s, ~150 000 actions/s, ~240 actions par partie ; fins : 96 % figée, 2 % alignement, 2 % limite de tours. Les kills effacent tout contact : sans déplacement, le plateau finit sans aucun ennemi voisin. Signal pour l'anti-stagnation. |
