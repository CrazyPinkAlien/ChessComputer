use bevy::app::App;
use bevy::prelude::{
    Component, Event, EventReader, EventWriter, Plugin, PostUpdate, PreUpdate, ResMut, Resource,
    Startup, Update,
};
use strum_macros::EnumIter;

use crate::fen::Fen;

use self::castling_rights::CastlingRights;
use self::r#move::Move;

mod castling_rights;
pub(super) mod r#move;
mod piece;

const BOARD_SIZE: usize = 8;

pub(super) struct ChessBoardPlugin;

impl Plugin for ChessBoardPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.add_event::<ResetBoardEvent>()
            .add_event::<PieceMoveEvent>()
            .add_event::<PieceCreateEvent>()
            .add_event::<RequestMoveEvent>()
            .init_resource::<ChessBoard>()
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, game_end_checker)
            .add_systems(Update, reset_board_state)
            .add_systems(PostUpdate, make_move);
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Component, Eq)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

impl ToString for PieceColor {
    fn to_string(&self) -> String {
        match self {
            PieceColor::White => "White".to_string(),
            PieceColor::Black => "Black".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum GameEndStatus {
    Checkmate,
    Resignation,
    Stalemate,
    DeadPosition,
    FlagFall,
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

    pub fn rank(&self) -> &usize {
        &self.rank
    }

    pub fn file(&self) -> &usize {
        &self.file
    }
}

/// Event sent to the [ChessBoard] to reset it with the given [Fen].
#[derive(Debug, Clone, Event)]
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

/// Event sent by the [ChessBoard] to notify that a piece has been moved.
#[derive(Event)]
pub struct PieceMoveEvent {
    from: BoardPosition,
    to: BoardPosition,
}

impl PieceMoveEvent {
    pub fn new(from: BoardPosition, to: BoardPosition) -> Self {
        PieceMoveEvent { from, to }
    }

    pub fn from(&self) -> &BoardPosition {
        &self.from
    }

    pub fn to(&self) -> &BoardPosition {
        &self.to
    }
}

/// Event sent to the [ChessBoard] to request that a move is made.
#[derive(Event)]
pub struct RequestMoveEvent {
    piece_move: Move,
}

impl RequestMoveEvent {
    pub fn new(piece_move: Move) -> Self {
        RequestMoveEvent { piece_move }
    }

    pub fn piece_move(&self) -> &Move {
        &self.piece_move
    }
}

/// Event sent by the [ChessBoard] to notify that a piece has been placed on the board.
#[derive(Event)]
pub struct PieceCreateEvent {
    position: BoardPosition,
    piece_type: PieceType,
    color: PieceColor,
}

impl PieceCreateEvent {
    pub fn position(&self) -> &BoardPosition {
        &self.position
    }

    pub fn piece_type(&self) -> &PieceType {
        &self.piece_type
    }

    pub fn color(&self) -> &PieceColor {
        &self.color
    }
}

#[derive(Resource, Clone)]
pub struct ChessBoard {
    board: [[Option<Box<dyn piece::Piece>>; 8]; 8],
    active_color: Option<PieceColor>,
    past_moves: Vec<Move>,
    move_number: i32,
    castling_rights: CastlingRights,
    winner: Option<PieceColor>,
    game_end_status: Option<GameEndStatus>,
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
            active_color: None,
            past_moves: Vec::new(),
            move_number: 1,
            castling_rights: CastlingRights::default(),
            winner: None,
            game_end_status: None,
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
            "w" => Some(PieceColor::White),
            "b" => Some(PieceColor::Black),
            _ => panic!("Unrecognised active color in FEN: {}", fen.active_color()),
        };
        // Set move number
        board_state.move_number = *fen.move_number();
        // Set castling rights
        board_state.castling_rights = CastlingRights::from_fen_string(fen.castling_rights());

        board_state
    }

    pub fn active_color(&self) -> &Option<PieceColor> {
        &self.active_color
    }

    pub fn past_moves(&self) -> &Vec<Move> {
        &self.past_moves
    }

    pub fn move_number(&self) -> &i32 {
        &self.move_number
    }

    pub fn game_end_status(&self) -> &Option<GameEndStatus> {
        &self.game_end_status
    }

    pub fn winner(&self) -> &Option<PieceColor> {
        &self.winner
    }

    pub fn valid_move(
        &self,
        piece_move: &Move,
        active_color: &Option<PieceColor>,
        check_for_check: &bool,
    ) -> bool {
        // Throw away any move with both the capture and castle tags
        if piece_move.is_castle() && piece_move.is_capture() {
            return false;
        }

        // Get piece
        if self.board[piece_move.from().rank][piece_move.from().file].is_none() {
            return false;
        }
        let piece = self.board[piece_move.from().rank][piece_move.from().file]
            .as_ref()
            .unwrap();

        let file_move_direction = *piece_move.to().file() as i32 - *piece_move.from().file() as i32;

        // Check that there is an active colour
        active_color.is_some()
        // Check that the piece is the active colour
        && (*piece.get_color() == active_color.unwrap())
        // Check whether or not there are any pieces there
        && match self.get_piece_color(piece_move.to()) {
            Some(color) => if color == *piece.get_color() {
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
        && (!piece.is_sliding() || self.no_piece_between_squares(piece_move.from(), piece_move.to()))
        // The move must not put the active color in check
        && (!check_for_check
        ||{
                let mut test_board = self.clone();
                test_board.move_piece(piece_move.from(), piece_move.to());
                !test_board.in_check(&active_color.unwrap())
            })
        // Check if a castle is possible
        && (!check_for_check || !piece_move.is_castle() || (
            // Check that this is a valid direction in which to castle
            self.castling_rights.valid_castle_direction(&active_color.unwrap(), file_move_direction)
            // Check that there are no pieces between the king and the rook
            && self.no_piece_between_squares(piece_move.from(), &BoardPosition::new(*piece_move.from().rank(), (*piece_move.from().file() as i32 + file_move_direction * BOARD_SIZE as i32).clamp(1, BOARD_SIZE as i32 - 1) as usize))
            // Check that the king is not currently in check
            && !self.in_check(&active_color.unwrap())
            // Check that the king will not move through check
            && {
                let mut test_board = self.clone();
                test_board.move_piece(piece_move.from(), piece_move.to());
                !test_board.in_check(&active_color.unwrap())
            }
        ))
    }

    pub fn get_valid_moves(
        &self,
        active_color: &Option<PieceColor>,
        check_for_check: &bool,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if self.board[rank][file].is_some() {
                    let piece = &self.board[rank][file].as_ref().unwrap();
                    let piece_moves = piece.get_moves(&true);
                    for move_to in piece_moves {
                        let piece_move =
                            Move::from_board(BoardPosition::new(rank, file), move_to, self);
                        if self.valid_move(&piece_move, active_color, check_for_check) {
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

    fn move_piece(&mut self, from: &BoardPosition, to: &BoardPosition) {
        if self.board[*from.rank()][*from.file()].is_none() {
            panic!("No piece at start location.");
        }
        self.board[*from.rank()][*from.file()]
            .as_mut()
            .unwrap()
            .set_position(to);
        self.board[*to.rank()][*to.file()] = self.board[*from.rank()][*from.file()].clone();
        self.board[*from.rank()][*from.file()] = None;
    }

    pub fn get_piece_type(&self, position: &BoardPosition) -> Option<PieceType> {
        self.board[position.rank][position.file]
            .as_ref()
            .map(|piece| *piece.get_type())
    }

    fn get_piece_color(&self, position: &BoardPosition) -> Option<PieceColor> {
        self.board[position.rank][position.file]
            .as_ref()
            .map(|piece| *piece.get_color())
    }

    fn in_check(&self, color: &PieceColor) -> bool {
        // Get king location
        let mut king_location = BoardPosition::new(0, 0);
        'outer: for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if self.board[rank][file].is_some()
                    && *self.board[rank][file].as_ref().unwrap().get_type() == PieceType::King
                    && self.board[rank][file].as_ref().unwrap().get_color() == color
                {
                    king_location = BoardPosition::new(rank, file);
                    break 'outer;
                }
            }
        }
        // Get valid moves
        let moves = self.get_valid_moves(&Some(color.opposite()), &false);
        // Check if any valid moves can take the king
        for piece_move in moves {
            if *piece_move.to() == king_location {
                return true;
            }
        }
        false
    }

    fn no_piece_between_squares(&self, start: &BoardPosition, end: &BoardPosition) -> bool {
        let mut rank = *start.rank() as i32;
        let mut file = *start.file() as i32;
        rank += (*end.rank() as i32 - *start.rank() as i32).signum();
        file += (*end.file() as i32 - *start.file() as i32).signum();
        while rank as usize != *end.rank() || file as usize != *end.file() {
            if self.board[rank as usize][file as usize].is_some() {
                return false;
            }
            rank += (*end.rank() as i32 - *start.rank() as i32).signum();
            file += (*end.file() as i32 - *start.file() as i32).signum();
        }
        true
    }
}

fn setup(mut create_event: EventWriter<PieceCreateEvent>, mut board: ResMut<ChessBoard>) {
    *board = ChessBoard::from_fen(&Fen::default(), &mut create_event);
}

fn make_move(
    mut request_events: EventReader<RequestMoveEvent>,
    mut move_events: EventWriter<PieceMoveEvent>,
    mut board: ResMut<ChessBoard>,
) {
    for request_event in request_events.iter() {
        // First confirm that the move is valid
        if board.valid_move(request_event.piece_move(), board.active_color(), &true) {
            // Move the piece
            board.move_piece(
                request_event.piece_move().from(),
                request_event.piece_move().to(),
            );
            move_events.send(PieceMoveEvent::new(
                *request_event.piece_move().from(),
                *request_event.piece_move().to(),
            ));

            // If the move was a castle, also move the rook
            if request_event.piece_move().is_castle() {
                let file_move_direction = *request_event.piece_move().to().file() as i32
                    - *request_event.piece_move().from().file() as i32;
                let from = BoardPosition::new(
                    *request_event.piece_move().from().rank(),
                    (*request_event.piece_move().from().file() as i32
                        + file_move_direction * BOARD_SIZE as i32)
                        .clamp(1, BOARD_SIZE as i32 - 1) as usize,
                );
                let to = BoardPosition::new(
                    *request_event.piece_move().to().rank(),
                    (*request_event.piece_move().to().file() as i32 - file_move_direction.signum())
                        as usize,
                );
                board.move_piece(&from, &to);
                move_events.send(PieceMoveEvent::new(from, to));
            }

            // Change the active color
            board.active_color = Some(board.active_color.unwrap().opposite());

            // Make a record of the move
            board.past_moves.push(*request_event.piece_move());

            // Increment the move number if it is now white's turn
            if board.active_color == Some(PieceColor::White) {
                board.move_number += 1;
            }

            // Update castling rights
            board
                .castling_rights
                .update_after_move(request_event.piece_move());
        }
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

fn game_end_checker(mut board: ResMut<ChessBoard>, mut events: EventReader<PieceMoveEvent>) {
    for _event in events.iter() {
        // Check for checkmate or stalemate
        if board.active_color().is_some()
            && board
                .get_valid_moves(board.active_color(), &true)
                .is_empty()
        {
            if board.in_check(&board.active_color().unwrap()) {
                // Checkmate
                board.game_end_status = Some(GameEndStatus::Checkmate);
                board.winner = Some(board.active_color().unwrap().opposite());
            } else {
                // Stalemate
                board.game_end_status = Some(GameEndStatus::Stalemate);
            }
            // The game has ended, set the active color to None.
            board.active_color = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Events, Startup};

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
    fn test_chess_board_empty_board() {
        let empty_board = ChessBoard::empty_board();

        assert_eq!(*empty_board.active_color(), None);
        assert_eq!(empty_board.past_moves.len(), 0);
        assert_eq!(*empty_board.move_number(), 1);
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
        app.add_systems(Update, reset_board_state);

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
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .active_color(),
            Some(PieceColor::Black)
        );

        // Check past moves
        assert_eq!(
            app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .past_moves
                .len(),
            0
        );

        // Check move number
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .move_number(),
            1
        );

        // Check pieces
        let board = &app.world.get_resource::<ChessBoard>().unwrap().board;
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if pieces[rank][file].is_none() {
                    assert!(board[rank][file].is_none());
                } else {
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_type(),
                        pieces[rank][file].unwrap().0
                    );
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_color(),
                        pieces[rank][file].unwrap().1
                    );
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_position(),
                        BoardPosition::new(rank, file)
                    );
                }
            }
        }
    }

    // TODO: This test should expect the message: "Unrecognised symbol in FEN: X"
    #[test]
    #[should_panic]
    fn test_chess_board_from_fen_unrecognised_symbol() {
        let fen = Fen::from_string(
            "rk1x1bb1/ppp1pp1p/3n2n1/1q1p2p1/4P3/1N2Q1PP/PPPP1P2/RK2RBBN b - - 0 1",
        );

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();
    }

    // TODO: This test should expect the message: "Unrecognised active color in FEN: l"
    #[test]
    #[should_panic]
    fn test_chess_board_from_fen_unrecognised_active_color() {
        let fen = Fen::from_string(
            "rk1r1bb1/ppp1pp1p/3n2n1/1q1p2p1/4P3/1N2Q1PP/PPPP1P2/RK2RBBN l - - 0 1",
        );

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();
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
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Create move
        let board = &app.world.get_resource::<ChessBoard>().unwrap();
        let piece_move =
            Move::from_board(BoardPosition::new(5, 2), BoardPosition::new(3, 1), board);

        // Confirm that the move is valid
        assert!(board.valid_move(&piece_move, board.active_color(), &true));
    }

    #[test]
    fn test_chess_board_valid_move_false() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Create move
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        let piece_move =
            Move::from_board(BoardPosition::new(6, 3), BoardPosition::new(5, 3), board);

        // Confirm that the move is not valid
        assert!(!board.valid_move(&piece_move, board.active_color(), &true));
    }

