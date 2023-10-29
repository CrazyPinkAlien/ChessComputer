//! Contains the [Fen] struct which interprets a [Forsyth–Edwards Notation (FEN)](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation) string.

use crate::castling_rights::CastlingRights;
use crate::chess_board::{BoardPosition, PieceColor, PieceType};

/// The FEN which represents the default starting position.
const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// A representation of a board state based on FEN notation.
#[derive(Debug, Clone)]
pub struct Fen {
    /// The piece placement.
    piece_placement: [[Option<(PieceColor, PieceType)>; 8]; 8],
    /// The active color.
    active_color: PieceColor,
    /// The castling rights.
    castling_rights: CastlingRights,
    /// A square over which a pawn has just passed after moving two squares, if available.
    ep_target_square: Option<BoardPosition>,
    /// The number of halfmoves since the last capture or pawn advance.
    halfmove_clock: i32,
    /// The number of the full moves. It starts at 1 and is incremented after Black's move.
    fullmove_number: i32,
}

impl Fen {
    /// Creates a new [Fen] from the given string.
    pub fn from_string(fen_string: &str) -> Self {
        // First split fen into sections separated by spaces
        let split_fen = fen_string.split_whitespace().collect::<Vec<&str>>();

        // Get piece placement data
        let piece_placement_string = split_fen[0];
        // Create an empty board state
        let mut piece_placement = [[None; 8]; 8];
        // Populate it from the given fen string
        let mut rank = 0;
        let mut file = 0;
        for rank_str in piece_placement_string.split('/') {
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
                    piece_placement[rank][file] = Some((piece_color, piece_type));
                    file += 1;
                }
                if file >= 8 {
                    rank += 1;
                    file = 0;
                };
            }
        }

        // Get active color
        let active_color = match split_fen[1] {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => panic!("Unrecognised active color in FEN: {}", split_fen[1]),
        };

        // Get castling rights
        let castling_rights = CastlingRights::from_fen_string(split_fen[2]);

        // Get en passant target square
        let ep_target_square = match split_fen[3] {
            "-" => None,
            _ => Some(BoardPosition::new(
                Self::char_to_rank(split_fen[3].chars().nth(1).unwrap()),
                Self::char_to_file(split_fen[3].chars().next().unwrap()),
            )),
        };

        // Create Fen object
        Fen {
            piece_placement,
            active_color,
            castling_rights,
            ep_target_square,
            halfmove_clock: split_fen[4].parse::<i32>().unwrap(),
            fullmove_number: split_fen[5].parse::<i32>().unwrap(),
        }
    }

    /// Returns the piece placement.
    pub fn piece_placement(&self) -> &[[Option<(PieceColor, PieceType)>; 8]; 8] {
        &self.piece_placement
    }

    /// Returns the active color.
    pub fn active_color(&self) -> &PieceColor {
        &self.active_color
    }

    /// Returns the castling rights.
    pub fn castling_rights(&self) -> &CastlingRights {
        &self.castling_rights
    }

    /// Returns the en passant target square.
    pub fn ep_target_square(&self) -> &Option<BoardPosition> {
        &self.ep_target_square
    }

    /// Returns the number of halfmoves since the last capture or pawn advance.
    pub fn halfmove_clock(&self) -> &i32 {
        &self.halfmove_clock
    }

    /// Returns the number of the full moves.
    pub fn fullmove_number(&self) -> &i32 {
        &self.fullmove_number
    }

    /// Converts the given rank char to the corresponding board index.
    fn char_to_rank(char: char) -> usize {
        match char {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            _ => panic!("Unexpected rank char: {}.", char),
        }
    }

    /// Converts the given file char to the corresponding board index.
    fn char_to_file(char: char) -> usize {
        match char {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!("Unexpected file char: {}.", char),
        }
    }
}

impl Default for Fen {
    fn default() -> Self {
        Fen::from_string(STARTING_FEN)
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the [Fen] module.
    use crate::chess_board::BOARD_SIZE;

    use super::*;

    #[test]
    fn test_fen_from_string() {
        // Randomly generated fen
        let fen_string = "5R2/2p4n/1Q6/6Pp/1R2P3/2P2b1K/P2krq2/2N5 w - - 0 1";

        // Create a new fen from the above string
        let fen = Fen::from_string(fen_string);

        // Confirm that the fen has the correct properties
        let expected_placement = [
            [
                None,
                None,
                None,
                None,
                None,
                Some((PieceColor::White, PieceType::Rook)),
                None,
                None,
            ],
            [
                None,
                None,
                Some((PieceColor::Black, PieceType::Pawn)),
                None,
                None,
                None,
                None,
                Some((PieceColor::Black, PieceType::Knight)),
            ],
            [
                None,
                Some((PieceColor::White, PieceType::Queen)),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                None,
                None,
                Some((PieceColor::White, PieceType::Pawn)),
                Some((PieceColor::Black, PieceType::Pawn)),
            ],
            [
                None,
                Some((PieceColor::White, PieceType::Rook)),
                None,
                None,
                Some((PieceColor::White, PieceType::Pawn)),
                None,
                None,
                None,
            ],
            [
                None,
                None,
                Some((PieceColor::White, PieceType::Pawn)),
                None,
                None,
                Some((PieceColor::Black, PieceType::Bishop)),
                None,
                Some((PieceColor::White, PieceType::King)),
            ],
            [
                Some((PieceColor::White, PieceType::Pawn)),
                None,
                None,
                Some((PieceColor::Black, PieceType::King)),
                Some((PieceColor::Black, PieceType::Rook)),
                Some((PieceColor::Black, PieceType::Queen)),
                None,
                None,
            ],
            [
                None,
                None,
                Some((PieceColor::White, PieceType::Knight)),
                None,
                None,
                None,
                None,
                None,
            ],
        ];
        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                if fen.piece_placement[rank][file].is_some() {
                    assert_eq!(
                        fen.piece_placement[rank][file].unwrap().0,
                        expected_placement[rank][file].unwrap().0
                    );
                    assert_eq!(
                        fen.piece_placement[rank][file].unwrap().1,
                        expected_placement[rank][file].unwrap().1
                    );
                } else {
                    assert_eq!(expected_placement[rank][file], None);
                }
            }
        }

        assert_eq!(fen.active_color, PieceColor::White);
        assert_eq!(
            fen.castling_rights,
            CastlingRights {
                white: [false, false],
                black: [false, false]
            }
        );
        assert_eq!(fen.ep_target_square, None);
        assert_eq!(fen.halfmove_clock, 0);
        assert_eq!(fen.fullmove_number, 1);
    }
}
