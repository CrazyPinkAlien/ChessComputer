use bevy::ecs::system::Commands;
use bevy::prelude::{Color, Component, FromWorld, Res, Resource, Vec2, MouseButton};

mod square;

#[derive(Resource)]
pub struct BoardProperties {
    color_white: Color,
    color_black: Color,
    center: Vec2,
    square_size: f32,
}

impl BoardProperties {
    pub(super) fn position_to_transform(&self, rank: u32, file: u32) -> (f32, f32) {
        let x = (file as f32 - 4.0) * self.square_size + self.center.x;
        let y = -1.0 * (rank as f32 - 4.0) * self.square_size + self.center.y;
        (x, y)
    }

    pub(super) fn transform_to_position(&self, transform: Vec2) -> Option<BoardPosition> {
        let rank = ((transform[0] - self.center.x) / self.square_size + 4.0).round() as u32;
        let file = (-1.0 * (transform[1] - self.center.y) / self.square_size + 4.0).round() as u32;
        if (rank < 0) || (rank > 7) || (file < 0) || (file > 7) {
            None
        } else {
            Some(BoardPosition { rank, file } )
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

#[derive(Component)]
pub(crate) struct BoardPosition {
    pub(crate) rank: u32,
    pub(crate) file: u32,
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
    pub(crate) input: MouseButton,
}

pub(super) fn setup(mut commands: Commands, properties: Res<BoardProperties>) {
    let mut squares = Vec::with_capacity(64);
    for rank in 0..8 {
        for file in 0..8 {
            let color = properties.position_to_color(rank, file);
            let (x, y) = properties.position_to_transform(rank, file);
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
