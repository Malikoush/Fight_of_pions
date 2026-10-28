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
- [ ] **Arène et bots** (tâche de la phase C, avancée avant B2 le 2026-10-27 : mesurer chaque réglage au lieu de le deviner). Le moteur n'a pas été modifié. Codée, testée et relue par le gardien ; reste la validation de Martin dans le labo, puis le commit.
  - Membre `bots/` : une bibliothèque, dans les membres par défaut (donc vérifiée par `cargo test`) ; l'arène est l'exemple `arene`, pour que `cargo run` lance toujours le terminal. Alias `cargo arene`.
  - Trait `Bot` sans état ; `RandomBot` (l'étalon) ; `GreedyBot`, réglable en glouton (dépense tout) ou banquier (garde le maximum permis). Le glouton gagne s'il peut, bloque les menaces, garde ses actions s'il est banquier, puis pose sur une des meilleures cases selon les fenêtres.
  - Aucune règle recopiée dans les bots : légalité par `legal_actions()`, attaques jouées jusqu'au bout sur des copies (`AttackPlan`), actions du prochain tour adverse lues sur une copie où l'on finit son tour (actions gardées et limite de tours comprises), géométrie lue dans le moteur (`Windows`). Seule lecture de règle : poser coûte `place_cost` actions et un pion de la réserve.
  - `bots::run` et `Report` : victoires par place, nuls par raison, tours, attaques par carte, pions tués, tours finis après avoir agi en gardant des actions, passes (les deux sont exclusifs), refus. Une partie abandonnée sur un refus ne laisse que ce refus dans le bilan. Les variantes sont des `GameConfig` nommées (aujourd'hui dans `bots::presets`) ; les paires de bots tournent en parallèle, et chaque variante est aussi jouée contre l'étalon aléatoire.
  - Tests : fenêtres (150 sur un 10×10 avec 6 à aligner, 8 sur un morpion), glouton (finir sa ligne, bloquer toutes les menaces, percer une ligne bloquée avec l'Ours, garder ses actions), arène (aucun refus sur cinq configurations, dont un 2×2 et deux où poser coûte 2 actions, et toutes les paires ; bilan reproductible ; le glouton bat l'aléatoire 10 fois sur 10 dans les deux places ; configuration invalide refusée).
  - Limites connues du glouton : il ignore les attaques de l'adversaire dans ses menaces ; il n'attaque que pour gagner ou se défendre ; en défense, il compare **une** pose à une attaque entière, et peut donc choisir l'Ours quand deux poses auraient suffi (cela gonfle un peu le nombre d'attaques mesuré).

> **Ce que l'arène montre déjà** (2026-10-27, `cargo arene 100`, chaque joueur avec l'Ours R et É). Ce sont des bots simples : un écart de quelques points ne prouve rien, un écart de 50 points si.
> - **Le glouton bat l'aléatoire 100 % du temps**, en 6 à 7 tours, dans les deux places : l'étalon est franchi.
> - **Garder 2 actions décide la partie** : banquier contre glouton, 97 % de victoires pour le banquier quand il est joueur 1 (74 % en joueur 2), sans jamais perdre ; banquier contre banquier, le joueur 2 gagne 100 %.
> - **Entre gloutons, 100 % de nuls par limite de tours**, avec 91 % de tours passés. Une fois les réserves vides, tuer redonne un pion à l'adversaire, et le glouton n'attaque que pour gagner ou se défendre : il passe. « Partie figée » ne le voit pas, puisqu'une attaque reste possible. C'est en partie un comportement de bot, mais la règle permet bien cette impasse (passer est gratuit et illimité).
> - **Banque max 1** : le banquier ne gagne plus que 20 à 26 % contre le glouton, mais 68 à 80 % des parties finissent à la limite de tours.
> - **Ours Rare à 1 action** : entre banquiers, 195 attaques par partie de 200 tours. Tuer, reposer, tuer : une boucle sans fin.
> - À mesurer ensuite, après être devenus des options de configuration : départage, fin sur passes répétées, pion posé ce tour qui ne peut pas attaquer, une attaque par tour.

