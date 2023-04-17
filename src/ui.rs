// UI functionality
use bevy::app::{App, Plugin};
use bevy::prelude::{Camera2dBundle, Commands};

// Create UI plugin
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup);
    }
}

// Systems

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
