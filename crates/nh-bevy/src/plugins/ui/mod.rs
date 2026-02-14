//! UI plugins for NetHack-rs Bevy client

mod character;
pub mod direction;
mod discoveries;
mod extended_commands;
mod help;
mod hud;
mod inventory;
pub mod item_picker;
mod key_bindings;
mod menus;
pub mod messages;
mod minimap;
pub mod monster_picker;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub use direction::DirectionSelectState;
pub use discoveries::DiscoveriesState;
pub use extended_commands::ExtendedCommandsState;
pub use inventory::InventoryState;
pub use item_picker::ItemPickerState;
pub use key_bindings::KeyBindingsState;
pub use menus::GameSettings;
pub use monster_picker::MonsterPickerState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin).add_plugins((
            hud::HudPlugin,
            messages::MessagesPlugin,
            inventory::InventoryPlugin,
            item_picker::ItemPickerPlugin,
            monster_picker::MonsterPickerPlugin,
            direction::DirectionPlugin,
            discoveries::DiscoveriesPlugin,
            menus::MenusPlugin,
            minimap::MinimapPlugin,
            character::CharacterPlugin,
            help::HelpPlugin,
            extended_commands::ExtendedCommandsPlugin,
            key_bindings::KeyBindingsPlugin,
        ));
    }
}
