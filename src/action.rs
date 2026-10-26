use crate::position::Position;


#[derive(Debug,Clone, Copy,PartialEq, Eq)]
pub enum Action {
    Place(Position),
    EndTurn,
}

impl Action {
    pub fn cost (&self) -> u32{
        match self {
            Action::EndTurn => 0,
            Action::Place(_)=>1,

        }
    }
}