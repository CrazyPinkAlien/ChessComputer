use bevy::app::App;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::{
    Camera, EventReader, EventWriter, GlobalTransform, Plugin, Query, Res, SystemSet, With,
};
use bevy::window::Windows;

use crate::ui::MainCamera;

use self::board::{BoardPosition, BoardProperties};
use crate::core::piece::PieceMoveEvent;

mod board;
mod fen;
mod move_checker;
mod piece;
mod state;

static BOARD_SIZE: usize = 8;

pub(super) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<board::BoardProperties>()
            .init_resource::<state::BoardState>()
            .add_startup_system(board::setup)
            .add_startup_system(piece::setup)
            .add_event::<BoardClickEvent>()
            .add_event::<ResetBoardEvent>()
            .add_event::<PieceMoveEvent>()
            .add_system_set(
                SystemSet::new()
                    .label("mouse")
                    .with_system(mouse_event_handler),
            )
            .add_system_set(
                SystemSet::new()
                    .label("piece_pre_move")
                    .after("mouse")
                    .with_system(piece::piece_click_handler)
                    .with_system(piece::piece_dragger)
                    .with_system(piece::reset_pieces),
            )
            .add_system_set(
                SystemSet::new()
                    .label("piece_move")
                    .after("piece_pre_move")
                    .with_system(piece::piece_mover),
            )
            .add_system_set(
                SystemSet::new()
                    .label("piece_post_move")
                    .after("piece_move")
                    .with_system(piece::piece_move_audio)
                    .with_system(piece::piece_taker),
            )
            .add_system_set(
                SystemSet::new()
                    .label("state")
                    .after("piece_post_move")
                    .with_system(state::piece_move_handler)
                    .with_system(state::reset_board_state),
            )
            .add_system_set(
                SystemSet::new()
                    .label("board")
                    .after("state")
                    .with_system(board::highlight_valid_squares),
            );
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ResetBoardEvent;

#[derive(Debug, Copy, Clone)]
pub struct BoardClickEvent {
    pub position: Option<BoardPosition>,
    pub input: MouseButtonInput,
}

fn mouse_event_handler(
    windows: Res<Windows>,
    mut mouse_input: EventReader<MouseButtonInput>,
    properties: Res<BoardProperties>,
    mut board_click_event: EventWriter<BoardClickEvent>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let window = windows.get_primary().unwrap();
    let (camera, camera_transform) = camera.single();
    for input in mouse_input.iter() {
        // Check if the cursor is in the window and convert to world coordinates
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            // Check if the mouse is over the board
            let board_position = properties.transform_to_position(world_position);
            // Send a board click event
            let event = BoardClickEvent {
                position: board_position,
                input: *input,
            };
            board_click_event.send(event);
        }
    }
}
