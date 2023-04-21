use bevy::app::App;
use bevy::winit::WinitSettings;
use bevy::DefaultPlugins;

use crate::core::CorePlugin;
use crate::ui::UIPlugin;

mod core;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(UIPlugin)
        .add_plugin(CorePlugin)
        .insert_resource(WinitSettings::desktop_app())
        .run();
}
