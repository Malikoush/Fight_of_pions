//! Tuer, et tout ce qui s'ensuit : les effets du pion s'arrêtent, il part à l'infirmerie
//! en laissant sa tombe (`clock.rs`), ou dans la réserve, et son pacte (Meilleurs amis) le
//! fait venger.
//!
//! Est un kill tout pion envoyé à l'infirmerie ou au cimetière ; l'assassin est le pion
//! qui a lancé l'attaque, même s'il a bougé (carnet, 2026-10-30).

use super::lasting::EffectKind;
use super::state::Game;
use crate::{Event, PionId, Position};

/// Un pion tué pendant une attaque, de quoi décider de sa vengeance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Death {
    /// Son ami, s'il était lié par un pacte.
    friend: Option<PionId>,
    /// L'assassin, s'il y en a un.
    killer: Option<PionId>,
}

impl Game {
    /// Tue le pion de cette case : il quitte le plateau, ses effets s'arrêtent, il part à
    /// l'infirmerie ou dans la réserve. Renvoie de quoi le venger ; `None` s'il n'y avait
    /// pas de pion.
    pub(super) fn kill(
        &mut self,
        at: Position,
        by: Option<PionId>,
        events: &mut Vec<Event>,
    ) -> Option<Death> {
        let pion = self.board.remove_pion(at)?;
        let friend = self
            .effects
            .iter()
            .find_map(|effect| effect.friend_of(pion.id()));
        events.push(Event::PionKilled { pion, at, by });
        self.end_effects_on(pion.id(), events);
        self.send_to_infirmary(pion, at, events);
        Some(Death { friend, killer: by })
    }

    /// Les vengeances des pactes, à la fin d'une attaque (carnet, 2026-10-30) : pour chaque
    /// ami tué pendant l'attaque, l'autre ami, s'il est vivant, tue l'assassin. Si les deux
    /// amis sont morts pendant la même attaque, personne ne se venge. Le pacte de l'ami
    /// survivant prend fin, même si l'assassin est protégé ou déjà mort. Une vengeance est
    /// un kill comme un autre : elle peut déclencher la vengeance d'un autre pacte. La
    /// chaîne s'arrête, puisque chaque vengeance met fin à un pacte.
    pub(super) fn avenge(&mut self, mut deaths: Vec<Death>, events: &mut Vec<Event>) {
        while !deaths.is_empty() {
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
                if self.has_effect(assassin_at, EffectKind::Protected) {
                    continue;
                }
                if let Some(revenge) = self.kill(assassin_at, Some(friend), events) {
                    next.push(revenge);
                }
            }
            deaths = next;
        }
    }
}
