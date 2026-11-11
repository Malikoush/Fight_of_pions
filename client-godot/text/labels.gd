class_name Labels
extends RefCounted
## Tout le texte affiché par le client, en français, réuni ici. Le serveur envoie des
## identifiants (`One`, `EndTurn`…) ; ce fichier dit comment les montrer. Les messages
## d'erreur du serveur, déjà en français, sont affichés tels quels.

const GAME_NAME := "Fight of Pions"
## Le nom du jeu en grand, au démarrage, sur deux lignes (maquettes).
const GAME_NAME_STACKED := "Fight of
Pions"
## La phrase d'accroche du plateau de l'accueil, en paysage.
const SLOGAN := "Aligne 6 pions
avant ton adversaire."

const CONNECTING := "Connexion au serveur…"
const UNREACHABLE := "Serveur injoignable, nouvel essai…"
const RETRY := "RÉESSAYER"

const PLAY_BOT := "JOUER CONTRE LE BOT"
## Le bot à affronter (`Settings.bot_opponent`) : son nom sur sa pastille, et ce qu'il
## promet sous le bouton.
const BOT_LEVELS := {"random": "FACILE", "mcts": "DIFFICILE"}
const BOT_DETAILS := {
	"random": "Pour s'entraîner, sans récompense",
	"mcts": "Chaque victoire débloque une carte",
}
const FRIENDS_SECTION := "PARTIE ENTRE AMIS"
const CREATE_ROOM := "CRÉER UNE SALLE"
const CREATE_ROOM_DETAIL := "Un code à envoyer"
const JOIN := "REJOINDRE"
const JOIN_DETAIL := "Avec le code de ton ami"
## La partie entre amis en paysage, à droite de l'accueil.
const FRIENDS_TITLE := "Partie entre amis"
const FRIENDS_INTRO := "Crée une salle, envoie le code à ton ami et lance la partie quand il arrive."
const OR := "ou"
const JOIN_WITH_CODE := "Rejoindre avec un code"
const CODE_HINT := "Tape le code au clavier."
const RESUME_TIP := "Déconnecté pendant une partie ? Reviens vite pour la reprendre."
## Le pseudo, tant que le compte n'est pas connu.
const DEFAULT_PLAYER_NAME := "Joueur"
const JOIN_TITLE := "Rejoindre une salle"
const JOIN_HELP := "Entre le code à 5 caractères que ton ami t'a envoyé."
const BACK := "RETOUR"
const FRIEND := "Ton ami"
## La salle d'attente d'une partie entre amis.
const LOBBY_TITLE := "Créer une salle"
const LOBBY_WAITING := "Envoie ce code à ton ami, puis lance la partie quand il arrive."
const LOBBY_READY := "Ton ami est arrivé. Tu peux lancer la partie."
## La salle d'attente chez l'ami qui l'a rejointe, et qui commence (tiré au sort).
const LOBBY_GUEST_TITLE := "Salle de ton ami"
const LOBBY_GUEST_READY := "C'est bon, ton ami va lancer la partie."
const LOBBY_GUEST_ALONE := "Ton ami s'est déconnecté. On attend son retour."
const LOBBY_YOU_START := "Tirage au sort : tu commences."
const LOBBY_FRIEND_STARTS := "Tirage au sort : ton ami commence."
const PLAYER_READY := "PRÊT"
const PLAYER_WAITING := "EN ATTENTE"
const VERSUS := "contre"
const LAUNCH := "LANCER LA PARTIE"
const CLOSE_ROOM := "FERMER LA SALLE"
const LEAVE_ROOM := "QUITTER LA SALLE"
## La question avant de partir de la salle d'attente, chez son créateur et chez son ami.
const CLOSE_ROOM_TITLE := "Fermer la salle ?"
const CLOSE_ROOM_TEXT := "Ton ami ne pourra plus la rejoindre avec ce code."
const LEAVE_ROOM_TITLE := "Quitter la salle ?"
const LEAVE_ROOM_TEXT := "Tu pourras la rejoindre avec le code tant que la partie n'est pas lancée."

