// UI functionality
use bevy::app::{App, Plugin};
use bevy::ecs::system::ResMut;
use bevy::prelude::IntoSystemDescriptor;
use bevy_egui::EguiContext;
use bevy_egui::egui::{CentralPanel, SidePanel};

// Create UI plugin
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(left_side_menu)
            .add_system(board_ui.after(left_side_menu));
    }
}

// Systems

// Left hand side menu
fn left_side_menu(mut egui_context: ResMut<EguiContext>) {
    SidePanel::left("side_panel").default_width(200.0).show(egui_context.ctx_mut(), |ui|  {

    });
}

// UI for the main menu
fn board_ui(mut egui_context: ResMut<EguiContext>) {
    // Draw UI
    CentralPanel::default().show(egui_context.ctx_mut(), |ui| {

    });
}
