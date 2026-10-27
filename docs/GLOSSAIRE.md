# Glossaire

Dernière mise à jour : 2026-10-27.

Un même mot désigne toujours la même chose, dans le GDD, dans le carnet de règles et dans le code.

- **Code du moteur** : les noms sont en anglais, sauf « Pion », qui est le nom même du jeu et s'écrit ainsi partout : `Pion`, `PionId`, `PionPlaced`, `pions_per_player`, `reserve_pions`. Jamais « pawn ». Les commentaires et la doc sont en français. Le labo, outil interne, est entièrement en français.
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