## Le draft.
const ROOM_CODE := "CODE DE LA SALLE"
const COPY := "COPIER"
const COPIED := "COPIÉ"
## L'espace entre une icône et son texte, dans une même ligne.
const ICON_SPACE := " "
## Le curseur, dans la case de la prochaine lettre du code.
const CODE_CARET := "|"
## Partager, sans le partage natif du téléphone (à venir avec les exports, R9) : le message
## d'invitation est copié, prêt à coller.
const SHARE := "PARTAGER"
const INVITE_COPIED := "INVITATION COPIÉE"
const INVITE := "Rejoins-moi sur Fight of Pions avec le code %s."
const ROOM_WAITING := "Envoie ce code à ton ami pour qu'il te rejoigne."
const MY_ARSENAL := "MON ARSENAL"
const DRAFT_CHOOSE := "CHOISIS UNE CARTE"
const DRAFT_WAIT := "%s choisit…"
const ALL_RARITIES := "TOUTES"
## Le nom de chaque phase du draft, dans la barre des étapes.
const DRAFT_PHASES := {"Ban": "BANS", "Pick": "PICKS", "Cache": "CACHE"}
## Ce qu'on fait à chaque étape : le titre, l'aide, le bouton (avec le nom de la carte).
const DRAFT_TITLES := {"Ban": "Bannis une carte", "Pick": "À toi de prendre", "Cache": "Choisis ta carte cachée"}
const DRAFT_HELP := {
	"Ban": "Une carte bannie ne pourra être prise par personne.",
	"Pick": "La carte prise rejoint ton arsenal. Ton adversaire voit tes picks.",
	"Cache": "Ton adversaire ne la découvrira que quand tu la lanceras.",
}
const DRAFT_ACTIONS := {"Ban": "BANNIR %s", "Pick": "PRENDRE %s", "Cache": "VALIDER MA CACHE"}
## Ce que fait l'adversaire pendant qu'il choisit.
const DRAFT_THINKING := {"Ban": "bannit une carte…", "Pick": "prend une carte…", "Cache": "choisit sa cache…"}
## Le rang du pick en cours (maquettes : « Pick 4 sur 6 · en serpent »).
const DRAFT_PICK_RANK := "Pick %d sur %d · en serpent"
## Les étiquettes des cartes tirées : qui l'a prise (le nom court du joueur), bannie, à bannir.
const TAG_YOU := "TOI"
const TAG_BOT := "BOT"
const TAG_FRIEND := "AMI"
const TAG_BANNED := "BANNI"
const TAG_BANNED_BY := "BANNI · %s"
const TAG_TO_BAN := "BANNIR ?"
## Les places encore vides d'un arsenal, par sorte de choix.
const DRAFT_SLOTS := {"Pick": "PICK", "Cache": "CACHE"}
## Ce que le draft ajoute à mon arsenal (« 3 picks + 1 cache ») : une part par sorte, au
## singulier ou au pluriel.
const DRAFT_PLAN_PARTS := {"Pick": ["%d pick", "%d picks"], "Cache": ["%d cache", "%d caches"]}
const DRAFT_PLAN_JOIN := " + "
## Ce que vient de faire l'adversaire, au draft.
const DRAFT_DONE := {"Ban": "a banni %s", "Pick": "a pris %s"}
const DRAFT_CACHED := "a mis une carte en cache"
## Les raretés, pour filtrer le catalogue pendant le choix de la cache.
const RARITY_FILTERS := {"Common": "COMMUNE", "Rare": "RARE", "Epic": "ÉPIQUE", "Legendary": "LÉGENDAIRE"}

const YOU := "Toi"
const BOT := "Bot"
const OPPONENT := "Adversaire"
const YOUR_TURN := "à toi de jouer"
const WAITS := "attend"
const PLAYS := "joue"
const THINKS := "réfléchit…"
## Sous le tour, pendant que l'adversaire décide.
const BOT_THINKING := "Le bot réfléchit…"
const OPPONENT_PLAYING := "L'adversaire joue…"

const WAITING := "EN ATTENTE"
## Le bandeau du début de ton tour.
const YOUR_TURN_BANNER := "À toi !"
const HOME := "ACCUEIL"

const FREE_PLACEMENT := "Pose gratuite disponible"
const PLACEMENT := "Pose disponible"
const PLACEMENTS := "%d poses disponibles"
const NO_PLACEMENT := "Plus de pose ce tour"
const GAME_OVER := "Partie finie"

## Les cartes et leur choix.
const SECRET_TAG := "CACHE"
## La fiche d'une carte.
const SHEET_COST := "COÛT"
const SHEET_COOLDOWN := "RECHARGE"
const SHEET_YOUR_CACHE := "TA CACHE"
const SHEET_NO_COOLDOWN := "aucune"
## Sous le bouton d'action, en paysage (PC).
const ENTER_SHORTCUT := "Raccourci : Entrée"
const STEP_LAUNCHER := "LANCEUR"
const STEP_TARGETS := "CIBLES"
const CHOOSE_LAUNCHER := "Choisis le lanceur"
const CHOOSE_TARGET := "Choisis une cible"
const READY_TO_PLAY := "Prête à jouer"
const CANCEL := "ANNULER"
const PLAY_CARD := "JOUER LA CARTE"
const AIM_TOKENS := "VISER LES JETONS"
const HARE_DESTINATION := "Touche la case où va ton pion."

