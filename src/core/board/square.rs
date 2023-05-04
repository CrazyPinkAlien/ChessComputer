use bevy::prelude::{default, Bundle, Component, Res, Transform, Vec2};
use bevy::sprite::{Sprite, SpriteBundle};

use crate::core::piece::PieceColor;

use super::{BoardPosition, BoardProperties};

#[derive(Component)]
pub struct Square;

#[derive(Component)]
pub struct SquareColor(pub PieceColor);

#[derive(Bundle)]
pub struct SquareBundle {
    _p: Square,
    position: BoardPosition,
    color: SquareColor,

    #[bundle]
    sprite: SpriteBundle,
}

impl SquareBundle {
    pub fn new(
        position: BoardPosition,
        size: f32,
        color: PieceColor,
        properties: &Res<BoardProperties>,
    ) -> Self {
        let (x, y) = properties.position_to_transform(position);
        let square_color = match color {
            PieceColor::Black => properties.color_black,
            PieceColor::White => properties.color_white,
        };
        let sprite_bundle = SpriteBundle {
            sprite: Sprite {
                color: square_color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 0.),
            ..default()
        };
        SquareBundle {
            _p: Square,
            position,
            color: SquareColor(color),
            sprite: sprite_bundle,
        }
    }
}
