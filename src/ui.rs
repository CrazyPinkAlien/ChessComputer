use bevy::app::{App, Plugin};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::{
    in_state, Camera, Camera2dBundle, Commands, Component, Event, EventReader, EventWriter,
    GlobalTransform, IntoSystemConfigs, NextState, Query, Res, ResMut, Startup, Update, With,
};
use bevy::window::Window;
use bevy_egui::{egui, EguiContexts};

use crate::chess_board::{BoardPosition, ChessBoard, GameEndStatus, ResetBoardEvent};
use crate::fen::Fen;
use crate::AppState;

mod board;
mod piece;

pub(super) struct UIPlugin;

impl Plugin for UIPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        use bevy_egui::EguiPlugin;

        app.add_plugins(EguiPlugin)
            .init_resource::<piece::PieceProperties>()
            .init_resource::<board::BoardProperties>()
            .add_event::<BoardClickEvent>()
            .add_systems(Startup, (setup, board::setup))
            .add_systems(Update, (ui_system, piece::piece_undragger))
            .add_systems(
                Update,
                (
                    mouse_event_handler,
                    piece::piece_creator,
                    piece::piece_click_handler,
                    piece::piece_move_audio,
                    piece::piece_dragger,
                    piece::piece_mover,
                    piece::piece_resetter,
                    board::highlight_valid_squares,
                )
                    .distributive_run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn ui_system(
    mut contexts: EguiContexts,
    mut setup_event: EventWriter<ResetBoardEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    board: Res<ChessBoard>,
) {
    let ctx = contexts.ctx_mut();
    egui::SidePanel::left("left_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            // Reset board button
            if ui.button("Reset Board").clicked() {
                setup_event.send(ResetBoardEvent::new(Fen::default()));
                next_state.set(AppState::InGame);
            }
        });

    egui::SidePanel::right("right_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            // Past moves list
            ui.heading("Past Moves");

            let text_style = egui::TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            let total_rows = (board.past_moves().len() as f32 / 2.0).ceil() as usize;
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    for row in row_range {
                        let white_move = board.past_moves()[row * 2].as_algebraic();
                        let black_move =
                            if (row == total_rows - 1) && ((board.past_moves().len() & 1) == 1) {
                                "".to_string()
                            } else {
                                board.past_moves()[row * 2 + 1].as_algebraic()
                            };
                        let mut move_number = row + *board.move_number() as usize - total_rows;
                        if (board.past_moves().len() & 1) == 1 {
                            move_number += 1;
                        }
                        let move_text = format!("{}. {} {}", move_number, white_move, black_move);
                        ui.label(move_text);
                    }
                });

            // Game end status
            if board.game_end_status().is_some() {
                ui.label(match board.game_end_status().unwrap() {
                    GameEndStatus::Checkmate => "Checkmate",
                    GameEndStatus::Resignation => "Resignation",
                    GameEndStatus::Stalemate => "Stalemate",
                    GameEndStatus::DeadPosition => "Dead Position",
                    GameEndStatus::FlagFall => "Flag Fall",
                });
                ui.label(format!(
                    "Winner: {}",
                    match board.winner() {
                        Some(x) => x.to_string(),
                        None => "Draw".to_string(),
                    }
                ));
            }
        });
}

#[derive(Debug, Copy, Clone, Event)]
struct BoardClickEvent {
    position: Option<BoardPosition>,
    input: MouseButtonInput,
}

fn mouse_event_handler(
    windows: Query<&Window>,
    mut mouse_input: EventReader<MouseButtonInput>,
    properties: Res<board::BoardProperties>,
    mut board_click_event: EventWriter<BoardClickEvent>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let window = windows.get_single().expect("No window has been created.");
    let (camera, camera_transform) = camera.single();
    for input in mouse_input.iter() {
        // Check if the cursor is in the window and convert to world coordinates
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            // Check if the mouse is over the board
            let board_position = properties.transform_to_position(&world_position);
            // Send a board click event
            let event = BoardClickEvent {
                position: board_position,
                input: *input,
            };
            board_click_event.send(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        input::InputPlugin,
        prelude::{AssetPlugin, Entity, Events, Vec2},
        window::{Window, WindowPlugin},
        MinimalPlugins,
    };

    use crate::ui::board::BoardProperties;

    use super::*;

    #[test]
    fn test_setup() {
        // Setup app
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_systems(Startup, setup);

        // Run systems
        app.update();

        // Confirm that the camera has been created
        assert_eq!(app.world.query::<&Camera>().iter(&app.world).len(), 1);
        assert_eq!(app.world.query::<&MainCamera>().iter(&app.world).len(), 1);
    }

    #[test]
    #[ignore]
    fn test_mouse_event_handler() {
        // Setup app
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            WindowPlugin::default(),
            InputPlugin,
        ));
        app.init_resource::<board::BoardProperties>();
        app.add_event::<BoardClickEvent>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, mouse_event_handler);

        // Run systems
        app.update();

        // Send MouseInputEvent
        let click_position = Vec2::new(34.0, 765.0);
        let (entity, _window) = app.world.query::<(Entity, &Window)>().single(&app.world);
        let mouse_button_input = MouseButtonInput {
            button: bevy::prelude::MouseButton::Left,
            state: bevy::input::ButtonState::Pressed,
            window: entity,
        };
        app.world
            .resource_mut::<Events<MouseButtonInput>>()
            .send(mouse_button_input);

        // Run systems
        app.update();

        // Get BoardClickEvent event reader
        let board_click_events = app.world.resource::<Events<BoardClickEvent>>();
        let mut board_click_reader = board_click_events.get_reader();
        let board_click = board_click_reader.iter(board_click_events).next().unwrap();

        // Check the event has been sent
        let board_properties = app.world.get_resource::<BoardProperties>().unwrap();
        assert_eq!(
            board_click.position,
            board_properties.transform_to_position(&click_position)
        );
        assert_eq!(board_click.input, mouse_button_input);
    }
}
