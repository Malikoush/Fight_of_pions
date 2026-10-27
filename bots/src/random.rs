use crate::Bot;
use fight_of_pions::{Action, Game, Rng};

/// Joue une action tirée uniformément parmi les actions légales, choix d'attaque
/// compris. Il sert d'étalon : un bot qui ne le bat pas largement ne sait pas jouer.
pub struct RandomBot;

impl Bot for RandomBot {
    fn name(&self) -> &'static str {
        "aléatoire"
    }

    fn choose(&self, game: &Game, rng: &mut Rng) -> Option<Action> {
        rng.pick(&game.legal_actions()).copied()
    }
}
