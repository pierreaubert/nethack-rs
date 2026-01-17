//! UI plugins for NetHack-rs Bevy client

mod character;
pub mod direction;
mod help;
mod hud;
mod inventory;
mod menus;
mod messages;
mod minimap;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub use direction::DirectionSelectState;
pub use inventory::InventoryState;
pub use menus::GameSettings;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins((
                hud::HudPlugin,
                messages::MessagesPlugin,
                inventory::InventoryPlugin,
                direction::DirectionPlugin,
                menus::MenusPlugin,
                minimap::MinimapPlugin,
                character::CharacterPlugin,
                help::HelpPlugin,
            ));
    }
}
