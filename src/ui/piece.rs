use bevy::input::ButtonState;
use bevy::prelude::{
    default, AssetServer, Assets, Bundle, Camera, Changed, Commands, Component, Entity,
    EventReader, EventWriter, FromWorld, GlobalTransform, Handle, MouseButton, Query, Res,
    Resource, Transform, Vec2, Vec3, With,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::window::Windows;
use bevy_kira_audio::{Audio, AudioControl};

use crate::chess_board::r#move::Move;
use crate::chess_board::{
    BoardPosition, ChessBoard, PieceColor, PieceCreateEvent, PieceDestroyEvent, PieceMoveEvent,
    ResetBoardEvent,
};

use super::board::BoardProperties;
use super::{BoardClickEvent, MainCamera};

#[derive(Resource, Debug)]
pub(super) struct PieceProperties {
    texture_atlas_handle: Handle<TextureAtlas>,
    move_audio_volume: f64,
    sprite_scale: f32,
}

impl FromWorld for PieceProperties {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
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
        // Add the piece_info textures to the texture atlas and return
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        PieceProperties {
            texture_atlas_handle,
            move_audio_volume: 0.75,
            sprite_scale: 0.25,
        }
    }
}

#[derive(Component, Clone)]
pub(super) struct Dragging(bool);

impl Dragging {
    pub(super) fn get(&self) -> bool {
        self.0
    }
}

#[derive(Component)]
pub(super) struct StartingPosition(BoardPosition);

#[derive(Component)]
pub(super) struct PieceTag;

#[derive(Bundle)]
struct PieceBundle {
    dragging: Dragging,
    position: BoardPosition,
    sprite: SpriteSheetBundle,
    color: PieceColor,
    starting_position: StartingPosition,
    tag: PieceTag,
}

pub(super) fn piece_creator(
    mut events: EventReader<PieceCreateEvent>,
    mut commands: Commands,
    board_properties: Res<BoardProperties>,
    piece_properties: Res<PieceProperties>,
) {
    for event in events.iter() {
        let sprite_sheet_index = (event.piece_type() as u8) + 6 * (event.color() as u8);
        let (x, y) = board_properties.position_to_transform(*event.position());
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(sprite_sheet_index.into()),
            texture_atlas: piece_properties.texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x, y, 1.)
                .with_scale(Vec3::splat(piece_properties.sprite_scale)),
            ..default()
        };
        commands.spawn(PieceBundle {
            dragging: Dragging(false),
            position: *event.position(),
            sprite,
            color: event.color(),
            starting_position: StartingPosition(*event.position()),
            tag: PieceTag,
        });
    }
}

pub(super) fn piece_destroyer(
    mut events: EventReader<PieceDestroyEvent>,
    query: Query<(Entity, &BoardPosition), With<PieceTag>>,
    mut commands: Commands,
) {
    for event in events.iter() {
        for (entity, position) in query.iter() {
            if event.position() == position {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub(super) fn piece_click_handler(
    mut board_click_events: EventReader<BoardClickEvent>,
    mut query: Query<(&mut Dragging, &BoardPosition), With<PieceTag>>,
    mut piece_move_event: EventWriter<PieceMoveEvent>,
    board: Res<ChessBoard>,
) {
    for click in board_click_events.iter() {
        for (mut dragging, piece_position) in query.iter_mut() {
            match click.input.button {
                MouseButton::Left => {
                    if click.input.state == ButtonState::Pressed {
                        if (click.position.is_some())
                            && (click.position.unwrap() == *piece_position)
                        {
                            // Start dragging the piece
                            dragging.0 = true;
                        }
                    } else if click.input.state == ButtonState::Released && dragging.0 {
                        if click.position.is_some() {
                            let potential_move =
                                Move::new(*piece_position, click.position.unwrap());
                            // When the button is released move the piece to that square if it is a valid move
                            if board.valid_move(potential_move) {
                                let event = PieceMoveEvent::new(potential_move);
                                piece_move_event.send(event);
                            }
                        }
                        // Stop dragging the piece
                        dragging.0 = false;
                    }
                }
                MouseButton::Right => {
                    // If the right button was clicked, stop dragging and return the piece to its original position
                    if click.input.state == ButtonState::Pressed && dragging.0 {
                        // Stop dragging the piece
                        dragging.0 = false;
                    }
                }
                _ => {}
            }
        }
    }
}

pub(super) fn piece_mover(
    mut piece_move_events: EventReader<PieceMoveEvent>,
    mut query: Query<(&BoardPosition, &mut Transform), With<PieceTag>>,
    board_properties: Res<BoardProperties>,
) {
    for event in piece_move_events.iter() {
        for (position, mut transform) in query.iter_mut() {
            // TODO I don't think this check is correct
            if *position == event.piece_move().from() {
                // Change its transform
                let new_transform = board_properties.position_to_transform(event.piece_move().to());
                *transform =
                    transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
            }
        }
    }
}

pub(super) fn piece_resetter(
    mut board_reset_events: EventReader<ResetBoardEvent>,
    mut query: Query<(&StartingPosition, &mut Transform, &mut Dragging), With<PieceTag>>,
    board_properties: Res<BoardProperties>,
) {
    for _event in board_reset_events.iter() {
        for (starting_position, mut transform, mut dragging) in query.iter_mut() {
            // Change its transform
            let new_transform = board_properties.position_to_transform(starting_position.0);
            *transform =
                transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
            // Disable dragging
            dragging.0 = false;
        }
    }
}

pub(super) fn piece_move_audio(
    mut events: EventReader<PieceMoveEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    piece_properties: Res<PieceProperties>,
) {
    for _event in events.iter() {
        audio
            .play(asset_server.load("sounds/chess_move_on_alabaster.wav"))
            .with_volume(piece_properties.move_audio_volume);
    }
}

pub(super) fn piece_dragger(
    mut query: Query<(&Dragging, &mut Transform, &PieceColor), With<PieceTag>>,
    board: Res<ChessBoard>,
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
        for (dragging, mut transform, piece_color) in query.iter_mut() {
            // If this piece is being dragged and is the current active color
            if dragging.0 && board.active_color() == *piece_color {
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

pub(super) fn piece_undragger(
    mut query: Query<
        (&Dragging, &mut Transform, &BoardPosition),
        (Changed<Dragging>, With<PieceTag>),
    >,
    board_properties: Res<BoardProperties>,
) {
    for (dragging, mut transform, position) in query.iter_mut() {
        // If this piece has stopped being dragged, change its transform to the correct position
        if !dragging.0 {
            // Change its transform
            let new_transform = board_properties.position_to_transform(*position);
            *transform =
                transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
        }
    }
}
