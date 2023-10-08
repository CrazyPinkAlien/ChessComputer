#[derive(Debug, Clone, PartialEq)]
pub struct Fen {
    piece_placement: String,
    active_color: String,
    castling_rights: String,
    move_number: i32,
}

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Fen {
    pub fn from_string(fen_string: &str) -> Self {
        // First split fen into sections separated by spaces
        let split_fen = fen_string.split_whitespace().collect::<Vec<&str>>();
        // Get piece_info placement data
        let piece_placement = split_fen[0];
        // Get active color
        let active_color = split_fen[1];
        // Get castling rights
        let castling_rights = split_fen[2];
        // Create Fen object
        Fen {
            piece_placement: piece_placement.to_string(),
            active_color: active_color.to_string(),
            castling_rights: castling_rights.to_string(),
            move_number: split_fen[5].parse::<i32>().unwrap(),
        }
    }

    pub fn piece_placement(&self) -> &String {
        &self.piece_placement
    }

    pub fn active_color(&self) -> &String {
        &self.active_color
    }

    pub fn castling_rights(&self) -> &String {
        &self.castling_rights
    }

    pub fn move_number(&self) -> &i32 {
        &self.move_number
    }
}

impl Default for Fen {
    fn default() -> Self {
        Fen::from_string(STARTING_FEN)
    }
}

#[cfg(test)]
mod tests {
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
}
