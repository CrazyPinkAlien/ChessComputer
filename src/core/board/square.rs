use bevy::prelude::{default, Bundle, Color, Transform, Vec2};
use bevy::sprite::{Sprite, SpriteBundle};

#[derive(Bundle)]
pub struct SquareBundle {
    #[bundle]
    sprite: SpriteBundle,
}

impl SquareBundle {
    pub fn new(x: f32, y: f32, size: f32, color: Color) -> Self {
        let sprite_bundle = SpriteBundle {
            sprite: Sprite {
                color: color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 0.),
            ..default()
        };
        SquareBundle {
            sprite: sprite_bundle,
        }
    }
}
