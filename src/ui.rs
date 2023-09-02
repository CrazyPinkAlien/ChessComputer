use bevy::app::{App, Plugin};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::{
    default, AssetServer, BuildChildren, ButtonBundle, Camera, Camera2dBundle, Changed, Color,
    Commands, Component, Event, EventReader, EventWriter, GlobalTransform, NodeBundle, Query, Res,
    Startup, TextBundle, Update, With,
};
use bevy::text::{Text, TextStyle};
use bevy::ui::{
    AlignItems, BackgroundColor, FlexDirection, Interaction, JustifyContent, Style, UiRect, Val,
};
use bevy::window::Window;

use crate::chess_board::{BoardPosition, ChessBoard, PieceColor, PieceMoveEvent, ResetBoardEvent};
use crate::fen::Fen;

mod board;
mod piece;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const TEXT_SIZE: f32 = 40.0;

pub(super) struct UIPlugin;

impl Plugin for UIPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        use bevy::prelude::PostUpdate;

        app.init_resource::<piece::PieceProperties>()
            .init_resource::<board::BoardProperties>()
            .add_event::<BoardClickEvent>()
            .add_systems(Startup, (setup, board::setup))
            .add_systems(
                Update,
                (
                    reset_board_button,
                    mouse_event_handler,
                    reset_past_moves_text,
                    piece::piece_creator,
                    piece::piece_click_handler,
                    piece::piece_move_audio,
                    piece::piece_dragger,
                    piece::piece_undragger,
                    piece::piece_mover,
                    piece::piece_resetter,
                    board::highlight_valid_squares,
                ),
            )
            .add_systems(PostUpdate, past_moves_text);
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct ResetBoardButton;

#[derive(Component)]
struct PastMovesText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.spawn((Camera2dBundle::default(), MainCamera));

    commands
        // Top level flex box
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect {
                    left: Val::Percent(2.),
                    right: Val::Percent(2.),
                    top: Val::Percent(2.),
                    bottom: Val::Percent(2.),
                },
                ..default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            // Left Side
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexStart,
                        ..default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    // Reset board button
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    // horizontally center child text
                                    justify_content: JustifyContent::Center,
                                    // vertically center child text
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            ResetBoardButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Reset Board",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: TEXT_SIZE,
                                    color: TEXT_COLOR,
                                },
                            ));
                        });
                });

            // Right Side
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    // Past moves title
                    parent.spawn(TextBundle::from_section(
                        "Past Moves",
                        TextStyle {
                            font: font.clone(),
                            font_size: TEXT_SIZE,
                            color: TEXT_COLOR,
                        },
                    ));

                    // Past moves list
                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: font.clone(),
                                font_size: TEXT_SIZE - 4.0,
                                color: TEXT_COLOR,
                            },
                        ),
                        PastMovesText,
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

fn reset_board_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ResetBoardButton>),
    >,
    mut setup_event: EventWriter<ResetBoardEvent>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                setup_event.send(ResetBoardEvent::new(Fen::default()));
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

fn past_moves_text(
    mut events: EventReader<PieceMoveEvent>,
    mut query: Query<&mut Text, With<PastMovesText>>,
    board: Res<ChessBoard>,
) {
    for event in events.iter() {
        let mut text = query.single_mut();

        // Add the move number if the active color is white
        if *board.active_color() == PieceColor::White {
            text.sections[0].value += &board.move_number().to_string();
            text.sections[0].value.push_str(". ");
        } else {
            text.sections[0].value.push_str("    ");
        }
        // Add the move in algebraic notation
        text.sections[0].value += &event.piece_move().as_algebraic();

        text.sections[0].value.push('\n');
    }
}

fn reset_past_moves_text(
    mut events: EventReader<ResetBoardEvent>,
    mut query: Query<&mut Text, With<PastMovesText>>,
) {
    for _event in events.iter() {
        let mut text = query.single_mut();
        text.sections[0].value = "".to_owned();
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        input::InputPlugin,
        prelude::{AssetPlugin, Button, Entity, Events, Vec2},
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
            board_properties.transform_to_position(&click_position)
        );
        assert_eq!(board_click.input, mouse_button_input);
    }
}
