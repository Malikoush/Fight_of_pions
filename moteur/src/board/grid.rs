use crate::{Cell, Direction, GameError, Pion, PionId, Player, Position};

/// Le damier carré de `size × size` cases.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    size: usize,
    cells: Vec<Cell>,
}
impl Board {
    /// Nombre de cases par côté.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Un plateau vide de `size × size` cases.
    pub(crate) fn new(size: usize) -> Self {
        Self {
            size,
            cells: vec![Cell::EMPTY; size * size],
        }
    }
    fn index(&self, pos: Position) -> Option<usize> {
        if pos.row < self.size && pos.col < self.size {
            return Some(pos.row * self.size + pos.col);
        }
        None
    }
    /// Toutes les positions du plateau, rangée par rangée, dans un ordre toujours identique.
    pub fn positions(&self) -> impl Iterator<Item = Position> + use<> {
        let size = self.size;
        (0..size).flat_map(move |row| (0..size).map(move |col| Position { row, col }))
    }

    /// Vrai si la position est sur le plateau.
    pub fn contains(&self, pos: Position) -> bool {
        self.index(pos).is_some()
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
        self.positions()
            .find(|&pos| self.tomb(pos).is_some_and(|dead| dead.id() == pion))
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
        self.positions().find(|&pos| {
            self.get(pos)
                .and_then(Cell::pion)
                .is_some_and(|found| found.id() == pion)
        })
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
