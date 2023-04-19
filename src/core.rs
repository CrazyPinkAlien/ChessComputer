use bevy::app::App;
use bevy::prelude::Plugin;

pub mod board;
pub mod pieces;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<board::BoardProperties>()
            .init_resource::<pieces::PieceProperties>()
            .add_startup_system(board::setup)
            .add_startup_system(pieces::setup);
    }
}
