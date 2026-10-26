# Glossaire

Dernière mise à jour : 2026-10-26.

Un même mot désigne toujours la même chose, dans le GDD, dans le carnet de règles et dans le code.

- **Code du moteur** : les noms sont en anglais, sauf « Pion », qui est le nom même du jeu. Les commentaires et la doc sont en français. Le labo, outil interne, est entièrement en français.
- **Joueur** : le texte affiché est en français, via `Display` ou le thème du client.
- **Vocabulaire du GDD** : en cas de doute, c'est lui qui gagne (« attaque », pas « pouvoir »).

**(cible)** = pas encore dans le code ; le nom est réservé.

## Partie et tour

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Partie | `Game` | L'état complet et le seul point d'entrée des actions |
| Configuration | `GameConfig` | Tous les réglages chiffrés, validés au démarrage |
| Joueur | `Player` | Les deux camps : `One` (commence) et `Two` |
| Tour | `turn` | Le tour **d'un** joueur. Le compteur avance à chaque fin de tour. |
| Manche | `round` (cible) | Un tour de chaque joueur (le « Tour 1 » du tableau du GDD) |
| Action | `Action` | Une décision envoyée au moteur : `Place`, `EndTurn`, puis `UseAttack`, `Choose`, `Pass`… |
| Coût d'une action | `Game::cost` (privé) | Le nombre d'actions de tour qu'elle consomme, lu dans la configuration |
| Coût de la pose | `place_cost` | Actions que coûte la pose d'un pion (1 par défaut) |
| Actions du tour | `actions_left` | Ce qu'il reste à dépenser au joueur courant |
| Actions en réserve | `banked_actions` | Actions non utilisées gardées pour le tour suivant (jusqu'à `max_banked_actions`, 2 par défaut). **À ne pas confondre avec la réserve de pions.** |
| Événement | `Event` | Ce que le moteur annonce après une action |
| Refus | `GameError` | Pourquoi une action ou une configuration est refusée |
| Configuration incohérente | `ConfigError` | La raison précise du refus d'une configuration (`GameError::InvalidConfig`) |
| Décision attendue | `Decision`, `pending_decision()` (cible) | Qui doit choisir quoi, maintenant |
| Actions légales | `legal_actions()` (cible) | La liste exacte des actions acceptées maintenant |
| Issue | `Outcome` (cible) | `Won(Player)` ou `Draw` |
| Graine | `seed` (cible) | Initialise le générateur `Rng`. Elle fait partie de la partie. |
| Enregistrement | `GameRecord` (cible) | Configuration + graine + actions : de quoi rejouer une partie |
| Vue joueur | `PlayerView` (cible) | Ce qu'un joueur a le droit de voir |

## Plateau et pions

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Plateau | `Board` | Le damier `board_size × board_size` |
| Case | `Cell` | Le contenu d'une case. Aujourd'hui un enum, demain une structure en couches (T3). |
| Position | `Position` | Rangée et colonne à partir de 0, affichée « C5 » |
| Direction | `Direction` | Un pas d'une case parmi les 8 directions |
| Pion | `Cell::Pion` → `Pawn` (T3) | Un pion posé, appartenant à un joueur |
| Identité d'un pion | `PawnId` (cible) | Numéro stable d'un pion, pour qu'un effet le suive quand il bouge |
| Réserve de pions | `reserve_pions` | Les pions qu'un joueur peut encore poser (15 au départ) |
| Pion lanceur | `launcher` (cible) | Le pion déjà posé depuis lequel une attaque est lancée |
| Tombe | couche `tomb` de `Cell` (cible) | Ce que laisse un pion tué. Elle ne bloque pas la case. |
| Trou | couche terrain de `Cell` (cible) | Case détruite par le Coup du taureau, injouable |
| Zombie | type de pion (cible) | Pion ressuscité par le Médecin |
| Axe d'alignement | `Direction::AXES`, `axis` | Une des 4 directions de ligne : horizontale, verticale, deux diagonales |
| Alignement | `rules::complete_line` | Une ligne droite d'au moins `align_to_win` pions d'un joueur |
| Règle d'or | victoire en attente (cible) | Une ligne obtenue par une attaque doit survivre à un tour adverse |

## Attaques

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Attaque | `Attack` (cible) | Une carte qu'on déclenche en dépensant des actions |
| Famille d'attaque | `AttackKind` (cible) | Ours, Lièvre, Beauté fatale… Une famille par fichier dans `attacks/`. |
| Rareté | `Rarity` (cible) | `Common`, `Rare`, `Epic`, `Legendary` : la variante de l'attaque et sa place dans le draft, **sans lien imposé avec le coût** |
| Caractéristiques | `AttackStats` (cible) | Coût et recharge d'une variante (famille + rareté) |
| Catalogue | `AttackCatalog` (cible) | Les caractéristiques de toutes les variantes, dans `GameConfig` |
| Arsenal | `Arsenal` (cible) | Les attaques d'un joueur pendant la partie et leur recharge |
| Utiliser une attaque | `Action::UseAttack` (cible) | Choisir une attaque de son arsenal ; les choix suivants passent par `Choose` |
| Recharge | `cooldown` (cible) | Nombre de tours avant de pouvoir réutiliser une attaque |
| Cible | `Target` (cible) | Où s'applique une attaque |
| Effet primitif | `Effect` (cible) | Opération de base produite par une attaque : tuer, déplacer, poser, changer de camp… |
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
