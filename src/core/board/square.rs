use bevy::prelude::{default, Bundle, Color, Transform, Vec2, Component};
use bevy::sprite::{Sprite, SpriteBundle};

#[derive(Component)]
struct Square;

#[derive(Bundle)]
pub(super) struct SquareBundle {
    _p: Square,
    
    #[bundle]
    sprite: SpriteBundle,
}

impl SquareBundle {
    pub(super) fn new(x: f32, y: f32, size: f32, color: Color) -> Self {
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
            _p: Square,
            sprite: sprite_bundle,
        }
    }
}
