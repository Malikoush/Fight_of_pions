//! Le bot MCTS : une recherche arborescente de Monte-Carlo. Il essaie des suites de coups
//! sur des copies de la partie, juge les positions où elles mènent, et joue le coup le
//! plus exploré, c'est-à-dire celui dont les suites se sont montrées les meilleures.
//!
//! - **Aucune règle** : les coups viennent de `legal_actions`, et le moteur les joue sur
//!   des copies. Immobilisation, protection, déplacement, bonus : tout ce que le moteur
//!   applique, le MCTS le subit dans ses simulations, sans rien changer ici.
//! - **Une attaque entière est un seul coup** de l'arbre (carte, lanceur et cibles,
//!   jouées jusqu'au bout sur une copie) : la recherche voit plus loin à effort égal.
//! - **Les coups qui entrent dans l'arbre** : les meilleures poses selon les fenêtres ;
//!   pour chaque carte jouable maintenant, sa meilleure façon de la jouer selon
//!   l'évaluation, même si son effet ne paie que plus tard (c'est la recherche qui
//!   tranche, pas ce premier jugement) ; et toute autre action légale, fin de tour
//!   comprise, même d'une sorte que ce module ne connaît pas encore.
//! - **Pas de partie jouée au hasard jusqu'au bout** : au hasard, personne n'aligne
//!   jamais. Une position est jugée par l'évaluation ([`Weights`]), exacte quand la partie
//!   est finie.
//! - **Sans état et reproductible** : l'arbre est reconstruit à chaque décision, avec un
//!   nombre fixe d'itérations (jamais une durée), et son hasard vient du générateur que
//!   l'arène lui passe. Les calculs en flottant n'utilisent que +, −, ×, ÷ et la racine
//!   carrée, exacts selon la norme IEEE : le même coup sur toutes les machines.
//!
//! Sa limite : au-delà de son horizon, il ne juge que par l'évaluation. Un effet qui ne
//! change ni les fenêtres ni les actions disponibles, et ne paie qu'après cet horizon,
//! est sous-estimé.

use crate::reading::{attack_plans, placeable};
use crate::{Bot, Weights, Windows};
use fight_of_pions::{Action, Decision, Game, Player, Position, Rng};
use std::cmp::Reverse;

/// Les poses qui entrent dans l'arbre à chaque position : les meilleures selon les fenêtres.
const PLACEMENT_CANDIDATES: usize = 8;
/// Les attaques entières qui entrent dans l'arbre à chaque position, au moins : la
/// meilleure de chaque carte, puis les meilleures autres façons de jouer jusqu'à ce nombre.
const ATTACK_CANDIDATES: usize = 4;
/// Le poids de la curiosité face à ce qui a déjà marché : plus il est grand, plus la
/// recherche s'étale sur des coups peu essayés au lieu de creuser les meilleurs.
const EXPLORATION: f64 = 0.25;

/// Le bot MCTS (voir le module).
pub struct MctsBot {
    iterations: u32,
    weights: Weights,
}

impl MctsBot {
    /// Un bot qui fait `iterations` simulations par décision et juge les positions avec
    /// ces poids. Plus d'itérations : plus fort, et plus lent.
    pub const fn new(iterations: u32, weights: Weights) -> MctsBot {
        MctsBot {
            iterations,
            weights,
        }
    }
}

impl Bot for MctsBot {
    fn name(&self) -> &'static str {
        "MCTS"
    }

    fn choose(&self, game: &Game, rng: &mut Rng) -> Option<Action> {
        game.pending_decision()?;
        let windows = Windows::new(game.board(), game.config().align_to_win);
        let mut tree = Tree::new(&self.weights, &windows, game.clone(), rng);
        if let Some(action) = tree.forced_action() {
            return Some(action);
        }
        for _ in 0..self.iterations {
            tree.grow(rng);
        }
        tree.best_action()
    }
}

