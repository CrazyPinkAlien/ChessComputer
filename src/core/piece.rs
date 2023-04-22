use bevy::input::ButtonState;
use bevy::prelude::{
    default, AssetServer, Assets, Bundle, Camera, Commands, Component, EventReader, EventWriter,
    FromWorld, GlobalTransform, Handle, MouseButton, Query, Res, ResMut, Resource, Transform, Vec2,
    Vec3, Visibility, With,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::window::Windows;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::ui::MainCamera;

use self::pawn::Pawn;

use super::board::{BoardClickEvent, BoardPosition, BoardProperties};
use super::fen::Fen;
use super::state::BoardState;
use super::SetupBoardEvent;

mod pawn;

#[derive(Resource)]
pub struct PieceProperties {
    scale: f32,
    spawn_numbers: [u32; 6],
}

impl FromWorld for PieceProperties {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        PieceProperties {
            scale: 0.25,
            spawn_numbers: [1, 1, 2, 2, 2, 8],
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Component, Clone, Copy, PartialEq)]
pub struct PieceInfo {
    piece_color: PieceColor,
    piece_type: PieceType,
}

impl PieceInfo {
    pub fn new(piece_color: PieceColor, piece_type: PieceType) -> Self {
        PieceInfo {
            piece_color,
            piece_type,
        }
    }
}

#[derive(Component)]
pub struct Dragging(bool);

#[derive(Bundle)]
pub(super) struct PieceBundle<T: Piece + Sync + Send + 'static + Component> {
    piece: T,
    piece_info: PieceInfo,
    dragging: Dragging,

    #[bundle]
    sprite: SpriteSheetBundle,
}

impl<T: Piece + Sync + Send + 'static + Component> PieceBundle<T> {
    fn new(
        piece_info: PieceInfo,
        texture_atlas_handle: &Handle<TextureAtlas>,
        board_position: BoardPosition,
        piece_properties: &Res<PieceProperties>,
        board_properties: &Res<BoardProperties>,
    ) -> Self {
        let sprite_sheet_index = (piece_info.piece_type as u8) + 6 * (piece_info.piece_color as u8);
        let (x, y) = board_properties.position_to_transform(board_position);
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(sprite_sheet_index.into()),
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x, y, 1.)
                .with_scale(Vec3::splat(piece_properties.scale)),
            ..default()
        };
        PieceBundle {
            piece: piece_from_type(piece_info.piece_type),
            piece_info: piece_info,
            dragging: Dragging(false),
            sprite: sprite,
        }
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    board_properties: Res<BoardProperties>,
    piece_properties: Res<PieceProperties>,
    mut setup_events: EventWriter<SetupBoardEvent>,
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
    // Add the piece_info textures to the texture atlas
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // Instantiate pieces
    let mut pieces: Vec<PieceBundle<_>> = Vec::with_capacity(32);
    for piece_color in PieceColor::iter() {
        for (index, piece_type) in PieceType::iter().enumerate() {
            for _number in 0..piece_properties.spawn_numbers[index] {
                pieces.push(PieceBundle::<T: Piece + Sync + Send + 'static + Component>::new(
                    PieceInfo {
                        piece_color,
                        piece_type,
                    },
                    &texture_atlas_handle,
                    BoardPosition::new(0, 0),
                    &piece_properties,
                    &board_properties,
                ));
            }
        }
    }
    commands.spawn_batch(pieces);
    // Load starting board state
    let board_state = BoardState::from_fen(Fen::from_file("assets/fens/starting_position.fen"));
    // Setup the board
    setup_events.send(SetupBoardEvent { state: board_state });
}

pub fn setup_pieces(
    mut setup_events: EventReader<SetupBoardEvent>,
    mut query: Query<(&PieceInfo, &mut BoardPosition, &mut Transform, &mut Visibility)>,
    properties: Res<BoardProperties>,
) {
    // Create array of bools to track which squares have been populated
    let mut row = Vec::new();
    row.resize(8, false);
    let mut populated = Vec::new();
    populated.resize(8, row.clone());
    for event in setup_events.iter() {
        for (piece_info, mut position, mut transform, mut visibility) in query.iter_mut() {
            *visibility = Visibility::INVISIBLE;
            'outer: for rank in 0..8 {
                for file in 0..8 {
                    if (event.state.board[rank][file].is_some())
                        && (event.state.board[rank][file].unwrap() == *piece_info)
                        && !populated[rank][file]
                    {
                        let new_position = BoardPosition::new(rank as u32, file as u32);
                        move_piece(&mut position, transform.as_mut(), new_position, &properties);
                        *visibility = Visibility::VISIBLE;
                        populated[rank][file] = true;
                        break 'outer;
                    }
                }
            }
        }
    }
}

pub fn handle_piece_clicks(
    mut board_click_events: EventReader<BoardClickEvent>,
    mut query: Query<(
        &mut Visibility,
        &mut BoardPosition,
        &mut Transform,
        &mut Dragging,
    )>,
    properties: Res<BoardProperties>,
) {
    for click in board_click_events.iter() {
        for (mut visibility, mut piece_position, mut piece_transform, mut dragging) in
            query.iter_mut()
        {
            match click.input.button {
                MouseButton::Left => {
                    if click.input.state == ButtonState::Pressed {
                        if (click.position.is_some())
                            && (click.position.unwrap() == *piece_position)
                        {
                            // Start dragging the piece_info
                            dragging.0 = true;
                        } else {
                            dragging.0 = false;
                        }
                    } else if click.input.state == ButtonState::Released {
                        if (click.position.is_some()) && (dragging.0) {
                            // When the button is released move the piece_info to that square
                            move_piece(
                                &mut piece_position,
                                &mut piece_transform,
                                click.position.unwrap(),
                                &properties,
                            );
                            dragging.0 = false;
                        } else if click.position.is_some() {
                            // Take any pieces that were already there
                            if click.position.unwrap() == *piece_position {
                                take_piece(visibility.as_mut());
                            }
                        } else {
                            // Stop dragging and return the piece_info to its original position
                            dragging.0 = false;
                            let new_position = piece_position.clone();
                            move_piece(
                                &mut piece_position,
                                &mut piece_transform,
                                new_position,
                                &properties,
                            );
                        }
                    }
                }
                MouseButton::Right => {
                    // If the right button was clicked, stop dragging and return the piece_info to its original position
                    if click.input.state == ButtonState::Pressed {
                        dragging.0 = false;
                        let new_position = piece_position.clone();
                        move_piece(
                            &mut piece_position,
                            &mut piece_transform,
                            new_position,
                            &properties,
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

pub fn dragged_piece(
    mut query: Query<(&Dragging, &mut Transform), With<PieceInfo>>,
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
            // Check if this piece_info is selected
            if dragging.0 == true {
                // Move this piece_info to follow the mouse
                *transform = transform.with_translation(Vec3::new(
                    world_position[0],
                    world_position[1],
                    2.0,
                ));
            }
        }
    }
}

fn move_piece(
    position: &mut BoardPosition,
    transform: &mut Transform,
    new_position: BoardPosition,
    properties: &Res<BoardProperties>,
) {
    // Change its board position
    *position = new_position;

    // Change its transform
    let new_transform = properties.position_to_transform(new_position);
    *transform = transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
}

fn take_piece(visibility: &mut Visibility) {
    // Make the piece_info invisible
    *visibility = Visibility::INVISIBLE;
}

trait Piece {
    fn new() -> Self;
    fn get_moves(&self) -> Vec<BoardPosition>;
}

fn piece_from_type(piece_type: PieceType) -> impl Piece {
    match piece_type {
        _ => Pawn::new()
    }
}
