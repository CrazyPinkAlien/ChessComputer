use bevy::prelude::{Color, Transform, Vec3, Vec2, default, Res, Resource, FromWorld};
use bevy::ecs::system::Commands;
use bevy::sprite::{Sprite, SpriteBundle};

pub mod square;

#[derive(Resource)]
pub struct BoardProperties {
    color_white: Color,
    color_black: Color,
    center: Vec3,
    square_size: i32
}

// TODO: Change to Default?
impl FromWorld for BoardProperties {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        return BoardProperties {
            color_white: Color::WHITE,
            color_black: Color::BLACK,
            center: Vec3::new(0., 0., 0.),
            square_size: 60,
        };
    }
}

pub fn setup(mut commands: Commands, properties: Res<BoardProperties>) {
    let mut squares = Vec::with_capacity(64);
    for i in 0..64 {
        let color = if ((i % 8) % 2 == 0) == ((i / 8) % 2 == 0) {
            properties.color_black
        } else {
            properties.color_white
        };
        let x = (i % 8 - 4) * properties.square_size;
        let y = (i / 8 - 4) * properties.square_size;
        let sprite_bundle = SpriteBundle {
            sprite: Sprite {
                color: color,
                custom_size: Some(Vec2::new(properties.square_size as f32, properties.square_size as f32)),
                ..default()
            },
            transform: Transform::from_translation(properties.center + Vec3::new(x as f32, y as f32, 0.)),
            ..default()
        };
        squares.push(square::SquareBundle {
            sprite_bundle: sprite_bundle,
        });
    }
    commands.spawn_batch(squares);
}