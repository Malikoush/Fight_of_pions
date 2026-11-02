use crate::{Cell, Direction, GameError, Pion, PionId, Player, Position};

/// Le damier carré de `size × size` cases, mur compris.
///
/// Le mur entoure le plateau de jeu (carnet, 2026-11-03) : ses cases ne sont pas sur le
/// plateau (ni libres, ni visibles, ni franchissables) tant que le Coup du taureau ne les a
/// pas creusées ; une case creusée devient une case normale pour toujours.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    size: usize,
    cells: Vec<Cell>,
    /// Vrai pour chaque case de mur pas encore creusée, dans l'ordre des cases.
    walls: Vec<bool>,
}
impl Board {
    /// Nombre de cases par côté, mur compris : c'est la grille que les noms « A1 »
    /// couvrent.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Un plateau vide de `size × size` cases, sans mur.
    #[cfg(test)]
    pub(crate) fn new(size: usize) -> Self {
        Self::walled(size, 0)
    }

    /// Un plateau vide de `size × size` cases, entouré de `wall` rangées de mur : la grille
    /// fait `size + 2 × wall` cases par côté.
    pub(crate) fn walled(size: usize, wall: usize) -> Self {
        let side = size + 2 * wall;
        let in_wall = |i: usize| i < wall || i >= wall + size;
        Self {
            size: side,
            cells: vec![Cell::EMPTY; side * side],
            walls: (0..side * side)
                .map(|i| in_wall(i / side) || in_wall(i % side))
                .collect(),
        }
    }

    /// L'indice de la case dans la grille, mur compris.
    fn grid_index(&self, pos: Position) -> Option<usize> {
        if pos.row < self.size && pos.col < self.size {
            return Some(pos.row * self.size + pos.col);
        }
        None
    }

    /// L'indice de la case, si elle est sur le plateau (pas dans le mur).
    fn index(&self, pos: Position) -> Option<usize> {
        self.grid_index(pos)
            .filter(|&i| self.walls.get(i) == Some(&false))
    }

