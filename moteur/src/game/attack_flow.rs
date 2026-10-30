//! Le déroulement d'une attaque : la carte, puis le lanceur, puis les cibles, et enfin la
//! résolution d'un bloc. Tant que l'attaque n'est pas jouée, rien ne change ; et aucun
//! choix proposé ne mène à une impasse, puisque le moteur n'a pas d'« annuler ».
//!
//! L'attaque est jouée dès qu'il n'y a plus rien à choisir (nombre maximal de cibles, ou
//! plus aucune cible utile), ou plus tôt avec [`crate::Action::FinishAttack`] quand elle
//! est déjà complète (« jusqu'à 2 »).

use super::state::{Game, PendingAttack};
use crate::attacks::{
    AttackContext, Effect, Launch, can_act, can_choose_more, can_complete, is_complete,
};
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
        if !self.attack_has_a_target(player, attack, Launch::Now) {
            // Des cibles, mais seulement depuis des pions qui ne peuvent pas lancer
            // maintenant (posés ce tour, immobilisés) : le dire.
            return Err(
                if self.attack_has_a_target(player, attack, Launch::Someday) {
                    GameError::NoReadyLauncher(attack)
                } else {
                    GameError::NoTarget(attack)
                },
            );
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
            // Le lanceur : un de ses pions prêt à lancer, depuis lequel l'attaque peut
            // aller au bout.
            None => {
                self.can_launch(player, pos, Launch::Now)
                    && self.can_complete_from(player, attack, pos, Launch::Now)
            }
            // Une cible : proposée par la famille, et après laquelle on peut encore finir.
            Some(launcher) => {
                let context = self.context(player, attack, launcher, Launch::Now);
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

    /// Les seules cases qui peuvent répondre au choix attendu, rangée par rangée : ses
    /// pions pour le lanceur, puis les cibles que propose la famille. C'est
    /// `validate_choice` qui décide ; cette liste évite seulement de tester tout le plateau.
    pub(super) fn choice_candidates(&self, pending: &PendingAttack) -> Vec<Position> {
        let player = self.current_player;
        let mut cells: Vec<Position> = match pending.launcher {
            None => self
                .board
                .positions()
                .filter(|&pos| self.board.owner(pos) == Some(player))
                .collect(),
            Some(launcher) => {
                let context = self.context(player, pending.attack, launcher, Launch::Now);
                pending
                    .attack
                    .kind
                    .rules()
                    .candidate_targets(&context, &pending.targets)
            }
        };
        cells.sort_by_key(|pos| (pos.row, pos.col));
        cells
    }

    /// Jouer l'attaque avec les cibles déjà choisies : permis si elle est déjà complète.
    pub(super) fn validate_finish(&self, pending: &PendingAttack) -> Result<(), GameError> {
        let Some(launcher) = pending.launcher else {
            return Err(GameError::UnexpectedAction);
        };
        let attack = pending.attack;
        let context = self.context(self.current_player, attack, launcher, Launch::Now);
        if is_complete(attack.kind.rules(), &context, &pending.targets) {
            Ok(())
        } else {
            Err(GameError::TargetsMissing(attack))
        }
    }

    /// Vrai si ce joueur a, quelque part sur le plateau, un lanceur prêt (`when`) depuis
    /// lequel cette attaque peut aller au bout. Ne regarde ni le coût ni la recharge.
    pub(super) fn attack_has_a_target(&self, player: Player, attack: Attack, when: Launch) -> bool {
        self.board
            .positions()
            .filter(|&pos| self.can_launch(player, pos, when))
            .any(|launcher| self.can_complete_from(player, attack, launcher, when))
    }

    /// Vrai si le pion de cette case est à ce joueur et peut lancer une attaque `when` :
    /// maintenant, ni posé ce tour (mal du débarquement), ni immobilisé.
    fn can_launch(&self, player: Player, pos: Position, when: Launch) -> bool {
        self.board
            .get(pos)
            .and_then(Cell::pion)
            .is_some_and(|pion| can_act(pion, player, when, &self.effects, self.sick_from()))
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
        if self.nothing_left_to_choose() {
            self.resolve_attack(events);
        }
    }

    /// Vrai si l'attaque en cours est complète et qu'aucune autre cible ne peut s'y
    /// ajouter : elle est alors jouée sans attendre [`crate::Action::FinishAttack`].
    fn nothing_left_to_choose(&self) -> bool {
        let Some(pending) = &self.pending_attack else {
            return false;
        };
        let Some(launcher) = pending.launcher else {
            return false;
        };
        let attack = pending.attack;
        let rules = attack.kind.rules();
        let context = self.context(self.current_player, attack, launcher, Launch::Now);
        let mut chosen = pending.targets.clone();
        is_complete(rules, &context, &chosen) && !can_choose_more(rules, &context, &mut chosen)
    }

    /// Joue l'attaque dont les choix sont faits : paie le coût, met la carte en recharge,
    /// applique les effets, puis vérifie la fin du tour ou de la partie.
    ///
    /// L'alignement n'est pas vérifié ici : une ligne faite par une carte (un déplacement,
    /// un échange), pour l'un ou l'autre joueur, gagne au début du tour de son
    /// propriétaire si elle y est encore, avec ou sans la règle d'or (carnet, « Fin de
    /// partie »).
    pub(super) fn resolve_attack(&mut self, events: &mut Vec<Event>) {
        let Some(pending) = self.pending_attack.take() else {
            return;
        };
        let Some(launcher) = pending.launcher else {
            return;
        };
        let player = self.current_player;
        let attack = pending.attack;
        let stats = self.attack_stats(attack);
        let effects = attack.kind.rules().effects(
            &self.context(player, attack, launcher, Launch::Now),
            &pending.targets,
        );
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
            self.apply_effect(effect, attack, killer, events);
        }
        self.after_action(events);
    }

    /// Le résolveur central : le seul endroit où un effet d'attaque modifie la partie.
    fn apply_effect(
        &mut self,
        effect: Effect,
        source: Attack,
        by: Option<PionId>,
        events: &mut Vec<Event>,
    ) {
        match effect {
            Effect::Kill { at } => {
                let Some(pion) = self.board.get(at).and_then(Cell::pion) else {
                    return;
                };
                if self.board.set(at, Cell::EMPTY).is_err() {
                    return;
                }
                events.push(Event::PionKilled { pion, at, by });
                self.end_effects_on(pion.id(), events);
                self.send_to_infirmary(pion, events);
            }
            Effect::Apply { at, kind, turns } => {
                self.start_effect(at, kind, turns, source, events);
            }
            Effect::RemoveEnemyEffects { at } => {
                let Some(pion) = self.board.get(at).and_then(Cell::pion) else {
                    return;
                };
                let enemy = self.current_player.opponent();
                self.end_effects(|effect| effect.is_on_by(pion.id(), enemy), events);
            }
            Effect::Move { from, to } => {
                let Some(pion) = self.board.get(from).and_then(Cell::pion) else {
                    return;
                };
                if self.board.move_pion(from, to).is_ok() {
                    events.push(Event::PionMoved { pion, from, to });
                }
            }
            Effect::Swap { a, b } => {
                let pions = (
                    self.board.get(a).and_then(Cell::pion),
                    self.board.get(b).and_then(Cell::pion),
                );
                let (Some(first), Some(second)) = pions else {
                    return;
                };
                if self.board.swap(a, b).is_ok() {
                    events.push(Event::PionsSwapped {
                        first,
                        first_at: a,
                        second,
                        second_at: b,
                    });
                }
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

    fn context(
        &self,
        player: Player,
        attack: Attack,
        launcher: Position,
        launch: Launch,
    ) -> AttackContext<'_> {
        AttackContext {
            board: &self.board,
            effects: &self.effects,
            player,
            launcher,
            rarity: attack.rarity,
            launch,
            sick_from: self.sick_from(),
        }
    }

    /// Vrai si l'attaque peut aller au bout depuis ce lanceur, ses autres pions devant
    /// pouvoir agir `when`.
    fn can_complete_from(
        &self,
        player: Player,
        attack: Attack,
        launcher: Position,
        when: Launch,
    ) -> bool {
        let context = self.context(player, attack, launcher, when);
        can_complete(attack.kind.rules(), &context, &mut Vec::new())
    }
}
