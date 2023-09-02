use bevy::app::App;
use bevy::prelude::States;
use bevy::winit::WinitSettings;
use bevy::DefaultPlugins;

use crate::chess_board::ChessBoardPlugin;
use crate::ui::UIPlugin;

mod chess_board;
mod fen;
mod ui;

#[cfg(not(tarpaulin_include))]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((ChessBoardPlugin, UIPlugin))
        .insert_resource(WinitSettings::desktop_app())
        .add_state::<AppState>()
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
enum AppState {
    #[default]
    InGame,
    GameEnd,
}
