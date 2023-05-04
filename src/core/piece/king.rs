use bevy::prelude::Component;

use crate::core::board::BoardPosition;

use super::{Piece, PieceColor, PieceType};

#[derive(Component, Clone, Debug)]
pub struct King {
    color: PieceColor,
    starting_position: BoardPosition,
    position: BoardPosition,
    moved: bool,
}

impl King {
    pub fn new(position: BoardPosition, color: PieceColor) -> Box<Self> {
        Box::new(King {
            color,
            starting_position: position,
            position,
            moved: false,
        })
    }
}

impl Piece for King {
    fn get_type(&self) -> PieceType {
        PieceType::King
    }

    fn get_color(&self) -> PieceColor {
        self.color
    }

    fn get_position(&self) -> BoardPosition {
        self.position
    }

    fn set_position(&mut self, new_position: BoardPosition, moved: bool) {
        self.position = new_position;
        if moved {
            self.moved = true;
        }
    }

    fn reset(&mut self) {
        self.position = self.starting_position;
        self.moved = false;
    }

    fn get_moves(&self) -> Vec<BoardPosition> {
        let mut moves = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                if (rank == self.position.rank && self.position.file.abs_diff(file) == 1)
                    || (file == self.position.file && self.position.rank.abs_diff(rank) == 1)
                    || (self.position.file.abs_diff(file) == 1
                        && self.position.rank.abs_diff(rank) == 1)
                {
                    moves.push(BoardPosition::new(rank, file));
                }
            }
        }
        moves
    }

    fn possible_move(&self, new_position: BoardPosition) -> bool {
        let valid_moves = self.get_moves();
        valid_moves.contains(&new_position)
    }

    fn possible_capture(&self, new_position: BoardPosition) -> bool {
        self.possible_move(new_position)
    }

    fn is_sliding(&self) -> bool {
        true
    }
}
