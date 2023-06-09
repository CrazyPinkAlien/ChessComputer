use bevy::prelude::Component;

use crate::chess_board::{r#move::Move, BoardPosition};

use super::{Piece, PieceColor, PieceType};

#[derive(Component, Clone, Debug)]
pub(super) struct Bishop {
    color: PieceColor,
    starting_position: BoardPosition,
    position: BoardPosition,
}

impl Bishop {
    pub(super) fn new(position: BoardPosition, color: PieceColor) -> Box<Self> {
        Box::new(Bishop {
            color,
            starting_position: position,
            position,
        })
    }
}

impl Piece for Bishop {
    fn get_type(&self) -> PieceType {
        PieceType::Bishop
    }

    fn get_color(&self) -> PieceColor {
        self.color
    }

    fn get_position(&self) -> BoardPosition {
        self.position
    }

    fn set_position(&mut self, new_position: BoardPosition, _moved: bool) {
        self.position = new_position;
    }

    fn get_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                let rank_diff = rank as i32 - self.position.rank as i32;
                let file_diff = file as i32 - self.position.file as i32;
                if (rank_diff == file_diff || rank_diff == -file_diff) && rank != self.position.rank
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

    fn get_starting_position(&self) -> crate::chess_board::BoardPosition {
        self.starting_position
    }

    fn valid_move(&self, end_position: crate::chess_board::BoardPosition) -> bool {
        let valid_moves = self.get_moves();
        valid_moves.contains(&Move::new(self.get_position(), end_position))
    }

    fn valid_capture(&self, end_position: crate::chess_board::BoardPosition) -> bool {
        self.valid_move(end_position)
    }
}