## Ce qui dure, dans le panneau d'un joueur.
const FACT_SCORCHED := "Terre brûlée · jusqu'au tour %d"
const FACT_INFIRMARY := "Infirmerie %d"
const FACT_GRAVEYARD := "Cimetière %d"
const FACT_GUARDS := "Gardes %d"

## Les onglets du menu.
const NAV_PLAY := "JOUER"
const NAV_CARDS := "CARTES"
const NAV_RULES := "RÈGLES"
const NAV_SETTINGS := "RÉGLAGES"
const CARDS_TITLE := "Cartes"
## Le compte des cartes (« 12 familles · 5 réactions »), le filtre des réactions.
const CARDS_COUNT := "%d familles · %d réactions"
const REACTIONS_FILTER := "RÉACTIONS"
## Le filtre de la collection : ses cartes seulement (par défaut), ou tout le catalogue ;
## avec leur nombre. Une carte pas encore débloquée est grisée, étiquetée.
const OWNED_FILTER := "MES CARTES · %d"
const CATALOG_FILTER := "TOUTES · %d"
const TAG_NOT_OWNED := "À DÉBLOQUER"
const NO_CARD_MATCHES := "Aucune de tes cartes ne correspond à ce filtre. Gagne des parties pour en débloquer."
## Le trio de cache, choisi dans la page Cartes et gardé par le serveur : au draft, la cache
## se choisit dans ce trio. Le bouton de la fiche, et où il en est.
const TRIO_ADD := "AJOUTER À MA CACHE"
const TRIO_REMOVE := "RETIRER DE MA CACHE"
const TRIO_FULL := "CACHE PLEINE"
const TRIO_NOT_OWNED := "PAS ENCORE DÉBLOQUÉE"
const TRIO_TITLE := "MA CACHE"
const TRIO_SLOTS := "%d / %d"
const TRIO_FREE := "LIBRE"
const TRIO_HELP := "Touche une de tes cartes pour l'ajouter à ta cache. Sans cartes, ta cache est tirée au hasard dans ta collection à chaque partie."
const CARDS_LOADING := "Chargement des cartes…"
const COLLECTION_LOADING := "Chargement de ta collection…"
const RULES_TITLE := "Règles"
## Le carrousel des règles : la page (« 1 / 6 »), les boutons, et ce que montrent ses dessins.
const RULES_PAGE := "%d / %d"
const PREVIOUS := "PRÉCÉDENT"
const NEXT := "SUIVANT"
const PLAY := "JOUER"
const RULES_ONE_PLACE := "1 POSE"
const RULES_ONE_POINT := "1 POINT"
const RULES_BANK := "BANQUE"
const RULES_DRAFT_PLAN := "3 PICKS + 1 CACHE"
const SETTINGS_TITLE := "Réglages"
## Les réglages (maquettes) : ses sections et ses lignes.
const SETTINGS_PROFILE := "PROFIL"
const SETTING_NAME := "Pseudo"
const SETTING_NAME_NOTE := "Appuie sur Entrée pour valider."
const SETTINGS_SOUND := "SON"
const SETTING_MUSIC := "Musique"
const SETTING_EFFECTS := "Effets"
const SETTING_VIBRATION := "Vibrations (téléphone)"
const SETTINGS_ABOUT := "À PROPOS"
const ABOUT_TEXT := "Fight of Pions, version de développement.\nUn jeu de Martin Ouedraogo et Théo Da Costa."
const SETTINGS_NOTE := "Le son et les vibrations ne valent que pour cet appareil."
## Une valeur en pourcentage (« 60 »).
const PERCENT := "%d"
## L'essentiel des règles, section par section (`[titre, texte]`), d'après le carnet de
## règles (docs/REGLES.md) ; le détail des cartes est dans la page Cartes.
const RULES := [
	["Aligne 6 pions", "Aligne 6 pions de ta couleur, en ligne, en colonne ou en diagonale, avant ton adversaire. Si plus personne ne peut jouer, ou après la limite de tours, c'est match nul."],
	["Ton tour", "Chaque tour, tu peux poser un pion gratuitement et tu gagnes 1 point d'action. Les points que tu ne dépenses pas restent en banque, jusqu'à 3."],
	["Cartes et points", "Une carte coûte des points d'action, puis se recharge quelques tours. Elle part d'un de tes pions, le lanceur, posé avant ce tour. Un pion tué part à l'infirmerie quelques tours, puis revient dans ta réserve."],
	["Réactions", "Certaines cartes se jouent pendant le tour adverse, juste après un coup qui les déclenche, payées avec ta banque. Une seule par tour adverse."],
	["Tenir sa ligne", "Une ligne ne gagne qu'au début de ton tour suivant, si elle tient encore. Ton adversaire a donc un tour pour la casser."],
	["Le draft", "Avant la partie, 10 cartes sont tirées : chacun en bannit une, puis en prend 3 en serpent. Enfin, chacun choisit une carte cachée parmi les 3 cartes de sa cache. L'adversaire ne la découvre que quand tu la lances."],
]

