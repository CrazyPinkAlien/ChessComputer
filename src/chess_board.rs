use bevy::app::App;
use bevy::prelude::{info, Component, EventReader, EventWriter, Plugin, ResMut, Resource};
use strum_macros::EnumIter;

use crate::fen::Fen;

use self::r#move::Move;

pub(super) mod r#move;
mod piece;

static BOARD_SIZE: usize = 8;

pub(super) struct ChessBoardPlugin;

impl Plugin for ChessBoardPlugin {
    #[cfg(not(tarpaulin_include))]
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

    pub fn valid_move(&self, piece_move: Move, check_for_check: bool) -> bool {
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
        && (!piece.is_sliding() || self.no_piece_between_squares(&piece_move.from(), &piece_move.to()))
        // Finally, the move must not put the active color in check
        && check_for_check
        &&{
                let mut test_board = self.clone();
                test_board.move_piece(piece_move);
                let check_status = test_board.in_check();
                check_status.is_none() || check_status.unwrap() != piece.get_color()
            }
    }

    fn get_valid_moves(&self, check_for_check: bool) -> Vec<Move> {
        let mut moves = Vec::new();
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if self.board[rank][file].is_some() {
                    let piece = &self.board[rank][file].as_ref().unwrap();
                    let piece_moves = piece.get_moves(true);
                    for piece_move in piece_moves {
                        if self.valid_move(piece_move, check_for_check) {
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
            panic!("No piece at start location.");
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
        let moves = self.get_valid_moves(false);
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

    fn no_piece_between_squares(&self, start: &BoardPosition, end: &BoardPosition) -> bool {
        let mut rank = start.rank() as i32;
        let mut file = start.file() as i32;
        rank += (end.rank() as i32 - start.rank() as i32).signum();
        file += (end.file() as i32 - start.file() as i32).signum();
        while rank as usize != end.rank() || file as usize != end.file() {
            if self.board[rank as usize][file as usize].is_some() {
                return false;
            }
            rank += (end.rank() as i32 - start.rank() as i32).signum();
            file += (end.file() as i32 - start.file() as i32).signum();
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
    fn test_board_position_new_rank_too_large() {
        BoardPosition::new(8, 4);
    }

    #[test]
    #[should_panic(expected = "Invalid rank or file value: 1, 10")]
    fn test_board_position_new_file_too_large() {
        BoardPosition::new(1, 10);
    }

    #[test]
    fn test_board_position_rank() {
        let position = BoardPosition::new(1, 4);
        assert_eq!(position.rank(), 1);
    }

    #[test]
    fn test_board_position_file() {
        let position = BoardPosition::new(1, 6);
        assert_eq!(position.file(), 6);
    }

    #[test]
    fn test_reset_board_event_fen() {
        let fen = Fen::from_string(
            "rk1r1bb1/ppp1pp1p/3n2n1/1q1p2p1/4P3/1N2Q1PP/PPPP1P2/RK2RBBN b - - 0 1",
        );
        let event = ResetBoardEvent::new(fen.clone());
        assert_eq!(*event.fen(), fen);
    }

    #[test]
    fn test_piece_move_event_piece_move() {
        let piece_move = Move::new(BoardPosition::new(0, 0), BoardPosition::new(2, 3));
        let event = PieceMoveEvent::new(piece_move);
        assert_eq!(*event.piece_move(), piece_move);
    }

    #[test]
    fn test_piece_create_event_position() {
        let position = BoardPosition::new(1, 3);
        let event = PieceCreateEvent {
            position,
            piece_type: PieceType::Bishop,
            color: PieceColor::Black,
        };
        assert_eq!(*event.position(), position);
    }

    #[test]
    fn test_piece_create_event_piece_type() {
        let piece_type = PieceType::Knight;
        let event = PieceCreateEvent {
            position: BoardPosition::new(1, 3),
            piece_type,
            color: PieceColor::Black,
        };
        assert_eq!(event.piece_type(), piece_type);
    }

    #[test]
    fn test_piece_create_event_color() {
        let color = PieceColor::White;
        let event = PieceCreateEvent {
            position: BoardPosition::new(1, 3),
            piece_type: PieceType::Bishop,
            color,
        };
        assert_eq!(event.color(), color);
    }

    #[test]
    fn test_piece_destroy_event_position() {
        let position = BoardPosition::new(4, 7);
        let event = PieceDestroyEvent { position };
        assert_eq!(*event.position(), position);
    }

    #[test]
    fn test_chess_board_default() {
        let default_board = ChessBoard::default();

        assert_eq!(default_board.active_color(), PieceColor::White);
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                assert!(default_board.board[rank][file].is_none());
            }
        }
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
        let fen = Fen::from_string(
            "rk1r1bb1/ppp1pp1p/3n2n1/1q1p2p1/4P3/1N2Q1PP/PPPP1P2/RK2RBBN b - - 0 1",
        );

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<PieceCreateEvent>();
        app.add_event::<ResetBoardEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that the chessboard has been set up correctly
        let pieces = vec![
            vec![
                Some((PieceType::Rook, PieceColor::Black)),
                Some((PieceType::King, PieceColor::Black)),
                None,
                Some((PieceType::Rook, PieceColor::Black)),
                None,
                Some((PieceType::Bishop, PieceColor::Black)),
                Some((PieceType::Bishop, PieceColor::Black)),
                None,
            ],
            vec![
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
            ],
            vec![
                None,
                None,
                None,
                Some((PieceType::Knight, PieceColor::Black)),
                None,
                None,
                Some((PieceType::Knight, PieceColor::Black)),
                None,
            ],
            vec![
                None,
                Some((PieceType::Queen, PieceColor::Black)),
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
                None,
                Some((PieceType::Pawn, PieceColor::Black)),
                None,
            ],
            vec![
                None,
                None,
                None,
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                None,
                None,
            ],
            vec![
                None,
                Some((PieceType::Knight, PieceColor::White)),
                None,
                None,
                Some((PieceType::Queen, PieceColor::White)),
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
            ],
            vec![
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                Some((PieceType::Pawn, PieceColor::White)),
                None,
                None,
            ],
            vec![
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::King, PieceColor::White)),
                None,
                None,
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Knight, PieceColor::White)),
            ],
        ];

        // Check active color
        assert_eq!(
            app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .active_color(),
            PieceColor::Black
        );

        // Check pieces
        let board = &app.world.get_resource::<ChessBoard>().unwrap().board;
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if pieces[rank][file].is_none() {
                    assert!(board[rank][file].is_none());
                } else {
                    assert_eq!(
                        board[rank][file].as_ref().unwrap().get_type(),
                        pieces[rank][file].unwrap().0
                    );
                    assert_eq!(
                        board[rank][file].as_ref().unwrap().get_color(),
                        pieces[rank][file].unwrap().1
                    );
                    assert_eq!(
                        board[rank][file].as_ref().unwrap().get_position(),
                        BoardPosition::new(rank, file)
                    );
                }
            }
        }
    }

    #[test]
    #[ignore]
    #[should_panic(expected = "Unrecognised symbol in FEN: X")]
    fn test_chess_board_from_fen_unrecognised_symbol() {
        let fen = Fen::from_string(
            "rk1x1bb1/ppp1pp1p/3n2n1/1q1p2p1/4P3/1N2Q1PP/PPPP1P2/RK2RBBN b - - 0 1",
        );

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();
    }

    #[test]
    #[ignore]
    #[should_panic(expected = "Unrecognised active color in FEN: l")]
    fn test_chess_board_from_fen_unrecognised_active_color() {
        let fen = Fen::from_string(
            "rk1r1bb1/ppp1pp1p/3n2n1/1q1p2p1/4P3/1N2Q1PP/PPPP1P2/RK2RBBN l - - 0 1",
        );

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();
    }

    #[test]
    fn test_chess_board_active_color() {
        let mut chess_board = ChessBoard::empty_board();
        chess_board.active_color = PieceColor::Black;

        assert_eq!(chess_board.active_color(), PieceColor::Black);
    }

    #[test]
    fn test_chess_board_valid_move_true() {
        let fen =
            Fen::from_string("rnb1kb1r/pp1ppp1p/5n2/qp4p1/4P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Create move
        let piece_move = Move::new(BoardPosition::new(5, 2), BoardPosition::new(3, 1));

        // Confirm that the move is valid
        let board = &app.world.get_resource::<ChessBoard>().unwrap();
        assert!(board.valid_move(piece_move, true));
    }

    #[test]
    #[ignore]
    fn test_chess_board_valid_move_false() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Create move
        let piece_move = Move::new(BoardPosition::new(6, 3), BoardPosition::new(5, 3));

        // Confirm that the move is not valid
        let board = &app.world.get_resource::<ChessBoard>().unwrap();
        assert!(!board.valid_move(piece_move, true));
    }

    #[test]
    fn test_chess_board_valid_move_no_piece() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Create move
        let piece_move = Move::new(BoardPosition::new(5, 3), BoardPosition::new(5, 3));

        // Confirm that the move is not valid
        let board = &app.world.get_resource::<ChessBoard>().unwrap();
        assert!(!board.valid_move(piece_move, true));
    }

    #[test]
    fn test_chess_board_get_valid_moves() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Expected valid moves
        let expected_valid_moves = vec![
            Move::new(BoardPosition::new(3, 1), BoardPosition::new(1, 0)),
            Move::new(BoardPosition::new(3, 1), BoardPosition::new(1, 2)),
            Move::new(BoardPosition::new(3, 1), BoardPosition::new(2, 3)),
            Move::new(BoardPosition::new(3, 1), BoardPosition::new(4, 3)),
            Move::new(BoardPosition::new(3, 1), BoardPosition::new(5, 0)),
            Move::new(BoardPosition::new(3, 1), BoardPosition::new(5, 2)),
            Move::new(BoardPosition::new(4, 4), BoardPosition::new(3, 4)),
            Move::new(BoardPosition::new(4, 4), BoardPosition::new(3, 3)),
            Move::new(BoardPosition::new(5, 5), BoardPosition::new(3, 4)),
            Move::new(BoardPosition::new(5, 5), BoardPosition::new(3, 6)),
            Move::new(BoardPosition::new(5, 5), BoardPosition::new(4, 3)),
            Move::new(BoardPosition::new(5, 5), BoardPosition::new(4, 7)),
            Move::new(BoardPosition::new(5, 5), BoardPosition::new(7, 6)),
            Move::new(BoardPosition::new(6, 0), BoardPosition::new(5, 0)),
            Move::new(BoardPosition::new(6, 0), BoardPosition::new(4, 0)),
            Move::new(BoardPosition::new(6, 1), BoardPosition::new(5, 1)),
            Move::new(BoardPosition::new(6, 1), BoardPosition::new(4, 1)),
            Move::new(BoardPosition::new(6, 2), BoardPosition::new(5, 2)),
            Move::new(BoardPosition::new(6, 2), BoardPosition::new(4, 2)),
            Move::new(BoardPosition::new(6, 3), BoardPosition::new(5, 3)),
            Move::new(BoardPosition::new(6, 3), BoardPosition::new(4, 3)),
            Move::new(BoardPosition::new(6, 6), BoardPosition::new(5, 6)),
            Move::new(BoardPosition::new(6, 6), BoardPosition::new(4, 6)),
            Move::new(BoardPosition::new(6, 7), BoardPosition::new(5, 7)),
            Move::new(BoardPosition::new(6, 7), BoardPosition::new(4, 7)),
            Move::new(BoardPosition::new(7, 0), BoardPosition::new(7, 1)),
            Move::new(BoardPosition::new(7, 3), BoardPosition::new(6, 4)),
            Move::new(BoardPosition::new(7, 4), BoardPosition::new(6, 4)),
            Move::new(BoardPosition::new(7, 4), BoardPosition::new(7, 5)),
            Move::new(BoardPosition::new(7, 7), BoardPosition::new(7, 5)),
            Move::new(BoardPosition::new(7, 7), BoardPosition::new(7, 6)),
        ];

        // Get valid moves
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        let valid_moves = board.get_valid_moves(true);

        // Confirm that the results match
        assert_eq!(expected_valid_moves, valid_moves);
    }

    #[test]
    fn test_chess_board_move_piece() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that the piece starts in the expected location
        let mut board = app.world.get_resource_mut::<ChessBoard>().unwrap();
        assert!(board.board[2][5].is_some());
        assert_eq!(
            board.board[2][5].as_ref().unwrap().get_color(),
            PieceColor::Black
        );
        assert_eq!(
            board.board[2][5].as_ref().unwrap().get_type(),
            PieceType::Knight
        );

        // Move the piece
        let piece_move = Move::new(BoardPosition::new(2, 5), BoardPosition::new(4, 6));
        board.move_piece(piece_move);

        // Confirm that the piece has been moved
        assert!(board.board[2][5].is_none());
        assert!(board.board[4][6].is_some());
        assert_eq!(
            board.board[4][6].as_ref().unwrap().get_color(),
            PieceColor::Black
        );
        assert_eq!(
            board.board[4][6].as_ref().unwrap().get_type(),
            PieceType::Knight
        );
    }

    #[test]
    #[should_panic(expected = "No piece at start location.")]
    fn test_chess_board_move_piece_no_piece() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Attempt to move a non-existent piece
        let mut board = app.world.get_resource_mut::<ChessBoard>().unwrap();
        let piece_move = Move::new(BoardPosition::new(2, 1), BoardPosition::new(4, 6));
        board.move_piece(piece_move);
    }

    #[test]
    fn test_chess_board_get_piece_color() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that get_piece_color returns the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert_eq!(
            board.get_piece_color(BoardPosition::new(1, 4)),
            Some(PieceColor::Black)
        );
        assert_eq!(board.get_piece_color(BoardPosition::new(2, 6)), None);
        assert_eq!(
            board.get_piece_color(BoardPosition::new(7, 2)),
            Some(PieceColor::White)
        );
    }

    #[test]
    #[ignore]
    fn test_chess_board_in_check_white() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/8/qN1p2N1/4P3/2Pn4/PP1P2PP/1RBQK2R w Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert_eq!(board.in_check(), Some(PieceColor::White));
    }

    #[test]
    #[ignore]
    fn test_chess_board_in_check_black() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2p2p/5p2/qN1p2NQ/4P3/2Pn4/PP1P2PP/1RB2K1R w Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert_eq!(board.in_check(), Some(PieceColor::Black));
    }

    #[test]
    fn test_chess_board_in_check_none() {
        let fen =
            Fen::from_string("rnbk1b1r/pp2p2p/5p2/qN1p2NQ/4P3/2Pn4/PP1P2PP/1RB2K1R b Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert!(board.in_check().is_none());
    }

    #[test]
    fn test_chess_board_no_piece_between_squares_true() {
        let fen =
            Fen::from_string("rnbk1b1r/pp2p2p/5p2/qN1p2NQ/4P3/2Pn4/PP1P2PP/1RB2K1R b Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert!(
            board.no_piece_between_squares(&BoardPosition::new(2, 1), &BoardPosition::new(6, 5))
        );
    }

    #[test]
    fn test_chess_board_no_piece_between_squares_false() {
        let fen =
            Fen::from_string("rnbk1b1r/pp2p2p/5p2/qN1p2NQ/4P3/2Pn4/PP1P2PP/1RB2K1R b Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_system(reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert!(
            !board.no_piece_between_squares(&BoardPosition::new(1, 6), &BoardPosition::new(4, 3))
        );
    }
}
