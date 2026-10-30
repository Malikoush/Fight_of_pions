//! L'horloge de la partie : ce qui dure après une action (effets continus, infirmerie),
//! quand cela commence et quand cela s'arrête. Une seule règle pour tout : une durée de N
//! tours se compte en tours de l'adversaire de celui qui a joué l'effet, le tour en cours
//! ne compte jamais, et elle s'éteint à la fin de son dernier tour (carnet, 2026-10-30).

use super::lasting::{ActiveEffect, EffectKind, Patient, pion_has};
use super::state::Game;
use crate::{Attack, Cell, Event, Pion, PionId, Player, Position};

impl Game {
    /// Les effets continus en jeu, dans l'ordre de leur création. Chaque changement est
    /// annoncé par [`Event::EffectStarted`] et [`Event::EffectEnded`].
    pub fn active_effects(&self) -> &[ActiveEffect] {
        &self.effects
    }

    /// Vrai si le pion de cette case a un effet de cette sorte (immobilisé, protégé…).
    pub fn has_effect(&self, pos: Position, kind: EffectKind) -> bool {
        pion_has(&self.board, &self.effects, pos, kind)
    }

    /// Les pions des deux joueurs à l'infirmerie, dans l'ordre de leur arrivée. Chaque
    /// changement est annoncé par [`Event::InfirmaryEntered`] et [`Event::InfirmaryLeft`].
    pub fn infirmary(&self) -> &[Patient] {
        &self.infirmary
    }

    /// Le dernier tour d'une durée de `turns` tours (au moins 1), jouée par le joueur
    /// courant : elle se compte en tours de son adversaire, à partir du tour suivant. Les
    /// réactions (B8), jouées pendant le tour de l'adversaire, devront sauter ce tour-là.
    fn last_turn_of(&self, turns: u32) -> u32 {
        self.turn
            .saturating_add(1)
            .saturating_add(turns.saturating_sub(1).saturating_mul(2))
    }

    /// Donne au pion de cette case un effet de `turns` tours, créé par le joueur courant
    /// avec cette carte. Il se compte en tours de l'adversaire du joueur courant.
    pub(super) fn start_effect(
        &mut self,
        at: Position,
        kind: EffectKind,
        turns: u32,
        source: Attack,
        events: &mut Vec<Event>,
    ) {
        let Some(pion) = self.board.get(at).and_then(Cell::pion) else {
            return;
        };
        let creator = self.current_player;
        let last_turn = self.last_turn_of(turns);
        let effect = ActiveEffect::new(
            self.next_effect_id,
            kind,
            pion.id(),
            creator,
            source,
            last_turn,
        );
        self.next_effect_id = self.next_effect_id.next();
        self.effects.push(effect);
        events.push(Event::EffectStarted(effect));
    }

    /// Arrête les effets qui remplissent cette condition, en annonçant chacun.
    pub(super) fn end_effects(
        &mut self,
        ends: impl Fn(&ActiveEffect) -> bool,
        events: &mut Vec<Event>,
    ) {
        let (ended, kept): (Vec<ActiveEffect>, Vec<ActiveEffect>) =
            std::mem::take(&mut self.effects)
                .into_iter()
                .partition(|effect| ends(effect));
        self.effects = kept;
        events.extend(ended.into_iter().map(Event::EffectEnded));
    }

    /// Les effets visant ce pion s'arrêtent : il vient de quitter le plateau.
    pub(super) fn end_effects_on(&mut self, pion: PionId, events: &mut Vec<Event>) {
        self.end_effects(|effect| effect.pion() == pion, events);
    }

    /// Un pion tué va à l'infirmerie pour [`crate::GameConfig::infirmary_turns`] tours de
    /// l'adversaire du joueur qui l'a tué (le joueur courant), ou directement dans la
    /// réserve si cette durée est 0. La règle (carnet, 2026-10-30) compte en tours de son
    /// propriétaire : c'est la même chose tant qu'on ne tue que des pions adverses. Le cas
    /// d'un pion tué par une carte de son propriétaire viendra en B6.
    pub(super) fn send_to_infirmary(&mut self, pion: Pion, events: &mut Vec<Event>) {
        let turns = self.config.infirmary_turns;
        if turns == 0 {
            self.return_to_reserve(pion.owner(), events);
            return;
        }
        let last_turn = self.last_turn_of(turns);
        let patient = Patient::new(pion, last_turn);
        self.infirmary.push(patient);
        events.push(Event::InfirmaryEntered(patient));
    }

    fn return_to_reserve(&mut self, owner: Player, events: &mut Vec<Event>) {
        let reserve = &mut self.reserve_pions[owner.index()];
        *reserve = reserve.saturating_add(1);
        events.push(self.reserve_changed(owner));
    }

    /// À la fin du tour en cours : les effets dont c'était le dernier tour s'arrêtent, et
    /// les pions dont c'était le dernier tour à l'infirmerie reviennent dans la réserve.
    /// Vrai si quelque chose a changé.
    pub(super) fn end_what_expires(&mut self, events: &mut Vec<Event>) -> bool {
        let turn = self.turn;
        let before = events.len();
        self.end_effects(|effect| effect.last_turn() <= turn, events);
        let (leaving, staying): (Vec<Patient>, Vec<Patient>) = std::mem::take(&mut self.infirmary)
            .into_iter()
            .partition(|patient| patient.last_turn() <= turn);
        self.infirmary = staying;
        for patient in leaving {
            events.push(Event::InfirmaryLeft(patient));
            self.return_to_reserve(patient.pion().owner(), events);
        }
        events.len() > before
    }

    /// Vrai si quelque chose est programmé pour changer la partie plus tard : un effet
    /// continu qui s'arrêtera, un pion qui sortira de l'infirmerie.
    pub(super) fn something_is_scheduled(&self) -> bool {
        !self.effects.is_empty() || !self.infirmary.is_empty()
    }
}
