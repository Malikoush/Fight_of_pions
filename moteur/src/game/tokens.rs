//! Les jetons du Massacre divin et ce qu'ils font des kills : des points rendus, ou un
//! départ au cimetière, d'où un pion ne revient jamais. Le cimetière seul peut faire
//! perdre un joueur par manque de pions (carnet, « Massacre divin », 2026-11-02).
//!
//! Une pile de jetons est portée par un joueur, pas par un pion : elle ne fait pas partie
//! des effets continus des pions ([`crate::ActiveEffect`]), mais c'est, comme eux, un
//! effet entier, qu'Attila Rare et Épique annulent.

use super::lasting::{CardPlay, PlayId};
use super::state::Game;
use crate::{Attack, Event, Outcome, Pion, Player};

/// Ce que rapporte chaque jeton d'une pile, au kill qui l'use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TokenReward {
    /// Ce nombre de points d'action, utilisables tout de suite.
    Points(u32),
    /// Le pion tué va au cimetière au lieu de l'infirmerie.
    Graveyard,
}

/// Une pile de jetons du Massacre divin, posée devant son propriétaire. Chacun de ses
/// prochains kills use un jeton de chacune de ses piles (carnet, 2026-11-02) ; une pile
/// vide disparaît.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenPile {
    play: PlayId,
    source: Attack,
    owner: Player,
    reward: TokenReward,
    tokens: u32,
}

impl TokenPile {
    /// Le jeu de carte qui l'a posée : son identité, et celle de son effet entier.
    pub fn play(self) -> PlayId {
        self.play
    }

    /// La carte qui l'a posée.
    pub fn source(self) -> Attack {
        self.source
    }

    /// Le joueur qui la porte : ce sont ses kills qui usent ses jetons.
    pub fn owner(self) -> Player {
        self.owner
    }

    /// Ce que rapporte chaque jeton.
    pub fn reward(self) -> TokenReward {
        self.reward
    }

    /// Les jetons qui restent.
    pub fn tokens(self) -> u32 {
        self.tokens
    }
}

impl Game {
    /// Les piles de jetons des deux joueurs, dans l'ordre où elles ont été posées. Chaque
    /// changement est annoncé par [`Event::TokenPileStarted`], [`Event::TokenUsed`] et
    /// [`Event::TokenPileEnded`].
    pub fn token_piles(&self) -> &[TokenPile] {
        &self.token_piles
    }

    /// Les pions des deux joueurs au cimetière, dans l'ordre de leur arrivée. Ils ne
    /// reviennent jamais. Chaque arrivée est annoncée par [`Event::GraveyardEntered`].
    pub fn graveyard(&self) -> &[Pion] {
        &self.graveyard
    }

    /// Les pions qu'un joueur a encore : sur le plateau, en réserve et à l'infirmerie. Sous
    /// [`crate::GameConfig::align_to_win`], il ne peut plus aligner, et perd s'il a un pion
    /// au cimetière.
    pub fn living_pions(&self, player: Player) -> u32 {
        let on_board = self
            .board
            .positions()
            .filter(|&pos| self.board.owner(pos) == Some(player))
            .count();
        let in_infirmary = self
            .infirmary
            .iter()
            .filter(|patient| patient.pion().owner() == player)
            .count();
        let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        count(on_board)
            .saturating_add(count(in_infirmary))
            .saturating_add(self.reserve_pions[player.index()])
    }

    /// Pose devant le joueur qui agit une pile de ces jetons (Massacre divin).
    pub(super) fn add_tokens(
        &mut self,
        reward: TokenReward,
        tokens: u32,
        play: CardPlay,
        events: &mut Vec<Event>,
    ) {
        let pile = TokenPile {
            play: play.id,
            source: play.attack,
            owner: self.acting_player(),
            reward,
            tokens,
        };
        self.token_piles.push(pile);
        events.push(Event::TokenPileStarted(pile));
    }

    /// Annule toutes les piles de jetons de l'adversaire du joueur qui agit (Attila Rare et
    /// Épique) : chacune est un effet entier.
    pub(super) fn cancel_enemy_tokens(&mut self, events: &mut Vec<Event>) {
        let enemy = self.acting_player().opponent();
        let (ended, kept): (Vec<TokenPile>, Vec<TokenPile>) = std::mem::take(&mut self.token_piles)
            .into_iter()
            .partition(|pile| pile.owner == enemy);
        self.token_piles = kept;
        events.extend(ended.into_iter().map(Event::TokenPileEnded));
    }

    /// Un kill de ce joueur use un jeton de chacune de ses piles : il reçoit leurs points,
    /// et le pion tué va au cimetière si l'une d'elles le demande (vrai alors).
    pub(super) fn use_tokens(&mut self, killer: Player, events: &mut Vec<Event>) -> bool {
        let mut points: u32 = 0;
        let mut to_graveyard = false;
        for pile in self
            .token_piles
            .iter_mut()
            .filter(|pile| pile.owner == killer)
        {
            pile.tokens = pile.tokens.saturating_sub(1);
            match pile.reward {
                TokenReward::Points(n) => points = points.saturating_add(n),
                TokenReward::Graveyard => to_graveyard = true,
            }
            events.push(if pile.tokens == 0 {
                Event::TokenPileEnded(*pile)
            } else {
                Event::TokenUsed(*pile)
            });
        }
        self.token_piles.retain(|pile| pile.tokens > 0);
        if points > 0 {
            self.give_points(killer, points, events);
        }
        to_graveyard
    }

    /// Des points rendus à ce joueur : pendant son tour, ils s'ajoutent à ses points du tour
    /// (sa banque redescend à son maximum à la fin du tour) ; pendant le tour adverse, à sa
    /// banque, dans la limite de son maximum (carnet, 2026-11-02).
    fn give_points(&mut self, player: Player, points: u32, events: &mut Vec<Event>) {
        if player == self.current_player {
            self.actions_left = self.actions_left.saturating_add(points);
        } else {
            let bank = &mut self.banked_actions[player.index()];
            let before = *bank;
            *bank = bank
                .saturating_add(points)
                .min(self.config.max_banked_actions);
            if *bank == before {
                return;
            }
        }
        events.push(self.actions_changed(player));
    }

    /// Le pion tué part au cimetière, sans tombe : il ne reviendra jamais.
    pub(super) fn send_to_graveyard(&mut self, pion: Pion, events: &mut Vec<Event>) {
        self.graveyard.push(pion);
        events.push(Event::GraveyardEntered(pion));
    }

    /// Le résultat si un joueur a perdu par manque de pions : il a un pion au cimetière (seul
    /// le cimetière fait baisser le total) et moins de pions qu'il n'en faut pour aligner. Si
    /// les deux joueurs sont dans ce cas, match nul (carnet, 2026-11-02).
    pub(super) fn lack_of_pions(&self) -> Option<Outcome> {
        let lost = |player: Player| {
            self.graveyard.iter().any(|pion| pion.owner() == player)
                && usize::try_from(self.living_pions(player))
                    .is_ok_and(|living| living < self.config.align_to_win)
        };
        match (lost(Player::One), lost(Player::Two)) {
            (false, false) => None,
            (true, false) => Some(Outcome::Won(Player::Two)),
            (false, true) => Some(Outcome::Won(Player::One)),
            (true, true) => Some(Outcome::Draw),
        }
    }
}
