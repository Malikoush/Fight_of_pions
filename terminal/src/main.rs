mod terminal;

use fight_of_pions::{Event, Game, GameConfig, Outcome};
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

    println!("Fight of Pions\n{}", terminal::help(game.config()));

    loop {
        println!("\n{}", terminal::render(&game));
        print!("> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        match terminal::parse_command(&line) {
            Ok(Command::Quit) => break,
            Ok(Command::Help) => println!("{}", terminal::help(game.config())),
            Ok(Command::Play(action)) => match game.apply(action) {
                Ok(events) => {
                    for event in events {
                        match event {
                            Event::PionPlaced { .. } => {}
                            Event::TurnEnded { next } => {
                                println!("Au tour de {}.", terminal::player_name(next));
                            }
                            Event::GameEnded(end) => {
                                println!("\n{}", terminal::render(&game));
                                match end.outcome {
                                    Outcome::Won(winner) => println!(
                                        "\n{} gagne la partie ({}) !",
                                        terminal::player_name(winner),
                                        end.reason
                                    ),
                                    Outcome::Draw => println!("\nMatch nul ({}).", end.reason),
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Coup refusé : {}", e),
            },
            Err(message) => println!("{}", message),
        }

        // Le break est ici, hors du for : un break dans le for n'arrêterait que le for.
        if game.end().is_some() {
            break;
        }
    }
}
