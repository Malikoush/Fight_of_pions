use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Direction {
    pub d_row: isize,
    pub d_col: isize,
}

impl Direction {
    pub const N: Direction = Direction {
        d_row: -1,
        d_col: 0,
    };
    pub const NE: Direction = Direction {
        d_row: -1,
        d_col: 1,
    };
    pub const E: Direction = Direction { d_row: 0, d_col: 1 };
    pub const SE: Direction = Direction { d_row: 1, d_col: 1 };
    pub const S: Direction = Direction { d_row: 1, d_col: 0 };
    pub const SW: Direction = Direction {
        d_row: 1,
        d_col: -1,
    };
    pub const W: Direction = Direction {
        d_row: 0,
        d_col: -1,
    };
    pub const NW: Direction = Direction {
        d_row: -1,
        d_col: -1,
    };

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
    pub const AXES: [Direction; 4] = [Direction::E, Direction::S, Direction::SE, Direction::SW];

    pub fn opposite(self) -> Direction {
        Direction {
            d_row: -self.d_row,
            d_col: -self.d_col,
        }
    }
}

impl Position {
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
