// Core functionality for the game
use bevy::app::App;
use bevy::prelude::Plugin;

pub mod board;

// Create core plugin
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<board::BoardProperties>()
            .add_startup_system(board::setup);
    }
}
