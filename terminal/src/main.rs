mod terminal;

use fight_of_pions::{Event, Game, GameConfig, Outcome};
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use terminal::Command;

/// Une graine différente à chaque partie, tirée de l'horloge : le moteur, lui, n'utilise
/// jamais l'horloge.
fn seed_from_clock() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() ^ u64::from(elapsed.subsec_nanos()))
        .unwrap_or(0)
}

fn main() {
    // Chaque joueur a toutes les attaques jouables, en attendant le draft.
    let config = GameConfig::default().with_full_arsenals();
    let mut game = match Game::new(config, seed_from_clock()) {
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
                            // Le plateau et les compteurs sont réaffichés à chaque tour.
                            Event::PionPlaced { .. }
                            | Event::ActionsChanged { .. }
                            | Event::PlacementsChanged { .. }
                            | Event::ReserveChanged { .. }
                            | Event::CooldownChanged { .. } => {}
                            Event::AttackUsed {
                                player,
                                attack,
                                launcher,
                            } => println!(
                                "{} lance {} depuis {}.",
                                terminal::player_name(player),
                                attack,
                                launcher
                            ),
                            Event::QueenShot { queen, target } => {
                                println!("Un garde de la reine en {} tire sur {}.", queen, target)
                            }
                            Event::PionKilled { pion, at, .. } => println!(
                                "Le pion {} de {} est tué en {}.",
                                pion.id(),
                                terminal::player_name(pion.owner()),
                                at
                            ),
                            Event::PionMoved { pion, from, to } => println!(
                                "Le pion {} de {} va de {} en {}.",
                                pion.id(),
                                terminal::player_name(pion.owner()),
                                from,
                                to
                            ),
                            Event::PionsSwapped {
                                first,
                                first_at,
                                second,
                                second_at,
                            } => println!(
                                "Les pions {} ({}) et {} ({}) échangent leurs cases.",
                                first.id(),
                                first_at,
                                second.id(),
                                second_at
                            ),
                            Event::EffectStarted(effect) => println!(
                                "Le pion {} est {}{} ({}).",
                                effect.pion(),
                                effect.kind(),
                                terminal::until(effect),
                                effect.source()
                            ),
                            Event::EffectChanged(effect) => println!(
                                "Le pion {} dépense un jeton : encore {}{}.",
                                effect.pion(),
                                effect.kind(),
                                terminal::until(effect)
                            ),
                            Event::EffectEnded(effect) => {
                                println!("Le pion {} n'est plus {}.", effect.pion(), effect.kind())
                            }
                            Event::InfirmaryEntered(patient) => println!(
                                "Le pion {} de {} part à l'infirmerie jusqu'à la fin du tour {}.",
                                patient.pion().id(),
                                terminal::player_name(patient.pion().owner()),
                                patient.last_turn()
                            ),
                            Event::TombAdded { pion, at } => {
                                println!("Le pion {} laisse sa tombe en {}.", pion.id(), at)
                            }
                            Event::TombRemoved { pion, at } => {
                                println!("La tombe du pion {} s'efface en {}.", pion.id(), at)
                            }
                            Event::InfirmaryLeft(patient) => println!(
                                "Le pion {} de {} sort de l'infirmerie.",
                                patient.pion().id(),
                                terminal::player_name(patient.pion().owner())
                            ),
                            Event::GraveyardEntered(pion) => println!(
                                "Le pion {} de {} part au cimetière, pour toujours.",
                                pion.id(),
                                terminal::player_name(pion.owner())
                            ),
                            Event::TokenPileStarted(pile) | Event::TokenUsed(pile) => {
                                println!("{}.", terminal::token_pile(pile))
                            }
                            Event::TokenPileEnded(pile) => println!(
                                "La pile de {} ({}) disparaît.",
                                terminal::player_name(pile.owner()),
                                pile.source()
                            ),
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
