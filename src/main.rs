use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::winit::WinitSettings;

use crate::ui::UIPlugin;
use crate::core::CorePlugin;

pub mod core;
pub mod ui;

// Main entry point
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(UIPlugin)
        .add_plugin(CorePlugin)

        .insert_resource(WinitSettings::desktop_app())

        .run();
}
