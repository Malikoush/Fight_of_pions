//! La reine (fiche n°03) : son pas gratuit et le tir de ses gardes. La pose de la reine et
//! le bouclier de ses gardes passent par les règles communes (la carte Reine, `kill.rs`).
//!
//! - **Pas** : une case dans les 8 directions vers une case libre, une fois par tour, à
//!   n'importe quel moment du tour, gratuit ; ce n'est pas une attaque.
//! - **Tir** : une attaque, une fois par tour, qui coûte
//!   [`crate::GameConfig::guard_shot_cost`] et un garde ; il tue le premier pion rencontré
//!   en ligne droite, seulement s'il est adverse et non protégé.
//! - Ni l'un ni l'autre le tour de la pose de la reine, ni quand elle est immobilisée.
//!
//! Une seule reine à la fois par joueur (carnet, 2026-10-31).

use super::kill::Hit;
use super::lasting::{EffectKind, queen_of};
use super::state::Game;
use crate::attacks::{Launch, around, can_act, first_in_lines};
use crate::{Cell, Event, GameError, Player, Position};

impl Game {
    /// La case de la reine de ce joueur, s'il en a une.
    pub fn queen_of(&self, player: Player) -> Option<Position> {
        queen_of(&self.board, &self.effects, player)
    }

    /// La reine du joueur courant, si elle peut agir maintenant : c'est un lanceur comme un
    /// autre (ni immobilisée, ni arrivée ce tour), sauf qu'elle attend toujours le tour
    /// suivant sa pose, même sans le mal du débarquement (fiche n°03).
    fn ready_queen(&self) -> Option<Position> {
        let at = self.queen_of(self.current_player)?;
        let queen = self.board.get(at).and_then(Cell::pion)?;
        let first_new = Some(self.turn_start_pion_id);
        can_act(
            queen,
            self.current_player,
            Launch::Now,
            &self.effects,
            first_new,
        )
        .then_some(at)
    }

    /// Les cases où la reine du joueur courant peut faire son pas maintenant.
    pub(super) fn queen_steps(&self) -> Vec<Position> {
        if self.queen_stepped {
            return Vec::new();
        }
        self.ready_queen().map_or_else(Vec::new, |at| {
            around(&self.board, at)
                .filter(|&cell| self.board.is_free(cell))
                .collect()
        })
    }

    /// Les pions que les gardes de cette reine peuvent viser : le premier rencontré dans
    /// chaque direction, adverse et non protégé.
    fn shot_targets_from(&self, at: Position) -> Vec<Position> {
        let Some(owner) = self.board.owner(at) else {
            return Vec::new();
        };
        first_in_lines(&self.board, at)
            .filter(|&target| {
                self.board.owner(target) == Some(owner.opponent())
                    && !self.has_effect(target, EffectKind::Protected)
            })
            .collect()
    }

    /// Les pions que la reine du joueur courant peut viser maintenant.
    pub(super) fn queen_shots(&self) -> Vec<Position> {
        if self.queen_shot {
            return Vec::new();
        }
        self.ready_queen()
            .filter(|&at| self.has_guard(at))
            .map_or_else(Vec::new, |at| self.shot_targets_from(at))
    }

    fn has_guard(&self, at: Position) -> bool {
        self.board
            .get(at)
            .and_then(Cell::pion)
            .is_some_and(|queen| self.guards_of(queen.id()) > 0)
    }

    /// Vrai si la reine de ce joueur pourra encore agir un jour (pour la partie figée) :
    /// une case libre autour d'elle, ou un garde et une cible.
    pub(super) fn queen_can_ever_act(&self, player: Player) -> bool {
        self.queen_of(player).is_some_and(|at| {
            around(&self.board, at).any(|cell| self.board.is_free(cell))
                || (self.has_guard(at) && !self.shot_targets_from(at).is_empty())
        })
    }

    pub(super) fn validate_step(&self, to: Position) -> Result<(), GameError> {
        if self.queen_steps().contains(&to) {
            Ok(())
        } else {
            Err(GameError::QueenCannotStep(to))
        }
    }

    pub(super) fn validate_shot(&self, at: Position) -> Result<(), GameError> {
        if !self.queen_shots().contains(&at) {
            return Err(GameError::QueenCannotShoot(at));
        }
        self.ensure_actions(self.config.guard_shot_cost)
    }

    pub(super) fn execute_step(&mut self, to: Position, events: &mut Vec<Event>) {
        let Some(from) = self.queen_of(self.current_player) else {
            return;
        };
        let Some(queen) = self.board.get(from).and_then(Cell::pion) else {
            return;
        };
        if self.board.move_pion(from, to).is_err() {
            return;
        }
        self.queen_stepped = true;
        events.push(Event::PionMoved {
            pion: queen,
            from,
            to,
        });
        self.after_action(events);
    }

    pub(super) fn execute_shot(&mut self, target: Position, events: &mut Vec<Event>) {
        let player = self.current_player;
        let Some(at) = self.queen_of(player) else {
            return;
        };
        let Some(queen) = self.board.get(at).and_then(Cell::pion) else {
            return;
        };
        // Le coût a été vérifié par `validate_shot`.
        self.actions_left = self
            .actions_left
            .saturating_sub(self.config.guard_shot_cost);
        self.queen_shot = true;
        events.push(Event::QueenShot { queen: at, target });
        events.push(self.actions_changed(player));
        self.use_guard(queen.id(), events);
        if let Hit::Killed(death) = self.kill(target, Some(queen.id()), player, events) {
            self.avenge(vec![death], events);
        }
        self.after_action(events);
    }
}
