use bevy::prelude::Component;

use crate::chess_board::{r#move::Move, BoardPosition};

use super::{Piece, PieceColor, PieceType};

#[derive(Component, Clone, Debug)]
pub(super) struct Pawn {
    color: PieceColor,
    starting_position: BoardPosition,
    position: BoardPosition,
    moved: bool,
}

impl Pawn {
    pub(super) fn new(position: BoardPosition, color: PieceColor) -> Box<Self> {
        Box::new(Pawn {
            color,
            starting_position: position,
            position,
            moved: false,
        })
    }

    fn move_direction(&self) -> i32 {
        match self.color {
            PieceColor::White => -1,
            PieceColor::Black => 1,
        }
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

    fn get_moves(&self, include_captures: bool) -> Vec<Move> {
        let mut moves = Vec::new();
        // Can move forward 1
        moves.push(Move::new(
            self.get_position(),
            BoardPosition::new(
                (self.position.rank as i32 + self.move_direction()).clamp(0, 7) as usize,
                self.position.file,
            ),
        ));
        if ((self.color == PieceColor::White) && (self.position.rank() == 6))
            || ((self.color == PieceColor::Black) && (self.position.rank() == 1))
        {
            // Can move forward 2
            moves.push(Move::new(
                self.get_position(),
                BoardPosition::new(
                    (self.position.rank as i32 + 2 * self.move_direction()).clamp(0, 7) as usize,
                    self.position.file,
                ),
            ));
        }
        if include_captures {
            moves.push(Move::new(
                self.position,
                BoardPosition::new(
                    (self.position.rank as i32 + 1 * self.move_direction()).clamp(0, 7) as usize,
                    (self.position.file as i32 + 1).clamp(0, 7) as usize,
                ))
            );
            moves.push(Move::new(
                self.position,
                BoardPosition::new(
                    (self.position.rank as i32 + 1 * self.move_direction()).clamp(0, 7) as usize,
                    (self.position.file as i32 - 1).clamp(0, 7) as usize,
                ))
            );
        }
        moves
    }

    fn is_sliding(&self) -> bool {
        true
    }

    fn get_starting_position(&self) -> BoardPosition {
        self.starting_position
    }

    fn valid_move(&self, end_position: BoardPosition) -> bool {
        let valid_moves = self.get_moves(false);
        valid_moves.contains(&Move::new(self.get_position(), end_position))
    }

    fn valid_capture(&self, end_position: BoardPosition) -> bool {
        if (0 <= self.position.rank as i32 + self.move_direction())
            && (self.position.rank as i32 + self.move_direction() < 8)
            && (self.position.rank as i32 + self.move_direction() == end_position.rank as i32)
            && (((self.position.file > 0) && (end_position.file == self.position.file - 1))
                || ((self.position.file < 7) && (end_position.file == self.position.file + 1)))
        {
            return true;
        }
        false
    }
}