/// Un coup de l'arbre : une pose, une fin de tour ou une attaque entière, avec la partie
/// qu'il laisse et ce qu'elle vaut pour le joueur qui l'a choisi.
struct Move {
    /// La prochaine action à envoyer au moteur pour jouer ce coup.
    first: Action,
    mover: Player,
    after: Game,
    /// Les chances de gagner de `mover` dans `after`, selon l'évaluation.
    value: f64,
}

/// Une position de l'arbre.
struct Node {
    game: Game,
    /// Le coup qui mène ici, et le joueur qui l'a choisi ; `None` pour la racine.
    from: Option<(Action, Player)>,
    parent: Option<usize>,
    children: Vec<usize>,
    /// Les coups pas encore essayés, le plus prometteur à la fin ; `None` tant que la
    /// position n'a pas été revisitée. Une feuille est jugée par la valeur de son coup,
    /// et la plupart ne sont jamais revisitées : préparer leurs coups d'avance coûterait
    /// des copies de partie pour rien.
    untried: Option<Vec<Move>>,
    visits: u32,
    /// La somme des chances de gagner du joueur 1, sur les simulations passées par ici.
    wins_of_first: f64,
}

impl Node {
    /// Les chances de gagner moyennes, vues par le joueur qui a choisi le coup menant ici.
    fn mean_for_mover(&self) -> f64 {
        let mean = self.wins_of_first / f64::from(self.visits.max(1));
        match self.from {
            Some((_, Player::Two)) => 1.0 - mean,
            _ => mean,
        }
    }
}

/// L'arbre d'une décision, rangé dans un tableau : la racine est la case 0.
struct Tree<'a> {
    weights: &'a Weights,
    windows: &'a Windows,
    nodes: Vec<Node>,
}

