//! Le draft (B9a, carnet « Draft et cache secrète ») : avant la partie, les joueurs font
//! leurs choix dans l'ordre de [`crate::DraftRules::steps`]. La cache se choisit dans tout
//! le catalogue ; les bans et les picks, parmi des cartes tirées avec la graine à la
//! création de la partie. Pendant le draft, seul [`crate::Action::Draft`] est permis.

use super::arsenal::ArsenalSlot;
use super::state::Game;
use crate::{Action, Attack, DraftStep, Event, GameConfig, GameError, Rng};

/// Le draft en cours.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct DraftState {
    /// Les cartes tirées qui restent, dans l'ordre du catalogue.
    pool: Vec<Attack>,
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
        let mut chosen = order[..drawn].to_vec();
        chosen.sort_unstable();
        Some(DraftState {
            pool: chosen.into_iter().map(|index| catalog[index]).collect(),
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
    /// vide hors du draft.
    pub fn draft_pool(&self) -> &[Attack] {
        self.draft.as_ref().map_or(&[], |draft| &draft.pool)
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
        if let Some(slot) = slot {
            self.arsenals[step.player().index()].push(slot);
        }
        draft.next_step += 1;
        events.push(Event::Drafted { step, attack });
        if self.draft_step().is_none() {
            self.draft = None;
        }
    }
}
