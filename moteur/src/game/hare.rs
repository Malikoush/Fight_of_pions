//! Les jetons de pas du Lièvre Épique : à chacun de ses tours, le pion qui les porte peut
//! en dépenser un pour avancer de jusqu'à 2 pas. C'est un geste gratuit, comme le pas de
//! la reine, pas une attaque : une dépense par tour et par pion, à n'importe quel moment du
//! tour, jamais quand le pion est immobilisé (ses jetons attendent) ; il doit finir sur une
//! autre case que sa case de départ (carnet, 2026-11-02).
//!
//! Chaque pas est une action ([`crate::Action::HareStep`]) : le premier dépense le jeton,
//! le second, facultatif, peut venir plus tard dans le tour.

use super::lasting::{EffectKind, has_effect_of};
use super::state::Game;
use crate::attacks::{HARE_TOKEN_STEPS, around};
use crate::{Action, Cell, Event, GameError, PionId, Player, Position};

/// Le trajet d'un pion qui a dépensé un jeton de pas ce tour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct HareWalk {
    pion: PionId,
    start: Position,
    steps: u32,
}

impl Game {
    /// Les pas du Lièvre que le joueur courant peut faire maintenant, dans l'ordre des cases
    /// des pions puis des directions.
    pub(super) fn hare_steps(&self) -> Vec<Action> {
        let player = self.current_player;
        self.board
            .positions()
            .filter(|&pos| self.board.owner(pos) == Some(player))
            .flat_map(|from| {
                self.hare_destinations(from)
                    .into_iter()
                    .map(move |to| Action::HareStep { from, to })
            })
            .collect()
    }

    /// Les cases où le pion de `from`, s'il est au joueur courant, peut faire un pas du
    /// Lièvre maintenant.
    fn hare_destinations(&self, from: Position) -> Vec<Position> {
        let Some(pion) = self.board.get(from).and_then(Cell::pion) else {
            return Vec::new();
        };
        if pion.owner() != self.current_player
            || has_effect_of(&self.effects, pion.id(), EffectKind::Immobilized)
        {
            return Vec::new();
        }
        let walk = self.hare_walks.iter().find(|walk| walk.pion == pion.id());
        let can_step = match walk {
            Some(walk) => walk.steps < HARE_TOKEN_STEPS,
            None => self.step_tokens_of(pion.id()),
        };
        if !can_step {
            return Vec::new();
        }
        // Le dernier pas ne revient jamais sur la case de départ.
        let last_step = walk.map_or(HARE_TOKEN_STEPS == 1, |walk| {
            walk.steps + 1 == HARE_TOKEN_STEPS
        });
        around(&self.board, from)
            .filter(|&cell| self.board.is_free(cell))
            .filter(|&cell| !(last_step && walk.is_some_and(|walk| walk.start == cell)))
            .collect()
    }

    /// Vrai si ce pion porte des jetons de pas.
    fn step_tokens_of(&self, pion: PionId) -> bool {
        has_effect_of(&self.effects, pion, EffectKind::StepTokens)
    }

    /// Vrai si un pion de ce joueur porte des jetons de pas et a une case libre autour de
    /// lui (pour la partie figée).
    pub(super) fn hare_can_ever_act(&self, player: Player) -> bool {
        self.board
            .positions()
            .filter(|&pos| self.board.owner(pos) == Some(player))
            .any(|pos| {
                self.board
                    .get(pos)
                    .and_then(Cell::pion)
                    .is_some_and(|pion| {
                        self.step_tokens_of(pion.id())
                            && around(&self.board, pos).any(|cell| self.board.is_free(cell))
                    })
            })
    }

    pub(super) fn validate_hare_step(&self, from: Position, to: Position) -> Result<(), GameError> {
        if self.hare_destinations(from).contains(&to) {
            Ok(())
        } else {
            Err(GameError::HareCannotStep { from, to })
        }
    }

    pub(super) fn execute_hare_step(
        &mut self,
        from: Position,
        to: Position,
        events: &mut Vec<Event>,
    ) {
        let Some(pion) = self.board.get(from).and_then(Cell::pion) else {
            return;
        };
        if self.board.move_pion(from, to).is_err() {
            return;
        }
        match self
            .hare_walks
            .iter_mut()
            .find(|walk| walk.pion == pion.id())
        {
            Some(walk) => walk.steps += 1,
            None => {
                self.spend_step_token(pion.id(), events);
                self.hare_walks.push(HareWalk {
                    pion: pion.id(),
                    start: from,
                    steps: 1,
                });
            }
        }
        events.push(Event::PionMoved { pion, from, to });
        self.after_action(events);
    }

    /// Dépense un jeton de pas de ce pion : l'effet change, ou s'arrête avec son dernier
    /// jeton.
    fn spend_step_token(&mut self, pion: PionId, events: &mut Vec<Event>) {
        let Some(index) = self
            .effects
            .iter()
            .position(|effect| effect.pion() == pion && effect.kind() == EffectKind::StepTokens)
        else {
            return;
        };
        let Some(effect) = self.effects.get_mut(index) else {
            return;
        };
        match effect.with_one_token_less() {
            Some(fewer) => {
                *effect = fewer;
                events.push(Event::EffectChanged(fewer));
            }
            None => {
                let spent = self.effects.remove(index);
                events.push(Event::EffectEnded(spent));
            }
        }
    }
}
