use bevy::prelude::Res;

use super::{board::BoardPosition, piece::PieceInfo, state::BoardState};

pub fn is_legal_move(
    piece: &PieceInfo,
    new_position: BoardPosition,
    state: &Res<BoardState>,
) -> bool {
    is_active_color(piece, state)
        && (can_capture(piece, new_position, state)
        || piece.piece.possible_move(new_position))
        && no_friendly_piece(piece, new_position, state)
        && no_piece_in_the_way(piece, new_position, state)
        && wont_cause_check(piece, new_position, state)
}

fn is_active_color(piece: &PieceInfo, state: &Res<BoardState>) -> bool {
    piece.piece.get_color() == state.active_color
}

fn no_friendly_piece(
    piece: &PieceInfo,
    new_position: BoardPosition,
    state: &Res<BoardState>,
) -> bool {
    state.board[new_position.rank][new_position.file].is_none()
        || (state.board[new_position.rank][new_position.file].unwrap().0 != piece.piece.get_color())
}

fn can_capture(piece: &PieceInfo, new_position: BoardPosition, state: &Res<BoardState>) -> bool {
    state.board[new_position.rank][new_position.file].is_none()
        || piece.piece.possible_capture(new_position)
}

fn no_piece_in_the_way(
    piece: &PieceInfo,
    new_position: BoardPosition,
    state: &Res<BoardState>,
) -> bool {
    if !piece.piece.is_sliding() {
        return true;
    }
    let position = piece.piece.get_position();
    let mut rank = position.rank as i32;
    let mut file = position.file as i32;
    let move_direction = [(new_position.rank as i32 - position.rank as i32).clamp(-1, 1), (new_position.file as i32 - position.file as i32).clamp(-1, 1)];
    while (rank != new_position.rank as i32 - move_direction[0]) || (file != new_position.file as i32 - move_direction[1]) {
        rank += move_direction[0];
        file += move_direction[1];
        if state.board[rank as usize][file as usize].is_some() {
            return false;
        }
    }
    true
}

fn wont_cause_check(
    piece: &PieceInfo,
    new_position: BoardPosition,
    state: &Res<BoardState>,
) -> bool {
    let mut new_state: BoardState = **state.clone();
    new_state.move_piece(piece.piece.get_position(), new_position);
    !new_state.is_check(piece.piece.get_color())
}
