//! Le déroulement d'une attaque : la carte, puis le lanceur, puis les cibles, et enfin la
//! résolution d'un bloc. Tant que l'attaque n'est pas jouée, rien ne change ; et aucun
//! choix proposé ne mène à une impasse, puisque le moteur n'a pas d'« annuler ».
//!
//! L'attaque est jouée dès qu'il n'y a plus rien à choisir (nombre maximal de cibles, ou
//! plus aucune cible utile), ou plus tôt avec [`crate::Action::FinishAttack`] quand elle
//! est déjà complète (« jusqu'à 2 »).

use super::kill::{Death, Hit};
use super::lasting::CardPlay;
use super::state::{Game, PendingAttack};
use crate::attacks::{
    AttackContext, Effect, Launch, Targets, can_act, can_choose_more, can_complete,
    can_complete_with_enemy_tokens, effects_of, is_complete,
};
use crate::{Attack, AttackStats, Cell, Event, GameError, Pion, PionId, Player, Position};

/// Une attaque dont les choix sont faits, prête à s'appliquer : ce qu'elle fera est fixé au
/// dernier choix. Elle peut attendre la réponse de l'autre joueur (Feintaille) ; les
/// effets portent alors sur les cases choisies, d'où un pion qui esquive est parti.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Resolution {
    /// L'emplacement de la carte dans l'arsenal du joueur.
    slot: usize,
    /// La carte jouée, payée et mise en recharge.
    pub(super) card: Attack,
    /// L'attaque qu'elle fait jouer (l'attaque copiée, pour un Perroquet).
    attack: Attack,
    /// La case du lanceur.
    launcher: Position,
    /// Le pion lanceur, l'assassin de ses kills.
    killer: Option<PionId>,
    /// Ses effets, dans l'ordre.
    effects: Vec<Effect>,
}

/// Ce qu'a donné un effet, pour la suite de l'attaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Applied {
    /// Joué (ou sans objet) : l'attaque continue.
    Done,
    /// Un garde a sauvé la reine de cette case : l'attaque ne lui fait plus rien.
    Shielded(Position),
    /// Un déplacement impossible : le reste de l'attaque n'a pas lieu.
    Stopped,
}

