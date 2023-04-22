use bevy::prelude::Component;

use crate::core::board::BoardPosition;

use super::Piece;

#[derive(Component)]
pub struct Pawn {
    position: BoardPosition,
    moved: bool,
}

impl Piece for Pawn {
    fn new() -> Self {
        Pawn { position: BoardPosition::new(0, 0), moved: false }
    }

    fn get_moves(&self) -> Vec<BoardPosition> {
        let mut moves = Vec::new();
        // Can move forward 1
        moves.push(BoardPosition::new(self.position.rank, self.position.file + 1));
        if !self.moved {
            // Can move forward 2
            moves.push(BoardPosition::new(self.position.rank, self.position.file + 2));
        }
        moves
    }
}
