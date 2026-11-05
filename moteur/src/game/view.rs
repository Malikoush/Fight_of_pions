//! La vue joueur : ce qu'un joueur voit de la partie. C'est tout ce qu'un client (Godot,
//! le serveur pour chaque joueur) doit afficher, sans connaître aucune règle — la décision
//! attendue et les coups permis sont donnés —, et rien de ce que ce joueur ne doit pas
//! savoir : la cache adverse pas encore lancée, les cartes tirées pendant le choix des
//! caches.
//!
//! Ce qui est secret n'est décidé qu'à trois endroits :
//! [`Game::arsenal_seen_by`] et [`Game::visible_draft_pool`] pour l'état, [`Event::seen_by`] pour
//! les événements.
//! Ajouter une information visible, c'est ajouter un champ ici, lu sur la partie.

use super::state::Game;
use crate::{
    Action, ActiveEffect, ArsenalSlot, Attack, Cell, Decision, DraftStep, EndReason, Event,
    GameConfig, GameEnd, Outcome, Patient, Pion, Player, Position, TokenPile, rules,
};

/// Ce qu'un joueur voit de la partie, à un instant.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerView {
    /// Le joueur qui regarde.
    pub viewer: Player,
    /// Les réglages de la partie (taille, catalogue et coûts des cartes…), publics.
    pub config: GameConfig,
    /// Le numéro du tour en cours.
    pub turn: u32,
    /// Le joueur dont c'est le tour.
    pub current_player: Player,
    /// La décision attendue et qui doit la prendre ; `None` si la partie est finie.
    pub decision: Option<Decision>,
    /// Les coups que `viewer` peut jouer maintenant, dans l'ordre de
    /// [`Game::legal_actions`] ; vide si ce n'est pas à lui de décider.
    pub legal_actions: Vec<Action>,
    /// Le plateau.
    pub board: BoardView,
    /// Ce que chacun a (indice : [`Player::index`]).
    pub sides: [SideView; 2],
    /// Les effets continus en jeu, dans l'ordre de leur création.
    pub effects: Vec<ActiveEffect>,
    /// Les piles de jetons du Massacre divin.
    pub token_piles: Vec<TokenPile>,
    /// Les pions à l'infirmerie, dans l'ordre d'arrivée.
    pub infirmary: Vec<Patient>,
    /// Les pions au cimetière, dans l'ordre d'arrivée.
    pub graveyard: Vec<Pion>,
    /// Les cartes tirées du draft qui restent, quand on peut les voir (voir
    /// [`Game::visible_draft_pool`]) ; vide hors du draft.
    pub draft_pool: Vec<Attack>,
    /// La fin de la partie, si elle est finie.
    pub end: Option<GameEnd>,
    /// Les cases de la ligne gagnante (toutes les lignes d'assez de pions du gagnant), si la
    /// partie est gagnée par alignement ; vide sinon. Le client l'allume sans rien calculer.
    #[cfg_attr(feature = "serde", serde(default))]
    pub winning_line: Vec<Position>,
}

/// Le plateau tel qu'on le voit : toute la grille, mur compris.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoardView {
    /// Le nombre de cases par côté de la grille, mur compris.
    pub size: usize,
    /// Les cases rangée par rangée (indice : `row * size + col`) ; `None` pour une case de
    /// mur.
    pub cells: Vec<Option<Cell>>,
}

impl BoardView {
    /// La case de cette position, ou `None` pour le mur et hors de la grille.
    pub fn cell(&self, pos: Position) -> Option<Cell> {
        if pos.row >= self.size || pos.col >= self.size {
            return None;
        }
        self.cells
            .get(pos.row * self.size + pos.col)
            .copied()
            .flatten()
    }
}

/// Ce qu'un joueur a, vu par le joueur qui regarde.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SideView {
    /// Ses pions en réserve.
    pub reserve: u32,
    /// Ses actions en banque.
    pub banked_actions: u32,
    /// Ses actions du tour en cours ; 0 si ce n'est pas son tour.
    pub actions_left: u32,
    /// Ses poses possibles ce tour ; 0 si ce n'est pas son tour.
    pub placements_left: u32,
    /// Son arsenal, emplacement par emplacement ; `None` pour une carte que le joueur qui
    /// regarde ne peut pas voir.
    pub arsenal: Vec<Option<ArsenalSlot>>,
    /// La case de sa reine, s'il en a une.
    pub queen: Option<Position>,
    /// Le dernier tour où il est sous Terre brûlée, s'il l'est.
    pub scorched_until: Option<u32>,
}

impl Game {
    /// La partie telle que `viewer` la voit.
    pub fn view(&self, viewer: Player) -> PlayerView {
        let decision = self.pending_decision();
        let decides = decision.is_some_and(|decision| decision.player() == viewer);
        PlayerView {
            viewer,
            config: self.config.clone(),
            turn: self.turn,
            current_player: self.current_player,
            decision,
            legal_actions: if decides {
                self.legal_actions()
            } else {
                Vec::new()
            },
            board: self.board_view(),
            sides: [Player::One, Player::Two].map(|player| self.side_view(player, viewer)),
            effects: self.active_effects().to_vec(),
            token_piles: self.token_piles().to_vec(),
            infirmary: self.infirmary().to_vec(),
            graveyard: self.graveyard().to_vec(),
            draft_pool: self.visible_draft_pool().to_vec(),
            end: self.end,
            winning_line: self.winning_line(),
        }
    }

    /// Les cases de la ligne gagnante, si la partie est gagnée par alignement.
    pub fn winning_line(&self) -> Vec<Position> {
        match self.end {
            Some(GameEnd {
                outcome: Outcome::Won(winner),
                reason: EndReason::Alignment,
            }) => rules::line_cells(&self.board, winner, self.config.align_to_win),
            _ => Vec::new(),
        }
    }

    fn board_view(&self) -> BoardView {
        let size = self.board.size();
        let cells = (0..size)
            .flat_map(|row| (0..size).map(move |col| Position { row, col }))
            .map(|pos| self.board.get(pos))
            .collect();
        BoardView { size, cells }
    }

    fn side_view(&self, player: Player, viewer: Player) -> SideView {
        let is_current = player == self.current_player && self.end.is_none();
        SideView {
            reserve: self.reserve_pions(player),
            banked_actions: self.banked_actions(player),
            actions_left: if is_current { self.actions_left } else { 0 },
            placements_left: if is_current {
                self.placements_left()
            } else {
                0
            },
            arsenal: self.arsenal_seen_by(player, viewer),
            queen: self.queen_of(player),
            scorched_until: self.scorched_until(player),
        }
    }
}

impl Event {
    /// Cet événement tel que `viewer` le voit : tel quel, sauf la carte que l'adversaire met
    /// dans sa cache, masquée en [`Event::CardCached`] (elle se révèle par
    /// [`Event::CardRevealed`] quand il la lance). Rien n'est retiré : un client qui suit
    /// ses seuls événements reste à jour, emplacements compris.
    pub fn seen_by(self, viewer: Player) -> Event {
        match self {
            Event::Drafted {
                step: DraftStep::Cache(owner),
                ..
            } if owner != viewer => Event::CardCached { player: owner },
            event => event,
        }
    }
}
