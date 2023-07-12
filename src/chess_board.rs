use bevy::app::App;
use bevy::prelude::{
    info, warn, Component, EventReader, EventWriter, Events, Plugin, ResMut, Resource,
};
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
        if (rank > BOARD_SIZE) | (file > BOARD_SIZE) {
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

#[derive(Debug, Copy, Clone)]
pub struct ResetBoardEvent;

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

    fn from_fen(fen: Fen, create_event: &mut EventWriter<PieceCreateEvent>) -> Self {
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
        } // TODO No piece in the way for sliding pieces
        // Finally, the move must not put the active color in check
        && {let mut test_board = self.clone();
            test_board.move_piece(piece_move);
            let check_status = test_board.in_check();
            check_status.is_none()
            || check_status.unwrap() != piece.get_color()
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
        mut events: Events<PieceDestroyEvent>,
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
        // TODO
        None
    }
}

fn setup(mut create_event: EventWriter<PieceCreateEvent>, mut board: ResMut<ChessBoard>) {
    *board = ChessBoard::from_fen(
        Fen::from_file("assets/fens/starting_position.fen"),
        &mut create_event,
    );
}

fn make_move(mut move_events: EventReader<PieceMoveEvent>, mut board: ResMut<ChessBoard>) {
    for event in move_events.iter() {
        // Move the piece
        board.move_piece(event.piece_move);

        // TODO Take any pieces that were there

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
    for _event in setup_events.iter() {
        // TODO: Despawn all pieces

        *board = ChessBoard::from_fen(
            Fen::from_file("assets/fens/starting_position.fen"),
            &mut create_event,
        );
    }
}
