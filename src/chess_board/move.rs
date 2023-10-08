use super::{BoardPosition, ChessBoard, PieceColor, PieceType};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Move {
    pub(super) from: BoardPosition,
    pub(super) to: BoardPosition,
    pub(super) piece_type: PieceType,
    pub(super) piece_color: PieceColor,
    pub(super) is_capture: bool,
    pub(super) is_castle: bool,
}

impl Move {
    pub fn from_board(from: BoardPosition, to: BoardPosition, board: &ChessBoard) -> Self {
        Move {
            from,
            to,
            piece_type: board.get_piece_type(&from).expect("No piece found."),
            piece_color: board.get_piece_color(&from).unwrap(),
            is_capture: board.get_piece_type(&to).is_some(),
            is_castle: board.get_piece_type(&from).unwrap() == PieceType::King
                && from.file.abs_diff(to.file) == 2,
        }
    }

    pub fn from(&self) -> &BoardPosition {
        &self.from
    }

    pub fn to(&self) -> &BoardPosition {
        &self.to
    }

    pub fn piece_type(&self) -> &PieceType {
        &self.piece_type
    }

    pub fn piece_color(&self) -> &PieceColor {
        &self.piece_color
    }

    pub fn is_castle(&self) -> bool {
        self.is_castle
    }

    pub fn is_capture(&self) -> bool {
        self.is_capture
    }

    pub fn as_algebraic(&self) -> String {
        if self.is_castle {
            match (self.to.file as i32 - self.from.file as i32).signum() {
                1 => "0-0".to_string(),
                -1 => "0-0-0".to_string(),
                _ => panic!("Invalid castle from {:?} to {:?}.", self.from, self.to),
            }
        } else {
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
            _ => panic!("Unexpected file for moved piece: {}.", file),
        }
    }
}
