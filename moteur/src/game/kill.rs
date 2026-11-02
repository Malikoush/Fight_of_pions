//! Tuer, et tout ce qui s'ensuit : les effets du pion s'arrêtent, il part à l'infirmerie
//! en laissant sa tombe (`clock.rs`), ou dans la réserve, et son pacte (Meilleurs amis) le
//! fait venger.
//!
//! Est un kill tout pion envoyé à l'infirmerie ou au cimetière ; l'assassin est le pion
//! qui a lancé l'attaque, même s'il a bougé (carnet, 2026-10-30). Un garde de la reine la
//! sauve de ce qui la tuerait, une fois (carnet, 2026-10-31). Le kill d'un pion adverse,
//! pendant son tour ou non, use un jeton de chacune des piles du Massacre divin du joueur
//! qui tue (`tokens.rs`), qui peuvent l'envoyer au cimetière ; un pion de son propre camp
//! (le retour d'Interchange) n'en use aucun (carnet, 2026-11-02).

use super::lasting::EffectKind;
use super::state::Game;
use crate::{Cell, Event, PionId, Player, Position};

/// Ce qu'a donné un kill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hit {
    /// Le pion est mort ; de quoi décider de sa vengeance.
    Killed(Death),
    /// Un garde a sauvé la reine : l'attaque ne lui fait plus rien.
    Shielded,
    /// Il n'y avait pas de pion.
    Missed,
}

/// Un pion tué pendant une attaque, de quoi décider de sa vengeance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Death {
    /// Son ami, s'il était lié par un pacte.
    friend: Option<PionId>,
    /// L'assassin, s'il y en a un.
    killer: Option<PionId>,
}

impl Game {
    /// Tue le pion de cette case, pour le joueur `killer` : il quitte le plateau, ses effets
    /// s'arrêtent, le kill use les jetons de `killer` si le pion est à l'adversaire, puis il
    /// part au cimetière, à l'infirmerie ou dans la réserve. Une reine gardée dépense un
    /// garde à la place, et survit.
    pub(super) fn kill(
        &mut self,
        at: Position,
        by: Option<PionId>,
        killer: Player,
        events: &mut Vec<Event>,
    ) -> Hit {
        let Some(pion) = self.board.get(at).and_then(Cell::pion) else {
            return Hit::Missed;
        };
        if self.use_guard(pion.id(), events) {
            return Hit::Shielded;
        }
        if self.board.remove_pion(at).is_none() {
            return Hit::Missed;
        }
        let friend = self
            .effects
            .iter()
            .find_map(|effect| effect.friend_of(pion.id()));
        events.push(Event::PionKilled { pion, at, by });
        self.end_effects_on(pion.id(), events);
        let enemy_kill = pion.owner() != killer;
        if enemy_kill && self.use_tokens(killer, events) {
            self.send_to_graveyard(pion, events);
        } else {
            self.send_to_infirmary(pion, at, events);
        }
        Hit::Killed(Death { friend, killer: by })
    }

    /// Les vengeances des pactes, à la fin d'une attaque (carnet, 2026-10-30) : pour chaque
    /// ami tué pendant l'attaque, l'autre ami, s'il est vivant, tue l'assassin. Si les deux
    /// amis sont morts pendant la même attaque, personne ne se venge. Le pacte de l'ami
    /// survivant prend fin, même si l'assassin est protégé ou déjà mort. Une vengeance est
    /// un kill comme un autre : elle peut déclencher la vengeance d'un autre pacte. La
    /// chaîne s'arrête, puisque chaque vengeance met fin à un pacte. Un ami ne se venge
    /// jamais sur un pion de son propre camp (un retour d'Interchange, carnet, 2026-11-02) :
    /// son pacte prend fin quand même.
    ///
    /// Avant chaque vague de vengeances, les enfants que leur famille n'encadre plus perdent
    /// leur protection (carnet, 2026-11-02) : un enfant sorti de sa famille par l'attaque
    /// n'échappe pas à la vengeance.
    pub(super) fn avenge(&mut self, mut deaths: Vec<Death>, events: &mut Vec<Event>) {
        while !deaths.is_empty() {
            self.end_unframed_children(events);
            let mut next = Vec::new();
            for death in &deaths {
                // Un ami mort lui aussi (dans la même attaque) n'est plus sur le plateau :
                // personne ne se venge.
                let Some(friend) = death
                    .friend
                    .filter(|&friend| self.board.position_of(friend).is_some())
                else {
                    continue;
                };
                self.end_effects(|effect| effect.friend_of(friend).is_some(), events);
                let assassin_at = death
                    .killer
                    .and_then(|killer| self.board.position_of(killer));
                let Some(assassin_at) = assassin_at else {
                    continue;
                };
                let Some(avenger) = self
                    .board
                    .position_of(friend)
                    .and_then(|pos| self.board.owner(pos))
                else {
                    continue;
                };
                if self.board.owner(assassin_at) == Some(avenger)
                    || self.has_effect(assassin_at, EffectKind::Protected)
                {
                    continue;
                }
                if let Hit::Killed(revenge) = self.kill(assassin_at, Some(friend), avenger, events)
                {
                    next.push(revenge);
                }
            }
            deaths = next;
        }
    }
}
