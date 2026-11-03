# Glossaire

Dernière mise à jour : 2026-11-02.

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
| Action | `Action` | Une décision envoyée au moteur : `Place`, `UseAttack`, `Choose`, `FinishAttack`, `Pass` (ne pas réagir), `EndTurn`… |
| Coût de la pose | `place_cost` | Actions que coûte la pose d'un pion (0 dans les règles actuelles, 1 dans les premières). 0 (pose gratuite) seulement si les poses sont limitées par tour. |
| Poses par tour | `max_placements_per_turn`, `GameError::NoPlacementLeft` | Le plus de pions qu'un joueur peut poser dans un tour ; elles ne s'accumulent pas. 1 dans les règles actuelles ; `None` (premières règles) : seules les actions limitent les poses. Une pose de trop est refusée (`NoPlacementLeft`) ; une limite de 0 est refusée à la configuration (`ConfigError::NoPlacementPerTurn`), comme une pose gratuite sans limite (`ConfigError::FreePlacement`). |
| Poses restantes | `Game::placements_left()` | Combien de pions le joueur courant peut encore poser ce tour : selon ses actions, ses poses permises, sa réserve et les cases libres |
| Actions du tour | `actions_left` | Ce qu'il reste à dépenser au joueur courant. Dans les règles actuelles, on parle de **points d'action** : ils ne servent qu'aux cartes. |
| Actions en réserve | `banked_actions`, `Game::banked_actions()` | Actions non utilisées gardées pour le tour suivant (jusqu'à `max_banked_actions` : 3 dans les règles actuelles, 2 dans les premières). **À ne pas confondre avec la réserve de pions.** |
| Banque de départ, la Pièce | `starting_banked_actions`, `ConfigError::StartingBankTooLarge` | Actions que chaque joueur a en banque au début de la partie (la Pièce dans les règles actuelles, 0 dans les premières), au plus `max_banked_actions`. J1 les reçoit avec son premier tour, J2 les trouve en banque. La Pièce : J2 commence avec 1, pour compenser le premier joueur. |
| Règles actuelles | `GameConfig::default()`, `bots::current_rules()`, variante « règles actuelles » | Les règles par défaut depuis le 2026-11-02 (les anciennes « règles cibles ») : 1 pose gratuite par tour, 1 point d'action par tour gardé jusqu'à 3, mal du débarquement, la Pièce, la règle d'or, l'infirmerie de 2 tours (voir le carnet) |
| Premières règles | `GameConfig::first_rules()`, `bots::first_rules()`, variante « premières règles » | Les règles par défaut jusqu'au 2026-11-02, gardées au cas où : 2 actions par tour partagées entre poser (1 action) et attaquer. Les tests de règles partent de cette configuration. |
| Attaques impayables retirées | `GameConfig::without_unaffordable_attacks()` | La configuration sans les attaques qu'aucun tour ne peut payer, même en économisant (catalogue et arsenaux) : pour régler moins d'actions sans rendre la configuration invalide |
| Événement | `Event` | Ce que le moteur annonce après une action ; suffit à suivre tout l'état visible |
| Compteurs d'actions annoncés | `Event::ActionsChanged` | Actions restantes et gardées d'un joueur, émis à chaque changement |
| Poses restantes annoncées | `Event::PlacementsChanged` | Poses restantes d'un joueur (`Game::placements_left`), émis à chaque changement, toujours avant la fin de partie |
| Attaque en cours de choix | `Decision::is_attack_choice()`, `Game::attack_in_progress()` | Une attaque commencée attend son lanceur ou une cible : seules `Action::Choose` et `Action::FinishAttack` sont permises |
| Réserve annoncée | `Event::ReserveChanged` | Pions en réserve d'un joueur, émis à chaque changement (pose, kill sans infirmerie, sortie de l'infirmerie) |
| Effet annoncé | `Event::EffectStarted`, `Event::EffectChanged`, `Event::EffectEnded` | Un effet continu commence, change sans s'arrêter en gardant son identité (un jeton de pas dépensé), ou s'arrête (durée écoulée, retiré par une carte, pion tué) |
| Infirmerie annoncée | `Event::InfirmaryEntered`, `Event::InfirmaryLeft` | Un pion tué entre à l'infirmerie, ou en sort pour revenir dans la réserve |
| Pion déplacé, pions échangés | `Event::PionMoved`, `Event::PionsSwapped` | Un pion change de case (saut, pas), ou deux pions échangent leurs cases d'un coup ; ils gardent leur identité, leurs effets et leur ancienneté |
| Refus | `GameError` | Pourquoi une action ou une configuration est refusée |
| Configuration incohérente | `ConfigError` | La raison précise du refus d'une configuration (`GameError::InvalidConfig`) |
| Décision attendue | `Decision`, `pending_decision()` | Qui doit choisir quoi, maintenant : `MainAction`, `ChooseLauncher`, `ChooseTarget` |
| Actions légales | `legal_actions()` | La liste exacte des actions acceptées maintenant |
| Fin de partie | `GameEnd`, `Event::GameEnded` | Le résultat et la raison de la fin |
| Résultat | `Outcome` | `Won(Player)` ou `Draw` (match nul) |
| Raison de la fin | `EndReason` | `Alignment`, `Frozen` (partie figée) ou `TurnLimit` |
| Partie figée | `EndReason::Frozen` | Plus personne ne pourra jamais agir autrement qu'en finissant son tour, et rien de programmé ne peut changer la situation (aucun effet continu, personne à l'infirmerie) |
| Limite de tours | `max_turns` | Le tour après lequel la partie s'arrête en match nul (200 par défaut) : filet de sécurité |
| Graine | `seed`, `Game::seed()` | Choisie par le client à la création (`Game::new(config, seed)`). Elle fait partie de la partie et initialise le hasard du jeu. |
| Générateur | `Rng` | Hasard déterministe (SplitMix64) : même graine, même suite. Tout le hasard passe par lui. |
| Enregistrement | `GameRecord` | Configuration + graine + actions acceptées : `replay()` reconstruit la partie |
| Échec de relecture | `ReplayError` | `Start` (configuration invalide) ou `RefusedAction { index, error }` (action refusée) |
| Vue joueur | `PlayerView` (cible) | Ce qu'un joueur a le droit de voir |

