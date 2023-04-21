use std::fs::read_to_string;

use bevy::input::ButtonState;
use bevy::prelude::{
    default, AssetServer, Assets, Bundle, Camera, Changed, Commands, Component, EventReader,
    FromWorld, GlobalTransform, Handle, MouseButton, Query, Res, ResMut, Resource, Transform, Vec2,
    Vec3, With,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::window::Windows;

use crate::ui::MainCamera;

use super::board::{BoardClickEvent, BoardPosition, BoardProperties};
use super::Selected;

enum PieceColor {
    White,
    Black,
}

enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Resource)]
pub struct PieceProperties {
    scale: f32,
}

impl FromWorld for PieceProperties {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        PieceProperties { scale: 0.25 }
    }
}

#[derive(Component)]
pub(crate) struct Piece;

#[derive(Bundle)]
pub(super) struct PieceBundle {
    _p: Piece,
    position: BoardPosition,
    selected: Selected,

    #[bundle]
    sprite: SpriteSheetBundle,
}

impl PieceBundle {
    fn new(
        piece_color: PieceColor,
        piece_type: PieceType,
        texture_atlas_handle: &Handle<TextureAtlas>,
        rank: u32,
        file: u32,
        piece_properties: &Res<PieceProperties>,
        board_properties: &Res<BoardProperties>,
    ) -> Self {
        let sprite_sheet_index = (piece_type as u8) + 6 * (piece_color as u8);
        let (x, y) = board_properties.position_to_transform(rank, file);
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(sprite_sheet_index.into()),
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x, y, 1.)
                .with_scale(Vec3::splat(piece_properties.scale)),
            ..default()
        };
        PieceBundle {
            _p: Piece,
            selected: Selected(false),
            sprite: sprite,
            position: BoardPosition::new(rank, file),
        }
    }
}

pub(super) fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    board_properties: Res<BoardProperties>,
    piece_properties: Res<PieceProperties>,
) {
    // Load sprite sheet
    let texture_handle = asset_server.load("sprites/pieces.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(333.33334, 333.5),
        6,
        2,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // Load starting position FEN
    let fen = read_to_string("./assets/starting_position.fen").expect("Starting FEN not found.");

    // Instantiate pieces
    let mut pieces = Vec::with_capacity(32);
    // First split fen into sections separated by spaces
    let split_fen = fen.split_whitespace().collect::<Vec<&str>>();
    // Get piece placement data
    let piece_placement = split_fen[0];
    let mut rank = 0;
    let mut file = 0;
    for rank_str in piece_placement.split("/") {
        for symbol in rank_str.chars().collect::<Vec<char>>() {
            if symbol.is_digit(9) {
                file += symbol.to_digit(9).unwrap();
            } else {
                let piece_color = if symbol.is_uppercase() {
                    PieceColor::White
                } else {
                    PieceColor::Black
                };
                let piece_type = match symbol.to_uppercase().next().unwrap() {
                    'P' => PieceType::Pawn,
                    'N' => PieceType::Knight,
                    'B' => PieceType::Bishop,
                    'R' => PieceType::Rook,
                    'Q' => PieceType::Queen,
                    'K' => PieceType::King,
                    _ => panic!("Unrecognised symbol in FEN: {}", symbol),
                };
                pieces.push(PieceBundle::new(
                    piece_color,
                    piece_type,
                    &texture_atlas_handle,
                    rank,
                    file,
                    &piece_properties,
                    &board_properties,
                ));
                file += 1;
            }
            if file >= 8 {
                rank += 1;
                file = 0;
            };
        }
    }
    commands.spawn_batch(pieces);
}

pub(super) fn handle_piece_clicks(
    mut board_click_events: EventReader<BoardClickEvent>,
    mut query: Query<(&BoardPosition, &mut Selected), With<Piece>>,
) {
    for click in board_click_events.iter() {
        for (piece_position, mut selected) in query.iter_mut() {
            // Check if this piece was clicked on and the button was just pressed
            if (click.position == *piece_position) && (click.input.state == ButtonState::Pressed) {
                match click.input.button {
                    MouseButton::Left => {
                        // If the left button was clicked, select it
                        selected.0 = true;
                    }
                    MouseButton::Right => {
                        // If the right button was clicked, deselect it
                        selected.0 = false;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
    }
}

pub(super) fn selected_piece(
    mut query: Query<(&Selected, &mut Transform), (With<Piece>, Changed<Selected>)>,
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // Get window and camera
    let window = windows.get_primary().unwrap();
    let (camera, camera_transform) = camera.single();
    // Check if the cursor is in the window
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        for (selected, mut transform) in query.iter_mut() {
            // Check if this piece is selected
            if selected.0 == true {
                // Move this piece to follow the mouse
                *transform = transform.with_translation(Vec3::new(
                    world_position[0],
                    world_position[1],
                    transform.translation.x,
                ));
            }
        }
    }
}
