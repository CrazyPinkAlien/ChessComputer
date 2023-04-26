use bevy::ecs::component::TableStorage;
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
use dyn_clone::DynClone;

use crate::ui::MainCamera;

use self::pawn::Pawn;

use super::board::{BoardPosition, BoardProperties};
use super::fen::Fen;
use super::state::BoardState;
use super::{SetupBoardEvent, BoardClickEvent};

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

trait Piece: Send + Sync + DynClone + 'static + Component<Storage = TableStorage> {
    fn get_type(&self) -> PieceType;
    fn get_color(&self) -> PieceColor;
    fn get_position(&self) -> BoardPosition;
    fn set_position(&mut self, new_position: BoardPosition);
    fn get_moves(&self) -> Vec<BoardPosition>;
}

#[derive(Component)]
pub struct PieceInfo {
    piece: Box<dyn Piece>,
}

impl PieceInfo {
    pub fn new(piece_color: PieceColor, piece_type: PieceType, position: BoardPosition) -> Self {
        let piece = match piece_type {
            _ => Pawn::new(position, piece_color)
        };
        PieceInfo { piece }
    }
}

#[derive(Component)]
pub struct Dragging(bool);

#[derive(Bundle)]
pub(super) struct PieceBundle {
    piece: PieceInfo,
    dragging: Dragging,

    #[bundle]
    sprite: SpriteSheetBundle,
}

impl PieceBundle {
    fn new(
        piece_type: PieceType,
        piece_color: PieceColor,
        texture_atlas_handle: &Handle<TextureAtlas>,
        board_position: BoardPosition,
        piece_properties: &Res<PieceProperties>,
        board_properties: &Res<BoardProperties>,
    ) -> Self {
        let sprite_sheet_index = (piece_type as u8) + 6 * (piece_color as u8);
        let (x, y) = board_properties.position_to_transform(board_position);
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(sprite_sheet_index.into()),
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x, y, 1.)
                .with_scale(Vec3::splat(piece_properties.scale)),
            ..default()
        };
        PieceBundle {
            piece: PieceInfo::new(piece_color, piece_type, board_position),
            dragging: Dragging(false),
            sprite: sprite,
        }
    }
}

fn move_piece(transform: &mut Transform, piece: &mut PieceInfo, new_position: BoardPosition, board_properties: &Res<BoardProperties>, board_state: &mut ResMut<BoardState>) {
    // Update the board state
    board_state.move_piece(piece.piece.get_position(), new_position);
    // Change its position
    piece.piece.set_position(new_position);
    // Change its transform
    let new_transform = board_properties.position_to_transform(new_position);
    *transform = transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
}

fn remove_piece(piece: &mut PieceInfo, visibility: &mut Visibility, board_state: &mut ResMut<BoardState>) {
    // Update the board state
    board_state.remove_piece(piece.piece.get_position());
    // Make it invisible
    *visibility = Visibility::INVISIBLE;
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
    let mut pieces: Vec<PieceBundle> = Vec::with_capacity(32);
    for piece_color in PieceColor::iter() {
        for (index, piece_type) in PieceType::iter().enumerate() {
            for _number in 0..piece_properties.spawn_numbers[index] {
                pieces.push(PieceBundle::new(
                    piece_type,
                    piece_color,
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
    mut query: Query<(&mut PieceInfo, &mut Transform, &mut Visibility)>,
    properties: Res<BoardProperties>,
    mut state: ResMut<BoardState>
) {
    // Create array of bools to track which squares have been populated
    let mut populated = [[false; 8]; 8];
    for &event in setup_events.iter() {
        for (mut piece, mut transform, mut visibility) in query.iter_mut() {
            *visibility = Visibility::INVISIBLE;
            'outer: for rank in 0..8 {
                for file in 0..8 {
                    if (event.state.board[rank][file].is_some())
                        && (event.state.board[rank][file].unwrap().0 == piece.piece.get_color())
                        && (event.state.board[rank][file].unwrap().1 == piece.piece.get_type())
                        && !populated[rank][file]
                    {
                        let new_position = BoardPosition::new(rank, file);
                        move_piece(transform.as_mut(), piece.as_mut(), new_position, &properties, &mut state);
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
        &mut PieceInfo,
        &mut Transform,
        &mut Dragging,
    )>,
    properties: Res<BoardProperties>,
    mut state: ResMut<BoardState>
) {
    for click in board_click_events.iter() {
        for (mut visibility, mut piece, mut piece_transform, mut dragging) in
            query.iter_mut()
        {
            match click.input.button {
                MouseButton::Left => {
                    if click.input.state == ButtonState::Pressed {
                        if (click.position.is_some())
                            && (click.position.unwrap() == piece.piece.get_position())
                        {
                            // Start dragging the piece
                            dragging.0 = true;
                        } else {
                            dragging.0 = false;
                        }
                    } else if click.input.state == ButtonState::Released {
                        if (click.position.is_some()) && (dragging.0) {
                            // When the button is released move the piece to that square
                            move_piece(piece_transform.as_mut(), piece.as_mut(), click.position.unwrap(), &properties, &mut state);
                            dragging.0 = false;
                        } else if click.position.is_some() {
                            // Take any pieces that were already there
                            if click.position.unwrap() == piece.piece.get_position() {
                                remove_piece(piece.as_mut(), visibility.as_mut(), &mut state);
                            }
                        } else {
                            // Stop dragging and return the piece to its original position
                            dragging.0 = false;
                            let new_position = piece.piece.get_position().clone();
                            move_piece(piece_transform.as_mut(), piece.as_mut(), new_position, &properties, &mut state);
                        }
                    }
                }
                MouseButton::Right => {
                    // If the right button was clicked, stop dragging and return the piece to its original position
                    if click.input.state == ButtonState::Pressed {
                        dragging.0 = false;
                        let new_position = piece.piece.get_position().clone();
                        move_piece(piece_transform.as_mut(), piece.as_mut(), new_position, &properties, &mut state)
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
            // Check if this piece is selected
            if dragging.0 == true {
                // Move this piece to follow the mouse
                *transform = transform.with_translation(Vec3::new(
                    world_position[0],
                    world_position[1],
                    2.0,
                ));
            }
        }
    }
}