## Plateau et pions

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Plateau | `Board` | La grille de `board_size + 2 × wall_thickness` cases de côté ; le plateau de jeu en est la partie hors du mur (`Board::contains`, `Board::positions`) |
| Case | `Cell` | Le contenu d'une case du plateau : une structure en couches, un pion et une tombe, chacun optionnel (le mur n'est pas une couche : voir « Mur ») |
| Position | `Position` | Rangée et colonne à partir de 0, affichée « C5 » |
| Direction | `Direction` | Un pas d'une case parmi les 8 directions |
| Pion | `Pion` | Un pion posé, avec son identité et son propriétaire |
| Pion tué | `Event::PionKilled { pion, at, by }` | Un pion retiré du plateau ; `by` est l'identité du tueur. Ses effets s'arrêtent ; il va à l'infirmerie, ou directement dans la réserve de son propriétaire. |
| Identité d'un pion | `PionId` | Numéro stable d'un pion (affiché « #3 »), pour qu'un effet le suive quand il bouge |
| Case d'un pion | `Board::position_of(pion)` | Où se trouve un pion, retrouvé par son identité (un effet vise une identité, pas une case) |
| Réserve de pions | `reserve_pions` | Les pions qu'un joueur peut encore poser (15 au départ) |
| Pion lanceur | `launcher` | Le pion du joueur depuis lequel une attaque est lancée, choisi par `Action::Choose`. C'est aussi le pion qui agit : une carte qui déplace, protège ou échange « un de tes pions » s'applique au lanceur (décidé le 2026-10-30) |
| Mal du débarquement | `summoning_sickness`, `Game::is_summoning_sick(pos)`, `GameError::NoReadyLauncher` | Avec ce réglage, un pion posé ce tour ne peut pas lancer d'attaque (`NoReadyLauncher` vaut aussi pour un pion immobilisé). Un pion posé par une carte ou ressuscité non plus (nouvelle identité) ; un pion déplacé ou échangé garde son ancienneté. |
| Tombe | `Cell::tomb`, `Board::tomb(pos)`, `Board::tomb_of(pion)`, `Event::TombAdded`, `Event::TombRemoved` | Ce que laisse un pion tué qui part à l'infirmerie, sur la case où il est mort, jusqu'à ce qu'il en sorte. Elle ne bloque pas la case ; une case en porte au plus une. |
| Opérations sur les pions | `Board::place_pion`, `remove_pion`, `move_pion`, `swap` (internes) | Le plateau ne s'écrit que pion par pion : une tombe reste sur sa case quand un pion y arrive, en part, bouge ou est échangé |
| Ressusciter | `Effect::Resurrect { pion, at }`, `Game::leave_infirmary`, `Game::create_pion` (internes) | Un pion sort de l'infirmerie (sa tombe s'efface) et revient sur le plateau avec une nouvelle identité, posé ce tour, sans passer par la réserve |
| Mur | `wall_thickness`, `Board::is_wall(pos)`, `ConfigError::WallTooThick` | Les rangées qui entourent le plateau de jeu (3 par défaut). Une case de mur n'est pas sur le plateau (`Board::contains` est faux) : ni libre, ni visible, ni franchissable. `Board::size()` compte la grille entière, mur compris, et les noms « A1 » la couvrent. |
| Case creusée | `Effect::Dig { at }` (interne), `Board::dig` (interne), `Event::WallDug { at }` | Une case de mur que la charge du Coup du taureau a percée : une case normale, pour toujours |
| Axe d'alignement | `Direction::AXES`, `axis` | Une des 4 directions de ligne : horizontale, verticale, deux diagonales |
| Alignement | `rules::complete_line` | Une ligne droite d'au moins `align_to_win` pions d'un joueur |
| Règle d'or | `golden_rule`, `rules::has_line` | Une ligne ne gagne qu'au début du tour de son propriétaire, si elle y est encore. Une ligne présente au début de son tour gagne dans tous les cas ; une ligne en attente gagne aussi quand la partie s'arrête. |