## La fin de partie, les coupures et les départs (E3g).
const RESULT_WIN := "Victoire"
const RESULT_LOSS := "Défaite"
const RESULT_DRAW := "Match nul"
## La raison de la fin, par raison du moteur (`EndReason`) ; `%d` : les pions à aligner.
const END_REASONS := {
	"Alignment": "%d pions alignés.",
	"Frozen": "Plus personne ne pouvait jouer.",
	"TurnLimit": "Limite de tours atteinte.",
	"TooFewPions": "Plus assez de pions pour aligner.",
}
## La fin par abandon, selon qu'on gagne (l'adversaire est parti, ou absent trop longtemps)
## ou qu'on perd (on était absent trop longtemps : qui part lui-même ne voit pas la fin).
const FORFEIT_WON := "Ton adversaire a abandonné."
const FORFEIT_LOST := "Tu as été absent trop longtemps : défaite par abandon."
const REPLAY := "REJOUER"
## Sous la raison de la fin : contre qui, et à quel tour.
const END_CAPTION := "Contre %s · tour %d"
## La place gardée que le serveur n'a plus, au retour (salle fermée, partie perdue par
## absence).
const SEAT_GONE := "Ta partie a été fermée pendant ton absence."
const SEAT_TAKEN_TITLE := "Partie ouverte ailleurs"
const SEAT_TAKEN_TEXT := "Ta partie est ouverte sur un autre appareil ou dans un autre onglet."
const RESUME_HERE := "JOUER ICI"
## Les fenêtres de coupure (maquettes) : connexion perdue, bot muet, serveur complet.
const CONNECTION_LOST_TITLE := "Connexion perdue"
const CONNECTION_LOST_TEXT := "Reconnexion en cours… Ta place dans la partie est gardée."
const BOT_DOWN_TITLE := "Le bot ne répond plus"
const BOT_DOWN_TEXT := "La partie est en pause. Tu peux attendre qu'il revienne ou quitter la partie."
const WAIT := "ATTENDRE"
const SERVER_FULL_TITLE := "Serveur complet"
const SERVER_FULL_TEXT := "Trop de parties sont en cours. Réessaie dans un instant."
const BOT_FAILED := "Le bot ne répond plus, nouvel essai…"
const FRIEND_LEFT := "Ton ami s'est déconnecté. S'il ne revient pas vite, tu gagnes par abandon."
const QUIT_TITLE := "Quitter la partie ?"
const QUIT_TEXT := "Si tu quittes, tu abandonnes : ton adversaire gagne et tu ne pourras pas revenir dans la partie."
const STAY := "RESTER"
const QUIT := "QUITTER"

## Le tour adverse : l'annonce d'une carte adverse, sa cache révélée, la fenêtre de réaction.
const ATTACK_TAG := "ATTAQUE"
const REACTION_TAG := "RÉACTION"
const ANNOUNCE_TITLE := "%s lance %s"
const ANNOUNCE_FROM := "depuis son pion en %s"
const REVEAL_TAG := "CACHE RÉVÉLÉE"
const REVEAL_TITLE := "%s lance sa carte cachée"
const REVEAL_DETAIL := "%s · coût %d"
const TAP_TO_CONTINUE := "Toucher pour continuer"
## La carte gagnée par une victoire, montrée comme la cache révélée.
const REWARD_TAG := "NOUVELLE CARTE"
const REWARD_TITLE := "Tu débloques une carte"
const REWARD_DETAIL := "%s · ajoutée à ta collection"
const REACT_QUESTION := "Réagir ?"
const REACT_DEFAULT_TITLE := "L'adversaire vient de jouer"
const PLAY_REACTION := "JOUER %s"
const NOT_PLAYABLE_NOW := "pas jouable maintenant"

