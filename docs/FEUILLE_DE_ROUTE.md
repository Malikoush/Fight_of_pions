# Feuille de route

Dernière mise à jour : 2026-10-26.

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
  - **Code prévu pour des règles pas encore tranchées** (principe 6), à réintroduire en B1/B4 : `ensure_owned`, `GameError::NotYourPawn`, `GameError::IllegalMove`, `Position::isadjacent`.
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
- [ ] **T3 · Identité des pions**
  - `Pawn { id: PawnId, owner }`, `Cell` en structure (`pawn: Option<Pawn>`).
  - Compteur d'identifiants dans `Game`, déterministe.
  - `Event::PawnPlaced` porte l'identifiant.
  - `rules/alignment` est inchangé grâce à `board.owner()`.
- [ ] **T4 · Modèle de décision et fin de partie** (attend la réponse à la question « match nul »)
  - `Decision`, `pending_decision()` et `legal_actions()`.
  - `Outcome { Won, Draw }` et `max_turns` dans la configuration.
  - Tests :
    - toute action listée est acceptée ;
    - une action non listée est refusée ;
    - toute partie se termine.
- [ ] **T5 · Déterminisme, replay et invariants**
  - `rng.rs` (SplitMix64 maison, testé sur des valeurs connues), graine dans `Game`.
  - `GameRecord` et sa relecture. Feature `serde` optionnelle.
  - Tests sur des centaines de parties aléatoires :
    - le miroir d'événements est égal au plateau ;
    - même graine + mêmes actions = même état ;
    - un replay est identique à l'original.
  - Mesure du nombre de parties aléatoires par seconde, notée dans ce fichier.
- [ ] **T6 · Labo outillé** (toujours sans aucune règle dans le labo)
  - Affichage de la graine.
  - « Copier la partie » / « Charger une partie » (JSON dans le presse-papiers), pour reproduire un bug exactement.
  - Annuler un coup (rejouer le replay moins une action).
  - Jouer N coups aléatoires.
  - Surligner les cases légales, lues dans `legal_actions()`.

## Phase B · Attaques

De la plus stable à la plus floue. Aucune famille « EN REFONTE » avant que sa règle soit figée dans le carnet.

- [ ] **B1 · Squelette des attaques + Ours R/É** (tuer un adjacent)
  - Types : `AttackKind`, `Rarity`, `AttackStats`, `AttackCatalog` dans la configuration, `Arsenal` par joueur avec recharge.
  - Choix : `Action::UseAttack(slot)`, puis `Choose` pour le lanceur et la cible ; annulation.
  - Résolution : `Effect::Kill`, résolveur central, `Event::PawnKilled`, retour en réserve.
  - Labo : arsenal de test réglable, choix de la carte, clic du lanceur puis de la cible.
- [ ] **B2 · Règle d'or** : victoire en attente, avant la première attaque qui peut créer une ligne.
- [ ] **B3 · Effets en cours** : `ActiveEffect` et ses échéances ; Beauté fatale C et Saut du loup C (immobiliser).
- [ ] **B4 · Mouvement** : Beauté fatale R, Ours C, Saut du loup R/É/L (sauts enchaînés), Lièvre, Interchange.
- [ ] **B5 · Invocation et changement de camp** : Beauté fatale É/L, Familia, Meilleurs amis (vengeance).
- [ ] **B6 · Économie** : Massacre divin.
- [ ] **B7 · Terrain** : Coup du taureau (trous).
- [ ] **B8 · Réactions** : fenêtres de réaction dans le modèle de décision (avant ou après la résolution) ; Feintaille, Sentinelle, Queue du lézard, Perroquet, Hibou.
- [ ] **B9 · Refontes figées** : Reine, Attila, Médecin (tombes et zombies).
- [ ] **B10 · Draft et cache secrète** : tirage avec la graine, bans et picks, `PlayerView`, arsenal rempli par le draft.
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