impl<'a> Tree<'a> {
    fn new(weights: &'a Weights, windows: &'a Windows, game: Game, rng: &mut Rng) -> Tree<'a> {
        let mut tree = Tree {
            weights,
            windows,
            nodes: Vec::new(),
        };
        let untried = tree.moves(&game, rng);
        tree.nodes.push(Node {
            game,
            from: None,
            parent: None,
            children: Vec::new(),
            untried: Some(untried),
            visits: 0,
            wins_of_first: 0.0,
        });
        tree
    }

    /// La première action, si tous les coups possibles commencent par elle : rien à
    /// chercher.
    fn forced_action(&self) -> Option<Action> {
        let untried = self.nodes.first()?.untried.as_deref()?;
        let mut firsts = untried.iter().map(|mv| mv.first);
        let first = firsts.next()?;
        firsts.all(|action| action == first).then_some(first)
    }

    /// Une simulation : descendre par les coups les plus prometteurs jusqu'à une position
    /// qui a encore des coups à essayer, en essayer un, puis remonter ce que vaut la
    /// position atteinte.
    fn grow(&mut self, rng: &mut Rng) {
        let mut index = 0;
        while self.untried(index, rng).is_empty() {
            match self.most_promising_child(index) {
                Some(child) => index = child,
                None => break, // partie finie : il n'y a plus rien à essayer
            }
        }
        let wins_of_first = match self.untried(index, rng).pop() {
            Some(mv) => {
                let wins_of_first = match mv.mover {
                    Player::One => mv.value,
                    Player::Two => 1.0 - mv.value,
                };
                index = self.add_child(index, mv);
                wins_of_first
            }
            None => self
                .weights
                .win_chance(&self.nodes[index].game, self.windows, Player::One),
        };
        self.back_up(index, wins_of_first);
    }

    /// L'enfant à suivre pour la prochaine simulation : celui qui a donné les meilleures
    /// chances à son joueur, avec un bonus pour les moins essayés.
    fn most_promising_child(&self, index: usize) -> Option<usize> {
        let node = &self.nodes[index];
        let curiosity = EXPLORATION * f64::from(node.visits).sqrt();
        let promise = |child: usize| {
            let child = &self.nodes[child];
            child.mean_for_mover() + curiosity / (1.0 + f64::from(child.visits))
        };
        node.children
            .iter()
            .copied()
            .max_by(|&a, &b| promise(a).total_cmp(&promise(b)))
    }

    /// Les coups pas encore essayés depuis cette position, préparés au premier passage.
    fn untried(&mut self, index: usize, rng: &mut Rng) -> &mut Vec<Move> {
        if self.nodes[index].untried.is_none() {
            let moves = self.moves(&self.nodes[index].game, rng);
            self.nodes[index].untried = Some(moves);
        }
        self.nodes[index].untried.get_or_insert_default()
    }

    fn add_child(&mut self, parent: usize, mv: Move) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node {
            game: mv.after,
            from: Some((mv.first, mv.mover)),
            parent: Some(parent),
            children: Vec::new(),
            untried: None,
            visits: 0,
            wins_of_first: 0.0,
        });
        self.nodes[parent].children.push(index);
        index
    }

    /// Compte la simulation dans cette position et dans toutes celles au-dessus.
    fn back_up(&mut self, mut index: usize, wins_of_first: f64) {
        loop {
            let node = &mut self.nodes[index];
            node.visits += 1;
            node.wins_of_first += wins_of_first;
            match node.parent {
                Some(parent) => index = parent,
                None => return,
            }
        }
    }

    /// La première action du coup le plus exploré (à égalité, le mieux noté) ; à défaut
    /// de simulation, celle du coup le plus prometteur.
    fn best_action(&self) -> Option<Action> {
        let root = self.nodes.first()?;
        let best = root
            .children
            .iter()
            .map(|&child| &self.nodes[child])
            .max_by(|a, b| {
                a.visits
                    .cmp(&b.visits)
                    .then(a.mean_for_mover().total_cmp(&b.mean_for_mover()))
            });
        match best {
            Some(node) => node.from.map(|(action, _)| action),
            None => root.untried.as_deref()?.last().map(|mv| mv.first),
        }
    }

    /// Les coups qui entrent dans l'arbre depuis cette partie, jugés, le plus prometteur
    /// à la fin. Si une attaque est en cours, ce sont les fins possibles de cette attaque.
    fn moves(&self, game: &Game, rng: &mut Rng) -> Vec<Move> {
        let Some(mover) = game.pending_decision().map(Decision::player) else {
            return Vec::new();
        };
        let judge = |first: Action, after: Game| Move {
            value: self.weights.win_chance(&after, self.windows, mover),
            first,
            mover,
            after,
        };
        let mut moves = Vec::new();
        if !game.attack_in_progress() {
            let legal = game.legal_actions();
            // Les attaques sont jouées en entier plus bas ; toute autre action est un coup
            // à elle seule (fin de tour, et les sortes d'action à venir).
            let others = legal
                .iter()
                .copied()
                .filter(|action| !matches!(action, Action::Place(_) | Action::UseAttack(_)));
            let simple = self
                .best_placements(game, &legal, mover, rng)
                .into_iter()
                .map(Action::Place)
                .chain(others);
            for action in simple {
                let mut after = game.clone();
                // L'action vient de `legal_actions` : le moteur ne peut pas la refuser.
                if after.apply(action).is_ok() {
                    moves.push(judge(action, after));
                }
            }
        }
        let mut attacks: Vec<Move> = attack_plans(game)
            .into_iter()
            .map(|plan| judge(plan.first, plan.after))
            .collect();
        // À valeur égale, l'ordre vient du hasard, comme pour les poses.
        shuffle(&mut attacks, rng);
        moves.append(&mut best_of_each_card(attacks));
        moves.sort_by(|a, b| a.value.total_cmp(&b.value));
        moves
    }

    /// Les meilleures cases où poser selon les fenêtres, au plus `PLACEMENT_CANDIDATES`.
    fn best_placements(
        &self,
        game: &Game,
        legal: &[Action],
        mover: Player,
        rng: &mut Rng,
    ) -> Vec<Position> {
        let gains = self
            .weights
            .placement_gains(game.board(), self.windows, mover);
        let mut cells = placeable(legal);
        // À gain égal, l'ordre vient du hasard : les parties varient d'une graine à l'autre.
        shuffle(&mut cells, rng);
        cells.sort_by_key(|&pos| Reverse(gains.at(pos)));
        cells.truncate(PLACEMENT_CANDIDATES);
        cells
    }
}

