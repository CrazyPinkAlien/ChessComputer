use crate::chess_board::{r#move::Move, PieceColor, PieceType};

#[derive(Debug, Clone, Default, PartialEq, Eq, Copy)]
pub struct CastlingRights {
    pub white: [bool; 2],
    pub black: [bool; 2],
}

impl CastlingRights {
    pub fn from_fen_string(fen: &str) -> Self {
        Self {
            white: [fen.contains('K'), fen.contains('Q')],
            black: [fen.contains('k'), fen.contains('q')],
        }
    }

    pub fn valid_castle_direction(&self, color: &PieceColor, direction: i32) -> bool {
        let rights = match *color {
            PieceColor::White => &self.white,
            PieceColor::Black => &self.black,
        };
        match direction.signum() {
            1 => rights[0],
            -1 => rights[1],
            0 => false,
            _ => panic!("Unexpected castle direction: {}", direction),
        }
    }

    pub fn update_after_move(&mut self, piece_move: &Move) {
        let rights = match piece_move.piece_color() {
            PieceColor::White => &mut self.white,
            PieceColor::Black => &mut self.black,
        };
        if *piece_move.piece_type() == PieceType::King {
            *rights = [false; 2]
        } else if *piece_move.piece_type() == PieceType::Rook {
            if *piece_move.from().file() == 0 {
                rights[1] = false
            } else if *piece_move.from().file() == 7 {
                rights[0] = false
            }
        }
    }
}
