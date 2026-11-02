//! Les retours programmés de l'Interchange Commune (carnet, 2026-11-02). Au tout début du
//! tour de son créateur, avant la règle d'or, le pion téléporté revient sur sa case
//! d'origine, d'où qu'il soit, même immobilisé (ce n'est pas une attaque). Le pion qui
//! occupe cette case est tué, par le pion qui revient ; s'il est protégé, ou si c'est une
//! reine qu'un garde sauve, le retour échoue et le pion reste où il est. Un pion tué avant
//! son retour ne revient pas (son effet s'arrête avec lui) ; Attila peut l'annuler.

use super::kill::Hit;
use super::lasting::{ActiveEffect, EffectKind};
use super::state::Game;
use crate::{Cell, Event};

impl Game {
    /// Joue les retours du joueur courant, dans l'ordre de leur création. Vrai si au moins
    /// un était prévu, même s'il a échoué (son effet s'est alors arrêté).
    pub(super) fn play_returns(&mut self, events: &mut Vec<Event>) -> bool {
        let player = self.current_player;
        let due: Vec<ActiveEffect> = self
            .effects
            .iter()
            .filter(|effect| {
                matches!(effect.kind(), EffectKind::Return { .. }) && effect.creator() == player
            })
            .copied()
            .collect();
        for effect in &due {
            // Un retour précédent a pu tuer ce pion : son effet s'est alors arrêté.
            if !self.effects.contains(effect) {
                continue;
            }
            self.end_effects(|known| known.id() == effect.id(), events);
            let EffectKind::Return { origin } = effect.kind() else {
                continue;
            };
            let Some(from) = self.board.position_of(effect.pion()) else {
                continue;
            };
            if from == origin {
                continue;
            }
            let mut deaths = Vec::new();
            if self.board.get(origin).and_then(Cell::pion).is_some() {
                if self.has_effect(origin, EffectKind::Protected) {
                    continue;
                }
                match self.kill(origin, Some(effect.pion()), player, events) {
                    Hit::Killed(death) => deaths.push(death),
                    Hit::Shielded | Hit::Missed => continue,
                }
            }
            let Some(pion) = self.board.get(from).and_then(Cell::pion) else {
                continue;
            };
            if self.board.move_pion(from, origin).is_ok() {
                events.push(Event::PionMoved {
                    pion,
                    from,
                    to: origin,
                });
            }
            self.avenge(deaths, events);
        }
        !due.is_empty()
    }
}