    /// Toutes les positions du plateau, mur exclu, rangée par rangée, dans un ordre
    /// toujours identique.
    pub fn positions(&self) -> impl Iterator<Item = Position> + '_ {
        let size = self.size;
        (0..size)
            .flat_map(move |row| (0..size).map(move |col| Position { row, col }))
            .filter(|&pos| self.contains(pos))
    }

    /// Vrai si la position est sur le plateau : dans la grille, et pas dans le mur.
    pub fn contains(&self, pos: Position) -> bool {
        self.index(pos).is_some()
    }

    /// Vrai si cette case est du mur pas encore creusé.
    pub fn is_wall(&self, pos: Position) -> bool {
        self.grid_index(pos)
            .is_some_and(|i| self.walls.get(i) == Some(&true))
    }

    /// Creuse cette case de mur : elle devient une case normale, pour toujours. Erreur, sans
    /// rien changer, si ce n'est pas du mur.
    pub(crate) fn dig(&mut self, pos: Position) -> Result<(), GameError> {
        let i = self
            .grid_index(pos)
            .filter(|_| self.is_wall(pos))
            .ok_or(GameError::OutOfBounds(pos))?;
        let wall = self.walls.get_mut(i).ok_or(GameError::OutOfBounds(pos))?;
        *wall = false;
        Ok(())
    }
    /// Le contenu de la case, ou `None` hors du plateau.
    pub fn get(&self, pos: Position) -> Option<Cell> {
        let i = self.index(pos)?;
        Some(self.cells[i])
    }
    fn cell_mut(&mut self, pos: Position) -> Result<&mut Cell, GameError> {
        let i = self.index(pos).ok_or(GameError::OutOfBounds(pos))?;
        self.cells.get_mut(i).ok_or(GameError::OutOfBounds(pos))
    }

    /// Vrai si la case est sur le plateau et sans pion : la seule définition d'une case
    /// libre. Une tombe ne bloque pas la case.
    pub fn is_free(&self, pos: Position) -> bool {
        self.get(pos).is_some_and(|cell| cell.pion().is_none())
    }

    // Les opérations sur les pions ne touchent qu'à la couche des pions : une tombe reste
    // sur sa case quand un pion y arrive, en part, bouge ou est échangé.

    /// Pose ce pion sur cette case libre. Erreur, sans rien changer, si elle ne l'est pas.
    pub(crate) fn place_pion(&mut self, pos: Position, pion: Pion) -> Result<(), GameError> {
        if !self.is_free(pos) {
            return Err(GameError::CellOccupied(pos));
        }
        self.cell_mut(pos)?.pion = Some(pion);
        Ok(())
    }

    /// Retire le pion de cette case et le renvoie, s'il y en avait un.
    pub(crate) fn remove_pion(&mut self, pos: Position) -> Option<Pion> {
        self.cell_mut(pos).ok()?.pion.take()
    }

    /// Déplace le pion de `from` vers `to`, une case libre. Erreur, sans rien changer, si
    /// `from` n'a pas de pion ou si `to` n'est pas libre ou hors du plateau.
    pub(crate) fn move_pion(&mut self, from: Position, to: Position) -> Result<(), GameError> {
        let Some(pion) = self.get(from).and_then(Cell::pion) else {
            return Err(GameError::InvalidChoice(from));
        };
        self.place_pion(to, pion)?;
        self.remove_pion(from);
        Ok(())
    }

    /// Échange les pions de deux cases. Erreur, sans rien changer, si l'une est hors du
    /// plateau.
    pub(crate) fn swap(&mut self, a: Position, b: Position) -> Result<(), GameError> {
        let pion_a = self.get(a).ok_or(GameError::OutOfBounds(a))?.pion();
        let pion_b = self.get(b).ok_or(GameError::OutOfBounds(b))?.pion();
        self.cell_mut(a)?.pion = pion_b;
        self.cell_mut(b)?.pion = pion_a;
        Ok(())
    }

    /// Le pion tué dont la tombe est sur cette case, s'il y en a une.
    pub fn tomb(&self, pos: Position) -> Option<Pion> {
        self.get(pos).and_then(Cell::tomb)
    }

    /// La case de la tombe de ce pion, s'il en a une.
    pub fn tomb_of(&self, pion: PionId) -> Option<Position> {
        self.find_cell(|cell| cell.tomb().is_some_and(|dead| dead.id() == pion))
    }

    /// La première case, rangée par rangée, qui vérifie `test`. Parcourt les cases
    /// directement : aucun pion ni tombe n'est jamais dans le mur, puisque les écritures
    /// passent par `index`.
    fn find_cell(&self, test: impl Fn(Cell) -> bool) -> Option<Position> {
        let i = self.cells.iter().position(|&cell| test(cell))?;
        Some(Position {
            row: i / self.size,
            col: i % self.size,
        })
    }

    /// Met cette tombe sur la case (ou l'efface avec `None`) et renvoie celle qu'elle
    /// remplace : une case porte au plus une tombe (carnet, 2026-10-30).
    pub(crate) fn set_tomb(&mut self, pos: Position, tomb: Option<Pion>) -> Option<Pion> {
        let cell = self.cell_mut(pos).ok()?;
        std::mem::replace(&mut cell.tomb, tomb)
    }

    /// Le propriétaire du pion sur la case, ou `None` (case libre ou hors plateau).
    pub fn owner(&self, pos: Position) -> Option<Player> {
        let j = self.index(pos)?;
        self.cells[j].owner()
    }
    /// La case voisine dans une direction, ou `None` si elle sort du plateau.
    pub fn neighbor(&self, pos: Position, dir: Direction) -> Option<Position> {
        let i = pos.step(dir)?;
        if self.contains(i) {
            return Some(i);
        }
        None
    }

    /// La case où se trouve ce pion, ou `None` s'il n'est pas sur le plateau. Un effet
    /// continu vise un pion par son identité : c'est ainsi qu'on retrouve sa case.
    pub fn position_of(&self, pion: PionId) -> Option<Position> {
        self.find_cell(|cell| cell.pion().is_some_and(|found| found.id() == pion))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_board_is_empty() {
        let board = Board::new(10);
        assert_eq!(board.get(Position { row: 5, col: 5 }), Some(Cell::EMPTY));
    }
    #[test]
    fn a_tomb_stays_on_its_cell_when_pions_move_or_swap() {
        let mut board = Board::new(3);
        let at = |row, col| Position { row, col };
        let dead = Pion::new(PionId::FIRST, Player::One);
        let walker = Pion::new(PionId::FIRST.next(), Player::Two);
        let other = Pion::new(PionId::FIRST.next().next(), Player::One);
        board.set_tomb(at(0, 0), Some(dead));
        board.place_pion(at(0, 1), walker).unwrap();
        board.place_pion(at(2, 2), other).unwrap();

        // Un pion arrive sur la tombe, qui ne bloque pas la case, puis repart.
        board.move_pion(at(0, 1), at(0, 0)).unwrap();
        assert_eq!(board.tomb(at(0, 0)), Some(dead));
        board.swap(at(0, 0), at(2, 2)).unwrap();
        assert_eq!(board.tomb(at(0, 0)), Some(dead));
        assert_eq!(board.get(at(0, 0)).and_then(Cell::pion), Some(other));
        assert_eq!(board.tomb(at(2, 2)), None);
        assert_eq!(board.tomb_of(dead.id()), Some(at(0, 0)));
    }

    #[test]
    fn reading_outside_the_board_gives_nothing() {
        let board = Board::new(10);
        assert_eq!(board.get(Position { row: 11, col: 11 }), None);
    }
    #[test]
    fn the_wall_is_not_on_the_board_until_dug() {
        // 2 × 2 entouré d'une rangée de mur : grille de 4 × 4, plateau de B2 à C3.
        let mut board = Board::walled(2, 1);
        let at = |row, col| Position { row, col };
        assert_eq!(board.size(), 4);
        assert_eq!(
            board.positions().collect::<Vec<_>>(),
            vec![at(1, 1), at(1, 2), at(2, 1), at(2, 2)]
        );
        assert!(board.is_wall(at(0, 0)) && board.is_wall(at(1, 3)));
        assert!(!board.is_wall(at(1, 1)) && !board.is_wall(at(4, 4)));
        assert!(!board.contains(at(1, 0)) && !board.is_free(at(1, 0)));
        assert_eq!(board.neighbor(at(1, 1), Direction::W), None);
        let pion = Pion::new(PionId::FIRST, Player::One);
        assert!(board.place_pion(at(1, 0), pion).is_err());

        board.dig(at(1, 0)).unwrap();
        assert!(!board.is_wall(at(1, 0)) && board.is_free(at(1, 0)));
        assert_eq!(board.neighbor(at(1, 1), Direction::W), Some(at(1, 0)));
        assert!(board.place_pion(at(1, 0), pion).is_ok());
        assert_eq!(board.positions().count(), 5);
        // On ne creuse que du mur.
        assert!(board.dig(at(1, 0)).is_err());
        assert!(board.dig(at(4, 4)).is_err());
    }

    #[test]
    fn corner_cell_has_three_neighbors() {
        let board = Board::new(10);
        let pos = Position { row: 0, col: 0 };
        assert_eq!(
            Direction::ALL
                .iter()
                .filter(|dir| board.neighbor(pos, **dir).is_some())
                .count(),
            3
        );
    }
}
