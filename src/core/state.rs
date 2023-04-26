use bevy::prelude::{FromWorld, Resource};

use super::board::BoardPosition;
use super::fen::Fen;
use super::piece::{PieceColor, PieceType};

#[derive(Resource, Copy, Clone, Debug)]
pub struct BoardState {
    pub board: [[Option<(PieceColor, PieceType)>; 8]; 8]
}

impl FromWorld for BoardState {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        BoardState::empty_board()
    }
}

impl BoardState {
    fn empty_board() -> Self {
        let board = [[None; 8]; 8];
        BoardState { board }
    }

    pub fn from_fen(fen: Fen) -> Self {
        // Create an empty board state
        let mut board_state = BoardState::empty_board();
        // Populate it from the given fen
        let mut rank = 0;
        let mut file = 0;
        for rank_str in fen.piece_placement.split("/") {
            for symbol in rank_str.chars().collect::<Vec<char>>() {
                if symbol.is_digit(9) {
                    file += symbol.to_digit(9).unwrap() as usize;
                } else {
                    let piece_color = if symbol.is_uppercase() {
                        PieceColor::White
                    } else {
                        PieceColor::Black
                    };
                    let piece_type = match symbol.to_uppercase().next().unwrap() {
                        'P' => PieceType::Pawn,
                        'N' => PieceType::Knight,
                        'B' => PieceType::Bishop,
                        'R' => PieceType::Rook,
                        'Q' => PieceType::Queen,
                        'K' => PieceType::King,
                        _ => panic!("Unrecognised symbol in FEN: {}", symbol),
                    };
                    board_state.add_piece(
                        piece_color,
                        piece_type,
                        BoardPosition::new(rank, file),
                    );
                    file += 1;
                }
                if file >= 8 {
                    rank += 1;
                    file = 0;
                };
            }
        }
        board_state
    }

    fn add_piece(&mut self, piece_color: PieceColor, piece_type: PieceType, position: BoardPosition) {
        self.board[position.rank][position.file] = Some((piece_color, piece_type));
    }

    pub fn remove_piece(&mut self, position: BoardPosition) {
        self.board[position.rank][position.file] = None;
    }

    pub fn move_piece(&mut self, position: BoardPosition, new_position: BoardPosition) {
        self.board[new_position.rank][new_position.file] = self.board[position.rank][position.file];
        self.remove_piece(position);
    }

}
