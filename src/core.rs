use bevy::app::App;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::{Plugin, Component, EventReader, Res, EventWriter};
use bevy::window::CursorMoved;

use self::board::{BoardProperties, BoardClickEvent};

mod board;
mod piece;

pub(super) struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<board::BoardProperties>()
            .init_resource::<piece::PieceProperties>()
            .add_startup_system(board::setup)
            .add_startup_system(piece::setup)
            .add_event::<board::BoardClickEvent>()
            .add_system(mouse_event_handler);
    }
}

#[derive(Component)]
pub(crate) struct Selected;

fn mouse_event_handler(mut cursor_moved: EventReader<CursorMoved>, mut mouse_input: EventReader<MouseButtonInput>, properties: Res<BoardProperties>, mut board_click_event: EventWriter<BoardClickEvent>) {
    for input in mouse_input.iter() {
        for moved in cursor_moved.iter() {
            // Check if the mouse is over the board
            let board_position = properties.transform_to_position(moved.position);
            if board_position.is_some() {
                // Send a board click event
                board_click_event.send(BoardClickEvent {
                    position: board_position.unwrap(),
                    input: input.button,
                });
            }
        }
    }
}
