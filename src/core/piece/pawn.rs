use bevy::prelude::Component;

use crate::core::board::BoardPosition;

use super::{Piece, PieceColor, PieceType};

#[derive(Component, Clone, Debug)]
pub struct Pawn {
    color: PieceColor,
    starting_position: BoardPosition,
    position: BoardPosition,
    moved: bool,
}

impl Pawn {
    pub fn new(position: BoardPosition, color: PieceColor) -> Box<Self> {
        Box::new(Pawn {
            color,
            starting_position: position,
            position,
            moved: false,
        })
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
        // Can move forward 1
        moves.push(BoardPosition::new(
            (self.position.rank as i32 + self.move_direction()).clamp(0, 7) as usize,
            self.position.file,
        ));
        if !self.moved {
            // Can move forward 2
            moves.push(BoardPosition::new(
                (self.position.rank as i32 + 2 * self.move_direction()).clamp(0, 7) as usize,
                self.position.file,
            ));
        }
        moves
    }

    fn possible_move(&self, new_position: BoardPosition) -> bool {
        let valid_moves = self.get_moves();
        valid_moves.contains(&new_position)
    }

    fn possible_capture(&self, new_position: BoardPosition) -> bool {
        if (0 <= self.position.rank as i32 + self.move_direction())
            && (self.position.rank as i32 + self.move_direction() < 8)
            && (self.position.rank as i32 + self.move_direction() == new_position.rank as i32)
            && (((self.position.file > 0) && (new_position.file == self.position.file - 1))
                || ((self.position.file < 7) && (new_position.file == self.position.file + 1)))
        {
            return true;
        }
        false
    }

    fn is_sliding(&self) -> bool {
        true
    }
}

impl Pawn {
    fn move_direction(&self) -> i32 {
        match self.color {
            PieceColor::White => -1,
            PieceColor::Black => 1,
        }
    }
}
