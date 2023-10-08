use bevy::input::ButtonState;
use bevy::prelude::{
    default, AssetServer, Assets, AudioBundle, Bundle, Camera, Changed, Commands, Component,
    Entity, EventReader, EventWriter, FromWorld, GlobalTransform, Handle, MouseButton,
    PlaybackSettings, Query, Res, Resource, Transform, Vec2, Vec3, With,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite};
use bevy::window::Window;

use crate::chess_board::r#move::Move;
use crate::chess_board::{
    BoardPosition, ChessBoard, PieceColor, PieceCreateEvent, PieceMoveEvent, RequestMoveEvent,
    ResetBoardEvent,
};

use super::board::BoardProperties;
use super::{BoardClickEvent, MainCamera};

#[derive(Resource, Debug)]
pub(super) struct PieceProperties {
    texture_atlas_handle: Handle<TextureAtlas>,
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

#[derive(Component)]
pub(super) struct PieceMoveAudio;

#[derive(Bundle)]
struct PieceBundle {
    dragging: Dragging,
    position: BoardPosition,
    sprite: SpriteSheetBundle,
    color: PieceColor,
    starting_position: StartingPosition,
    tag: PieceTag,
}

impl PieceBundle {
    fn new(position: BoardPosition, sprite: SpriteSheetBundle, color: PieceColor) -> Self {
        PieceBundle {
            dragging: Dragging(false),
            position,
            sprite,
            color,
            starting_position: StartingPosition(position),
            tag: PieceTag,
        }
    }
}

pub(super) fn piece_creator(
    mut events: EventReader<PieceCreateEvent>,
    mut commands: Commands,
    board_properties: Res<BoardProperties>,
    piece_properties: Res<PieceProperties>,
) {
    for event in events.iter() {
        let sprite_sheet_index = (*event.piece_type() as u8) + 6 * (*event.color() as u8);
        let (x, y) = board_properties.position_to_transform(event.position());
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(sprite_sheet_index.into()),
            texture_atlas: piece_properties.texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x, y, 1.)
                .with_scale(Vec3::splat(piece_properties.sprite_scale)),
            ..default()
        };
        commands.spawn(PieceBundle::new(*event.position(), sprite, *event.color()));
    }
}

pub(super) fn piece_click_handler(
    mut board_click_events: EventReader<BoardClickEvent>,
    mut query: Query<(&mut Dragging, &BoardPosition), With<PieceTag>>,
    mut piece_move_event: EventWriter<RequestMoveEvent>,
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
                                Move::from_board(*piece_position, click.position.unwrap(), &board);
                            // When the button is released move the piece to that square if it is a valid move
                            if board.valid_move(&potential_move, board.active_color(), &true) {
                                let event = RequestMoveEvent::new(potential_move);
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
    mut query: Query<(Entity, &mut BoardPosition, &mut Transform), With<PieceTag>>,
    board_properties: Res<BoardProperties>,
    mut commands: Commands,
) {
    for event in piece_move_events.iter() {
        // Remove any piece that is already there
        for (entity, position, _transform) in query.iter() {
            if *event.to() == *position {
                commands.entity(entity).despawn();
            }
        }
        // Move the piece
        for (_entity, mut position, mut transform) in query.iter_mut() {
            if *position == *event.from() {
                // Change its transform
                let new_transform = board_properties.position_to_transform(event.to());
                *transform =
                    transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
                // Change its position
                *position = *event.to();
            }
        }
    }
}

pub(super) fn piece_resetter(
    mut board_reset_events: EventReader<ResetBoardEvent>,
    mut query: Query<Entity, With<PieceTag>>,
    mut commands: Commands,
) {
    for _event in board_reset_events.iter() {
        for entity in query.iter_mut() {
            // Despawn the piece
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn piece_move_audio(
    mut events: EventReader<PieceMoveEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for _event in events.iter() {
        commands.spawn((AudioBundle {
            source: asset_server.load("sounds/chess_move_on_alabaster.wav"),
            settings: PlaybackSettings::DESPAWN,
        },));
    }
}

pub(super) fn piece_dragger(
    mut query: Query<(&Dragging, &mut Transform, &PieceColor), With<PieceTag>>,
    board: Res<ChessBoard>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // Get window and camera
    let window = windows.single();
    let (camera, camera_transform) = camera.single();
    // Check if the cursor is in the window
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        for (dragging, mut transform, piece_color) in query.iter_mut() {
            // If this piece is being dragged and is the current active color
            if dragging.0
                && board.active_color().is_some()
                && board.active_color().unwrap() == *piece_color
            {
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
            let new_transform = board_properties.position_to_transform(position);
            *transform =
                transform.with_translation(Vec3::new(new_transform.0, new_transform.1, 1.0));
        }
    }
}
