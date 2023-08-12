use bevy::ecs::system::Commands;
use bevy::prelude::{Changed, Color, Query, Res, Resource, Vec2, With};
use bevy::sprite::Sprite;

use crate::chess_board::r#move::Move;
use crate::chess_board::{BoardPosition, ChessBoard, PieceColor};

use super::piece::{Dragging, PieceTag};

mod square;

#[derive(Resource)]
pub(super) struct BoardProperties {
    color_white: Color,
    color_black: Color,
    highlight_color_white: Color,
    highlight_color_black: Color,
    center: Vec2,
    square_size: f32,
}

impl BoardProperties {
    pub(super) fn position_to_transform(&self, position: BoardPosition) -> (f32, f32) {
        let x = (position.file() as f32 - 4.0) * self.square_size + self.center.x;
        let y = -1.0 * (position.rank() as f32 - 4.0) * self.square_size + self.center.y;
        (x, y)
    }

    pub(super) fn transform_to_position(&self, transform: Vec2) -> Option<BoardPosition> {
        let file = ((transform[0] - self.center.x) / self.square_size + 4.0).round() as i32;
        let rank = (-1.0 * (transform[1] - self.center.y) / self.square_size + 4.0).round() as i32;
        if !(0..=7).contains(&rank) || !(0..=7).contains(&file) {
            None
        } else {
            Some(BoardPosition::new(rank as usize, file as usize))
        }
    }

    fn position_to_color(&self, position: BoardPosition) -> PieceColor {
        if (position.rank() % 2 == 0) == (position.file() % 2 == 0) {
            PieceColor::White
        } else {
            PieceColor::Black
        }
    }
}

impl Default for BoardProperties {
    fn default() -> Self {
        BoardProperties {
            color_white: Color::WHITE,
            color_black: Color::GRAY,
            highlight_color_white: Color::AQUAMARINE,
            highlight_color_black: Color::TEAL,
            center: Vec2::new(0., 0.),
            square_size: 80.,
        }
    }
}

pub(super) fn setup(mut commands: Commands, properties: Res<BoardProperties>) {
    let mut squares = Vec::with_capacity(64);
    for rank in 0..8 {
        for file in 0..8 {
            let position = BoardPosition::new(rank, file);
            let color = properties.position_to_color(position);
            squares.push(square::SquareBundle::new(
                position,
                properties.square_size,
                color,
                &properties,
            ));
        }
    }
    commands.spawn_batch(squares);
}

pub(super) fn highlight_valid_squares(
    piece_query: Query<(&BoardPosition, &Dragging), (Changed<Dragging>, With<PieceTag>)>,
    mut square_query: Query<
        (&mut Sprite, &BoardPosition, &square::SquareColor),
        With<square::Square>,
    >,
    board: Res<ChessBoard>,
    properties: Res<BoardProperties>,
) {
    for (piece_position, dragging) in piece_query.iter() {
        for (mut sprite, position, color) in square_query.iter_mut() {
            // Highlight the square if it's valid
            let potential_move = Move::new(*piece_position, *position);
            let sprite_color = if dragging.get() && board.valid_move(potential_move, true) {
                match color.get() {
                    PieceColor::White => properties.highlight_color_white,
                    PieceColor::Black => properties.highlight_color_black,
                }
            } else {
                match color.get() {
                    PieceColor::White => properties.color_white,
                    PieceColor::Black => properties.color_black,
                }
            };
            sprite.color = sprite_color;
        }
    }
}
