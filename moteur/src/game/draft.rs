//! Le draft (B9a, carnet « Draft et cache secrète ») : avant la partie, les joueurs font
//! leurs choix dans l'ordre de [`crate::DraftRules::steps`]. La cache se choisit dans tout
//! le catalogue ; les bans et les picks, parmi des cartes tirées avec la graine à la
//! création de la partie. Pendant le draft, seul [`crate::Action::Draft`] est permis.

use super::arsenal::ArsenalSlot;
use super::state::Game;
use super::view::DraftEntry;
use crate::{Action, Attack, DraftStep, Event, GameConfig, GameError, Player, Rng};

/// Le draft en cours.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct DraftState {
    /// Les cartes tirées qui restent, dans l'ordre du catalogue.
    pool: Vec<Attack>,
    /// Les cartes bannies, dans l'ordre : les seuls choix que rien d'autre ne garde (un pick
    /// ou une cache est dans l'arsenal de son joueur).
    banned: Vec<Attack>,
    /// L'indice de l'étape en cours dans `DraftRules::steps`.
    next_step: usize,
}

impl DraftState {
    /// Le draft de cette configuration, ses cartes tirées avec cette graine ; `None` sans
    /// draft (ou sans aucun choix).
    pub(super) fn start(config: &GameConfig, seed: u64) -> Option<DraftState> {
        let rules = config
            .draft
            .as_ref()
            .filter(|rules| !rules.steps.is_empty())?;
        // Les `pool_size` premières cartes d'un mélange du catalogue (Fisher-Yates
        // partiel), remises dans l'ordre du catalogue pour se lire facilement.
        let catalog = config.catalog_attacks();
        let mut order: Vec<usize> = (0..catalog.len()).collect();
        let mut rng = Rng::new(seed);
        let drawn = rules.pool_size.min(catalog.len());
        for index in 0..drawn {
            let left = u64::try_from(catalog.len() - index).unwrap_or(u64::MAX);
            let offset = rng
                .below(left)
                .and_then(|offset| usize::try_from(offset).ok())
                .unwrap_or(0);
            order.swap(index, index + offset);
        }
        let mut drawn_cards = order[..drawn].to_vec();
        drawn_cards.sort_unstable();
        Some(DraftState {
            pool: drawn_cards
                .into_iter()
                .map(|index| catalog[index])
                .collect(),
            banned: Vec::new(),
            next_step: 0,
        })
    }
}

impl Game {
    /// L'étape du draft en cours, s'il y en a un.
    pub fn draft_step(&self) -> Option<DraftStep> {
        let draft = self.draft.as_ref()?;
        self.config
            .draft
            .as_ref()?
            .steps
            .get(draft.next_step)
            .copied()
    }

    /// Les cartes tirées qui restent à bannir ou à prendre, dans l'ordre du catalogue ;
    /// vide hors du draft. Toutes, même avant qu'un joueur puisse les voir : ce que voient
    /// les joueurs, c'est [`Game::visible_draft_pool`].
    pub fn draft_pool(&self) -> &[Attack] {
        self.draft.as_ref().map_or(&[], |draft| &draft.pool)
    }

    /// Les cartes tirées telles que les joueurs les voient : aucune tant qu'aucune étape ne
    /// les a utilisées. Un format qui fait choisir la cache d'abord la fait choisir sans les
    /// connaître (carnet, 2026-11-03) ; le format standard commence par les bans, et les
    /// montre dès le début (2026-11-05).
    pub fn visible_draft_pool(&self) -> &[Attack] {
        let shown = match (&self.draft, &self.config.draft) {
            (Some(draft), Some(rules)) => rules
                .steps
                .iter()
                .take(draft.next_step.saturating_add(1))
                .any(|step| step.uses_the_pool()),
            _ => false,
        };
        if shown { self.draft_pool() } else { &[] }
    }

    /// Les choix déjà faits du draft, dans l'ordre des étapes, tels que `viewer` les voit :
    /// la carte que son adversaire a mise en cache reste cachée, comme dans
    /// [`Event::seen_by`]. Les bans et les picks sont publics. Vide hors du draft.
    ///
    /// Un ban est relu dans les cartes bannies ; un pick ou une cache, dans l'arsenal de son
    /// joueur (emplacement suivant, après ses cartes de départ) : une partie aux secrets
    /// devinés ([`Game::with_secrets_guessed`]) n'y garde rien de la vraie cache.
    pub fn draft_history_seen_by(&self, viewer: Player) -> Vec<DraftEntry> {
        let (Some(draft), Some(rules)) = (&self.draft, &self.config.draft) else {
            return Vec::new();
        };
        let mut bans = draft.banned.iter();
        let mut next_slot = self.config.starting_attacks.each_ref().map(Vec::len);
        let mut history = Vec::new();
        for &step in rules.steps.iter().take(draft.next_step) {
            let attack = match step {
                DraftStep::Ban(_) => bans.next().copied(),
                DraftStep::Pick(player) | DraftStep::Cache(player) => {
                    let slot = &mut next_slot[player.index()];
                    let card = self.arsenals[player.index()]
                        .get(*slot)
                        .map(|slot| slot.attack());
                    *slot += 1;
                    card
                }
            };
            history.push(DraftEntry {
                step,
                attack: match step {
                    DraftStep::Cache(owner) if owner != viewer => None,
                    _ => attack,
                },
            });
        }
        history
    }

    /// Les cartes que l'étape en cours permet de choisir : tout le catalogue pour la cache,
    /// les cartes tirées qui restent pour un ban ou un pick.
    fn draft_choices(&self) -> Vec<Attack> {
        match self.draft_step() {
            Some(step) if step.uses_the_pool() => self.draft_pool().to_vec(),
            Some(_) => self.config.catalog_attacks(),
            None => Vec::new(),
        }
    }

    /// Les actions permises pendant le draft, dans l'ordre du catalogue.
    pub(super) fn draft_actions(&self) -> Vec<Action> {
        self.draft_choices()
            .into_iter()
            .map(Action::Draft)
            .collect()
    }

    pub(super) fn validate_draft(&self, action: Action) -> Result<(), GameError> {
        let Action::Draft(attack) = action else {
            return Err(GameError::UnexpectedAction);
        };
        if self.draft_choices().contains(&attack) {
            Ok(())
        } else {
            Err(GameError::NotInDraft(attack))
        }
    }

    /// Joue le choix, déjà validé, de l'étape en cours ; après la dernière, la partie
    /// commence.
    pub(super) fn execute_draft(&mut self, attack: Attack, events: &mut Vec<Event>) {
        let Some(step) = self.draft_step() else {
            return;
        };
        let Some(draft) = self.draft.as_mut() else {
            return;
        };
        if step.uses_the_pool() {
            draft.pool.retain(|&card| card != attack);
        }
        let slot = match step {
            DraftStep::Cache(_) => Some(ArsenalSlot::secret(attack)),
            DraftStep::Pick(_) => Some(ArsenalSlot::new(attack)),
            DraftStep::Ban(_) => None,
        };
        match slot {
            Some(slot) => self.arsenals[step.player().index()].push(slot),
            None => draft.banned.push(attack),
        }
        draft.next_step += 1;
        events.push(Event::Drafted { step, attack });
        if self.draft_step().is_none() {
            self.draft = None;
        }
    }
}
