use std::fmt;

    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct Position {
        pub row: usize,
        pub col: usize,
    }
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct Direction{
        pub d_row: isize,
        pub d_col: isize,
    }

    impl Direction {
        pub const N: Direction = Direction{d_row:-1 , d_col: 0};
        pub const NE: Direction = Direction{d_row:-1 , d_col: 1};
        pub const E: Direction = Direction{d_row:0 , d_col: 1};
        pub const SE: Direction = Direction{d_row:1 , d_col: 1};
        pub const S: Direction = Direction{d_row:1 , d_col: 0};
        pub const SW: Direction = Direction{d_row:1 , d_col: -1};
        pub const W: Direction = Direction{d_row:0 , d_col: -1};
        pub const NW: Direction = Direction{d_row:-1 , d_col: -1};
       
       pub const ALL: [Direction;8] = [Direction::E,Direction::N,Direction::NE,Direction::NW,Direction::S,Direction::SE,Direction::SW,Direction::W];
       pub const AXES : [Direction;4] = [Direction::E,Direction::S,Direction::SE,Direction::SW ];
    
    pub fn opposite(self) -> Direction {
        Direction { 
            d_row: self.d_row * -1,
             d_col: self.d_col *-1,
            }
        
    }
    
    }

    impl Position {
        pub fn step(self,dir:Direction) -> Option<Position> {
            let row = self.row as isize + dir.d_row;
            let col= self.col as isize + dir.d_col;
            if row >= 0 && col >= 0 {
                return Some(Position { row: row as usize, col: col as usize });
            }

            None
            
        }
        pub fn isadjacent(self,other :Position) -> bool{
            let ecart_l =self.row.abs_diff(other.row);
            let ecart_c =self.col.abs_diff(other.col);
            
            ecart_c <= 1 && ecart_l <= 1 && (
            ecart_c == 1 || ecart_l == 1 )
            
            
            
        }
    }

    impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "{}{}",(b'A' + self.col as u8) as char,self.row +1)
            
        
    }
}