    #[test]
    #[should_panic(expected = "No piece found.")]
    fn test_chess_board_valid_move_no_piece() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Create move
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        let _piece_move =
            Move::from_board(BoardPosition::new(5, 3), BoardPosition::new(5, 3), board);
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
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Expected valid moves
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        let expected_valid_moves = vec![
            Move {
                from: BoardPosition::new(3, 1),
                to: BoardPosition::new(1, 0),
                piece_type: PieceType::Knight,
                is_capture: true,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(3, 1),
                to: BoardPosition::new(1, 2),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(3, 1),
                to: BoardPosition::new(2, 3),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(3, 1),
                to: BoardPosition::new(4, 3),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(3, 1),
                to: BoardPosition::new(5, 0),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(3, 1),
                to: BoardPosition::new(5, 2),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(4, 4),
                to: BoardPosition::new(3, 4),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(4, 4),
                to: BoardPosition::new(3, 3),
                piece_type: PieceType::Pawn,
                is_capture: true,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(5, 5),
                to: BoardPosition::new(3, 4),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(5, 5),
                to: BoardPosition::new(3, 6),
                piece_type: PieceType::Knight,
                is_capture: true,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(5, 5),
                to: BoardPosition::new(4, 3),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(5, 5),
                to: BoardPosition::new(4, 7),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(5, 5),
                to: BoardPosition::new(7, 6),
                piece_type: PieceType::Knight,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 0),
                to: BoardPosition::new(5, 0),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 0),
                to: BoardPosition::new(4, 0),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 1),
                to: BoardPosition::new(5, 1),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 1),
                to: BoardPosition::new(4, 1),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 2),
                to: BoardPosition::new(5, 2),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 2),
                to: BoardPosition::new(4, 2),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 6),
                to: BoardPosition::new(5, 6),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 6),
                to: BoardPosition::new(4, 6),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 7),
                to: BoardPosition::new(5, 7),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(6, 7),
                to: BoardPosition::new(4, 7),
                piece_type: PieceType::Pawn,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(7, 0),
                to: BoardPosition::new(7, 1),
                piece_type: PieceType::Rook,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(7, 3),
                to: BoardPosition::new(6, 4),
                piece_type: PieceType::Queen,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(7, 4),
                to: BoardPosition::new(6, 4),
                piece_type: PieceType::King,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(7, 4),
                to: BoardPosition::new(7, 5),
                piece_type: PieceType::King,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(7, 4),
                to: BoardPosition::new(7, 6),
                piece_type: PieceType::King,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: true,
            },
            Move {
                from: BoardPosition::new(7, 7),
                to: BoardPosition::new(7, 5),
                piece_type: PieceType::Rook,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
            Move {
                from: BoardPosition::new(7, 7),
                to: BoardPosition::new(7, 6),
                piece_type: PieceType::Rook,
                is_capture: false,
                piece_color: PieceColor::White,
                is_castle: false,
            },
        ];

        // Get valid moves
        let valid_moves = board.get_valid_moves(board.active_color(), &true);

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
        app.add_systems(Update, reset_board_state);

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
            *board.board[2][5].as_ref().unwrap().get_color(),
            PieceColor::Black
        );
        assert_eq!(
            *board.board[2][5].as_ref().unwrap().get_type(),
            PieceType::Knight
        );

        // Move the piece
        board.move_piece(&BoardPosition::new(2, 5), &BoardPosition::new(4, 6));

        // Confirm that the piece has been moved
        assert!(board.board[2][5].is_none());
        assert!(board.board[4][6].is_some());
        assert_eq!(
            *board.board[4][6].as_ref().unwrap().get_color(),
            PieceColor::Black
        );
        assert_eq!(
            *board.board[4][6].as_ref().unwrap().get_type(),
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
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Attempt to move a non-existent piece
        let mut board = app.world.get_resource_mut::<ChessBoard>().unwrap();
        board.move_piece(&BoardPosition::new(2, 1), &BoardPosition::new(4, 6));
    }

    #[test]
    fn test_chess_board_get_piece_type() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/5n2/qN1p2p1/4P3/5N2/PPPP1PPP/R1BQK2R w KQkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that get_piece_type returns the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert_eq!(
            board.get_piece_type(&BoardPosition::new(1, 4)),
            Some(PieceType::Pawn)
        );
        assert_eq!(board.get_piece_type(&BoardPosition::new(2, 6)), None);
        assert_eq!(
            board.get_piece_type(&BoardPosition::new(7, 2)),
            Some(PieceType::Bishop)
        );
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
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that get_piece_color returns the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert_eq!(
            board.get_piece_color(&BoardPosition::new(1, 4)),
            Some(PieceColor::Black)
        );
        assert_eq!(board.get_piece_color(&BoardPosition::new(2, 6)), None);
        assert_eq!(
            board.get_piece_color(&BoardPosition::new(7, 2)),
            Some(PieceColor::White)
        );
    }

    #[test]
    fn test_chess_board_in_check_white() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2pp1p/8/qN1p2N1/4P3/2Pn4/PP1P2PP/1RBQK2R w Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert!(board.in_check(&PieceColor::White));
        assert!(!board.in_check(&PieceColor::Black));
    }

    #[test]
    fn test_chess_board_in_check_black() {
        let fen =
            Fen::from_string("rnb1kb1r/pp2p2p/5p2/qN1p2NQ/4P3/2Pn4/PP1P2PP/1RB2K1R w Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert!(board.in_check(&PieceColor::Black));
        assert!(!board.in_check(&PieceColor::White));
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
        app.add_systems(Update, reset_board_state);

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Confirm that we get the correct result
        let board = app.world.get_resource::<ChessBoard>().unwrap();
        assert!(!board.in_check(&PieceColor::White));
        assert!(!board.in_check(&PieceColor::Black));
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
        app.add_systems(Update, reset_board_state);

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
        app.add_systems(Update, reset_board_state);

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

    #[test]
    fn test_setup() {
        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<PieceCreateEvent>();
        app.add_systems(Startup, setup);

        // Run systems
        app.update();

        // Confirm that the chessboard has been set up correctly
        let pieces = vec![
            vec![
                Some((PieceType::Rook, PieceColor::Black)),
                Some((PieceType::Knight, PieceColor::Black)),
                Some((PieceType::Bishop, PieceColor::Black)),
                Some((PieceType::Queen, PieceColor::Black)),
                Some((PieceType::King, PieceColor::Black)),
                Some((PieceType::Bishop, PieceColor::Black)),
                Some((PieceType::Knight, PieceColor::Black)),
                Some((PieceType::Rook, PieceColor::Black)),
            ],
            vec![
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
                Some((PieceType::Pawn, PieceColor::Black)),
            ],
            vec![None, None, None, None, None, None, None, None],
            vec![None, None, None, None, None, None, None, None],
            vec![None, None, None, None, None, None, None, None],
            vec![None, None, None, None, None, None, None, None],
            vec![
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
                Some((PieceType::Pawn, PieceColor::White)),
            ],
            vec![
                Some((PieceType::Rook, PieceColor::White)),
                Some((PieceType::Knight, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Queen, PieceColor::White)),
                Some((PieceType::King, PieceColor::White)),
                Some((PieceType::Bishop, PieceColor::White)),
                Some((PieceType::Knight, PieceColor::White)),
                Some((PieceType::Rook, PieceColor::White)),
            ],
        ];

        // Check active color
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .active_color(),
            Some(PieceColor::White)
        );

        // Check past moves
        assert_eq!(
            app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .past_moves
                .len(),
            0
        );

        // Check move number
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .move_number(),
            1
        );

        // Check pieces
        let board = &app.world.get_resource::<ChessBoard>().unwrap().board;
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if pieces[rank][file].is_none() {
                    assert!(board[rank][file].is_none());
                } else {
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_type(),
                        pieces[rank][file].unwrap().0
                    );
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_color(),
                        pieces[rank][file].unwrap().1
                    );
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_position(),
                        BoardPosition::new(rank, file)
                    );
                }
            }
        }
    }

    #[test]
    fn test_make_move() {
        let fen =
            Fen::from_string("rnbk1b1r/pp2p2p/5p2/qN1p2NQ/4P3/2Pn4/PP1P2PP/1RB2K1R b Kkq - 0 1");

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<ResetBoardEvent>();
        app.add_event::<PieceCreateEvent>();
        app.add_event::<PieceMoveEvent>();
        app.add_event::<RequestMoveEvent>();
        app.add_systems(Update, (reset_board_state, make_move));

        // Trigger reset board event
        app.world
            .resource_mut::<Events<ResetBoardEvent>>()
            .send(ResetBoardEvent::new(fen));

        // Run systems
        app.update();

        // Trigger request move event
        let move_from = BoardPosition::new(2, 5);
        let move_to = BoardPosition::new(3, 6);
        app.world
            .resource_mut::<Events<RequestMoveEvent>>()
            .send(RequestMoveEvent::new(Move {
                from: move_from,
                to: move_to,
                piece_type: PieceType::Pawn,
                piece_color: PieceColor::Black,
                is_capture: true,
                is_castle: false,
            }));

        // Run systems
        app.update();

        // Confirm that the piece has been correctly moved
        let board = &app.world.get_resource::<ChessBoard>().unwrap().board;
        assert_eq!(
            app.world.get_resource::<ChessBoard>().unwrap().active_color,
            Some(PieceColor::White)
        );
        assert!(board[3][6].is_some());
        assert_eq!(
            *board[3][6].as_ref().unwrap().get_color(),
            PieceColor::Black
        );
        assert_eq!(*board[3][6].as_ref().unwrap().get_type(), PieceType::Pawn);
        assert_eq!(*board[3][6].as_ref().unwrap().get_position(), move_to);
        assert!(board[2][5].is_none());
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .active_color(),
            Some(PieceColor::White)
        );
        assert_eq!(
            &app.world.get_resource::<ChessBoard>().unwrap().past_moves,
            &vec![Move {
                from: move_from,
                to: move_to,
                piece_type: PieceType::Pawn,
                piece_color: PieceColor::Black,
                is_capture: true,
                is_castle: false
            }]
        );
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .move_number(),
            2
        );

        // Trigger request move event
        let move_from = BoardPosition::new(3, 7);
        let move_to = BoardPosition::new(3, 6);
        app.world
            .resource_mut::<Events<RequestMoveEvent>>()
            .send(RequestMoveEvent::new(Move {
                from: move_from,
                to: move_to,
                piece_type: PieceType::Pawn,
                piece_color: PieceColor::White,
                is_capture: false,
                is_castle: false,
            }));

        // Run systems
        app.update();

        // Confirm that the piece has been correctly moved
        let board = &app.world.get_resource::<ChessBoard>().unwrap().board;
        assert_eq!(
            app.world.get_resource::<ChessBoard>().unwrap().active_color,
            Some(PieceColor::Black)
        );
        assert!(board[3][6].is_some());
        assert_eq!(
            *board[3][6].as_ref().unwrap().get_color(),
            PieceColor::White
        );
        assert_eq!(*board[3][6].as_ref().unwrap().get_type(), PieceType::Queen);
        assert_eq!(*board[3][6].as_ref().unwrap().get_position(), move_to);
        assert!(board[3][7].is_none());
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .active_color(),
            Some(PieceColor::Black)
        );
        assert_eq!(
            &app.world.get_resource::<ChessBoard>().unwrap().past_moves,
            &vec![
                Move {
                    from: BoardPosition::new(2, 5),
                    to: BoardPosition::new(3, 6),
                    piece_type: PieceType::Pawn,
                    piece_color: PieceColor::Black,
                    is_capture: true,
                    is_castle: false
                },
                Move {
                    from: move_from,
                    to: move_to,
                    piece_type: PieceType::Pawn,
                    piece_color: PieceColor::White,
                    is_capture: false,
                    is_castle: false
                }
            ]
        );
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .move_number(),
            2
        );
    }

    #[test]
    fn test_reset_board_state() {
        let fen = Fen::from_string(
            "rk1r1bb1/ppp1pp1p/3n2n1/1q1p2p1/4P3/1N2Q1PP/PPPP1P2/RK2RBBN b - - 0 1",
        );

        // Setup app
        let mut app = App::new();
        app.insert_resource(ChessBoard::empty_board());
        app.add_event::<PieceCreateEvent>();
        app.add_event::<ResetBoardEvent>();
        app.add_systems(Update, reset_board_state);

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
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .active_color(),
            Some(PieceColor::Black)
        );

        // Check past moves
        assert_eq!(
            app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .past_moves
                .len(),
            0
        );

        // Check move number
        assert_eq!(
            *app.world
                .get_resource::<ChessBoard>()
                .unwrap()
                .move_number(),
            1
        );

        // Check pieces
        let board = &app.world.get_resource::<ChessBoard>().unwrap().board;
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if pieces[rank][file].is_none() {
                    assert!(board[rank][file].is_none());
                } else {
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_type(),
                        pieces[rank][file].unwrap().0
                    );
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_color(),
                        pieces[rank][file].unwrap().1
                    );
                    assert_eq!(
                        *board[rank][file].as_ref().unwrap().get_position(),
                        BoardPosition::new(rank, file)
                    );
                }
            }
        }
    }
}
