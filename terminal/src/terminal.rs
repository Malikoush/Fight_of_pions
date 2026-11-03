//! Interface texte : traduit du texte en Action, et une Game en texte.
//! Elle ne décide jamais si un coup est légal : c'est le travail du moteur.

use fight_of_pions::{
    Action, ActiveEffect, Decision, EffectEnd, Game, GameConfig, Player, Position, TokenPile,
    TokenReward,
};

/// Ce que le joueur peut taper.
pub enum Command {
    Play(Action),
    Help,
    Quit,
}

/// Le texte d'aide ; les nombres viennent de la configuration de la partie.
pub fn help(config: &GameConfig) -> String {
    format!(
        "\
Commandes :
  p C5   poser un pion en C5
  a 1    lancer l'attaque n°1 de son arsenal
  c C5   choisir C5 comme lanceur ou comme cible de l'attaque en cours
  j      viser les jetons de l'adversaire (Massacre divin) pendant une attaque
  v      jouer l'attaque en cours avec les cibles déjà choisies (« jusqu'à 2 »)
  r C5   faire avancer sa reine en C5 (son pas gratuit, une fois par tour)
  t C5   faire tirer un garde de sa reine sur le pion en C5
  l C5 D6  faire avancer d'un pas le pion en C5, qui a des jetons du Lièvre, vers D6
  f      finir le tour (les actions restantes sont gardées, {} maximum)
  n      ne pas réagir au coup adverse (une réaction se joue avec a 1, payée avec la banque)
  Sur le plateau : X et O les pions, + une tombe libre (le Médecin y ressuscite), # le mur
  (le Coup du taureau le creuse).
  h      afficher l'aide
  q      quitter",
        config.max_banked_actions
    )
}

/// « Joueur 1 : Massacre divin (Rare), 3 jeton(s), 1 point(s) par kill ».
pub fn token_pile(pile: TokenPile) -> String {
    let reward = match pile.reward() {
        TokenReward::Points(n) => format!("{n} point(s) par kill"),
        TokenReward::Graveyard => "le pion tué part au cimetière".to_string(),
    };
    format!(
        "{} : {}, {} jeton(s), {reward}",
        player_name(pile.owner()),
        pile.source(),
        pile.tokens()
    )
}

/// « jusqu'à la fin du tour 6 », « tant que sa famille l'encadre » (un enfant de la
/// Familia), « jusqu'au début du prochain tour de … » (un retour), « (2 jeton(s)) », ou
/// rien pour un autre effet sans durée (un pacte).
pub fn until(effect: ActiveEffect) -> String {
    match effect.end() {
        EffectEnd::Turn(turn) => format!(" jusqu'à la fin du tour {turn}"),
        EffectEnd::Framed(_) => " tant que sa famille l'encadre".to_string(),
        EffectEnd::NextTurn => format!(
            " jusqu'au début du prochain tour de {}",
            player_name(effect.creator())
        ),
        EffectEnd::Tokens(n) => format!(" ({n} jeton(s))"),
        EffectEnd::Never => String::new(),
    }
}

/// Le symbole d'un joueur sur le plateau.
fn symbol(player: Player) -> char {
    match player {
        Player::One => 'X',
        Player::Two => 'O',
    }
}

/// Le nom d'un joueur pour les messages.
pub fn player_name(player: Player) -> &'static str {
    match player {
        Player::One => "Joueur 1 (X)",
        Player::Two => "Joueur 2 (O)",
    }
}

