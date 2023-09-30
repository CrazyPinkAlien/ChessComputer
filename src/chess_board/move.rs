use bevy::prelude::Res;

use super::{BoardPosition, ChessBoard, PieceType};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Move {
    from: BoardPosition,
    to: BoardPosition,
    piece_type: PieceType,
    is_capture: bool,
}

impl Move {
    pub fn new_from_board(from: BoardPosition, to: BoardPosition, board: &Res<ChessBoard>) -> Self {
        Move {
            from,
            to,
            piece_type: board.get_piece_type(&from).unwrap(),
            is_capture: board.get_piece_type(&to).is_some(),
        }
    }

    pub fn new(
        from: BoardPosition,
        to: BoardPosition,
        piece_type: PieceType,
        is_capture: bool,
    ) -> Self {
        Move {
            from,
            to,
            piece_type,
            is_capture,
        }
    }

    pub fn from(&self) -> &BoardPosition {
        &self.from
    }

    pub fn to(&self) -> &BoardPosition {
        &self.to
    }

    pub fn as_algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push_str(match self.piece_type {
            PieceType::King => "K",
            PieceType::Queen => "Q",
            PieceType::Bishop => "B",
            PieceType::Knight => "N",
            PieceType::Rook => "R",
            PieceType::Pawn => "",
        });
        if self.is_capture {
            if self.piece_type == PieceType::Pawn {
                algebraic.push_str(&Self::file_to_string(self.from.file));
            }
            algebraic.push('x');
        }
        algebraic.push_str(&Self::file_to_string(self.to.file));
        algebraic += &(8 - self.to.rank).to_string();
        algebraic
    }

    fn file_to_string(file: usize) -> String {
        match file {
            0 => "a".to_string(),
            1 => "b".to_string(),
            2 => "c".to_string(),
            3 => "d".to_string(),
            4 => "e".to_string(),
            5 => "f".to_string(),
            6 => "g".to_string(),
            7 => "h".to_string(),
            _ => panic!("Unexpected file for moved piece: {}", file),
        }
    }
}
