use bevy::{ecs::component::TableStorage, prelude::Component};
use dyn_clone::DynClone;

use super::{BoardPosition, Move, PieceColor, PieceType};

mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
mod rook;

dyn_clone::clone_trait_object!(Piece);

pub(super) trait Piece:
    Send + Sync + DynClone + 'static + Component<Storage = TableStorage>
{
    // TODO some of these functions should return references
    fn get_type(&self) -> PieceType;
    fn get_color(&self) -> PieceColor;
    fn get_position(&self) -> BoardPosition;
    fn get_starting_position(&self) -> BoardPosition;
    fn set_position(&mut self, new_position: BoardPosition, moved: bool);
    fn get_moves(&self, include_captures: bool) -> Vec<Move>;
    fn valid_move(&self, end_position: BoardPosition) -> bool;
    fn valid_capture(&self, end_position: BoardPosition) -> bool;
    fn is_sliding(&self) -> bool;
}

pub(super) fn new_piece(
    piece_color: PieceColor,
    piece_type: PieceType,
    position: BoardPosition,
) -> Box<dyn Piece> {
    let piece: Box<dyn Piece> = match piece_type {
        PieceType::Pawn => pawn::Pawn::new(position, piece_color),
        PieceType::King => king::King::new(position, piece_color),
        PieceType::Queen => queen::Queen::new(position, piece_color),
        PieceType::Bishop => bishop::Bishop::new(position, piece_color),
        PieceType::Knight => knight::Knight::new(position, piece_color),
        PieceType::Rook => rook::Rook::new(position, piece_color),
    };
    piece
}