- [ ] **Labo : bots et arène à l'écran** (2026-10-27). Moteur inchangé. Codée, testée et relue par le gardien ; reste la validation de Martin dans le labo, puis le commit.
  - Socle partagé dans `bots/` : `presets` (bots disponibles `BotKind`, paires `STANDARD_PAIRS`, règle actuelle `current_rules()`, variantes `variants()`), `start_game` (moteur et hasard des bots avec la même graine, utilisé aussi par le labo), une seule boucle de partie pour les bilans et pour `record_game`, et `run_pairs` (paires en parallèle). L'exemple `arene` ne fait plus qu'afficher.
  - Labo, écran **Partie** : chaque joueur humain ou bot, lecture automatique avec attente réglable (arrêtée si le coup d'un bot est refusé), bouton « Coup du bot », variante de la nouvelle partie. Au tour d'un bot, les boutons de jeu sont désactivés et `app.rs` refuse tout geste de jeu ; une attaque qu'un humain choisissait est abandonnée si un bot prend sa place. Face à un bot, « Annuler le coup » remonte jusqu'au dernier coup d'un humain (rien si l'humain n'a pas encore joué), et le hasard des bots reprend son état d'alors : la partie revue reste celle de l'arène. Les coups au hasard passent par le bot aléatoire, avec le même hasard qu'avant.
  - « Nouvelle partie » lance la **prochaine partie** : on part d'une variante ou de la partie en cours (« Reprendre la partie en cours », utile après un chargement JSON), puis on retouche plateau, alignement, pions, actions, banque, coût de pose, limite de tours, coût et recharge de chaque attaque, arsenaux. Le titre indique la variante et si elle est retouchée ; une configuration incohérente affiche le refus de `GameConfig::validate` et « Nouvelle partie » est désactivé. Le panneau de l'écran Partie et les réglages de l'Arène défilent ; le journal a une hauteur maximale, au-delà il défile. Après « Revoir », la prochaine partie repart de la configuration revue ; cocher ou décocher une attaque garde l'ordre du catalogue (mêmes emplacements).
  - Labo, écran **Arène** : configuration simulée réglable comme la prochaine partie (même éditeur, `reglage.rs` : variante ou partie en cours, chaque réglage retouchable), paires (« Lancer » désactivé sans paire cochée ou si la configuration est incohérente), 1 à 500 parties par paire (50 proposées) ; calcul dans un fil à part ; bilans affichés comme par `cargo arene` ; « Revoir » ouvre une partie précise dans l'écran Partie ; une panique est affichée avec son message.
  - Tests : une partie de l'arène se revoit à l'identique dans le labo (2 configurations, dont une autre que celle par défaut, × 2 paires, dont une avec l'aléatoire, × 2 graines), y compris après une annulation en cours de route ; un coup du bot ne laisse jamais une attaque à moitié ; annuler face à un bot revient au tour de l'humain, ne fait rien si l'humain n'a pas joué, et entre deux bots retire un seul coup ; au tour d'un bot, les gestes de jeu humains sont refusés ; « Revoir » règle la partie, les bots et l'écran ; variantes valides et toutes différentes ; paires en parallèle = paires une à une ; partie enregistrée = partie du bilan (vainqueur, tours, attaques, tués) ; une partie refusée (action refusée ou aucune action) ne laisse que son refus dans le bilan ; la prochaine partie part de la règle actuelle sans retouche, une configuration incohérente ne se lance pas, repartir d'une variante efface les retouches, reprendre la partie en cours en repart sans retouche ; décocher puis recocher une attaque rend le même arsenal.
  - À savoir : les bots jouent dans le fil de l'interface. Sur les grands plateaux qu'on peut maintenant régler, un coup de bot prenait des centaines de millisecondes avant l'optimisation du moteur et des bots en debug (mesuré par le gardien sur 26×26 : jusqu'à ~360 ms pour le glouton, ~800 ms pour le banquier ; l'optimisation divise les temps par ~4 sur l'arène, pas remesuré sur 26×26) : la fenêtre peut répondre mal pendant la lecture automatique. Sur 10×10, rien de visible. Si cela gêne, faire jouer les bots dans un fil à part, comme l'Arène.
  - À savoir : le labo tourne en debug, mais le moteur et les bots y sont optimisés (voir la tâche suivante) : l'écran Arène va à la vitesse de `cargo arene`. Les parties entre gloutons (200 tours) coûtent le plus.

- [ ] **Outil : fourchettes à 95 % et moteur optimisé en debug** (2026-10-27). Moteur inchangé (seul le profil de compilation change).
  - Chaque bilan affiche, sous sa ligne, la fourchette à 95 % de chaque pourcentage de fin (victoires J1 et J2, nuls) : intervalle de Wilson, juste même avec peu de parties et à 0 ou 100 %. Le labo l'affiche aussi, puisqu'il montre les bilans tels quels. Tests : valeurs connues, extrêmes, fourchettes absentes sans partie jouée.
  - `Cargo.toml` : `fight_of_pions` et `bots` en `opt-level = 3` dans le profil dev ; le labo et le terminal restent en opt-level 0 (recompilation rapide), les contrôles de débordement restent actifs. Pour suivre le moteur pas à pas dans un débogueur, commenter ces deux blocs.
  - L'écran Arène propose 50 parties par paire (au lieu de 10) et en accepte jusqu'à 500.

- [ ] **Règles cibles : l'économie** (décidée le 2026-10-28, voir le carnet) : pose et points d'action séparés (1 pose facultative, 1 point d'action par tour, banque de 3), mal du débarquement (pose et invocation), la Pièce, limite de tours → égalité. En option de la configuration, règles actuelles par défaut, pour que l'arène compare les deux. Codée et testée le 2026-10-28 ; reste la relecture du gardien, la validation de Martin dans le labo, puis le commit.
  - Moteur : `max_placements_per_turn`, `place_cost` à 0 permis si les poses sont limitées, `starting_banked_actions`, `summoning_sickness` ; `Game::placements_left()` et `Game::arrived_this_turn(pos)`. Un pion « arrivé ce tour » se reconnaît à son identité (les identités suivent l'ordre des poses) : aucun suivi par pion. Le tour se termine seul quand il ne reste ni action ni pose. Les champs absents d'une ancienne partie copiée prennent leur valeur par défaut.
  - Bots : ils ne lisent plus aucune règle (les poses restantes viennent du moteur) ; le banquier fait quand même une pose gratuite. Variante « règles cibles » (`bots::target_rules()`), proposée par l'arène et le labo.
  - Labo : réglages « Banque de départ J1 / J2 », « Poses par tour », « Mal du débarquement » ; poses restantes affichées ; point orange sur les pions posés ce tour quand le mal du débarquement est actif.
  - Tests : pose gratuite sans annonce d'actions ni fin de tour ; une seule pose permise, et les poses non faites ne s'accumulent pas ; la Pièce ; fin de tour automatique sans action ni pose ; mal du débarquement (avec et sans) ; un pion nouveau qui pourra attaquer ne fige pas la partie ; configurations refusées ; ancienne partie copiée rechargée ; les configurations aléatoires des tests de propriétés (miroir d'événements, actions légales) couvrent les nouveaux réglages ; le banquier fait sa pose gratuite ; tous les bots jouent légalement dans les règles cibles.
  - Premiers résultats (`cargo arene 100`, Ours seuls, sans règle d'or, donc pire cas) : glouton contre glouton, 20 % de victoires J1 (fourchette 13–29 %), 7 % J2 (3–14 %), 73 % de limite de tours (64–81 %), contre 100 % de limite dans les règles actuelles ; 0,27 attaque par partie : le glouton n'attaque que pour gagner ou se défendre. Le banquier et le glouton jouent exactement pareil (la pose est gratuite, les actions ne servent qu'aux attaques) : la paire banquier n'apprend plus rien dans cette économie. Il faut des bots qui attaquent (MCTS) avant de conclure.
- [ ] **B2 · Règle d'or** : une ligne de 6 ou plus au début de son tour gagne (décidé le 2026-10-28), avant la première attaque qui peut créer une ligne.

> **À prévoir pour les familles suivantes** (relecture de B1, 2026-10-27). Rien n'oblige à réécrire le socle, mais :
> - **Attaques sans lanceur** : `Decision::ChooseTarget` et `Event::AttackUsed` exigent un lanceur. Si une famille sans lanceur est confirmée par le carnet, ces types publics devront devenir optionnels (impact labo, terminal, Godot).
> - **Nombre de cibles variable** (sauts enchaînés du Saut du loup, arrêt facultatif) : remplacer `target_count` par un `is_complete(contexte, choisies)` commun à `can_complete` et à la résolution, et peut-être une action pour s'arrêter.
> - **Effets en chaîne** (déplacer puis tuer, échanger) : viser les `PionId` plutôt que les positions dans `Effect`.
> - **Familles qui posent des pions** : élargir `AttackContext` (réserves, effets en cours).
> - **Réactions (B8)** : ajouter une phase « fenêtre de réaction » à `PendingAttack` ; aujourd'hui seul le joueur courant choisit.
> - **Coût de `can_complete`** : recherche en profondeur sans mémoire ; à mesurer avec `random_games` dès les sauts enchaînés (B4).
> - **« Une attaque est en cours de choix »** est testé à l'identique dans le labo (`en_plein_choix`) et dans les bots (`is_choosing`) : à la prochaine tâche qui touche le moteur, le déplacer dans `Decision` (par exemple `Decision::is_attack_choice()`) et l'utiliser des deux côtés.
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
- [ ] Membre `bots/` : bot aléatoire, bot glouton, arène de N parties avec statistiques par carte (avancé avant B2, voir la phase B, où il attend la validation de Martin).
- [ ] Bots plus forts (recherche à 2 coups, puis MCTS) ; statistiques de draft par carte (B10).
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
| 2026-10-28 | `cargo arene 100` avec la variante « règles cibles » (5 variantes × 7 paires × 100 parties, release) | ~2 min (34,6 s pour les règles cibles) |
| 2026-10-27 | `cargo run -p bots --example arene -- 10` en debug (4 variantes × 7 paires × 10 parties), avant puis après l'optimisation du moteur et des bots dans le profil dev | 52 s → 11,8 s (×4,4) : le debug va désormais à la vitesse de la release |
| 2026-10-27 | `cargo arene 100` : 4 variantes × 7 paires de bots × 100 parties, paires en parallèle, release | ~1 min 50 ; les parties entre gloutons durent 200 tours et coûtent le plus. `cargo test` des bots : ~3,5 s en debug. |
