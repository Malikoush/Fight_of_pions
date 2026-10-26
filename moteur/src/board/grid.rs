use crate::{Cell, Direction, GameError, Player, Position};

/// Le damier carré de `size × size` cases.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn new(size: usize) -> Self {
        Self {
            size,
            cells: vec![Cell::Empty; size * size],
        }
    }
    fn index(&self, pos: Position) -> Option<usize> {
        if pos.row < self.size && pos.col < self.size {
            return Some(pos.row * self.size + pos.col);
        }
        None
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
    /// Remplace le contenu d'une case. Erreur si la position est hors du plateau.
    pub fn set(&mut self, pos: Position, cell: Cell) -> Result<(), GameError> {
        let i = self.index(pos).ok_or(GameError::OutOfBounds(pos))?;

        self.cells[i] = cell;
        Ok(())
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
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_board_is_empty() {
        let board = Board::new(10);
        assert_eq!(board.get(Position { row: 5, col: 5 }), Some(Cell::Empty));
    }
    #[test]
    fn outside_none() {
        let board = Board::new(10);
        assert_eq!(board.get(Position { row: 11, col: 11 }), None);
    }
    #[test]
    fn coin_3_voisin() {
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