impl Game {
    pub(super) fn validate_use_attack(&self, slot: usize) -> Result<(), GameError> {
        let player = self.acting_player();
        let Some(slot_state) = self.arsenal(player).get(slot) else {
            return Err(GameError::NoSuchAttack(slot));
        };
        let card = slot_state.attack();
        self.ensure_right_moment(card)?;
        if slot_state.cooldown_left() > 0 {
            return Err(GameError::AttackRecharging(card));
        }
        self.ensure_points(player, self.attack_stats(card).cost)?;
        let attack = self.played_attack(card)?;
        if !self.attack_has_a_target(player, attack, Launch::Now) {
            if self.only_scorched_earth_forbids(player, attack) {
                return Err(GameError::ScorchedEarth(attack));
            }
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
        let player = self.acting_player();
        let attack = pending.attack;
        let allowed = match pending.launcher {
            // Le lanceur : un de ses pions prêt à lancer, depuis lequel l'attaque peut
            // aller au bout.
            None => {
                self.is_launcher(player, attack, pos, Launch::Now)
                    && self.can_complete_from(player, attack, pos, Launch::Now)
            }
            // Une cible : proposée par la famille, et après laquelle on peut encore finir.
            Some(launcher) => {
                let context = self.context(player, attack, launcher, Launch::Now);
                let rules = attack.kind.rules();
                let mut chosen = pending.targets.clone();
                let candidate = rules
                    .candidate_targets(&context, &chosen.cells)
                    .contains(&pos);
                chosen.cells.push(pos);
                candidate && can_complete(rules, &context, &mut chosen)
            }
        };
        if allowed {
            Ok(())
        } else {
            Err(GameError::InvalidChoice(pos))
        }
    }

    /// Viser les jetons de l'adversaire (Attila Rare et Épique) : permis après le lanceur,
    /// si l'attaque sait les viser et peut encore finir ensuite.
    pub(super) fn validate_enemy_tokens(&self, pending: &PendingAttack) -> Result<(), GameError> {
        let Some(launcher) = pending.launcher else {
            return Err(GameError::UnexpectedAction);
        };
        let attack = pending.attack;
        let rules = attack.kind.rules();
        let context = self.context(self.acting_player(), attack, launcher, Launch::Now);
        let mut chosen = pending.targets.clone();
        if can_complete_with_enemy_tokens(rules, &context, &mut chosen) {
            Ok(())
        } else {
            Err(GameError::UnexpectedAction)
        }
    }

    /// Les seules cases qui peuvent répondre au choix attendu, rangée par rangée : les
    /// lanceurs possibles, puis les cibles que propose la famille. C'est `validate_choice`
    /// qui décide ; cette liste évite seulement de tester tout le plateau.
    pub(super) fn choice_candidates(&self, pending: &PendingAttack) -> Vec<Position> {
        let player = self.acting_player();
        let mut cells: Vec<Position> = match pending.launcher {
            None => self
                .board
                .positions()
                .filter(|&pos| self.is_launcher(player, pending.attack, pos, Launch::Now))
                .collect(),
            Some(launcher) => {
                let context = self.context(player, pending.attack, launcher, Launch::Now);
                pending
                    .attack
                    .kind
                    .rules()
                    .candidate_targets(&context, &pending.targets.cells)
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
        let context = self.context(self.acting_player(), attack, launcher, Launch::Now);
        if is_complete(attack.kind.rules(), &context, &pending.targets) {
            Ok(())
        } else {
            Err(GameError::TargetsMissing(attack))
        }
    }

    /// Vrai si ce joueur a, quelque part sur le plateau, un lanceur prêt (`when`) depuis
    /// lequel cette attaque peut aller au bout. Ne regarde ni le coût ni la recharge.
    pub(super) fn attack_has_a_target(&self, player: Player, attack: Attack, when: Launch) -> bool {
        let mut launchers = self
            .board
            .positions()
            .filter(|&pos| self.is_launcher(player, attack, pos, when));
        if attack.kind.rules().ignores_launcher(attack.rarity) {
            // Les cibles ne dépendent pas du lanceur : le premier lanceur prêt suffit.
            return launchers
                .next()
                .is_some_and(|launcher| self.can_complete_from(player, attack, launcher, when));
        }
        launchers.any(|launcher| self.can_complete_from(player, attack, launcher, when))
    }

    /// Vrai si ce joueur est sous Terre brûlée et que, sans elle, l'attaque serait jouable
    /// maintenant : seulement pour dire pourquoi elle est refusée. La copie n'est faite que
    /// sous Terre brûlée.
    fn only_scorched_earth_forbids(&self, player: Player, attack: Attack) -> bool {
        if self.scorched_until(player).is_none() {
            return false;
        }
        let mut free = self.clone();
        free.scorched_until = [None, None];
        free.attack_has_a_target(player, attack, Launch::Now)
    }

    /// Vrai si cette attaque de ce joueur peut partir de cette case `when` : de son
    /// déclencheur pour une réaction qui en part, sinon d'un de ses pions prêt
    /// ([`Game::can_launch`]).
    fn is_launcher(&self, player: Player, attack: Attack, pos: Position, when: Launch) -> bool {
        self.is_trigger_launcher(player, attack, pos, when)
            .unwrap_or_else(|| self.can_launch(player, pos, when))
    }

    /// Vrai si le pion de cette case est à ce joueur et peut lancer une attaque `when` :
    /// maintenant, ni posé ce tour (mal du débarquement), ni immobilisé.
    pub(super) fn can_launch(&self, player: Player, pos: Position, when: Launch) -> bool {
        self.board
            .get(pos)
            .and_then(Cell::pion)
            .is_some_and(|pion| can_act(pion, player, when, &self.effects, self.sick_from()))
    }

    pub(super) fn start_attack(&mut self, slot: usize) {
        // L'emplacement a été validé ; à défaut, rien ne commence.
        let Some(slot_state) = self.arsenal(self.acting_player()).get(slot) else {
            return;
        };
        let card = slot_state.attack();
        // Validée : la carte a bien une attaque à faire jouer.
        let Ok(attack) = self.played_attack(card) else {
            return;
        };
        self.pending_attack = Some(PendingAttack {
            slot,
            card,
            attack,
            launcher: None,
            targets: Targets::default(),
        });
    }

    pub(super) fn execute_choice(&mut self, pos: Position, events: &mut Vec<Event>) {
        let Some(pending) = self.pending_attack.as_mut() else {
            return;
        };
        match pending.launcher {
            None => pending.launcher = Some(pos),
            Some(_) => pending.targets.cells.push(pos),
        }
        if self.nothing_left_to_choose() {
            self.resolve_attack(events);
        }
    }

    pub(super) fn execute_enemy_tokens(&mut self, events: &mut Vec<Event>) {
        let Some(pending) = self.pending_attack.as_mut() else {
            return;
        };
        pending.targets.enemy_tokens = true;
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
        let context = self.context(self.acting_player(), attack, launcher, Launch::Now);
        let mut chosen = pending.targets.clone();
        is_complete(rules, &context, &chosen) && !can_choose_more(rules, &context, &mut chosen)
    }

    /// L'attaque dont les choix sont faits : ses effets sont fixés, puis elle s'applique,
    /// sauf si elle vise un pion de l'autre joueur qui peut l'esquiver (Feintaille) : elle
    /// attend alors sa réponse ([`Game::open_targeted_window`]).
    pub(super) fn resolve_attack(&mut self, events: &mut Vec<Event>) {
        let Some(pending) = self.pending_attack.take() else {
            return;
        };
        let Some(launcher) = pending.launcher else {
            return;
        };
        let attack = pending.attack;
        let resolution = Resolution {
            slot: pending.slot,
            card: pending.card,
            attack,
            launcher,
            killer: self.board.get(launcher).and_then(Cell::pion).map(Pion::id),
            effects: effects_of(
                attack.kind.rules(),
                &self.context(self.acting_player(), attack, launcher, Launch::Now),
                &pending.targets,
            ),
        };
        // On ne réagit pas à une réaction.
        if self.reaction_window.is_none()
            && self.open_targeted_window(&resolution, &pending.targets.cells)
        {
            self.paused_attack = Some(resolution);
            return;
        }
        self.apply_resolution(resolution, events);
    }

    /// Joue l'attaque : paie le coût, met la carte en recharge, applique les effets, puis
    /// vérifie la fin du tour ou de la partie. Une réaction se joue de même, par le joueur
    /// qui réagit.
    ///
    /// L'alignement n'est pas vérifié ici : une ligne faite par une carte (un déplacement,
    /// un échange), pour l'un ou l'autre joueur, gagne au début du tour de son
    /// propriétaire si elle y est encore, avec ou sans la règle d'or (carnet, « Fin de
    /// partie »).
    pub(super) fn apply_resolution(&mut self, resolution: Resolution, events: &mut Vec<Event>) {
        let Resolution {
            slot,
            card,
            attack,
            launcher,
            killer,
            effects,
        } = resolution;
        let player = self.acting_player();
        let is_reaction = self.reaction_window.is_some();
        let had_a_line = self.has_winning_line(player);
        let stats = self.attack_stats(card);
        let play = CardPlay {
            id: self.next_play_id,
            attack,
        };
        self.next_play_id = play.id.next();

        // Le coût a été vérifié au début de l'attaque, et rien ne l'a changé depuis.
        self.pay(player, stats.cost);
        if let Some(arsenal_slot) = self.arsenals[player.index()].get_mut(slot) {
            arsenal_slot.set_cooldown(stats.cooldown);
            // Une carte de la cache est révélée quand elle est jouée pour la première fois.
            if arsenal_slot.is_secret() {
                arsenal_slot.reveal();
                events.push(Event::CardRevealed { player, slot });
            }
        }
        events.push(Event::AttackUsed {
            player,
            attack: card,
            launcher,
        });
        events.push(self.actions_changed(player));
        events.push(Event::CooldownChanged {
            player,
            slot,
            turns_left: stats.cooldown,
        });

        let mut deaths = Vec::new();
        // Les pions sauvés par un garde : l'attaque ne leur fait plus rien. Ils sont reconnus
        // par leur case, ce qui suffit puisqu'aucune famille ne pose ou ne pousse un autre
        // pion sur cette case après le kill : un déplacement impossible arrête l'attaque.
        let mut shielded = Vec::new();
        for effect in effects {
            if effect
                .single_target()
                .is_some_and(|at| shielded.contains(&at))
            {
                continue;
            }
            match self.apply_effect(effect, play, killer, &mut deaths, events) {
                Applied::Done => {}
                Applied::Shielded(at) => shielded.push(at),
                Applied::Stopped => break,
            }
        }
        // Les vengeances, une fois l'attaque entière jouée (carnet, 2026-10-30).
        self.avenge(deaths, events);
        if is_reaction {
            self.after_reaction(player, had_a_line, events);
        } else {
            self.after_action(events);
        }
    }

    /// Le résolveur central : le seul endroit où un effet d'attaque modifie la partie. Les
    /// pions tués s'ajoutent à `deaths`, pour les vengeances de fin d'attaque.
    fn apply_effect(
        &mut self,
        effect: Effect,
        play: CardPlay,
        by: Option<PionId>,
        deaths: &mut Vec<Death>,
        events: &mut Vec<Event>,
    ) -> Applied {
        match effect {
            Effect::Kill { at } => match self.kill(at, by, self.acting_player(), events) {
                Hit::Killed(death) => deaths.push(death),
                Hit::Shielded => return Applied::Shielded(at),
                Hit::Missed => {}
            },
            Effect::Apply { at, kind, turns } => {
                self.start_effect(at, kind, turns, play, events);
            }
            Effect::CancelEnemyEffects { at } => self.cancel_enemy_effects(at, events),
            Effect::CancelEnemyTokens => self.cancel_enemy_tokens(events),
            Effect::ScorchEarth => self.scorch_earth(events),
            Effect::AddTokens { reward, tokens } => self.add_tokens(reward, tokens, play, events),
            Effect::Attach { at, kind, end } => self.add_effect_at(at, kind, play, end, events),
            Effect::RemoveEnemyStatuses { at } => {
                if let Some(pion) = self.board.get(at).and_then(Cell::pion) {
                    let enemy = self.acting_player().opponent();
                    self.end_effects(|effect| effect.is_status_on_by(pion.id(), enemy), events);
                }
            }
            Effect::Place { at } => {
                self.put_pion(at, events);
            }
            Effect::Bind { a, b } => self.bind(a, b, play, events),
            Effect::Crown { at, guards } => self.crown(at, guards, play, events),
            Effect::MakeChild { at, framed_by } => self.make_child(at, framed_by, play, events),
            Effect::Resurrect { pion, at } => {
                let ours = self.infirmary.iter().any(|patient| {
                    patient.pion().id() == pion && patient.pion().owner() == self.acting_player()
                });
                if ours && self.board.is_free(at) && self.leave_infirmary(pion, events) {
                    self.create_pion(at, events);
                }
            }
            Effect::Move { from, to } => {
                // Un déplacement impossible arrête l'attaque : une charge ne passe pas une
                // reine qu'un garde vient de sauver.
                let Some(pion) = self.board.get(from).and_then(Cell::pion) else {
                    return Applied::Stopped;
                };
                if self.board.move_pion(from, to).is_err() {
                    return Applied::Stopped;
                }
                events.push(Event::PionMoved { pion, from, to });
            }
            Effect::Dig { at } => {
                if self.board.dig(at).is_ok() {
                    events.push(Event::WallDug { at });
                }
            }
            Effect::Swap { a, b } => {
                let pions = (
                    self.board.get(a).and_then(Cell::pion),
                    self.board.get(b).and_then(Cell::pion),
                );
                let (Some(first), Some(second)) = pions else {
                    return Applied::Done;
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
        Applied::Done
    }

    /// Le coût et la recharge d'une attaque. La configuration validée garantit que toute
    /// attaque d'un arsenal est au catalogue ; à défaut, l'attaque serait impayable.
    pub(super) fn attack_stats(&self, attack: Attack) -> AttackStats {
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
            token_piles: &self.token_piles,
            player,
            launcher,
            rarity: attack.rarity,
            launch,
            sick_from: self.sick_from(),
            reserve: self.reserve_pions[player.index()],
            // Terre brûlée s'arrêtera : « un jour », tout redevient permis.
            may_create_effects: launch == Launch::Someday || self.scorched_until(player).is_none(),
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
        can_complete(attack.kind.rules(), &context, &mut Targets::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameConfig, Rng};

    /// Une famille qui dit ignorer la case du lanceur doit avoir la même réponse depuis
    /// chacun de ses lanceurs prêts, à chaque coup de parties jouées au hasard.
    #[test]
    fn families_that_ignore_the_launcher_really_do() {
        let config = GameConfig {
            infirmary_turns: 2,
            ..GameConfig::first_rules().with_full_arsenals()
        };
        let mut checked = 0;
        for seed in 0..20 {
            let mut game = Game::new(config.clone(), seed).unwrap();
            let mut rng = Rng::new(seed);
            for _ in 0..150 {
                if game.end().is_some() {
                    break;
                }
                let player = game.current_player();
                for attack in config.catalog_attacks() {
                    if !attack.kind.rules().ignores_launcher(attack.rarity) {
                        continue;
                    }
                    let answers: Vec<bool> = game
                        .board
                        .positions()
                        .filter(|&pos| game.can_launch(player, pos, Launch::Now))
                        .map(|launcher| {
                            game.can_complete_from(player, attack, launcher, Launch::Now)
                        })
                        .collect();
                    assert!(
                        answers.windows(2).all(|pair| pair[0] == pair[1]),
                        "{attack} dépend du lanceur (partie {seed})"
                    );
                    checked += 1;
                }
                let legal = game.legal_actions();
                game.apply(*rng.pick(&legal).unwrap()).unwrap();
            }
        }
        assert!(checked > 1000, "seulement {checked} vérifications");
    }
}
