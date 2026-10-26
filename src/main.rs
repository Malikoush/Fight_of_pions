mod terminal;

use fight_of_pions::config::GameConfig;
use fight_of_pions::game::{Game, Outcome};
use std::io::{self, Write};
use terminal::Command;

fn main() {
    let mut game = match Game::new(GameConfig::default()) {
        Ok(game) => game,
        Err(e) => {
            eprintln!("Impossible de lancer la partie : {}", e);
            return;
        }
    };

    println!("Fight of Payon\n{}", terminal::HELP);

    loop {
        println!("\n{}", terminal::render(&game));
        print!("> ");
        io::stdout().flush().ok(); // affiche le "> " tout de suite

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => break, // fin de l'entrée ou erreur de lecture
            Ok(_) => {}
        }

        match terminal::parse_command(&line) {
            Ok(Command::Quit) => break,
            Ok(Command::Help) => println!("{}", terminal::HELP),
            Ok(Command::Play(action)) => match game.apply(action) {
                Ok(Outcome::Victory(winner)) => {
                    println!("\n{}", terminal::render(&game));
                    println!("\n{} gagne la partie !", terminal::player_name(winner));
                    break;
                }
                Ok(Outcome::TurnEnded { next }) => {
                    println!("Au tour de {}.", terminal::player_name(next));
                }
                Ok(Outcome::Continue) => {}
                Err(e) => println!("Coup refusé : {}", e),
            },
            Err(message) => println!("{}", message),
        }
    }
}