#![doc = include_str!("../README.md")]

use bevy::app::App;
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
        .run();
}
