//! Une salle : une partie et ses deux places. Toute la logique de jeu du serveur est ici,
//! sans réseau : qui a le droit de jouer, les coups des bots, ce que voit chacun. Le
//! moteur reste le seul juge des règles.

use crate::ServerError;
use bots::BotKind;
use fight_of_pions::{
    Action, Decision, Event, Game, GameConfig, GameRecord, Player, PlayerView, Rng,
};

/// Qui occupe une place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seat {
    /// Personne encore : un humain peut la prendre.
    Free,
    /// Un humain, reconnu à sa clé de place, secrète.
    Human {
        /// Sa clé.
        key: String,
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

/// Une recherche de bot, prête à tourner hors de la salle : une copie de la partie et du
/// hasard des bots. La salle reste libre pendant que le bot réfléchit.
pub struct BotSearch {
    kind: BotKind,
    game: Game,
    rng: Rng,
    /// Le nombre de coups joués quand la recherche a commencé.
    moves: usize,
}

/// Le coup trouvé par une recherche de bot, à jouer dans la salle.
pub struct BotMove {
    action: Option<Action>,
    rng: Rng,
    moves: usize,
}

#[cfg(test)]
impl BotMove {
    /// Un coup de bot écrit à la main, trouvé après `moves` coups.
    pub fn new(action: Option<Action>, moves: usize) -> BotMove {
        BotMove {
            action,
            rng: Rng::new(1),
            moves,
        }
    }
}

impl BotSearch {
    /// Cherche le coup du bot. Le MCTS cherche sans lire la cache adverse.
    pub fn think(mut self) -> BotMove {
        let action = self.kind.bot().choose(&self.game, &mut self.rng);
        BotMove {
            action,
            rng: self.rng,
            moves: self.moves,
        }
    }
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

    /// Assied un humain, avec cette clé, à cette place libre.
    pub fn sit(&mut self, player: Player, key: String) -> Result<(), ServerError> {
        let seat = &mut self.seats[player.index()];
        if *seat != Seat::Free {
            return Err(ServerError::RoomFull);
        }
        *seat = Seat::Human { key };
        Ok(())
    }

    /// Vérifie que cette clé est celle de l'humain assis à cette place.
    pub fn check_key(&self, player: Player, key: &str) -> Result<(), ServerError> {
        match &self.seats[player.index()] {
            Seat::Human { key: own } if own == key => Ok(()),
            _ => Err(ServerError::BadKey),
        }
    }

    /// Joue le coup de cet humain, s'il doit décider ; rend ce que le coup a produit.
    pub fn play(&mut self, player: Player, action: Action) -> Result<Vec<Event>, ServerError> {
        if self.decider() != Some(player) {
            return Err(ServerError::NotYourTurn);
        }
        self.apply(action)
    }

    /// Ce que verrait ce joueur au fil des choix d'une carte (la carte, son lanceur, ses
    /// cibles), essayés sur une copie : rien n'est joué.
    ///
    /// L'aperçu s'arrête au coup qui finit ces choix et rend alors `None` : la suite est
    /// prête à être jouée. Montrer ce qui vient après (la carte jouée, une fenêtre de
    /// réaction adverse) dévoilerait ce que l'adversaire cache, sans rien risquer.
    pub fn preview(
        &self,
        player: Player,
        actions: &[Action],
    ) -> Result<Option<PlayerView>, ServerError> {
        if self.decider() != Some(player) {
            return Err(ServerError::NotYourTurn);
        }
        // Tant qu'une carte attend ses choix, c'est à celui qui l'a lancée de choisir.
        let mut copy = self.game.clone();
        for (done, &action) in actions.iter().enumerate() {
            copy.apply(action)?;
            if !copy.attack_in_progress() {
                return if done + 1 == actions.len() {
                    Ok(None)
                } else {
                    Err(ServerError::PreviewPastTheCard)
                };
            }
        }
        Ok(Some(copy.view(player)))
    }

    /// De quoi faire chercher le bot qui doit décider maintenant, s'il y en a un.
    pub fn bot_search(&self) -> Option<BotSearch> {
        let kind = match self.seats[self.decider()?.index()] {
            Seat::Bot(kind) => kind,
            Seat::Free | Seat::Human { .. } => return None,
        };
        Some(BotSearch {
            kind,
            game: self.game.clone(),
            rng: self.bot_rng.clone(),
            moves: self.record.actions.len(),
        })
    }

    /// Joue le coup trouvé par le bot ; rend ce qu'il a produit (parfois rien : un choix
    /// d'attaque). `None` si ce n'est plus au bot de jouer, ou si la partie a changé
    /// pendant la recherche : le coup n'est pas joué.
    ///
    /// Un bot sans coup, ou dont le coup est refusé, ou dont la recherche a paniqué
    /// (`found` vaut `None`), est un bug du bot : pour que la partie ne reste pas figée, il
    /// joue à la place le premier coup permis par le moteur, et le serveur le signale.
    pub fn play_bot(&mut self, found: Option<BotMove>) -> Result<Option<Vec<Event>>, ServerError> {
        if self.bot_search().is_none() {
            return Ok(None);
        }
        if let Some(found) = found {
            if found.moves != self.record.actions.len() {
                return Ok(None);
            }
            if let Some(events) = found.action.and_then(|action| self.apply(action).ok()) {
                self.bot_rng = found.rng;
                return Ok(Some(events));
            }
        }
        eprintln!("le bot n'a pas trouvé de coup valable : il joue le premier coup permis");
        let fallback = self.game.legal_actions().first().copied();
        let fallback =
            fallback.ok_or_else(|| ServerError::BotFailed("aucun coup permis".to_string()))?;
        self.apply(fallback)
            .map(Some)
            .map_err(|error| ServerError::BotFailed(error.to_string()))
    }

    /// Ce que voit ce joueur.
    pub fn view(&self, player: Player) -> PlayerView {
        self.game.view(player)
    }

    /// Vrai si la partie est finie.
    pub fn is_over(&self) -> bool {
        self.decider().is_none()
    }

    /// Le joueur qui doit décider, si la partie n'est pas finie.
    fn decider(&self) -> Option<Player> {
        self.game.pending_decision().map(Decision::player)
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
    use fight_of_pions::{Attack, AttackKind, DraftRules, GameError, Position, Rarity};

    fn human(key: &str) -> Seat {
        Seat::Human {
            key: key.to_string(),
        }
    }

    fn duel() -> Room {
        Room::new(GameConfig::default(), 0, [human("un"), Seat::Free]).unwrap()
    }

    /// Les règles actuelles sans mur ni draft, J1 avec l'Ours Rare (emplacement 0).
    fn armed(seats: [Seat; 2]) -> Room {
        let ours = Attack {
            kind: AttackKind::Ours,
            rarity: Rarity::Rare,
        };
        let config = GameConfig {
            wall_thickness: 0,
            starting_attacks: [vec![ours], Vec::new()],
            ..GameConfig::default()
        };
        Room::new(config, 0, seats).unwrap()
    }

    #[test]
    fn a_human_takes_the_free_seat_with_a_key() {
        let mut room = duel();
        assert_eq!(room.free_seat(), Some(Player::Two));
        room.sit(Player::Two, "deux".to_string()).unwrap();
        assert_eq!(room.free_seat(), None);
        assert_eq!(
            room.sit(Player::Two, "autre".to_string()),
            Err(ServerError::RoomFull)
        );
        assert_eq!(room.check_key(Player::Two, "deux"), Ok(()));
        assert_eq!(room.check_key(Player::Two, "un"), Err(ServerError::BadKey));
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
    fn a_preview_follows_the_choices_of_a_card_without_playing_them() {
        let mut room = armed([human("un"), human("deux")]);
        let at = |row, col| Position { row, col };
        room.play(Player::One, Action::Place(at(4, 4))).unwrap();
        room.play(Player::One, Action::EndTurn).unwrap();
        room.play(Player::Two, Action::Place(at(4, 5))).unwrap();
        room.play(Player::Two, Action::EndTurn).unwrap();
        // Tour 3 : l'Ours Rare de J1, lancé depuis E5, vise F5.
        let start = [Action::UseAttack(0)];
        let launcher = room.preview(Player::One, &start).unwrap().unwrap();
        assert!(launcher.decision.is_some_and(Decision::is_attack_choice));
        let aimed = [start[0], Action::Choose(at(4, 4))];
        let target = room.preview(Player::One, &aimed).unwrap().unwrap();
        assert!(target.legal_actions.contains(&Action::Choose(at(4, 5))));
        let whole = [aimed[0], aimed[1], Action::Choose(at(4, 5))];
        assert_eq!(room.preview(Player::One, &whole), Ok(None));
        let past = [whole[0], whole[1], whole[2], Action::EndTurn];
        assert_eq!(
            room.preview(Player::One, &past),
            Err(ServerError::PreviewPastTheCard)
        );
        assert_eq!(room.record.actions.len(), 4, "rien n'est joué");
        assert_eq!(
            room.preview(Player::Two, &start),
            Err(ServerError::NotYourTurn)
        );
        assert_eq!(
            room.preview(Player::Two, &[]),
            Err(ServerError::NotYourTurn)
        );
    }

    #[test]
    fn a_preview_stops_at_the_move_that_ends_the_choices() {
        let room = armed([human("un"), human("deux")]);
        let place = Action::Place(Position { row: 4, col: 4 });
        // Une pose finit tout de suite : sa suite (une Sentinelle adverse ?) reste cachée.
        assert_eq!(room.preview(Player::One, &[place]), Ok(None));
        assert_eq!(
            room.preview(Player::One, &[place, Action::EndTurn]),
            Err(ServerError::PreviewPastTheCard)
        );
    }

    #[test]
    fn a_bot_move_is_dropped_if_the_game_changed_during_the_search() {
        let mut room = armed([human("un"), Seat::Bot(BotKind::Random)]);
        let place = Action::Place(Position { row: 4, col: 4 });
        assert_eq!(
            room.play_bot(Some(BotMove::new(Some(place), 0))),
            Ok(None),
            "c'est à J1"
        );
        room.play(Player::One, Action::EndTurn).unwrap();
        assert!(room.bot_search().is_some(), "c'est au bot");
        assert_eq!(room.play_bot(Some(BotMove::new(Some(place), 0))), Ok(None));
        assert_eq!(room.record.actions.len(), 1);
        assert!(
            room.play_bot(Some(BotMove::new(Some(place), 1)))
                .unwrap()
                .is_some()
        );
        assert_eq!(room.record.actions, [Action::EndTurn, place]);
    }

    #[test]
    fn a_failing_bot_plays_the_first_legal_move_instead() {
        // Sans coup, une recherche qui a paniqué, un coup refusé.
        for found in [
            Some(BotMove::new(None, 1)),
            None,
            Some(BotMove::new(Some(Action::Pass), 1)),
        ] {
            let mut room = armed([human("un"), Seat::Bot(BotKind::Random)]);
            room.play(Player::One, Action::EndTurn).unwrap();
            let first = room.game.legal_actions()[0];
            assert!(room.play_bot(found).unwrap().is_some());
            assert_eq!(room.record.actions, [Action::EndTurn, first]);
        }
    }

    #[test]
    #[ignore = "fait jouer un bot une partie entière : à lancer à la main"]
    fn a_bot_plays_its_whole_game_against_a_human_who_only_ends_turns() {
        let config = GameConfig {
            draft: Some(DraftRules::standard()),
            ..GameConfig::default()
        };
        let mut room = Room::new(config, 3, [human("un"), Seat::Bot(BotKind::Random)]).unwrap();
        let mut moves = 0;
        while !room.is_over() && moves < 2000 {
            if let Some(search) = room.bot_search() {
                assert!(room.play_bot(Some(search.think())).unwrap().is_some());
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
        assert!(room.is_over(), "partie finie en {moves} coups");
    }
}
