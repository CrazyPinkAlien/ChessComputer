use bevy::prelude::{FromWorld, Resource};

use super::board::BoardPosition;
use super::fen::Fen;
use super::piece::{PieceInfo, PieceColor, PieceType};

#[derive(Resource)]
pub struct BoardState {
    pub board: Vec<Vec<Option<PieceInfo>>>,
}

impl FromWorld for BoardState {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        BoardState::empty_board()
    }
}

impl BoardState {
    fn empty_board() -> Self {
        // Row of the array
        let mut row = Vec::new();
        row.resize(8, None);
        // Fill board with these rows
        let mut board = Vec::new();
        board.resize(8, row.clone());
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
                    file += symbol.to_digit(9).unwrap();
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
                        PieceInfo::new(piece_color, piece_type),
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

    fn add_piece(&mut self, piece_info: PieceInfo, position: BoardPosition) {
        let rank = position.rank as usize;
        let file = position.file as usize;
        self.board[rank][file] = Some(piece_info);
    }

    fn remove_piece(&mut self, position: BoardPosition) {
        let rank = position.rank as usize;
        let file = position.file as usize;
        self.board[rank][file] = None;
    }

    pub fn move_piece(
        &mut self,
        piece_info: PieceInfo,
        old_position: BoardPosition,
        new_position: BoardPosition,
    ) {
        self.remove_piece(old_position);
        self.add_piece(piece_info, new_position);
    }
}
