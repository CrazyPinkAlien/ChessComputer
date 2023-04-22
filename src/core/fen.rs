use std::fs::read_to_string;

pub struct Fen {
    pub piece_placement: String,
}

impl Fen {
    fn from_string(fen_string: String) -> Self {
        // First split fen into sections separated by spaces
        let split_fen = fen_string.split_whitespace().collect::<Vec<&str>>();
        // Get piece_info placement data
        let piece_placement = split_fen[0];
        // Create Fen object
        Fen {
            piece_placement: piece_placement.to_string(),
        }
    }

    pub fn from_file(filename: &str) -> Self {
        let fen_string = read_to_string(filename).expect("Starting FEN not found.");
        Fen::from_string(fen_string)
    }
}