/// Les attaques qui entrent dans l'arbre : la mieux jugée de chaque première action (de
/// chaque carte, ou de chaque choix d'une attaque en cours), puis les mieux jugées des
/// autres, jusqu'à `ATTACK_CANDIDATES` en tout. Ainsi chaque carte jouable est essayée par
/// la recherche, même quand son effet immédiat semble faible.
fn best_of_each_card(mut attacks: Vec<Move>) -> Vec<Move> {
    // Tri stable : à valeur égale, l'ordre reçu est gardé.
    attacks.sort_by(|a, b| b.value.total_cmp(&a.value));
    let (mut kept, mut others) = (Vec::new(), Vec::new());
    for attack in attacks {
        if kept.iter().any(|best: &Move| best.first == attack.first) {
            others.push(attack);
        } else {
            kept.push(attack);
        }
    }
    let room = ATTACK_CANDIDATES.saturating_sub(kept.len());
    kept.extend(others.into_iter().take(room));
    kept
}

/// Mélange la liste (Fisher-Yates), avec le hasard du bot.
fn shuffle<T>(items: &mut [T], rng: &mut Rng) {
    for last in (1..items.len()).rev() {
        let pick = u64::try_from(last + 1)
            .ok()
            .and_then(|bound| rng.below(bound))
            .and_then(|pick| usize::try_from(pick).ok());
        if let Some(pick) = pick {
            items.swap(last, pick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target_rules;

    #[test]
    fn each_position_is_judged_by_the_player_who_chose_it() {
        let mut game = Game::new(target_rules(), 0).unwrap();
        game.apply(Action::EndTurn).unwrap(); // à J2 de choisir
        let windows = Windows::new(game.board(), game.config().align_to_win);
        let weights = Weights::DEFAULT;
        let mut rng = Rng::new(0);
        let mut tree = Tree::new(&weights, &windows, game, &mut rng);
        let tried = tree.nodes[0].untried.as_deref().unwrap().last().unwrap();
        let chance_of_second = tried.value;
        assert!(chance_of_second > 0.5, "le meilleur coup de J2 l'avantage");

        tree.grow(&mut rng);
        // Le coup essayé est jugé par J2, qui l'a choisi ; la racine garde les chances de J1.
        let child = &tree.nodes[1];
        assert_eq!(child.from.map(|(_, player)| player), Some(Player::Two));
        assert!((child.mean_for_mover() - chance_of_second).abs() < 1e-12);
        assert!((tree.nodes[0].wins_of_first - (1.0 - chance_of_second)).abs() < 1e-12);
    }

    #[test]
    fn every_card_gets_its_best_attack_into_the_tree_even_a_weak_one() {
        let game = Game::new(target_rules(), 0).unwrap();
        let attack = |slot: usize, value: f64| Move {
            first: Action::UseAttack(slot),
            mover: Player::One,
            after: game.clone(),
            value,
        };
        // La carte 0 a quatre façons de jouer bien jugées, la carte 1 une seule, faible.
        let attacks = vec![
            attack(0, 0.9),
            attack(1, 0.1),
            attack(0, 0.8),
            attack(0, 0.7),
            attack(0, 0.6),
        ];
        let kept: Vec<(Action, f64)> = best_of_each_card(attacks)
            .iter()
            .map(|mv| (mv.first, mv.value))
            .collect();
        assert_eq!(
            kept,
            [
                (Action::UseAttack(0), 0.9),
                (Action::UseAttack(1), 0.1),
                (Action::UseAttack(0), 0.8),
                (Action::UseAttack(0), 0.7),
            ]
        );
    }
}
