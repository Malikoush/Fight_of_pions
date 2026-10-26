use std::fmt;
use std::str::FromStr;

/// Une case repérée par sa rangée et sa colonne, à partir de 0.
///
/// Affichée et lue au format « C5 » : lettre de colonne, puis numéro de rangée à partir de 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Rangée, de haut en bas, à partir de 0.
    pub row: usize,
    /// Colonne, de gauche à droite, à partir de 0.
    pub col: usize,
}
/// Un pas d'une case dans l'une des 8 directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Direction {
    /// Variation de rangée : -1, 0 ou 1.
    pub d_row: isize,
    /// Variation de colonne : -1, 0 ou 1.
    pub d_col: isize,
}

impl Direction {
    /// Vers le haut.
    pub const N: Direction = Direction {
        d_row: -1,
        d_col: 0,
    };
    /// Vers le haut à droite.
    pub const NE: Direction = Direction {
        d_row: -1,
        d_col: 1,
    };
    /// Vers la droite.
    pub const E: Direction = Direction { d_row: 0, d_col: 1 };
    /// Vers le bas à droite.
    pub const SE: Direction = Direction { d_row: 1, d_col: 1 };
    /// Vers le bas.
    pub const S: Direction = Direction { d_row: 1, d_col: 0 };
    /// Vers le bas à gauche.
    pub const SW: Direction = Direction {
        d_row: 1,
        d_col: -1,
    };
    /// Vers la gauche.
    pub const W: Direction = Direction {
        d_row: 0,
        d_col: -1,
    };
    /// Vers le haut à gauche.
    pub const NW: Direction = Direction {
        d_row: -1,
        d_col: -1,
    };

    /// Les 8 directions.
    pub const ALL: [Direction; 8] = [
        Direction::E,
        Direction::N,
        Direction::NE,
        Direction::NW,
        Direction::S,
        Direction::SE,
        Direction::SW,
        Direction::W,
    ];
    /// Une direction par axe d'alignement : horizontal, vertical et les deux diagonales.
    pub const AXES: [Direction; 4] = [Direction::E, Direction::S, Direction::SE, Direction::SW];

    /// La direction inverse.
    pub fn opposite(self) -> Direction {
        Direction {
            d_row: -self.d_row,
            d_col: -self.d_col,
        }
    }
}

impl Position {
    /// La position un pas plus loin, ou `None` si elle passe sous 0.
    /// Ne connaît pas la taille du plateau : voir [`crate::Board::neighbor`].
    pub fn step(self, dir: Direction) -> Option<Position> {
        let row = self.row as isize + dir.d_row;
        let col = self.col as isize + dir.d_col;
        if row >= 0 && col >= 0 {
            return Some(Position {
                row: row as usize,
                col: col as usize,
            });
        }

        None
    }
    /// Vrai si les deux cases se touchent, diagonales comprises.
    pub fn isadjacent(self, other: Position) -> bool {
        let ecart_l = self.row.abs_diff(other.row);
        let ecart_c = self.col.abs_diff(other.col);

        ecart_c <= 1 && ecart_l <= 1 && (ecart_c == 1 || ecart_l == 1)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", (b'A' + self.col as u8) as char, self.row + 1)
    }
}

/// « C5 » → Position { row: 4, col: 2 }.
/// Ne vérifie pas que la case est sur le plateau : le moteur s'en charge.
impl FromStr for Position {
    type Err = String;

    fn from_str(text: &str) -> Result<Position, String> {
        let mut chars = text.chars();

        let letter = match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => c.to_ascii_uppercase(),
            _ => {
                return Err(format!(
                    "« {} » : il faut une lettre puis un nombre, par exemple C5",
                    text
                ));
            }
        };

        let number: usize = match chars.as_str().parse() {
            Ok(n) if n >= 1 => n,
            _ => {
                return Err(format!(
                    "« {} » : le nombre après la lettre est invalide",
                    text
                ));
            }
        };

        let col = (letter as u8 - b'A') as usize;
        Ok(Position {
            row: number - 1,
            col,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_positions() {
        assert_eq!("C5".parse(), Ok(Position { row: 4, col: 2 }));
        assert_eq!("j10".parse(), Ok(Position { row: 9, col: 9 }));
        assert!("5C".parse::<Position>().is_err());
        assert!("A0".parse::<Position>().is_err());
        assert!("".parse::<Position>().is_err());
    }

    #[test]
    fn display_and_parse_agree() {
        let pos = Position { row: 4, col: 2 };
        assert_eq!(pos.to_string().parse(), Ok(pos));
    }
}
