//! Interface texte : traduit du texte en Action, et une Game en texte.
//! Elle ne décide jamais si un coup est légal : c'est le travail du moteur.

use fight_of_pions::action::Action;
use fight_of_pions::game::Game;
use fight_of_pions::player::Player;
use fight_of_pions::position::Position;

/// Ce que le joueur peut taper.
pub enum Command {
    Play(Action),
    Help,
    Quit,
}

pub const HELP: &str = "\
Commandes :
  p C5   poser un pion en C5
  f      finir le tour (les actions restantes sont gardées, 2 maximum)
  h      afficher l'aide
  q      quitter";

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
        out.push((b'A' + col as u8) as char);
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
    out
}

/// « C5 » → Position { row: 4, col: 2 }.
/// Ne vérifie pas que la case est sur le plateau : le moteur s'en charge.
pub fn parse_position(text: &str) -> Result<Position, String> {
    let mut chars = text.chars();

    let letter = match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => c.to_ascii_uppercase(),
        _ => return Err(format!("« {} » : il faut une lettre puis un nombre, par exemple C5", text)),
    };

    let number: usize = match chars.as_str().parse() {
        Ok(n) if n >= 1 => n,
        _ => return Err(format!("« {} » : le nombre après la lettre est invalide", text)),
    };

    let col = (letter as u8 - b'A') as usize;
    Ok(Position { row: number - 1, col })
}

/// Transforme une ligne tapée en commande.
pub fn parse_command(line: &str) -> Result<Command, String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    match words.as_slice() {
        ["p", pos] => Ok(Command::Play(Action::Place(parse_position(pos)?))),
        ["f"] => Ok(Command::Play(Action::EndTurn)),
        ["h"] => Ok(Command::Help),
        ["q"] => Ok(Command::Quit),
        _ => Err("Commande inconnue (tape h pour l'aide)".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_positions() {
        assert_eq!(parse_position("C5"), Ok(Position { row: 4, col: 2 }));
        assert_eq!(parse_position("j10"), Ok(Position { row: 9, col: 9 }));
        assert!(parse_position("5C").is_err());
        assert!(parse_position("A0").is_err());
        assert!(parse_position("").is_err());
    }
}