## Le journal des coups.
const JOURNAL_TITLE := "JOURNAL DES COUPS"
const JOURNAL_ORDER := "les plus récents en haut"
const JOURNAL_EMPTY := "Aucun coup pour l'instant."
const JOURNAL_TURN := "T%d"
## Les lettres des colonnes, de gauche à droite (le moteur s'arrête aussi à Z).
const COLUMN_LETTERS := "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
const CLOSE := "FERMER"
const JOURNAL_PLACE := "pose en %s"
const JOURNAL_REVEAL := "révèle sa carte cachée : %s"
const JOURNAL_ATTACK := "lance %s depuis %s"
const JOURNAL_SHOT := "tire avec un garde de sa reine sur %s"
const JOURNAL_KILLED := "pion tué en %s"
const JOURNAL_MOVED := "pion de %s à %s"
const JOURNAL_SWAPPED := "échange de %s et %s"
const JOURNAL_DUG := "mur creusé en %s"
const JOURNAL_GRAVEYARD := "un pion part au cimetière"
const JOURNAL_HEALED := "un pion sort de l'infirmerie"
const JOURNAL_SCORCHED := "sous Terre brûlée jusqu'au tour %d"
const JOURNAL_TOKENS := "reçoit une pile : %s"
const JOURNAL_CHILD := "enfant en %s"
## Les effets continus racontés quand ils commencent, par sorte (`%s` : la case du pion).
const JOURNAL_EFFECTS := {
	"Immobilized": "pion immobilisé en %s",
	"Protected": "pion protégé en %s",
	"Pact": "pacte en %s",
	"Crown": "reine couronnée en %s",
	"Guard": "un garde pour la reine en %s",
	"Return": "retour prévu pour le pion en %s",
	"StepTokens": "jetons de pas en %s",
}

## Le libellé de chaque coup proposé dans le menu d'une case.
const CELL_ACTIONS := {
	"Place": "POSER ICI",
	"StepQueen": "PAS DE LA REINE",
	"Shoot": "TIR DE LA REINE",
	"HareStep": "PAS DU LIÈVRE",
}

## Le libellé des coups sans case, sur le bouton d'action.
const ACTION_LABELS := {
	"EndTurn": "FINIR LE TOUR",
	"Pass": "PASSER",
}


## « TOUR 7 · À TOI » ou « TOUR 8 · ADVERSAIRE ».
static func turn_pill(turn: int, mine: bool) -> String:
	return "TOUR %d · %s" % [turn, "À TOI" if mine else "ADVERSAIRE"]


## Ce qui reste à poser ce tour, ou une chaîne vide quand ce n'est pas à dire.
static func placements(left: int, cost: int, my_turn: bool) -> String:
	if not my_turn:
		return ""
	if left == 0:
		return NO_PLACEMENT
	if left > 1:
		return PLACEMENTS % left
	return FREE_PLACEMENT if cost == 0 else PLACEMENT


## Une pile de jetons du Massacre divin : « 3 jetons · +1 point » ou « 2 jetons · cimetière ».
## `reward` est la récompense telle que la vue l'écrit (`{"Points": 1}` ou `"Graveyard"`).
static func token_pile(tokens: int, reward: Variant) -> String:
	var count := _plural(tokens, "jeton")
	var points: Variant = Wire.content(reward)
	if points is int:
		return "%s · +%s" % [count, _plural(points, "point")]
	return "%s · cimetière" % count


## « 1 tour », « 2 tours » (la recharge d'une carte).
static func turns(count: int) -> String:
	return _plural(count, "tour")


## « 1 cible choisie », « 3 cibles choisies ».
static func targets_chosen(count: int) -> String:
	return "%s %s" % [_plural(count, "cible"), "choisies" if count > 1 else "choisie"]


## « 1 jeton », « 3 jetons ».
static func _plural(count: int, word: String) -> String:
	return "%d %s%s" % [count, word, "s" if count > 1 else ""]


## Le nom d'une case, comme le moteur l'écrit (`Position`) : la lettre de sa colonne, puis
## le numéro de sa rangée, comptés sur toute la grille, mur compris (« C5 »).
static func cell_name(cell: Vector2i) -> String:
	if cell.x < 0 or cell.x >= COLUMN_LETTERS.length() or cell.y < 0:
		return "?"
	return "%s%d" % [COLUMN_LETTERS[cell.x], cell.y + 1]


## Un message du serveur, en phrase : une capitale au début, un point à la fin
## (« aucune salle n'a le code K7QXM » → « Aucune salle n'a le code K7QXM. »).
static func sentence(text: String) -> String:
	var trimmed := text.strip_edges()
	if trimmed.is_empty():
		return trimmed
	trimmed = trimmed[0].to_upper() + trimmed.substr(1)
	return trimmed if trimmed.ends_with(".") or trimmed.ends_with("!") or trimmed.ends_with("?") else trimmed + "."
