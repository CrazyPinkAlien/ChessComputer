use bevy::ecs::component::TableStorage;
use bevy::input::ButtonState;
use bevy::prelude::{
    default, info, AssetServer, Assets, Bundle, Camera, Commands, Component, EventReader,
    EventWriter, GlobalTransform, Handle, MouseButton, Query, Res, ResMut, Transform, Vec2, Vec3,
    Visibility, With,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::window::Windows;
use bevy_kira_audio::{Audio, AudioControl};
use dyn_clone::DynClone;
use strum_macros::EnumIter;

use crate::ui::MainCamera;

use self::bishop::Bishop;
use self::king::King;
use self::knight::Knight;
use self::pawn::Pawn;
use self::queen::Queen;
use self::rook::Rook;

use super::board::{BoardPosition, BoardProperties};
use super::move_checker::is_legal_move;
use super::state::BoardState;
use super::{BoardClickEvent, ResetBoardEvent, BOARD_SIZE};

mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
mod rook;

const PIECE_MOVE_VOLUME: f64 = 0.75;
const PIECE_SPRITE_SCALE: f32 = 0.25;

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

#[derive(Debug)]
pub struct PieceMoveEvent {
    pub from: BoardPosition,
    pub to: BoardPosition,
}

pub trait Piece: Send + Sync + DynClone + 'static + Component<Storage = TableStorage> {
    fn get_type(&self) -> PieceType;
    fn get_color(&self) -> PieceColor;
    fn get_position(&self) -> BoardPosition;
    fn set_position(&mut self, new_position: BoardPosition, moved: bool);
    fn reset(&mut self);
    fn get_moves(&self) -> Vec<BoardPosition>;
    fn possible_move(&self, new_position: BoardPosition) -> bool;
    fn possible_capture(&self, new_position: BoardPosition) -> bool;
    fn is_sliding(&self) -> bool;
}

#[derive(Component)]
pub struct PieceInfo {
    pub piece: Box<dyn Piece>,
}

impl PieceInfo {
    pub fn new(piece_color: PieceColor, piece_type: PieceType, position: BoardPosition) -> Self {
        let piece: Box<dyn Piece> = match piece_type {
            PieceType::Pawn => Pawn::new(position, piece_color),
            PieceType::King => King::new(position, piece_color),
            PieceType::Queen => Queen::new(position, piece_color),
            PieceType::Bishop => Bishop::new(position, piece_color),
            PieceType::Knight => Knight::new(position, piece_color),
            PieceType::Rook => Rook::new(position, piece_color),
        };
        PieceInfo { piece }
    }
}

#[derive(Component)]
pub struct Dragging(pub bool);

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
        board_properties: &Res<BoardProperties>,
    ) -> Self {
        let sprite_sheet_index = (piece_type as u8) + 6 * (piece_color as u8);
        let (x, y) = board_properties.position_to_transform(board_position);
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(sprite_sheet_index.into()),
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x, y, 1.).with_scale(Vec3::splat(PIECE_SPRITE_SCALE)),
            ..default()
        };
        PieceBundle {
            piece: PieceInfo::new(piece_color, piece_type, board_position),
            dragging: Dragging(false),
            sprite,
        }
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    board_properties: Res<BoardProperties>,
    board_state: Res<BoardState>,
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
    let mut pieces = Vec::with_capacity(BOARD_SIZE * BOARD_SIZE);
    for rank in 0..BOARD_SIZE {
        for file in 0..BOARD_SIZE {
            if board_state.board[rank][file].is_some() {
                let board_position = BoardPosition::new(rank, file);
                let piece = PieceBundle::new(
                    board_state.board[rank][file].unwrap().1,
                    board_state.board[rank][file].unwrap().0,
                    &texture_atlas_handle,
                    board_position,
                    &board_properties,
                );
                pieces.push(piece);
            }
        }
    }
    commands.spawn_batch(pieces);
}

pub fn reset_pieces(
    mut setup_events: EventReader<ResetBoardEvent>,
    mut query: Query<(
        &mut PieceInfo,
        &mut Transform,
        &mut Visibility,
        &mut Dragging,
    )>,
    board_properties: Res<BoardProperties>,
) {
    for _event in setup_events.iter() {
        for (mut piece, mut transform, mut visibility, mut dragging) in query.iter_mut() {
            piece.piece.reset();
            // Change its transform
            return_piece(piece.as_mut(), transform.as_mut(), &board_properties);
            // Make it visible
            *visibility = Visibility::VISIBLE;
            // Disable dragging
            dragging.0 = false;
        }
    }
}

