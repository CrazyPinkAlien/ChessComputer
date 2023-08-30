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
            algebraic.push_str("x");
        }
        algebraic.push_str(match self.to.file {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            6 => "g",
            7 => "h",
            _ => panic!("Unexpected rank for moved piece: {}", self.to.file),
        });
        algebraic += &(8 - self.to.rank).to_string();
        algebraic
    }

    pub fn from(&self) -> BoardPosition {
        self.from
    }

    pub fn to(&self) -> BoardPosition {
        self.to
    }
}
