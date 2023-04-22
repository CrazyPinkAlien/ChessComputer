use std::fs::read_to_string;

use bevy::input::ButtonState;
use bevy::prelude::{
    default, AssetServer, Assets, Bundle, Camera, Commands, Component, EventReader, FromWorld,
    GlobalTransform, Handle, MouseButton, Query, Res, ResMut, Resource, Transform, Vec2, Vec3,
    With, Visibility,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::window::Windows;

use crate::ui::MainCamera;

use super::board::{BoardClickEvent, BoardPosition, BoardProperties, BoardState};

#[derive(Clone, Copy)]
enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy)]
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

#[derive(Component, Clone, Copy)]
pub(crate) struct Piece {
    piece_color: PieceColor,
    piece_type: PieceType,
}

#[derive(Component)]
pub(crate) struct Dragging(bool);

#[derive(Bundle)]
pub(super) struct PieceBundle {
    piece: Piece,
    position: BoardPosition,
    dragging: Dragging,

    #[bundle]
    sprite: SpriteSheetBundle,
}

impl PieceBundle {
    fn new(
        piece: Piece,
        texture_atlas_handle: &Handle<TextureAtlas>,
        board_position: BoardPosition,
        piece_properties: &Res<PieceProperties>,
        board_properties: &Res<BoardProperties>,
        board_state: &mut ResMut<BoardState>
    ) -> Self {
        let sprite_sheet_index = (piece.piece_type as u8) + 6 * (piece.piece_color as u8);
        let (x, y) = board_properties.position_to_transform(board_position);
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(sprite_sheet_index.into()),
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x, y, 1.)
                .with_scale(Vec3::splat(piece_properties.scale)),
            ..default()
        };
        // Place the piece on the board
        board_state.add(piece, board_position);
        //board_state
        PieceBundle {
            piece: piece,
            dragging: Dragging(false),
            sprite: sprite,
            position: board_position,
        }
    }
}

pub(super) fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    board_properties: Res<BoardProperties>,
    piece_properties: Res<PieceProperties>,
    mut board_state: ResMut<BoardState>
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
                    Piece { piece_color, piece_type },
                    &texture_atlas_handle,
                    BoardPosition::new(rank, file),
                    &piece_properties,
                    &board_properties,
                    &mut board_state
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
    mut query: Query<
            (&mut Visibility,
            &mut BoardPosition,
            &mut Transform,
            &mut Dragging,
            &Piece)
    >,
    properties: Res<BoardProperties>,
    mut state: ResMut<BoardState>,
) {
    for click in board_click_events.iter() {
        for (mut visibility, mut piece_position, mut piece_transform, mut dragging, piece) in
            query.iter_mut()
        {
            match click.input.button {
                MouseButton::Left => {
                    if click.input.state == ButtonState::Pressed {
                        if click.position == *piece_position {
                            // Start dragging the piece
                            dragging.0 = true;
                        } else {
                            dragging.0 = false;
                        }
                    } else if click.input.state == ButtonState::Released {
                        if dragging.0 {
                            // When the button is released move the piece to that square
                            move_piece(
                                piece,
                                &mut piece_position,
                                &mut piece_transform,
                                click.position,
                                &properties,
                                &mut state,
                            );
                            dragging.0 = false;
                        } else {
                            // Take any pieces that were already there
                            if click.position == *piece_position {
                                take_piece(visibility.as_mut(), &click.position, &mut state);
                            }
                        }
                    }
                }
                MouseButton::Right => {
                    // If the right button was clicked, stop dragging and return the piece to its original position
                    if click.input.state == ButtonState::Pressed {
                        dragging.0 = false;
                        let new_position = piece_position.clone();
                        move_piece(
                            piece,
                            &mut piece_position,
                            &mut piece_transform,
                            new_position,
                            &properties,
                            &mut state
                        );
                    }
                }
                _ => {
                    break;
                }
            }
        }
    }
}

pub(super) fn dragged_piece(
    mut query: Query<(&Dragging, &mut Transform), With<Piece>>,
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
        for (dragging, mut transform) in query.iter_mut() {
            // Check if this piece is selected
            if dragging.0 == true {
                // Move this piece to follow the mouse
                *transform = transform.with_translation(Vec3::new(
                    world_position[0],
                    world_position[1],
                    transform.translation.z,
                ));
            }
        }
    }
}

fn move_piece(
    piece: &Piece,
    position: &mut BoardPosition,
    transform: &mut Transform,
    new_position: BoardPosition,
    properties: &Res<BoardProperties>,
    state: &mut ResMut<BoardState>,
) {
    // Change the board state
    // Remove the piece from its old position
    state.remove(*position);
    // Add the piece at the new position
    state.add(piece.clone(), new_position);

    // Change its board position
    *position = new_position;

    // Change its transform
    let new_transform = properties.position_to_transform(new_position);
    *transform = transform.with_translation(Vec3::new(
        new_transform.0,
        new_transform.1,
        transform.translation.z,
    ));
}

fn take_piece(
    visibility: &mut Visibility,
    position: &BoardPosition,
    state: &mut ResMut<BoardState>,
) {
    // Update the board state
    state.remove(*position);
    // Make the piece invisible
    *visibility = Visibility::INVISIBLE;
}
