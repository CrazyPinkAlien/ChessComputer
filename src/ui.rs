use bevy::app::{App, Plugin};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::{
    default, AssetServer, BuildChildren, Button, ButtonBundle, Camera, Camera2dBundle, Changed,
    Color, Commands, Component, Event, EventReader, EventWriter, GlobalTransform, NodeBundle,
    Query, Res, Startup, TextBundle, Update, With,
};
use bevy::text::TextStyle;
use bevy::ui::{AlignItems, BackgroundColor, Interaction, JustifyContent, Style, UiRect, Val};
use bevy::window::Window;

use crate::chess_board::{BoardPosition, ResetBoardEvent};
use crate::fen::Fen;

mod board;
mod piece;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub(super) struct UIPlugin;

impl Plugin for UIPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.init_resource::<piece::PieceProperties>()
            .init_resource::<board::BoardProperties>()
            .add_event::<BoardClickEvent>()
            .add_systems(Startup, (setup, board::setup, piece::setup))
            .add_systems(
                Update,
                (
                    button_system,
                    mouse_event_handler,
                    piece::piece_creator,
                    piece::piece_click_handler,
                    piece::piece_move_audio,
                    piece::piece_dragger,
                    piece::piece_undragger,
                    piece::piece_mover,
                    piece::piece_resetter,
                    board::highlight_valid_squares,
                ),
            );
    }
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
    commands
        .spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                margin: UiRect {
                    left: Val::Percent(1.),
                    right: Val::Percent(1.),
                    top: Val::Percent(1.),
                    bottom: Val::Percent(1.),
                },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Reset Board",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
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

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut setup_event: EventWriter<ResetBoardEvent>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                setup_event.send(ResetBoardEvent::new(Fen::from_file(
                    "assets/fens/starting_position.fen",
                )));
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
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

        // Confirm that the reset button has been created
        assert_eq!(app.world.query::<&Button>().iter(&app.world).len(), 1);
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
            board_properties.transform_to_position(click_position)
        );
        assert_eq!(board_click.input, mouse_button_input);
    }
}
