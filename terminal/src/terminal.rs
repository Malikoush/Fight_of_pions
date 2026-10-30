//! Interface texte : traduit du texte en Action, et une Game en texte.
//! Elle ne décide jamais si un coup est légal : c'est le travail du moteur.

use fight_of_pions::{Action, Decision, Game, GameConfig, Player, Position};

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
  v      jouer l'attaque en cours avec les cibles déjà choisies (« jusqu'à 2 »)
  f      finir le tour (les actions restantes sont gardées, {} maximum)
  h      afficher l'aide
  q      quitter",
        config.max_banked_actions
    )
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
            let c = match board.owner(Position { row, col }) {
                Some(player) => symbol(player),
                None => '.',
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
    for (index, slot) in game.arsenal(player).iter().enumerate() {
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
    for effect in game.active_effects() {
        if let Some(at) = board.position_of(effect.pion()) {
            out.push_str(&format!(
                "\n  {at} : {} jusqu'à la fin du tour {}",
                effect.kind(),
                effect.last_turn()
            ));
        }
    }
    for patient in game.infirmary() {
        out.push_str(&format!(
            "\n  à l'infirmerie : pion {} de {}, jusqu'à la fin du tour {}",
            patient.pion().id(),
            player_name(patient.pion().owner()),
            patient.last_turn()
        ));
    }
    match game.pending_decision() {
        Some(Decision::ChooseLauncher { attack, .. }) => {
            out.push_str(&format!("\n{attack} : choisis le pion lanceur (c C5)"));
        }
        Some(Decision::ChooseTarget { attack, .. }) => {
            let can_finish = game.legal_actions().contains(&Action::FinishAttack);
            let or_finish = if can_finish { ", ou joue-la (v)" } else { "" };
            out.push_str(&format!("\n{attack} : choisis une cible (c C5){or_finish}"));
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
        ["v"] => Ok(Command::Play(Action::FinishAttack)),
        ["f"] => Ok(Command::Play(Action::EndTurn)),
        ["h"] => Ok(Command::Help),
        ["q"] => Ok(Command::Quit),
        _ => Err("Commande inconnue (tape h pour l'aide)".to_string()),
    }
}
