// FILE TO BE DELETED

use bevy::prelude::Res;

use super::{BoardPosition, piece::PieceInfo, ChessBoard};

struct Move {
    from: BoardPosition,
    to: BoardPosition
}

impl Move {
    fn is_valid(&self, piece: &PieceInfo, board: &Res<ChessBoard>) -> bool {
        self.is_active_color(piece, board)
        && self.no_friendly_piece(piece, board)
        && ((!board.any_piece_is_here(self.to)
            && piece.piece.valid_move(self))
            || (board.any_piece_is_here(self.to)
                && piece.piece.valid_capture(self)))
        && self.no_piece_in_the_way(piece, board)
        && self.wont_cause_check(piece, board)
    }

    fn is_active_color(&self, piece: &PieceInfo, board: &Res<ChessBoard>) -> bool {
        piece.piece.get_color() == board.active_color
    }
    
    fn no_friendly_piece(
        &self,
        piece: &PieceInfo,
        board: &Res<ChessBoard>,
    ) -> bool {
        !board.any_piece_is_here(self.to)
            || (board.get_piece(self.to).piece.get_color() != piece.piece.get_color())
    }
    
    fn no_piece_in_the_way(
        &self,
        piece: &PieceInfo,
        board: &Res<ChessBoard>,
    ) -> bool {
        if !piece.piece.is_sliding() {
            return true;
        }
        let position = piece.piece.get_position();
        let mut rank = position.rank as i32;
        let mut file = position.file as i32;
        let move_direction = [
            (self.to.rank as i32 - position.rank as i32).clamp(-1, 1),
            (self.to.file as i32 - position.file as i32).clamp(-1, 1),
        ];
        while (rank != self.to.rank as i32 - move_direction[0])
            || (file != self.to.file as i32 - move_direction[1])
        {
            rank += move_direction[0];
            file += move_direction[1];
            if board.board[rank as usize][file as usize].is_some() {
                return false;
            }
        }
        true
    }
    
    fn wont_cause_check(
        &self,
        piece: &PieceInfo,
        board: &Res<ChessBoard>,
    ) -> bool {
        // TODO
        true
    }
}
