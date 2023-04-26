use bevy::ecs::system::Commands;
use bevy::prelude::{Color, Component, FromWorld, Res, Resource, Vec2};

mod square;

#[derive(Resource)]
pub struct BoardProperties {
    color_white: Color,
    color_black: Color,
    center: Vec2,
    square_size: f32,
}

impl BoardProperties {
    pub fn position_to_transform(&self, position: BoardPosition) -> (f32, f32) {
        let x = (position.file as f32 - 4.0) * self.square_size + self.center.x;
        let y = -1.0 * (position.rank as f32 - 4.0) * self.square_size + self.center.y;
        (x, y)
    }

    pub fn transform_to_position(&self, transform: Vec2) -> Option<BoardPosition> {
        let file = ((transform[0] - self.center.x) / self.square_size + 4.0).round() as i32;
        let rank = (-1.0 * (transform[1] - self.center.y) / self.square_size + 4.0).round() as i32;
        if (rank < 0) || (rank > 7) || (file < 0) || (file > 7) {
            None
        } else {
            Some(BoardPosition {
                rank: (rank as usize),
                file: (file as usize),
            })
        }
    }

    fn position_to_color(&self, position: BoardPosition) -> Color {
        if (position.rank % 2 == 0) == (position.file % 2 == 0) {
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
            color_black: Color::GRAY,
            center: Vec2::new(0., 0.),
            square_size: 80.,
        }
    }
}

#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub struct BoardPosition {
    pub rank: usize,
    pub file: usize,
}

impl BoardPosition {
    pub(super) fn new(rank: usize, file: usize) -> Self {
        if (rank > 8) | (file > 8) {
            panic!("Invalid rank or file value: {}, {}", rank, file)
        }
        BoardPosition { rank, file }
    }
}

pub fn setup(mut commands: Commands, properties: Res<BoardProperties>) {
    let mut squares = Vec::with_capacity(64);
    for rank in 0..8 {
        for file in 0..8 {
            let position = BoardPosition { rank, file };
            let color = properties.position_to_color(position);
            let (x, y) = properties.position_to_transform(position);
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
