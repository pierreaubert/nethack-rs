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
use bevy_egui::{EguiPlugin, egui};

pub use direction::DirectionSelectState;
pub use discoveries::DiscoveriesState;
pub use extended_commands::ExtendedCommandsState;
pub use inventory::InventoryState;
pub use item_picker::ItemPickerState;
pub use key_bindings::KeyBindingsState;
pub use menus::GameSettings;
pub use monster_picker::MonsterPickerState;

/// Get display name for an object from the object data table
pub(crate) fn item_name(item: &nh_core::object::Object) -> String {
    let objects = nh_core::data::objects::OBJECTS;
    if (item.object_type as usize) < objects.len() {
        let obj_def = &objects[item.object_type as usize];
        obj_def.name.to_string()
    } else {
        format!("{:?}", item.class)
    }
}

/// Get egui color for an object class
pub(crate) fn object_class_color(class: &nh_core::object::ObjectClass) -> egui::Color32 {
    use nh_core::object::ObjectClass;
    match class {
        ObjectClass::Weapon => egui::Color32::from_rgb(200, 200, 200),
        ObjectClass::Armor => egui::Color32::from_rgb(150, 150, 200),
        ObjectClass::Ring => egui::Color32::from_rgb(255, 215, 0),
        ObjectClass::Amulet => egui::Color32::from_rgb(255, 165, 0),
        ObjectClass::Tool => egui::Color32::from_rgb(139, 90, 43),
        ObjectClass::Food => egui::Color32::from_rgb(139, 69, 19),
        ObjectClass::Potion => egui::Color32::from_rgb(255, 105, 180),
        ObjectClass::Scroll => egui::Color32::from_rgb(245, 245, 220),
        ObjectClass::Spellbook => egui::Color32::from_rgb(138, 43, 226),
        ObjectClass::Wand => egui::Color32::from_rgb(0, 191, 255),
        ObjectClass::Coin => egui::Color32::GOLD,
        ObjectClass::Gem => egui::Color32::from_rgb(0, 255, 255),
        ObjectClass::Rock => egui::Color32::GRAY,
        ObjectClass::Ball => egui::Color32::from_rgb(105, 105, 105),
        ObjectClass::Chain => egui::Color32::from_rgb(192, 192, 192),
        ObjectClass::Venom => egui::Color32::from_rgb(0, 128, 0),
        ObjectClass::Random | ObjectClass::IllObj => egui::Color32::from_rgb(255, 0, 255),
    }
}

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
