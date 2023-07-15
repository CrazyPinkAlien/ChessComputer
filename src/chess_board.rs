use bevy::app::App;
use bevy::prelude::{info, warn, Component, EventReader, EventWriter, Plugin, ResMut, Resource};
use strum_macros::EnumIter;

use crate::fen::Fen;

use self::r#move::Move;

pub(super) mod r#move;
mod piece;

static BOARD_SIZE: usize = 8;

pub(super) struct ChessBoardPlugin;

impl Plugin for ChessBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetBoardEvent>()
            .add_event::<PieceMoveEvent>()
            .add_event::<PieceCreateEvent>()
            .add_event::<PieceDestroyEvent>()
            .init_resource::<ChessBoard>()
            .add_startup_system(setup)
            .add_system(make_move)
            .add_system(reset_board_state);
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Component)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Component, PartialEq, Debug, Copy, Clone, Eq)]
pub struct BoardPosition {
    rank: usize,
    file: usize,
}

impl BoardPosition {
    pub fn new(rank: usize, file: usize) -> Self {
        if (rank >= BOARD_SIZE) | (file >= BOARD_SIZE) {
            panic!("Invalid rank or file value: {}, {}", rank, file)
        }
        BoardPosition { rank, file }
    }

    pub fn rank(&self) -> usize {
        self.rank
    }

    pub fn file(&self) -> usize {
        self.file
    }
}

#[derive(Debug, Clone)]
pub struct ResetBoardEvent {
    fen: Fen,
}

impl ResetBoardEvent {
    pub fn new(fen: Fen) -> Self {
        ResetBoardEvent { fen }
    }

    pub fn fen(&self) -> &Fen {
        &self.fen
    }
}

pub struct PieceMoveEvent {
    piece_move: Move,
}

impl PieceMoveEvent {
    pub fn new(piece_move: Move) -> Self {
        PieceMoveEvent { piece_move }
    }

    pub fn piece_move(&self) -> &Move {
        &self.piece_move
    }
}

pub struct PieceCreateEvent {
    position: BoardPosition,
    piece_type: PieceType,
    color: PieceColor,
}

impl PieceCreateEvent {
    pub fn position(&self) -> &BoardPosition {
        &self.position
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn color(&self) -> PieceColor {
        self.color
    }
}

pub struct PieceDestroyEvent {
    position: BoardPosition,
}

impl PieceDestroyEvent {
    pub fn position(&self) -> &BoardPosition {
        &self.position
    }
}

#[derive(Resource, Clone)]
pub struct ChessBoard {
    board: [[Option<Box<dyn piece::Piece>>; 8]; 8],
    active_color: PieceColor,
}

impl Default for ChessBoard {
    fn default() -> Self {
        ChessBoard::empty_board()
    }
}

impl ChessBoard {
    fn empty_board() -> Self {
        let board: [[Option<Box<dyn piece::Piece>>; 8]; 8] = Default::default();
        ChessBoard {
            board,
            active_color: PieceColor::White,
        }
    }

