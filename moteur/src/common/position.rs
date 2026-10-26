use std::fmt;
use std::str::FromStr;

/// Une case repérée par sa rangée et sa colonne, à partir de 0.
///
/// Affichée et lue au format « C5 » : lettre de colonne, puis numéro de rangée à partir de 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Position {
    /// Rangée, de haut en bas, à partir de 0.
    pub row: usize,
    /// Colonne, de gauche à droite, à partir de 0.
    pub col: usize,
}
/// Un pas d'une case dans l'une des 8 directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Nombre de colonnes qu'on sait nommer par une lettre, de A à Z.
    /// C'est ce qui limite la taille du plateau.
    pub const NAMED_COLUMNS: usize = 26;

    /// La lettre d'une colonne (0 → 'A'), ou `None` au-delà de Z.
    pub fn column_letter(col: usize) -> Option<char> {
        let offset = u8::try_from(col).ok()?;
        if usize::from(offset) >= Self::NAMED_COLUMNS {
            return None;
        }
        Some(char::from(b'A' + offset))
    }

    /// La position un pas plus loin, ou `None` si elle sort des nombres représentables
    /// (sous 0, par exemple). Ne connaît pas la taille du plateau : voir
    /// [`crate::Board::neighbor`].
    pub fn step(self, dir: Direction) -> Option<Position> {
        Some(Position {
            row: self.row.checked_add_signed(dir.d_row)?,
            col: self.col.checked_add_signed(dir.d_col)?,
        })
    }
}

/// « C5 » pour une case nommable. Une position impossible à nommer (colonne au-delà de Z,
/// rangée démesurée) s'affiche avec ses indices bruts, sans jamais planter.
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (Self::column_letter(self.col), self.row.checked_add(1)) {
            (Some(letter), Some(number)) => write!(f, "{}{}", letter, number),
            _ => write!(
                f,
                "(rangée {}, colonne {}, comptées depuis 0)",
                self.row, self.col
            ),
        }
    }
}

/// « C5 » → Position { row: 4, col: 2 }.
/// Ne vérifie pas que la case est sur le plateau : le moteur s'en charge.
impl FromStr for Position {
    type Err = String;

    fn from_str(text: &str) -> Result<Position, String> {
        let mut chars = text.chars();

        let letter = match chars
            .next()
            .and_then(|c| u8::try_from(c).ok())
            .map(|byte| byte.to_ascii_uppercase())
        {
            Some(byte) if byte.is_ascii_uppercase() => byte,
            _ => {
                return Err(format!(
                    "« {} » : il faut une lettre puis un nombre, par exemple C5",
                    text
                ));
            }
        };

        // `parse` accepterait un « + » au début : on exige des chiffres seulement.
        let digits = chars.as_str();
        let only_digits = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
        let number: usize = match digits.parse() {
            Ok(n) if only_digits && n >= 1 => n,
            _ => {
                return Err(format!(
                    "« {} » : le nombre après la lettre est invalide",
                    text
                ));
            }
        };

        // `letter` est une majuscule ASCII : l'écart avec 'A' tient entre 0 et 25.
        let col = usize::from(letter - b'A');
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
        assert!("é3".parse::<Position>().is_err());
        assert!("C+5".parse::<Position>().is_err());
        assert!("C 5".parse::<Position>().is_err());
        assert!("".parse::<Position>().is_err());
    }

    #[test]
    fn display_and_parse_agree() {
        let pos = Position { row: 4, col: 2 };
        assert_eq!(pos.to_string().parse(), Ok(pos));
    }

    #[test]
    fn column_letters_go_from_a_to_z() {
        assert_eq!(Position::column_letter(0), Some('A'));
        assert_eq!(Position::column_letter(25), Some('Z'));
        assert_eq!(Position::column_letter(26), None);
        assert_eq!(Position::column_letter(usize::MAX), None);
    }

    #[test]
    fn unnamable_position_displays_without_panicking() {
        let far = Position {
            row: usize::MAX,
            col: 200,
        };
        assert_eq!(
            far.to_string(),
            format!("(rangée {}, colonne 200, comptées depuis 0)", usize::MAX)
        );
    }

    #[test]
    fn step_returns_none_instead_of_overflowing() {
        let corner = Position { row: 0, col: 0 };
        assert_eq!(corner.step(Direction::N), None);
        let far = Position {
            row: usize::MAX,
            col: 0,
        };
        assert_eq!(far.step(Direction::S), None);
        assert_eq!(
            corner.step(Direction::SE),
            Some(Position { row: 1, col: 1 })
        );
    }
}
