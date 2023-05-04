use bevy::prelude::Component;

use crate::core::board::BoardPosition;

use super::{Piece, PieceColor, PieceType};

#[derive(Component, Clone, Debug)]
pub struct Knight {
    color: PieceColor,
    starting_position: BoardPosition,
    position: BoardPosition,
}

impl Knight {
    pub fn new(position: BoardPosition, color: PieceColor) -> Box<Self> {
        Box::new(Knight {
            color,
            starting_position: position,
            position,
        })
    }
}

impl Piece for Knight {
    fn get_type(&self) -> PieceType {
        PieceType::Knight
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

    fn reset(&mut self) {
        self.position = self.starting_position;
    }

    fn get_moves(&self) -> Vec<BoardPosition> {
        let mut moves = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                let rank_diff = rank as i32 - self.position.rank as i32;
                let file_diff = file as i32 - self.position.file as i32;
                if (((rank_diff == 1) || (rank_diff == -1))
                    && ((file_diff == 2) || (file_diff == -2)))
                    || (((file_diff == 1) || (file_diff == -1))
                        && ((rank_diff == 2) || (rank_diff == -2)))
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
        false
    }
}