    fn from_fen(fen: &Fen, create_event: &mut EventWriter<PieceCreateEvent>) -> Self {
        // Create an empty board state
        let mut board_state = ChessBoard::empty_board();
        // Populate it from the given fen
        let mut rank = 0;
        let mut file = 0;
        for rank_str in fen.piece_placement().split('/') {
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
                        create_event,
                    );
                    file += 1;
                }
                if file >= 8 {
                    rank += 1;
                    file = 0;
                };
            }
        }
        // Set active color
        board_state.active_color = match fen.active_color().as_str() {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => panic!("Unrecognised active color in FEN: {}", fen.active_color()),
        };
        board_state
    }

    pub fn active_color(&self) -> PieceColor {
        self.active_color
    }

    pub fn valid_move(&self, piece_move: Move) -> bool {
        // Get piece
        if self.board[piece_move.from().rank][piece_move.from().file].is_none() {
            return false;
        }
        let piece = self.board[piece_move.from().rank][piece_move.from().file]
            .as_ref()
            .unwrap();

        // Check that the piece is the active colour
        (piece.get_color() == self.active_color)
        // Check whether or not there are any pieces there
        && match self.get_piece_color(piece_move.to()) {
            Some(color) => if color == piece.get_color() {
                // If a friendly piece is here this move is invalid
                false
            } else {
                // If an enemy piece is here the move must be a valid capture
                piece.valid_capture(piece_move.to())
            }
            // If no piece is here the move must be a valid move
            None => piece.valid_move(piece_move.to())
        }
        // No piece in the way for sliding pieces
        && (!piece.is_sliding() || self.no_piece_along_line(&piece_move.from(), &piece_move.to()))
        // Finally, the move must not put the active color in check
        && {
                let mut test_board = self.clone();
                test_board.move_piece(piece_move);
                let check_status = test_board.in_check();
                check_status.is_none() || check_status.unwrap() != piece.get_color()
            }
    }

    fn get_valid_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if self.board[rank][file].is_some() {
                    let piece = &self.board[rank][file].as_ref().unwrap();
                    let piece_moves = piece.get_moves();
                    for piece_move in piece_moves {
                        if self.valid_move(piece_move) {
                            moves.push(piece_move);
                        }
                    }
                }
            }
        }
        moves
    }

    fn add_piece(
        &mut self,
        piece_color: PieceColor,
        piece_type: PieceType,
        position: BoardPosition,
        create_event: &mut EventWriter<PieceCreateEvent>,
    ) {
        let new_piece = piece::new_piece(piece_color, piece_type, position);
        self.board[position.rank][position.file] = Some(new_piece);
        create_event.send(PieceCreateEvent {
            position,
            piece_type,
            color: piece_color,
        });
    }

    fn remove_piece(
        &mut self,
        remove_position: BoardPosition,
        events: &mut EventWriter<PieceDestroyEvent>,
    ) {
        self.board[remove_position.rank][remove_position.file] = None;
        events.send(PieceDestroyEvent {
            position: remove_position,
        });
    }

    fn move_piece(&mut self, piece_move: Move) {
        if self.board[piece_move.from().rank][piece_move.from().file].is_none() {
            warn!("No piece moved.");
            return;
        }
        self.board[piece_move.from().rank][piece_move.from().file]
            .as_mut()
            .unwrap()
            .set_position(piece_move.to(), true);
        self.board[piece_move.to().rank][piece_move.to().file] =
            self.board[piece_move.from().rank][piece_move.from().file].clone();
        self.board[piece_move.from().rank][piece_move.from().file] = None;
    }

    fn get_piece_color(&self, position: BoardPosition) -> Option<PieceColor> {
        self.board[position.rank][position.file]
            .as_ref()
            .map(|piece| piece.get_color())
    }

    fn in_check(&self) -> Option<PieceColor> {
        // Get king locations
        let mut white_king_location = BoardPosition::new(0, 0);
        let mut black_king_location = BoardPosition::new(0, 0);
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if self.board[rank][file].is_some()
                    && self.board[rank][file].as_ref().unwrap().get_type() == PieceType::King
                {
                    match self.board[rank][file].as_ref().unwrap().get_color() {
                        PieceColor::White => white_king_location = BoardPosition::new(rank, file),
                        PieceColor::Black => black_king_location = BoardPosition::new(rank, file),
                    }
                }
            }
        }
        // Get valid moves
        let moves = self.get_valid_moves();
        // Check if any valid moves can take the king
        for piece_move in moves {
            if piece_move.to() == white_king_location {
                return Some(PieceColor::White);
            } else if piece_move.to() == black_king_location {
                return Some(PieceColor::Black);
            }
        }
        None
    }

    fn no_piece_along_line(&self, start: &BoardPosition, end: &BoardPosition) -> bool {
        let mut rank = start.rank() as i32;
        let mut file = start.file() as i32;
        while rank as usize != end.rank() || file as usize != end.file() {
            rank += end.rank() as i32 - start.rank() as i32;
            file += end.file() as i32 - start.file() as i32;
            if self.board[rank as usize][file as usize].is_some() {
                return false;
            }
        }
        true
    }
}

fn setup(mut create_event: EventWriter<PieceCreateEvent>, mut board: ResMut<ChessBoard>) {
    *board = ChessBoard::from_fen(
        &Fen::from_file("assets/fens/starting_position.fen"),
        &mut create_event,
    );
}

fn make_move(
    mut move_events: EventReader<PieceMoveEvent>,
    mut destroy_events: EventWriter<PieceDestroyEvent>,
    mut board: ResMut<ChessBoard>,
) {
    for event in move_events.iter() {
        // Move the piece
        board.move_piece(event.piece_move);

        // Take any pieces that were there
        board.remove_piece(event.piece_move.to(), &mut destroy_events);

        // Change the active color
        board.active_color = match board.active_color {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        };
        info!("Active color: {:?}", board.active_color);
    }
}

fn reset_board_state(
    mut setup_events: EventReader<ResetBoardEvent>,
    mut board: ResMut<ChessBoard>,
    mut create_event: EventWriter<PieceCreateEvent>,
) {
    for event in setup_events.iter() {
        *board = ChessBoard::from_fen(event.fen(), &mut create_event);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Events;

    use super::*;

    #[test]
    #[should_panic(expected = "Invalid rank or file value: 8, 4")]
    fn test_board_position_rank_to_large() {
        BoardPosition::new(8, 4);
    }

    #[test]
    #[should_panic(expected = "Invalid rank or file value: 1, 10")]
    fn test_board_position_file_to_large() {
        BoardPosition::new(1, 10);
    }

    #[test]
    fn test_chess_board_empty_board() {
        let empty_board = ChessBoard::empty_board();

        assert_eq!(empty_board.active_color(), PieceColor::White);
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                assert!(empty_board.board[rank][file].is_none());
            }
        }
    }

    #[test]
    fn test_chess_board_from_fen() {
        let fen = Fen::from_file("assets/fens/test_position.fen");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<PieceCreateEvent>();
        app.add_event::<ResetBoardEvent>();
        app.add_system(reset_board_state);

        // Set reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // TODO Confirm that the chessboard has been set up correctly
    }
}
