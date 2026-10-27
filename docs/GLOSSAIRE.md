# Glossaire

Dernière mise à jour : 2026-10-27.

Un même mot désigne toujours la même chose, dans le GDD, dans le carnet de règles et dans le code.

- **Code du moteur** : les noms sont en anglais, sauf « Pion », qui est le nom même du jeu et s'écrit ainsi partout : `Pion`, `PionId`, `PionPlaced`, `pions_per_player`, `reserve_pions`. Jamais « pawn ». Les commentaires et la doc sont en français. Le labo, outil interne, est entièrement en français. Les bots suivent la convention du moteur : c'est une bibliothèque, que les futurs `py_bindings` réutiliseront.
- **Joueur** : le texte affiché est en français, via `Display` ou le thème du client.
- **Vocabulaire du GDD** : en cas de doute, c'est lui qui gagne (« attaque », pas « pouvoir »).

**(cible)** = pas encore dans le code ; le nom est réservé.

## Partie et tour

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Partie | `Game` | L'état complet et le seul point d'entrée des actions |
| Configuration | `GameConfig` | Tous les réglages (nombres, catalogue des attaques, attaques de départ), validés au démarrage |
| Joueur | `Player` | Les deux camps : `One` (commence) et `Two` |
| Tour | `turn` | Le tour **d'un** joueur. Le compteur avance à chaque fin de tour. |
| Manche | `round` (cible) | Un tour de chaque joueur (le « Tour 1 » du tableau du GDD) |
| Action | `Action` | Une décision envoyée au moteur : `Place`, `UseAttack`, `Choose`, `EndTurn` (plus tard `Pass` pour les réactions…) |
| Coût de la pose | `place_cost` | Actions que coûte la pose d'un pion (1 par défaut) |
| Actions du tour | `actions_left` | Ce qu'il reste à dépenser au joueur courant |
| Actions en réserve | `banked_actions`, `Game::banked_actions()` | Actions non utilisées gardées pour le tour suivant (jusqu'à `max_banked_actions`, 2 par défaut). **À ne pas confondre avec la réserve de pions.** |
| Événement | `Event` | Ce que le moteur annonce après une action ; suffit à suivre tout l'état visible |
| Compteurs d'actions annoncés | `Event::ActionsChanged` | Actions restantes et gardées d'un joueur, émis à chaque changement |
| Réserve annoncée | `Event::ReserveChanged` | Pions en réserve d'un joueur, émis à chaque changement (pose, kill…) |
| Refus | `GameError` | Pourquoi une action ou une configuration est refusée |
| Configuration incohérente | `ConfigError` | La raison précise du refus d'une configuration (`GameError::InvalidConfig`) |
| Décision attendue | `Decision`, `pending_decision()` | Qui doit choisir quoi, maintenant : `MainAction`, `ChooseLauncher`, `ChooseTarget` |
| Actions légales | `legal_actions()` | La liste exacte des actions acceptées maintenant |
| Fin de partie | `GameEnd`, `Event::GameEnded` | Le résultat et la raison de la fin |
| Résultat | `Outcome` | `Won(Player)` ou `Draw` (match nul) |
| Raison de la fin | `EndReason` | `Alignment`, `Frozen` (partie figée) ou `TurnLimit` |
| Partie figée | `EndReason::Frozen` | Plus personne ne pourra jamais agir autrement qu'en finissant son tour, et rien de programmé ne peut changer la situation |
| Limite de tours | `max_turns` | Le tour après lequel la partie s'arrête en match nul (200 par défaut) : filet de sécurité |
| Graine | `seed`, `Game::seed()` | Choisie par le client à la création (`Game::new(config, seed)`). Elle fait partie de la partie et initialise le hasard du jeu. |
| Générateur | `Rng` | Hasard déterministe (SplitMix64) : même graine, même suite. Tout le hasard passe par lui. |
| Enregistrement | `GameRecord` | Configuration + graine + actions acceptées : `replay()` reconstruit la partie |
| Échec de relecture | `ReplayError` | `Start` (configuration invalide) ou `RefusedAction { index, error }` (action refusée) |
| Vue joueur | `PlayerView` (cible) | Ce qu'un joueur a le droit de voir |

## Plateau et pions

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Plateau | `Board` | Le damier `board_size × board_size` |
| Case | `Cell` | Le contenu d'une case : une structure en couches (pour l'instant, un pion optionnel) |
| Position | `Position` | Rangée et colonne à partir de 0, affichée « C5 » |
| Direction | `Direction` | Un pas d'une case parmi les 8 directions |
| Pion | `Pion` | Un pion posé, avec son identité et son propriétaire |
| Pion tué | `Event::PionKilled { pion, at, by }` | Un pion retiré du plateau ; `by` est l'identité du tueur. Il retourne dans la réserve de son propriétaire. |
| Identité d'un pion | `PionId` | Numéro stable d'un pion (affiché « #3 »), pour qu'un effet le suive quand il bouge |
| Réserve de pions | `reserve_pions` | Les pions qu'un joueur peut encore poser (15 au départ) |
| Pion lanceur | `launcher` | Le pion du joueur depuis lequel une attaque est lancée, choisi par `Action::Choose` |
| Tombe | couche `tomb` de `Cell` (cible) | Ce que laisse un pion tué. Elle ne bloque pas la case. |
| Trou | couche terrain de `Cell` (cible) | Case détruite par le Coup du taureau, injouable |
| Zombie | type de pion (cible) | Pion ressuscité par le Médecin |
| Axe d'alignement | `Direction::AXES`, `axis` | Une des 4 directions de ligne : horizontale, verticale, deux diagonales |
| Alignement | `rules::complete_line` | Une ligne droite d'au moins `align_to_win` pions d'un joueur |
| Règle d'or | victoire en attente (cible) | Une ligne obtenue par une attaque doit survivre à un tour adverse |

## Attaques

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Attaque | `Attack` | Une carte : une famille dans une rareté (« Ours (Rare) ») |
| Famille d'attaque | `AttackKind` | Ours, et plus tard Lièvre, Beauté fatale… Une famille par fichier dans `attacks/`. |
| Rareté | `Rarity` | `Common`, `Rare`, `Epic`, `Legendary` : la variante de l'attaque et sa place dans le draft, **sans lien imposé avec le coût** |
| Caractéristiques | `AttackStats` | Coût et recharge d'une attaque |
| Catalogue | `attack_catalog`, `CatalogEntry` | Dans `GameConfig` : les attaques jouables et leurs caractéristiques |
| Attaques de départ | `starting_attacks` | Dans `GameConfig` : l'arsenal de chaque joueur au début (plus tard, sa cache secrète) |
| Arsenal | `Game::arsenal(player)`, `ArsenalSlot` | Les attaques d'un joueur pendant la partie, chacune avec sa recharge en cours |
| Utiliser une attaque | `Action::UseAttack(slot)` | Commencer l'attaque d'un emplacement de son arsenal ; le lanceur et les cibles se choisissent ensuite |
| Choisir une case | `Action::Choose(pos)` | Le lanceur, puis chaque cible de l'attaque en cours |
| Recharge | `cooldown`, `cooldown_left`, `Event::CooldownChanged` | Tours du propriétaire avant de pouvoir relancer une attaque (1 = une fois par tour) |
| Cible | case choisie | Une case visée par l'attaque en cours, proposée par la famille |
| Attaque lancée | `Event::AttackUsed` | Annonce d'une attaque résolue ; ses effets suivent |
| Effet primitif | `Effect` (interne au moteur) | Opération de base produite par une famille : tuer, et plus tard déplacer, poser, changer de camp… Seul le résolveur l'applique. |
| Effet en cours | `ActiveEffect` (cible) | Un effet qui dure plusieurs tours (immobilisé, protégé…) |
| Réaction | attaque avec déclencheur (cible) | Attaque jouée pendant le tour adverse, payée avec les actions en réserve |
| Axe d'attaque | aucun (documentation seulement) | Crowd control, Mouvement, Kill, Invocation, Buff, Spécial : sert à l'équilibrage, pas au code. Toujours dire « axe d'attaque », pour ne pas confondre avec l'axe d'alignement. |

## Bots et arène

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Bot | `Bot` (membre `bots/`) | Un joueur automatique, sans état : il choisit parmi `legal_actions()` et ne contient aucune règle |
| Bot aléatoire | `RandomBot` | Joue une action légale au hasard : l'étalon |
| Bot glouton | `GreedyBot` | Gagne s'il peut, bloque les menaces, puis pose sur une des meilleures cases selon les fenêtres |
| Glouton, banquier | `Banking::SpendAll`, `Banking::KeepMax` | Le glouton dépense toutes ses actions ; le banquier garde le maximum permis, sauf pour gagner ou se défendre |
| Fenêtre | `Windows`, `WindowContent` | Une suite de `align_to_win` cases consécutives sur un axe d'alignement. Sans pion adverse, c'est une ligne encore possible. |
| Menace | `threats` (interne aux bots) | Une fenêtre que l'adversaire peut finir en posant à son prochain tour |
| Poses possibles | `placement_budget` (interne) | Combien de pions un joueur peut poser dans son tour (en cours, ou le prochain lu sur une copie) : actions, coût de la pose et réserve |
| Plan d'attaque | `AttackPlan` (interne) | Une attaque jouée jusqu'au bout sur une copie de la partie, pour voir son effet sans connaître sa règle |
| Arène | `bots::run`, `bots::run_pairs`, `cargo arene` | Joue N parties entre deux bots (graines 0 à N−1) et en fait le bilan ; `run_pairs` le fait pour plusieurs paires en parallèle |
| Bots disponibles | `BotKind` | La liste des bots proposés par l'arène et le labo : aléatoire, glouton, banquier |
| Paires de l'arène | `STANDARD_PAIRS` | Les paires (joueur 1, joueur 2) jouées par défaut |
| Règle actuelle | `current_rules()` | La configuration par défaut où chaque joueur a toutes les attaques : la première variante, et la partie de départ du labo |
| Variante | `Variant`, `variants()` | Une configuration nommée, comparée à la règle actuelle ; la liste est partagée par l'arène et le labo |
| Début d'une partie de l'arène | `start_game` | La partie et le hasard de ses bots, initialisés avec la même graine ; le labo commence ainsi ses parties |
| Partie enregistrée | `record_game` | Une partie précise de l'arène (configuration, graine, deux bots) sous forme de `GameRecord` : la référence qui vérifie que le labo revoit la même partie |
| Bilan | `Report` | Victoires par place, nuls par raison, durée, attaques par carte, pions tués, tours où l'on garde des actions, passes, refus |
| Passe | `Report::passes` | Un tour fini sans aucune action |
| Tour où l'on garde | `Report::turns_keeping_actions` | Un tour fini après avoir agi, avec des actions gardées pour le suivant (jamais compté aussi comme passe) |
| Fourchette à 95 % | `Report` (affichage), `wilson_interval` | Les valeurs plausibles d'un pourcentage du bilan (victoires, nuls), vu le nombre de parties : intervalle de Wilson à 95 %. Avec 10 parties, 70 % de victoires donne une fourchette d'environ 40 à 89 % ; deux bilans dont les fourchettes se chevauchent largement ne permettent pas de conclure. |
| Refus | `Report::refused` | Une partie abandonnée parce qu'un bot a proposé une action refusée, ou aucune : toujours un bug. Elle ne laisse que ce refus dans le bilan. |

## Labo

Le labo est entièrement en français, noms du code compris.

| Terme | Nom dans le code | Rôle |
|---|---|---|
| Écran | `Ecran::Partie`, `Ecran::Arene` | Les deux écrans du labo : jouer ou regarder une partie, simuler des parties entre bots |
| Contrôleur | `Panneau::controleur` | Qui joue un joueur : un humain (`None`) ou un bot (`BotKind`) |
| Lecture automatique | `Panneau::attente_des_bots` | Les bots jouent seuls, avec une attente réglable entre deux coups ; elle s'arrête si le coup d'un bot est refusé |
| Coup du bot | `Demande::CoupDuBot` | Un coup joué par un bot via `partie.rs` : une action, et toute la suite si c'est une attaque |
| Revoir | `Intention::Revoir` | Ouvrir dans l'écran Partie une partie de l'arène (configuration, graine, bots), qui se rejoue à l'identique, même après une annulation |
| Hasard avant chaque action | `Partie::hasard_avant` | L'état du hasard des bots avant chaque action enregistrée : annuler le remet, pour que les bots rejouent les mêmes coups |
| Simulation | `Simulation` (écran Arène) | Ce qui est demandé à l'arène : configuration (et d'où elle vient), paires, nombre de parties |
| Réglage de partie | `ReglageDePartie` | Une configuration qu'on règle à l'écran : on part d'une variante ou de la partie en cours, puis on retouche chaque réglage (plateau, pions, actions, catalogue, arsenaux). `GameConfig::validate` décide si elle se lance. Il y en a un par écran : la **prochaine partie** (que lance « Nouvelle partie ») et la **configuration simulée** (sur laquelle l'arène fait jouer les bots). |
| Retouchée | `ReglageDePartie::retouchee` | La configuration réglée n'est plus exactement celle dont on est parti (variante ou partie en cours) |

## Cartes avant la partie (cible, B10)

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Cache secrète | `SecretCache` | Les 3 attaques choisies dans sa collection, cachées à l'adversaire |
| Draft | `Draft` | Tirage de 14 cartes avec la graine, puis bans et picks en serpent |
| Gacha, collection | hors moteur | Le moteur reçoit les cartes déjà choisies |

## Conventions

- Une famille d'attaques par fichier.
- Modules et fonctions en `snake_case`, types en `PascalCase`.
- Aucune abréviation obscure.
- Un nouveau terme s'ajoute ici **avant** d'apparaître dans le code.
