use bevy::prelude::Component;

use crate::chess_board::{r#move::Move, BoardPosition};

use super::{Piece, PieceColor, PieceType};

#[derive(Component, Clone, Debug)]
pub(super) struct King {
    color: PieceColor,
    starting_position: BoardPosition,
    position: BoardPosition,
    moved: bool,
}

impl King {
    pub(super) fn new(position: BoardPosition, color: PieceColor) -> Box<Self> {
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

    fn get_moves(&self, _include_captures: bool) -> Vec<Move> {
        let mut moves = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                if (rank == self.position.rank && self.position.file.abs_diff(file) == 1)
                    || (file == self.position.file && self.position.rank.abs_diff(rank) == 1)
                    || (self.position.file.abs_diff(file) == 1
                        && self.position.rank.abs_diff(rank) == 1)
                {
                    moves.push(Move::new(self.get_position(), BoardPosition { rank, file }));
                }
            }
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
        self.valid_move(end_position)
    }
}
