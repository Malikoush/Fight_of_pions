//! Le déroulement d'une attaque : la carte, puis le lanceur, puis chaque cible, et enfin
//! la résolution d'un bloc. Tant que le dernier choix n'est pas fait, rien ne change ; et
//! aucun choix proposé ne mène à une impasse, puisque le moteur n'a pas d'« annuler ».

use super::state::{Game, PendingAttack};
use crate::attacks::{AttackContext, Effect, can_complete};
use crate::{Attack, AttackStats, Cell, Event, GameError, Pion, PionId, Player, Position};

impl Game {
    pub(super) fn validate_use_attack(&self, slot: usize) -> Result<(), GameError> {
        let player = self.current_player;
        let Some(slot_state) = self.arsenal(player).get(slot) else {
            return Err(GameError::NoSuchAttack(slot));
        };
        let attack = slot_state.attack();
        if slot_state.cooldown_left() > 0 {
            return Err(GameError::AttackRecharging(attack));
        }
        self.ensure_actions(self.attack_stats(attack).cost)?;
        if !self.attack_has_a_target(player, attack) {
            return Err(GameError::NoTarget(attack));
        }
        Ok(())
    }

    pub(super) fn validate_choice(
        &self,
        pending: &PendingAttack,
        pos: Position,
    ) -> Result<(), GameError> {
        let player = self.current_player;
        let attack = pending.attack;
        let allowed = match pending.launcher {
            // Le lanceur : un de ses pions, depuis lequel l'attaque a au moins une cible.
            None => {
                self.board.owner(pos) == Some(player)
                    && self.can_complete_from(player, attack, pos, &mut Vec::new())
            }
            // Une cible : proposée par la famille, et après laquelle on peut encore finir.
            Some(launcher) => {
                let context = self.context(player, attack, launcher);
                let rules = attack.kind.rules();
                let mut chosen = pending.targets.clone();
                let candidate = rules.candidate_targets(&context, &chosen).contains(&pos);
                chosen.push(pos);
                candidate && can_complete(rules, &context, &mut chosen)
            }
        };
        if allowed {
            Ok(())
        } else {
            Err(GameError::InvalidChoice(pos))
        }
    }

    /// Vrai si ce joueur a, quelque part sur le plateau, un lanceur depuis lequel cette
    /// attaque peut aller au bout. Ne regarde ni le coût ni la recharge.
    pub(super) fn attack_has_a_target(&self, player: Player, attack: Attack) -> bool {
        self.board
            .positions()
            .filter(|&pos| self.board.owner(pos) == Some(player))
            .any(|launcher| self.can_complete_from(player, attack, launcher, &mut Vec::new()))
    }

    pub(super) fn start_attack(&mut self, slot: usize) {
        // L'emplacement a été validé ; à défaut, rien ne commence.
        let Some(slot_state) = self.arsenal(self.current_player).get(slot) else {
            return;
        };
        self.pending_attack = Some(PendingAttack {
            slot,
            attack: slot_state.attack(),
            launcher: None,
            targets: Vec::new(),
        });
    }

    pub(super) fn execute_choice(&mut self, pos: Position, events: &mut Vec<Event>) {
        let Some(pending) = self.pending_attack.as_mut() else {
            return;
        };
        match pending.launcher {
            None => pending.launcher = Some(pos),
            Some(_) => pending.targets.push(pos),
        }
        let Some(pending) = self.pending_attack.as_ref() else {
            return;
        };
        let attack = pending.attack;
        let needed = attack.kind.rules().target_count(attack.rarity);
        if pending.launcher.is_some() && pending.targets.len() >= needed {
            self.resolve_attack(events);
        }
    }

    /// Résout l'attaque dont tous les choix sont faits : paie le coût, met la carte en
    /// recharge, applique les effets, puis vérifie la fin du tour ou de la partie.
    ///
    /// Tuer ne peut compléter aucune ligne : l'alignement n'est pas vérifié ici. Il le
    /// sera, avec la règle d'or (B2), pour les attaques qui posent ou déplacent des pions.
    fn resolve_attack(&mut self, events: &mut Vec<Event>) {
        let Some(pending) = self.pending_attack.take() else {
            return;
        };
        let Some(launcher) = pending.launcher else {
            return;
        };
        let player = self.current_player;
        let attack = pending.attack;
        let stats = self.attack_stats(attack);
        let effects = attack
            .kind
            .rules()
            .effects(&self.context(player, attack, launcher), &pending.targets);
        let killer = self.board.get(launcher).and_then(Cell::pion).map(Pion::id);

        // Le coût a été vérifié au début de l'attaque, et rien ne l'a changé depuis.
        self.actions_left = self.actions_left.saturating_sub(stats.cost);
        if let Some(slot) = self.arsenals[player.index()].get_mut(pending.slot) {
            slot.set_cooldown(stats.cooldown);
        }
        events.push(Event::AttackUsed {
            player,
            attack,
            launcher,
        });
        events.push(self.actions_changed(player));
        events.push(Event::CooldownChanged {
            player,
            slot: pending.slot,
            turns_left: stats.cooldown,
        });

        for effect in effects {
            self.apply_effect(effect, killer, events);
        }
        self.after_action(events);
    }

    /// Le résolveur central : le seul endroit où un effet d'attaque modifie la partie.
    fn apply_effect(&mut self, effect: Effect, by: Option<PionId>, events: &mut Vec<Event>) {
        match effect {
            Effect::Kill { at } => {
                let Some(pion) = self.board.get(at).and_then(Cell::pion) else {
                    return;
                };
                if self.board.set(at, Cell::EMPTY).is_err() {
                    return;
                }
                let owner = pion.owner();
                let reserve = &mut self.reserve_pions[owner.index()];
                *reserve = reserve.saturating_add(1);
                events.push(Event::PionKilled { pion, at, by });
                events.push(self.reserve_changed(owner));
            }
        }
    }

    /// Le coût et la recharge d'une attaque. La configuration validée garantit que toute
    /// attaque d'un arsenal est au catalogue ; à défaut, l'attaque serait impayable.
    fn attack_stats(&self, attack: Attack) -> AttackStats {
        self.config.attack_stats(attack).unwrap_or(AttackStats {
            cost: u32::MAX,
            cooldown: u32::MAX,
        })
    }

    fn context(&self, player: Player, attack: Attack, launcher: Position) -> AttackContext<'_> {
        AttackContext {
            board: &self.board,
            player,
            launcher,
            rarity: attack.rarity,
        }
    }

    fn can_complete_from(
        &self,
        player: Player,
        attack: Attack,
        launcher: Position,
        chosen: &mut Vec<Position>,
    ) -> bool {
        let context = self.context(player, attack, launcher);
        can_complete(attack.kind.rules(), &context, chosen)
    }
}
