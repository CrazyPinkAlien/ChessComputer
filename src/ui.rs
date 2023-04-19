use bevy::app::{App, Plugin};
use bevy::prelude::{Camera2dBundle, Commands};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
