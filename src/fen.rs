use std::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct Fen {
    piece_placement: String,
    active_color: String,
}

impl Fen {
    fn from_string(fen_string: &str) -> Self {
        // First split fen into sections separated by spaces
        let split_fen = fen_string.split_whitespace().collect::<Vec<&str>>();
        // Get piece_info placement data
        let piece_placement = split_fen[0];
        // Get active color
        let active_color = split_fen[1];
        // Create Fen object
        Fen {
            piece_placement: piece_placement.to_string(),
            active_color: active_color.to_string(),
        }
    }

    pub fn from_file(filename: &str) -> Self {
        let fen_string = read_to_string(filename).expect("Starting FEN not found.");
        Fen::from_string(&fen_string)
    }

    pub fn piece_placement(&self) -> &String {
        &self.piece_placement
    }

    pub fn active_color(&self) -> &String {
        &self.active_color
    }
}