## Attaques

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Attaque | `Attack` | Une carte : une famille dans une rareté (« Ours (Rare) ») |
| Famille d'attaque | `AttackKind` | `BeauteFatale`, `SautDuLoup`, `Reine`, `Attila`, `Interchange`, `Ours`, `Lievre`, `MeilleursAmis`, `Medecin`, `Familia`, `CoupDuTaureau`… Le nom de la carte, tel quel (« Beauté fatale »). Une famille par fichier dans `attacks/`. |
| Rareté | `Rarity` | `Common`, `Rare`, `Epic`, `Legendary` : la variante de l'attaque et sa place dans le draft, **sans lien imposé avec le coût** |
| Caractéristiques | `AttackStats` | Coût et recharge d'une attaque |
| Catalogue | `attack_catalog`, `CatalogEntry` | Dans `GameConfig` : les attaques jouables et leurs caractéristiques |
| Attaques de départ | `starting_attacks` | Dans `GameConfig` : l'arsenal de chaque joueur au début (plus tard, sa cache secrète) |
| Arsenal | `Game::arsenal(player)`, `ArsenalSlot` | Les attaques d'un joueur pendant la partie, chacune avec sa recharge en cours |
| Utiliser une attaque | `Action::UseAttack(slot)` | Commencer l'attaque d'un emplacement de son arsenal ; le lanceur et les cibles se choisissent ensuite |
| Choisir une case | `Action::Choose(pos)` | Le lanceur, puis chaque cible de l'attaque en cours ; l'attaque est jouée dès qu'il n'y a plus rien à choisir |
| Jouer l'attaque | `Action::FinishAttack`, `GameError::TargetsMissing` | Jouer l'attaque en cours avec les cibles déjà choisies, avant son maximum (« jusqu'à 2 ») ; refusé tant qu'elle n'en a pas assez |
| Nombre de cibles | `AttackRules::target_count` (interne) | De combien à combien de cibles une variante choisit (`1..=2` : « jusqu'à 2 » ; `0..=0` : aucune, jouée dès le choix du lanceur) |
| Peut s'arrêter | `AttackRules::can_stop` (interne) | Vrai si l'attaque peut être jouée après ces choix, quand leur nombre le permet ; par défaut toujours. Le Lièvre attend que le dernier pion choisi ait bougé. |
| Portée | `attacks/reach.rs` : `around`, `in_sight`, `in_line` (interne) | Où une carte trouve ses cibles, partagé entre familles : collé au lanceur, regard (ligne droite, sans pion entre : Beauté fatale), ligne droite (même derrière un pion : Interchange Rare). Pas de portée par défaut. |
| Pion prêt à agir | `can_act`, `AttackContext::can_act`, `Launch` (internes) | La seule définition d'un lanceur prêt : ni immobilisé, ni arrivé ce tour quand le mal du débarquement compte. Partagée par la partie (le lanceur) et par les familles qui font agir plusieurs de tes pions (Lièvre). `Launch::Someday` l'ignore, pour la partie figée. |
| Recharge | `cooldown`, `cooldown_left`, `Event::CooldownChanged` | Tours du propriétaire avant de pouvoir relancer une attaque (1 = une fois par tour) |
| Cible | case choisie | Une case visée par l'attaque en cours, proposée par la famille |
| Attaque lancée | `Event::AttackUsed` | Annonce d'une attaque résolue ; ses effets suivent |
| Effet primitif | `Effect` (interne au moteur) | Opération de base produite par une famille : `Kill` (tuer), `Apply` (donner un effet continu), `RemoveEnemyStatuses` (retirer les statuts adverses d'un pion), `Move` (déplacer), `Swap` (échanger), `Place` (poser depuis la réserve), `Bind` (lier par un pacte), `Resurrect` (ressusciter depuis l'infirmerie), `Crown` (couronner une reine, avec ses gardes), `MakeChild` (faire un enfant), `CancelEnemyEffects` (annuler des effets adverses entiers)… (catalogue dans l'architecture). Seul le résolveur l'applique. Il n'y a pas de « changer de camp » : séduire, c'est tuer puis poser. |
| Effet continu | `ActiveEffect`, `EffectId`, `Game::active_effects()` | Ce qu'une carte laisse en jeu après avoir été jouée : les statuts, le pacte, la couronne, les gardes et la protection d'un enfant ; plus tard retour programmé (les jetons du Massacre divin, posés devant un joueur, sont des piles de jetons, à part). Il vise un pion par son identité, retient son créateur (`creator`), sa carte (`source`), son jeu de carte (`play`) et son dernier tour (`last_turn`, `None` pour un effet sans durée comme le pacte). Les fiches disaient aussi « effet en cours ». Les pions, l'infirmerie, les tombes et les cases creusées n'en sont pas. |
| Statut | `EffectKind::Immobilized`, `EffectKind::Protected`, `Game::has_effect(pos, kind)` | Immobilisé (ne lance pas d'attaque, donc ne bouge pas par une carte de son propriétaire) ou protégé (ne peut pas être choisi comme cible d'un kill) : un effet continu sur un pion |
| Reine, couronne | `EffectKind::Crown`, `Game::queen_of(player)` | Le pion posé par la Reine Rare à Légendaire, une seule à la fois par joueur. La couronne est un effet sans durée, qui la suit. |
| Pas de la reine | `Action::StepQueen(pos)`, `GameError::QueenCannotStep` | Son pas gratuit d'une case, une fois par tour, à n'importe quel moment ; ce n'est pas une attaque |
| Garde, bouclier, tir | `EffectKind::Guard`, `Game::guards_of(pion)`, `Action::Shoot(pos)`, `Event::QueenShot`, `guard_shot_cost`, `GameError::QueenCannotShoot` | Un jeton sur la reine, qui sert une fois : il la sauve de ce qui la tuerait (bouclier, dans `kill`), ou tire sur le premier pion adverse en ligne droite (un tir par tour) |
| Ignorer le lanceur | `AttackRules::ignores_launcher` (interne) | Une variante dont les cibles ne dépendent pas de la case du lanceur (n'importe où) : savoir si elle a une cible ne demande qu'un lanceur prêt. Un test vérifie chaque déclaration. |
| Famille, enfant, parent | `AttackKind::Familia`, `Effect::MakeChild`, `ActiveEffect::framed_by()`, `AttackContext::is_child` et `Game::end_unframed_children` (internes) | La Familia fait des **enfants** : un de tes pions encadré en ligne droite par deux membres de sa **famille** (les **parents**, posés depuis la réserve, ou son frère). Sa protection est un statut protégé sans durée, qui retient les deux pions qui l'encadrent (`framed_by`) et tombe pour de bon quand ils ne l'encadrent plus. « Encadrer » : être sur les deux cases collées de part et d'autre, dans une même ligne (`both_sides`, interne). Dans le code, la famille d'un enfant ne s'appelle jamais `family` seul, pour ne pas la confondre avec une famille d'attaque (`AttackKind`) : on parle des pions qui l'encadrent. |
| Jeu de carte, effet entier | `PlayId`, `ActiveEffect::play()`, `EffectKind::is_cancellable()`, `CardPlay` (interne) | Un **jeu de carte** est une attaque jouée ; il reçoit une identité (`PlayId`), dans l'ordre des jeux. Tous les effets continus qu'il laisse forment un **effet entier**, qu'Attila Rare et Épique annulent d'un coup. Tout s'annule, sauf la couronne (`is_cancellable`). |
| Pacte, ami | `EffectKind::Pact { friend }`, `EffectKind::is_status()` | L'effet continu de Meilleurs amis, porté par chacun des deux amis, sans durée (`last_turn() == None`). Ce n'est pas un statut : Attila Commune ne le retire pas. |
| Kill, assassin, vengeance | `Game::kill`, `Game::avenge`, `Death` (internes, `game/kill.rs`) | Un kill envoie un pion à l'infirmerie ou au cimetière ; l'assassin est le lanceur de l'attaque. À la fin d'une attaque, l'ami survivant d'un pacte tue l'assassin de son ami (en chaîne). |
| Pose par une carte | `Effect::Place`, `Game::put_pion` (interne) | Un pion de la réserve posé par une carte (séduction, Meilleurs amis) : nouvelle identité, posé ce tour, sans utiliser la pose normale. La pose normale passe aussi par `put_pion`. |
| Durée, dernier tour | `last_turn` | Une durée de N tours se compte en tours de l'adversaire de celui qui a joué l'effet, sans le tour en cours ; l'effet s'arrête à la fin de son dernier tour (`clock.rs`) |
| Infirmerie | `infirmary_turns`, `Game::infirmary()`, `Patient` | Où va un pion tué, pour `infirmary_turns` tours (2 dans les règles actuelles, 0 dans les premières), avant de revenir dans la réserve de son propriétaire. Un `Patient` est un pion qui y attend. |
| Cimetière | `Game::graveyard()`, `Event::GraveyardEntered`, `TokenReward::Graveyard` | Où va un pion tué par un kill qui use un jeton du Massacre divin Épique ou Légendaire : sans tombe, il ne revient jamais |
| Fin d'un effet | `EffectEnd`, `ActiveEffect::end()` | Comment un effet continu s'arrête : à la fin d'un tour (`Turn`), au début du prochain tour de son créateur (`NextTurn`, le retour), quand sa famille n'encadre plus l'enfant (`Framed`), quand ses jetons sont dépensés (`Tokens`), ou jamais tout seul (`Never` : pacte, couronne, gardes). `last_turn()`, `framed_by()` et `tokens()` en sont des lectures. |
| Retour programmé, téléportation | `EffectKind::Return { origin }`, `Effect::Attach`, `game/returns.rs` | L'Interchange Commune téléporte le lanceur (`Effect::Move`) et lui attache un retour : au début du prochain tour de son créateur, il revient sur `origin`, et l'occupant est tué. |
| Jetons de pas, pas du Lièvre | `EffectKind::StepTokens`, `EffectEnd::Tokens`, `Action::HareStep { from, to }`, `Event::EffectChanged`, `GameError::HareCannotStep` | Les jetons du Lièvre Épique, sur un pion (un effet continu, contrairement aux piles du Massacre divin). Le premier pas du tour en dépense un (`EffectChanged`, ou `EffectEnded` pour le dernier), le second est facultatif. Un geste gratuit, pas une attaque. |
| Terre brûlée | `Effect::ScorchEarth`, `Game::scorched_until(player)`, `Event::ScorchedEarthStarted`, `Event::ScorchedEarthEnded`, `GameError::ScorchedEarth`, `Effect::creates_lasting` (interne), `AttackContext::may_create_effects` (interne) | Attila Légendaire : pendant son tour suivant, l'adversaire ne peut jouer aucun coup qui créerait un effet continu ou une pile de jetons. Ce n'est pas un effet continu : rien ne l'annule. |
| Jeton, pile de jetons | `TokenPile`, `TokenReward`, `Game::token_piles()`, `Effect::AddTokens`, `Event::TokenPileStarted`, `Event::TokenUsed`, `Event::TokenPileEnded` | Une **pile** est posée devant un joueur par un jeu du Massacre divin (pas sur un pion : ce n'est pas un `ActiveEffect`). Chacun de ses prochains kills use un **jeton** de chacune de ses piles, qui rend des points (`Points`) ou envoie au cimetière (`Graveyard`). Une pile est un effet entier : Attila Rare et Épique l'annulent (`Effect::CancelEnemyTokens`). Les gardes de la reine, sur la reine, ne sont pas des jetons. |
| Viser les jetons adverses | `Action::ChooseEnemyTokens`, `AttackRules::enemy_tokens_effects`, `Targets` (interne) | Une cible qui n'est pas une case : toutes les piles de l'adversaire (Attila Rare et Épique). Elle compte pour une cible. Les cibles d'une attaque en cours (`Targets`) sont des cases, plus peut-être les jetons adverses. |
| Pions en vie, manque de pions | `Game::living_pions(player)`, `EndReason::TooFewPions` | Les pions d'un joueur sur le plateau, en réserve et à l'infirmerie. Un joueur qui a un pion au cimetière et moins de `align_to_win` pions en vie a perdu (nul si les deux). |
| Réaction | `AttackKind::is_reaction`, `Trigger` (interne) | Carte de l'arsenal jouée pendant le tour adverse, juste après son déclencheur (une pose, un déplacement, la mort d'un de ses pions, une attaque ; ou, avant qu'elle ne s'applique, une attaque qui vise un de ses pions), payée avec les actions en réserve ; une par tour adverse |
| Fenêtre de réaction, réagir, passer | `Decision::React`, `Action::Pass`, `ReactionWindow` (interne) | Le moment où l'autre joueur peut répondre au coup qui vient d'être joué ; elle ne s'ouvre que s'il a une réaction jouable. Pendant elle, c'est lui qui décide (`Decision::player`) |
| Ligne tardive | `late_lines` (interne) | Une ligne qu'une réaction a donnée à son joueur : elle gagne un tour plus tard, après un tour adverse complet |
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
| Poses possibles | `placement_budget` (interne) | Combien de pions un joueur peut poser dans son tour : `Game::placements_left` pour le tour en cours, ou sur une copie où l'on a fini son tour pour le prochain |
| Plan d'attaque | `AttackPlan` (interne) | Une attaque jouée jusqu'au bout sur une copie de la partie, pour voir son effet sans connaître sa règle |
| Arène | `bots::run`, `bots::run_pairs`, `cargo arene` | Joue N parties entre deux bots (graines 0 à N−1) et en fait le bilan ; `run_pairs` le fait pour plusieurs paires en parallèle |
| Bots disponibles | `BotKind` | La liste des bots proposés par l'arène et le labo : aléatoire, glouton, banquier |
| Paires de l'arène | `STANDARD_PAIRS` | Les paires (joueur 1, joueur 2) jouées par défaut |
| Règles actuelles (bots) | `bots::current_rules()` | La configuration par défaut où chaque joueur a toutes les attaques : la première variante, et la partie de départ du labo |
| Variante | `Variant`, `variants()` | Une configuration nommée, comparée aux règles actuelles ; la liste est partagée par l'arène et le labo |
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
| Jouer l'attaque | `Intention::FinirAttaque` | Le bouton qui joue l'attaque en cours de choix avec les cibles déjà choisies (`Action::FinishAttack` au bout de la suite) ; activé seulement si le moteur le permet sur la copie |
| Revoir | `Intention::Revoir` | Ouvrir dans l'écran Partie une partie de l'arène (configuration, graine, bots), qui se rejoue à l'identique, même après une annulation |
| Hasard avant chaque action | `Partie::hasard_avant` | L'état du hasard des bots avant chaque action enregistrée : annuler le remet, pour que les bots rejouent les mêmes coups |
| Simulation | `Simulation` (écran Arène) | Ce qui est demandé à l'arène : configuration (et d'où elle vient), paires, nombre de parties |
| Réglage de partie | `ReglageDePartie` | Une configuration qu'on règle à l'écran : on part d'une variante ou de la partie en cours, puis on retouche chaque réglage (plateau, pions, actions, catalogue, arsenaux). `GameConfig::validate` décide si elle se lance. Il y en a un par écran : la **prochaine partie** (que lance « Nouvelle partie ») et la **configuration simulée** (sur laquelle l'arène fait jouer les bots). |
| Retouchée | `ReglageDePartie::retouchee` | La configuration réglée n'est plus exactement celle dont on est parti (variante ou partie en cours) |

## Cartes avant la partie (cible, B9)

| Terme du jeu | Nom dans le code | Rôle |
|---|---|---|
| Cache secrète | `DraftStep::Cache` | Les cartes (1 par défaut) que chaque joueur choisit dans tout le catalogue avant le draft ; cachées à l'adversaire jusqu'à leur premier usage (`ArsenalSlot::is_secret`, `Event::CardRevealed`, `Game::arsenal_seen_by`, `Game::with_secrets_guessed`) |
| Draft | `GameConfig::draft`, `DraftRules`, `DraftStep`, `Decision::Draft`, `Action::Draft`, `Event::Drafted`, `Game::draft_pool` | Avant la partie : la cache, puis des cartes tirées avec la graine (10 par défaut), bannies et prises dans l'ordre des étapes (1 ban chacun, 3 picks en serpent) |
| Gacha, collection | hors moteur | Le moteur reçoit les cartes déjà choisies |

## Conventions

- Une famille d'attaques par fichier.
- Modules et fonctions en `snake_case`, types en `PascalCase`.
- Aucune abréviation obscure.
- Un nouveau terme s'ajoute ici **avant** d'apparaître dans le code.
