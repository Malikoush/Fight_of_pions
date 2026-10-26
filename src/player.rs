
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub enum Player {
        One,
        Two,
    }

        impl Player {
        pub fn opponent(self) -> Player{
            match self {
                Player::One => Player::Two,
                Player::Two => Player::One,

            }
        }
        pub fn index(self) -> usize {
            match self {
               Player::One => 0,
               Player::Two => 1, 
            }
        }
    }