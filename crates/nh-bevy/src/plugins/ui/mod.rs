//! UI plugins for NetHack-rs Bevy client

mod character;
mod discoveries;
pub mod direction;
mod help;
mod hud;
mod inventory;
pub mod item_picker;
mod menus;
pub mod messages;
mod minimap;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub use direction::DirectionSelectState;
pub use discoveries::DiscoveriesState;
pub use inventory::InventoryState;
pub use item_picker::ItemPickerState;
pub use menus::GameSettings;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins((
                hud::HudPlugin,
                messages::MessagesPlugin,
                inventory::InventoryPlugin,
                item_picker::ItemPickerPlugin,
                direction::DirectionPlugin,
                discoveries::DiscoveriesPlugin,
                menus::MenusPlugin,
                minimap::MinimapPlugin,
                character::CharacterPlugin,
                help::HelpPlugin,
            ));
    }
}