/// Construit le texte du plateau et de l'état de la partie.
pub fn render(game: &Game) -> String {
    let board = game.board();
    let size = board.size();
    let mut out = String::new();

    // Ligne des lettres : A B C ...
    out.push_str("    ");
    for col in 0..size {
        out.push(Position::column_letter(col).unwrap_or('?'));
        out.push(' ');
    }
    out.push('\n');

    // Une ligne par rangée, avec son numéro à gauche.
    for row in 0..size {
        out.push_str(&format!("{:>3} ", row + 1));
        for col in 0..size {
            let pos = Position { row, col };
            let c = match (board.owner(pos), board.tomb(pos)) {
                _ if board.is_wall(pos) => '#',
                (Some(player), _) => symbol(player),
                (None, Some(_)) => '+',
                (None, None) => '.',
            };
            out.push(c);
            out.push(' ');
        }
        out.push('\n');
    }

    let player = game.current_player();
    out.push_str(&format!(
        "Tour {} — {} — actions : {} — pions en réserve : {}",
        game.turn(),
        player_name(player),
        game.actions_left(),
        game.reserve_pions(player),
    ));
    // Pendant une réaction, c'est l'autre joueur qui choisit : son arsenal et sa banque.
    let deciding = game.pending_decision().map_or(player, Decision::player);
    if deciding != player {
        out.push_str(&format!(
            "
{} réagit — banque : {}",
            player_name(deciding),
            game.banked_actions(deciding)
        ));
    }
    for (index, slot) in game.arsenal(deciding).iter().enumerate() {
        let state = match slot.cooldown_left() {
            0 => "prête".to_string(),
            1 => "en recharge (1 tour)".to_string(),
            turns => format!("en recharge ({turns} tours)"),
        };
        out.push_str(&format!(
            "\n  attaque {} : {} — {}",
            index + 1,
            slot.attack(),
            state
        ));
    }
    for player in [Player::One, Player::Two] {
        if let Some(at) = game.queen_of(player) {
            out.push_str(&format!("\n  reine de {} en {at}", player_name(player)));
        }
    }
    for effect in game.active_effects() {
        if let Some(at) = board.position_of(effect.pion()) {
            out.push_str(&format!("\n  {at} : {}{}", effect.kind(), until(*effect)));
        }
    }
    for patient in game.infirmary() {
        let tomb = board
            .tomb_of(patient.pion().id())
            .map_or(String::new(), |at| format!(", tombe en {at}"));
        out.push_str(&format!(
            "\n  à l'infirmerie : pion {} de {}, jusqu'à la fin du tour {}{tomb}",
            patient.pion().id(),
            player_name(patient.pion().owner()),
            patient.last_turn()
        ));
    }
    for player in [Player::One, Player::Two] {
        if let Some(turn) = game.scorched_until(player) {
            out.push_str(&format!(
                "\n  Terre brûlée : {} ne crée aucun effet continu jusqu'à la fin du tour {turn}",
                player_name(player)
            ));
        }
    }
    for pile in game.token_piles() {
        out.push_str(&format!("\n  {}", token_pile(*pile)));
    }
    for pion in game.graveyard() {
        out.push_str(&format!(
            "\n  au cimetière : pion {} de {}",
            pion.id(),
            player_name(pion.owner())
        ));
    }
    match game.pending_decision() {
        Some(Decision::ChooseLauncher { attack, .. }) => {
            out.push_str(&format!("\n{attack} : choisis le pion lanceur (c C5)"));
        }
        Some(Decision::ChooseTarget { attack, .. }) => {
            let legal = game.legal_actions();
            let or_tokens = if legal.contains(&Action::ChooseEnemyTokens) {
                ", les jetons adverses (j)"
            } else {
                ""
            };
            let or_finish = if legal.contains(&Action::FinishAttack) {
                ", ou joue-la (v)"
            } else {
                ""
            };
            out.push_str(&format!(
                "\n{attack} : choisis une cible (c C5){or_tokens}{or_finish}"
            ));
        }
        Some(Decision::React { player }) => {
            out.push_str(&format!(
                "
{} peut réagir : joue une réaction (a 1) ou passe (n)",
                player_name(player)
            ));
        }
        Some(Decision::MainAction { .. }) | None => {}
    }
    out
}

/// Transforme une ligne tapée en commande.
pub fn parse_command(line: &str) -> Result<Command, String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    match words.as_slice() {
        ["p", pos] => Ok(Command::Play(Action::Place(pos.parse()?))),
        ["c", pos] => Ok(Command::Play(Action::Choose(pos.parse()?))),
        ["a", number] => match number.parse::<usize>() {
            Ok(n) if n >= 1 => Ok(Command::Play(Action::UseAttack(n - 1))),
            _ => Err(format!(
                "« {number} » : il faut le numéro d'une attaque, à partir de 1"
            )),
        },
        ["j"] => Ok(Command::Play(Action::ChooseEnemyTokens)),
        ["v"] => Ok(Command::Play(Action::FinishAttack)),
        ["r", pos] => Ok(Command::Play(Action::StepQueen(pos.parse()?))),
        ["t", pos] => Ok(Command::Play(Action::Shoot(pos.parse()?))),
        ["l", from, to] => Ok(Command::Play(Action::HareStep {
            from: from.parse()?,
            to: to.parse()?,
        })),
        ["f"] => Ok(Command::Play(Action::EndTurn)),
        ["n"] => Ok(Command::Play(Action::Pass)),
        ["h"] => Ok(Command::Help),
        ["q"] => Ok(Command::Quit),
        _ => Err("Commande inconnue (tape h pour l'aide)".to_string()),
    }
}