pub fn piece_click_handler(
    mut board_click_events: EventReader<BoardClickEvent>,
    mut query: Query<(&mut PieceInfo, &mut Transform, &mut Dragging, &Visibility)>,
    mut piece_move_event: EventWriter<PieceMoveEvent>,
    board_properties: Res<BoardProperties>,
    state: Res<BoardState>,
) {
    for click in board_click_events.iter() {
        for (mut piece, mut transform, mut dragging, visibility) in query.iter_mut() {
            match click.input.button {
                MouseButton::Left => {
                    if click.input.state == ButtonState::Pressed {
                        if (click.position.is_some())
                            && (click.position.unwrap() == piece.piece.get_position()
                                && visibility.is_visible)
                        {
                            // Start dragging the piece
                            dragging.0 = true;
                        }
                    } else if click.input.state == ButtonState::Released && dragging.0 {
                        if click.position.is_some() {
                            // When the button is released move the piece to that square if it is a valid move
                            attempt_piece_move(
                                piece.as_mut(),
                                transform.as_mut(),
                                click.position.unwrap(),
                                &board_properties,
                                &mut piece_move_event,
                                &state,
                            );
                        } else {
                            // Return the piece to its original position
                            return_piece(piece.as_mut(), transform.as_mut(), &board_properties);
                        }
                        // Stop dragging the piece
                        dragging.0 = false;
                    }
                }
                MouseButton::Right => {
                    // If the right button was clicked, stop dragging and return the piece to its original position
                    if click.input.state == ButtonState::Pressed && dragging.0 {
                        // Stop dragging and return the piece to its original position
                        dragging.0 = false;
                        return_piece(piece.as_mut(), transform.as_mut(), &board_properties);
                    }
                }
                _ => {}
            }
        }
    }
}

fn attempt_piece_move(
    piece: &mut PieceInfo,
    transform: &mut Transform,
    new_position: BoardPosition,
    board_properties: &Res<BoardProperties>,
    piece_move_event: &mut EventWriter<PieceMoveEvent>,
    state: &Res<BoardState>,
) {
    if is_legal_move(piece, new_position, state) {
        // Send piece move event
        let event = PieceMoveEvent {
            from: piece.piece.get_position(),
            to: new_position,
        };
        piece_move_event.send(event);
        info!(
            "Piece moved from: {:?} to: {:?}",
            piece.piece.get_position(),
            new_position
        );
    } else {
        // Return piece to its original position
        return_piece(piece, transform, board_properties);
    }
}

fn move_piece(
    piece: &mut PieceInfo,
    transform: &mut Transform,
    new_position: BoardPosition,
    board_properties: &Res<BoardProperties>,
    moved: bool,
) {
    // Change its position
    piece.piece.set_position(new_position, moved);
    // Change its transform
    let new_transform = board_properties.position_to_transform(new_position);
    *transform = transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
}

fn return_piece(
    piece: &mut PieceInfo,
    transform: &mut Transform,
    board_properties: &Res<BoardProperties>,
) {
    move_piece(
        piece,
        transform,
        piece.piece.get_position(),
        board_properties,
        false,
    );
}

fn take_piece(visibility: &mut Visibility) {
    *visibility = Visibility::INVISIBLE;
}

pub fn piece_mover(
    mut events: EventReader<PieceMoveEvent>,
    mut query: Query<(&mut PieceInfo, &mut Transform)>,
    board_properties: Res<BoardProperties>,
) {
    for event in events.iter() {
        for (mut piece, mut transform) in query.iter_mut() {
            if piece.piece.get_position() == event.from {
                // Move piece
                move_piece(
                    piece.as_mut(),
                    transform.as_mut(),
                    event.to,
                    &board_properties,
                    true,
                );
            }
        }
    }
}

pub fn piece_taker(
    mut events: EventReader<PieceMoveEvent>,
    mut query: Query<(&mut Visibility, &PieceInfo)>,
    state: Res<BoardState>,
) {
    for event in events.iter() {
        for (mut visibility, piece) in query.iter_mut() {
            if piece.piece.get_position() == event.to
                && piece.piece.get_color() != state.active_color
            {
                take_piece(visibility.as_mut());
            }
        }
    }
}

pub fn piece_move_audio(
    mut events: EventReader<PieceMoveEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for _event in events.iter() {
        audio
            .play(asset_server.load("sounds/chess_move_on_alabaster.wav"))
            .with_volume(PIECE_MOVE_VOLUME);
    }
}

pub fn piece_dragger(
    mut query: Query<(&Dragging, &mut Transform, &PieceInfo)>,
    state: Res<BoardState>,
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
        for (dragging, mut transform, piece) in query.iter_mut() {
            // If this piece is being dragged and is the current active color
            if dragging.0 && state.active_color == piece.piece.get_color() {
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
