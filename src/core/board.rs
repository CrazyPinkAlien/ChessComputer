use bevy::ecs::system::Commands;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::{Color, Component, FromWorld, Res, Resource, Vec2};

use super::piece::Piece;

mod square;

#[derive(Resource)]
pub struct BoardProperties {
    color_white: Color,
    color_black: Color,
    center: Vec2,
    square_size: f32,
}

impl BoardProperties {
    pub(super) fn position_to_transform(&self, position: BoardPosition) -> (f32, f32) {
        let x = (position.file as f32 - 4.0) * self.square_size + self.center.x;
        let y = -1.0 * (position.rank as f32 - 4.0) * self.square_size + self.center.y;
        (x, y)
    }

    pub(super) fn transform_to_position(&self, transform: Vec2) -> Option<BoardPosition> {
        let file = ((transform[0] - self.center.x) / self.square_size + 4.0).round() as i32;
        let rank = (-1.0 * (transform[1] - self.center.y) / self.square_size + 4.0).round() as i32;
        if (rank < 0) || (rank > 7) || (file < 0) || (file > 7) {
            None
        } else {
            Some(BoardPosition {
                rank: (rank as u32),
                file: (file as u32),
            })
        }
    }

    fn position_to_color(&self, rank: u32, file: u32) -> Color {
        if (rank % 2 == 0) == (file % 2 == 0) {
            self.color_white
        } else {
            self.color_black
        }
    }
}

impl FromWorld for BoardProperties {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        BoardProperties {
            color_white: Color::WHITE,
            color_black: Color::DARK_GRAY,
            center: Vec2::new(0., 0.),
            square_size: 80.,
        }
    }
}

#[derive(Resource)]
pub struct BoardState {
    board: Vec<Vec<Option<Piece>>>
}

impl FromWorld for BoardState {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        // Row of the array
        let mut row = Vec::new();
        row.resize(8, None);
        // Fill board with these rows
        let mut board = Vec::new();
        board.resize(8, row.clone());
        BoardState {
            board
        }
    }
}

impl BoardState {
    pub(crate) fn add(&mut self, piece: Piece, position: BoardPosition) {
        let rank = position.rank as usize;
        let file = position.file as usize;
        self.board[rank][file] = Some(piece);
    }

    pub(crate) fn remove(&mut self, position: BoardPosition) {
        let rank = position.rank as usize;
        let file = position.file as usize;
        self.board[rank][file] = None;
    }
}

#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub(crate) struct BoardPosition {
    rank: u32,
    file: u32,
}

impl BoardPosition {
    pub(super) fn new(rank: u32, file: u32) -> Self {
        if (rank > 8) | (file > 8) {
            panic!("Invalid rank or file value: {}, {}", rank, file)
        }
        BoardPosition { rank, file }
    }
}

pub(crate) struct BoardClickEvent {
    pub(crate) position: BoardPosition,
    pub(crate) input: MouseButtonInput,
}

pub(super) fn setup(mut commands: Commands, properties: Res<BoardProperties>) {
    let mut squares = Vec::with_capacity(64);
    for rank in 0..8 {
        for file in 0..8 {
            let color = properties.position_to_color(rank, file);
            let (x, y) = properties.position_to_transform(BoardPosition { rank, file });
            squares.push(square::SquareBundle::new(
                x,
                y,
                properties.square_size,
                color,
            ));
        }
    }
    commands.spawn_batch(squares);
}
