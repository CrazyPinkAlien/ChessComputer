use bevy::prelude::Component;

use crate::core::board::BoardPosition;

use super::{Piece, PieceType, PieceColor};

#[derive(Component, Clone)]
pub struct Pawn {
    color: PieceColor,
    position: BoardPosition,
    moved: bool,
}

impl Pawn {
    pub fn new(position: BoardPosition, color: PieceColor) -> Box<Self> {
        Box::new(Pawn { color, position, moved: false })
    }
}

impl Piece for Pawn {
    fn get_type(&self) -> PieceType {
        PieceType::Pawn
    }

    fn get_color(&self) -> PieceColor {
        self.color
    }

    fn get_position(&self) -> BoardPosition {
        self.position
    }

    fn set_position(&mut self, new_position: BoardPosition) {
        self.position = new_position;
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
