//! Une salle : une partie et ses deux places. Toute la logique de jeu du serveur est ici,
//! sans réseau : qui a le droit de jouer, les coups des bots, ce que voit chacun. Le
//! moteur reste le seul juge des règles.

use crate::ServerError;
use bots::BotKind;
use fight_of_pions::{Action, Event, Game, GameConfig, GameRecord, Player, PlayerView, Rng};

/// Qui occupe une place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seat {
    /// Personne encore : un humain peut la prendre.
    Free,
    /// Un humain, reconnu à son jeton secret.
    Human {
        /// Son jeton.
        token: String,
    },
    /// Un bot du serveur.
    Bot(BotKind),
}

/// Une partie et ses deux places.
pub struct Room {
    game: Game,
    /// La partie comme liste d'actions : de quoi la rejouer à l'identique (le moteur est
    /// déterministe), la sauvegarder ou la recharger.
    record: GameRecord,
    seats: [Seat; 2],
    /// Le hasard des bots, initialisé avec la graine de la partie, comme dans l'arène.
    bot_rng: Rng,
}

impl Room {
    /// Une salle neuve avec cette configuration et cette graine ; ces places.
    pub fn new(config: GameConfig, seed: u64, seats: [Seat; 2]) -> Result<Room, ServerError> {
        let game = Game::new(config.clone(), seed)?;
        Ok(Room {
            game,
            record: GameRecord::new(config, seed),
            seats,
            bot_rng: Rng::new(seed),
        })
    }

    /// La première place libre, s'il y en a une.
    pub fn free_seat(&self) -> Option<Player> {
        [Player::One, Player::Two]
            .into_iter()
            .find(|player| self.seats[player.index()] == Seat::Free)
    }

    /// Assied un humain, avec ce jeton, à cette place libre.
    pub fn sit(&mut self, player: Player, token: String) -> Result<(), ServerError> {
        let seat = &mut self.seats[player.index()];
        if *seat != Seat::Free {
            return Err(ServerError::RoomFull);
        }
        *seat = Seat::Human { token };
        Ok(())
    }

    /// Vérifie que ce jeton est celui de l'humain assis à cette place.
    pub fn check_token(&self, player: Player, token: &str) -> Result<(), ServerError> {
        match &self.seats[player.index()] {
            Seat::Human { token: own } if own == token => Ok(()),
            _ => Err(ServerError::BadToken),
        }
    }

    /// Joue le coup de cet humain, s'il doit décider ; rend ce que le coup a produit.
    pub fn play(&mut self, player: Player, action: Action) -> Result<Vec<Event>, ServerError> {
        if self.decider() != Some(player) {
            return Err(ServerError::NotYourTurn);
        }
        self.apply(action)
    }

    /// Ce que verrait ce joueur après ces coups, tous les siens, essayés sur une copie :
    /// rien n'est joué.
    pub fn preview(&self, player: Player, actions: &[Action]) -> Result<PlayerView, ServerError> {
        let mut copy = self.game.clone();
        for &action in actions {
            if copy.pending_decision().map(|decision| decision.player()) != Some(player) {
                return Err(ServerError::NotYourTurn);
            }
            copy.apply(action)?;
        }
        Ok(copy.view(player))
    }

    /// Le bot qui doit décider maintenant, s'il y en a un.
    pub fn bot_to_play(&self) -> Option<BotKind> {
        match self.seats[self.decider()?.index()] {
            Seat::Bot(kind) => Some(kind),
            Seat::Free | Seat::Human { .. } => None,
        }
    }

    /// Fait jouer une action au bot qui doit décider ; rend ce qu'elle a produit (parfois
    /// rien : un choix d'attaque), ou `None` si aucun bot n'a joué. Le MCTS cherche sans
    /// lire la cache adverse. Un coup refusé serait un bug du bot.
    pub fn play_bot(&mut self) -> Result<Option<Vec<Event>>, ServerError> {
        let Some(kind) = self.bot_to_play() else {
            return Ok(None);
        };
        let Some(action) = kind.bot().choose(&self.game, &mut self.bot_rng) else {
            return Ok(None);
        };
        self.apply(action).map(Some)
    }

    /// Ce que voit ce joueur.
    pub fn view(&self, player: Player) -> PlayerView {
        self.game.view(player)
    }

    /// Le joueur qui doit décider, si la partie n'est pas finie.
    fn decider(&self) -> Option<Player> {
        self.game
            .pending_decision()
            .map(|decision| decision.player())
    }

    fn apply(&mut self, action: Action) -> Result<Vec<Event>, ServerError> {
        let events = self.game.apply(action)?;
        self.record.actions.push(action);
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fight_of_pions::{DraftRules, GameError};

    fn human(token: &str) -> Seat {
        Seat::Human {
            token: token.to_string(),
        }
    }

    fn duel() -> Room {
        Room::new(GameConfig::default(), 0, [human("un"), Seat::Free]).unwrap()
    }

    #[test]
    fn a_human_takes_the_free_seat_with_a_token() {
        let mut room = duel();
        assert_eq!(room.free_seat(), Some(Player::Two));
        room.sit(Player::Two, "deux".to_string()).unwrap();
        assert_eq!(room.free_seat(), None);
        assert_eq!(
            room.sit(Player::Two, "autre".to_string()),
            Err(ServerError::RoomFull)
        );
        assert_eq!(room.check_token(Player::Two, "deux"), Ok(()));
        assert_eq!(
            room.check_token(Player::Two, "un"),
            Err(ServerError::BadToken)
        );
    }

    #[test]
    fn only_the_deciding_player_plays_and_the_engine_judges() {
        let mut room = duel();
        let place = room.view(Player::One).legal_actions[0];
        assert_eq!(room.play(Player::Two, place), Err(ServerError::NotYourTurn));
        assert_eq!(
            room.play(Player::One, Action::Pass),
            Err(ServerError::Game(GameError::UnexpectedAction))
        );
        room.play(Player::One, place).unwrap();
        assert_eq!(room.record.actions, [place]);
        assert_eq!(room.record.replay().unwrap(), room.game);
    }

    #[test]
    fn a_preview_tries_moves_without_playing_them() {
        let mut room = duel();
        room.sit(Player::Two, "deux".to_string()).unwrap();
        let place = room.view(Player::One).legal_actions[0];
        let after = room.preview(Player::One, &[place]).unwrap();
        assert_ne!(after, room.view(Player::One));
        assert!(room.record.actions.is_empty(), "rien n'est joué");
        assert_eq!(
            room.preview(Player::Two, &[place]),
            Err(ServerError::NotYourTurn)
        );
    }

    #[test]
    fn a_bot_plays_its_whole_game_against_a_human_who_only_ends_turns() {
        let config = GameConfig {
            draft: Some(DraftRules::standard()),
            ..GameConfig::default()
        };
        let mut room = Room::new(config, 3, [human("un"), Seat::Bot(BotKind::Random)]).unwrap();
        let mut moves = 0;
        while room.view(Player::One).end.is_none() && moves < 2000 {
            if room.bot_to_play().is_some() {
                assert!(room.play_bot().unwrap().is_some());
            } else {
                let legal = room.view(Player::One).legal_actions;
                // Le premier coup permis : une carte au draft, sinon finir le tour.
                let action = legal
                    .iter()
                    .copied()
                    .find(|action| matches!(action, Action::EndTurn | Action::Pass))
                    .unwrap_or(legal[0]);
                room.play(Player::One, action).unwrap();
            }
            moves += 1;
        }
        assert!(
            room.view(Player::Two).end.is_some(),
            "partie finie en {moves} coups"
        );
    }
}
