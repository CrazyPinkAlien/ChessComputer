use bevy::app::App;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::{
    Camera, EventReader, EventWriter, GlobalTransform, Plugin, Query, Res, With,
};
use bevy::window::Windows;

use crate::ui::MainCamera;

use self::board::{BoardClickEvent, BoardProperties};

mod board;
mod piece;

pub(super) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<board::BoardProperties>()
            .init_resource::<piece::PieceProperties>()
            .init_resource::<board::BoardState>()
            .add_startup_system(board::setup)
            .add_startup_system(piece::setup)
            .add_event::<board::BoardClickEvent>()
            .add_system(mouse_event_handler)
            .add_system(piece::handle_piece_clicks)
            .add_system(piece::dragged_piece);
    }
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
            if board_position.is_some() {
                // Send a board click event
                board_click_event.send(BoardClickEvent {
                    position: board_position.unwrap(),
                    input: *input,
                });
            }
        }
    }
}
