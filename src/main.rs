use bevy::app::App;
use bevy::winit::WinitSettings;
use bevy::DefaultPlugins;
use bevy_kira_audio::AudioPlugin;

use crate::chess_board::ChessBoardPlugin;
use crate::ui::UIPlugin;

mod chess_board;
mod fen;
mod ui;

#[cfg(not(tarpaulin_include))]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(ChessBoardPlugin)
        .add_plugin(UIPlugin)
        .insert_resource(WinitSettings::desktop_app())
        .run();
}
