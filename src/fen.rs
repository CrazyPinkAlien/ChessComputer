//! Contains the [Fen] struct which reads and stores a [Forsyth–Edwards Notation (FEN)](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation) string.

use crate::chess_board::BoardPosition;

/// The FEN which represents the default starting position.
const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Reads and stores a FEN string.
#[derive(Debug, Clone, PartialEq)]
pub struct Fen {
    /// The piece placement section of the FEN.
    piece_placement: String,
    /// The active color section of the FEN.
    active_color: String,
    /// The castling rights section of the FEN.
    castling_rights: String,
    /// A square over which a pawn has just passed after moving two squares, if available.
    ep_target_square: Option<BoardPosition>,
    /// The full move number.
    move_number: i32,
}

impl Fen {
    /// Creates a new [Fen] from the given string.
    pub fn from_string(fen_string: &str) -> Self {
        // First split fen into sections separated by spaces
        let split_fen = fen_string.split_whitespace().collect::<Vec<&str>>();
        // Get piece_info placement data
        let piece_placement = split_fen[0];
        // Get active color
        let active_color = split_fen[1];
        // Get castling rights
        let castling_rights = split_fen[2];
        // Get en passant target square
        let ep_target_square = match split_fen[4] {
            "-" => None,
            _ => Some(BoardPosition::new(
                Self::char_to_rank(split_fen[4].chars().nth(1).unwrap()),
                Self::char_to_file(split_fen[4].chars().next().unwrap()),
            )),
        };
        // Create Fen object
        Fen {
            piece_placement: piece_placement.to_string(),
            active_color: active_color.to_string(),
            castling_rights: castling_rights.to_string(),
            ep_target_square,
            move_number: split_fen[5].parse::<i32>().unwrap(),
        }
    }

    /// Returns the piece placement part of the [Fen].
    pub fn piece_placement(&self) -> &String {
        &self.piece_placement
    }

    /// Returns the active color part of the [Fen].
    pub fn active_color(&self) -> &String {
        &self.active_color
    }

    /// Returns the castling rights part of the [Fen].
    pub fn castling_rights(&self) -> &String {
        &self.castling_rights
    }

    /// Returns the en passant target square.
    pub fn ep_target_square(&self) -> &Option<BoardPosition> {
        &self.ep_target_square
    }

    /// Returns the move number of the [Fen].
    pub fn move_number(&self) -> &i32 {
        &self.move_number
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
    use super::*;

    #[test]
    fn test_fen_from_string() {
        // Randomly generated fen
        let fen_string = "5R2/2p4n/1Q6/6Pp/1R2P3/2P2b1K/P2krq2/2N5 w - - 0 1";

        // Create a new fen from the above string
        let fen = Fen::from_string(fen_string);

        // Confirm that the fen has the correct properties
        assert_eq!(
            fen.piece_placement,
            "5R2/2p4n/1Q6/6Pp/1R2P3/2P2b1K/P2krq2/2N5"
        );
        assert_eq!(fen.active_color, "w");
    }

    #[test]
    fn test_fen_piece_placement() {
        // Randomly generated fen
        let fen_string = "5Q2/4PK2/p1pP4/3p4/N1P1P2p/5bB1/3kp2P/8 w - - 0 1";

        // Create a new fen from the above string
        let fen = Fen::from_string(fen_string);

        // Confirm that the function returns the correct result
        assert_eq!(
            fen.piece_placement(),
            "5Q2/4PK2/p1pP4/3p4/N1P1P2p/5bB1/3kp2P/8"
        );
    }

    #[test]
    fn test_fen_active_color() {
        // Randomly generated fen
        let fen_string = "5Q2/4PK2/p1pP4/3p4/N1P1P2p/5bB1/3kp2P/8 b - - 0 1";

        // Create a new fen from the above string
        let fen = Fen::from_string(fen_string);

        // Confirm that the function returns the correct result
        assert_eq!(fen.active_color(), "b");
    }

    #[test]
    fn test_fen_castling_rights() {
        // Randomly generated fen
        let fen_string = "5Q2/4PK2/p1pP4/3p4/N1P1P2p/5bB1/3kp2P/8 b - - 0 1";

        // Create a new fen from the above string
        let fen = Fen::from_string(fen_string);

        // Confirm that the function returns the correct result
        assert_eq!(fen.castling_rights(), "-");
    }

    #[test]
    fn test_fen_move_number() {
        // Randomly generated fen
        let fen_string = "5Q2/4PK2/p1pP4/3p4/N1P1P2p/5bB1/3kp2P/8 b - - 0 1";

        // Create a new fen from the above string
        let fen = Fen::from_string(fen_string);

        // Confirm that the function returns the correct result
        assert_eq!(*fen.move_number(), 1);
    }
}
