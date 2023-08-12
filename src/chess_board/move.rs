use super::BoardPosition;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Move {
    from: BoardPosition,
    to: BoardPosition,
}

impl Move {
    pub fn new(from: BoardPosition, to: BoardPosition) -> Self {
        Move { from, to }
    }

    pub fn from(&self) -> BoardPosition {
        self.from
    }

    pub fn to(&self) -> BoardPosition {
        self.to
    }
}
