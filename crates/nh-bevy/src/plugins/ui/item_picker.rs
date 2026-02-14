//! Item picker UI for selecting items for actions (eat, drop, etc.)

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use nh_core::action::Command;
use nh_core::object::{Object, ObjectClass};

use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use crate::plugins::ui::direction::{DirectionAction, DirectionSelectState};
use crate::resources::GameStateResource;

pub struct ItemPickerPlugin;

impl Plugin for ItemPickerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemPickerState>().add_systems(
            Update,
            (handle_picker_input, render_picker)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

#[derive(Resource, Default)]
pub struct ItemPickerState {
    pub active: bool,
    pub action: Option<PickerAction>,
    pub selected_index: usize,
    pub filtered_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PickerAction {
    Eat,
    Quaff,
    Read,
    Zap, // Needs direction
    Apply,
    Wield,
    Wear,
    TakeOff,
    PutOn,  // Ring/Amulet
    Remove, // Ring/Amulet
    Drop,
    Throw, // Needs direction
}

impl PickerAction {
    pub fn name(&self) -> &'static str {
        match self {
            PickerAction::Eat => "eat",
            PickerAction::Quaff => "drink",
            PickerAction::Read => "read",
            PickerAction::Zap => "zap",
            PickerAction::Apply => "apply",
            PickerAction::Wield => "wield",
            PickerAction::Wear => "wear",
            PickerAction::TakeOff => "take off",
            PickerAction::PutOn => "put on",
            PickerAction::Remove => "remove",
            PickerAction::Drop => "drop",
            PickerAction::Throw => "throw",
        }
    }

    /// Returns true if the object matches the filter for this action
    pub fn filter(&self, obj: &Object) -> bool {
        match self {
            PickerAction::Eat => obj.class == ObjectClass::Food,
            PickerAction::Quaff => obj.class == ObjectClass::Potion,
            PickerAction::Read => {
                obj.class == ObjectClass::Scroll || obj.class == ObjectClass::Spellbook
            }
            PickerAction::Zap => obj.class == ObjectClass::Wand,
            PickerAction::Apply => {
                // Tools and some other applyable things
                obj.class == ObjectClass::Tool
            }
            PickerAction::Wield => {
                obj.class == ObjectClass::Weapon || obj.class == ObjectClass::Tool // Pick-axe, etc
            }
            PickerAction::Wear => obj.class == ObjectClass::Armor,
            PickerAction::TakeOff => obj.class == ObjectClass::Armor, // Logic usually checks if worn
            PickerAction::PutOn => {
                obj.class == ObjectClass::Ring || obj.class == ObjectClass::Amulet
            }
            PickerAction::Remove => {
                obj.class == ObjectClass::Ring || obj.class == ObjectClass::Amulet
            }
            PickerAction::Drop => true,
            PickerAction::Throw => true, // Can throw anything
        }
    }
}

fn handle_picker_input(
    input: Res<ButtonInput<KeyCode>>,
    mut picker_state: ResMut<ItemPickerState>,
    mut commands: EventWriter<GameCommand>,
    mut dir_state: ResMut<DirectionSelectState>,
    game_state: Res<GameStateResource>,
) {
    if !picker_state.active {
        return;
    }

    // Close with Escape
    if input.just_pressed(KeyCode::Escape) {
        picker_state.active = false;
        picker_state.action = None;
        return;
    }

    let inventory = &game_state.0.inventory;

    // Update filtered indices if needed (could be optimized to run only on open)
    // But fast enough for now to just re-calculate or assume they are correct from open time
    // For safety, let's recalculate if empty and we have an action, but usually handled when opening.
    if picker_state.filtered_indices.is_empty() && !inventory.is_empty() {
        // If it's truly empty because no items match, that's fine.
        // We rely on the opener to populate this.
    }

    let list_len = picker_state.filtered_indices.len();

    if list_len > 0 {
        // Navigation
        if input.just_pressed(KeyCode::KeyJ) || input.just_pressed(KeyCode::ArrowDown) {
            picker_state.selected_index = (picker_state.selected_index + 1) % list_len;
        }
        if input.just_pressed(KeyCode::KeyK) || input.just_pressed(KeyCode::ArrowUp) {
            picker_state.selected_index = (picker_state.selected_index + list_len - 1) % list_len;
        }

        // Selection with Enter or Space
        if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space) {
            confirm_selection(&mut picker_state, &mut commands, &mut dir_state, inventory);
        }

        // Selection by Inventory Letter (a-z, A-Z)
        // This is a bit complex with KeyCode, we might need Char input event for robustness
        // But for now let's stick to navigation + enter, or implement basic alpha keys if crucial.
    }
}

fn confirm_selection(
    picker_state: &mut ItemPickerState,
    commands: &mut EventWriter<GameCommand>,
    dir_state: &mut ResMut<DirectionSelectState>,
    inventory: &[Object],
) {
    if picker_state.filtered_indices.is_empty() {
        return;
    }

    let inv_idx = picker_state.filtered_indices[picker_state.selected_index];
    if inv_idx >= inventory.len() {
        return;
    }

    let item = &inventory[inv_idx];
    let item_char = item.inv_letter; // Assuming inv_letter is char

    if let Some(action) = picker_state.action {
        match action {
            PickerAction::Eat => {
                commands.send(GameCommand(Command::Eat(item_char)));
            }
            PickerAction::Quaff => {
                commands.send(GameCommand(Command::Quaff(item_char)));
            }
            PickerAction::Read => {
                commands.send(GameCommand(Command::Read(item_char)));
            }
            PickerAction::Wield => {
                commands.send(GameCommand(Command::Wield(Some(item_char))));
            }
            PickerAction::Wear => {
                commands.send(GameCommand(Command::Wear(item_char)));
            }
            PickerAction::TakeOff => {
                commands.send(GameCommand(Command::TakeOff(item_char)));
            }
            PickerAction::PutOn => {
                commands.send(GameCommand(Command::PutOn(item_char)));
            }
            PickerAction::Remove => {
                commands.send(GameCommand(Command::Remove(item_char)));
            }
            PickerAction::Drop => {
                commands.send(GameCommand(Command::Drop(item_char)));
            }
            PickerAction::Apply => {
                commands.send(GameCommand(Command::Apply(item_char)));
            }

            // Actions requiring direction next
            PickerAction::Zap => {
                // Transition to direction selection, carrying the item info
                dir_state.active = true;
                dir_state.action = Some(DirectionAction::Zap(item_char));
            }
            PickerAction::Throw => {
                dir_state.active = true;
                dir_state.action = Some(DirectionAction::Throw(item_char));
            }
        }
    }

    // Close picker
    picker_state.active = false;
    picker_state.action = None;
}

fn render_picker(
    mut contexts: EguiContexts,
    picker_state: Res<ItemPickerState>,
    game_state: Res<GameStateResource>,
) {
    if !picker_state.active {
        return;
    }

    let action_name = picker_state.action.map(|a| a.name()).unwrap_or("select");
    let inventory = &game_state.0.inventory;

    egui::Window::new(format!("Select item to {}", action_name))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.set_min_width(300.0);

            if picker_state.filtered_indices.is_empty() {
                ui.label(egui::RichText::new("No applicable items found.").italics());
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (i, &inv_idx) in picker_state.filtered_indices.iter().enumerate() {
                            if inv_idx >= inventory.len() {
                                continue;
                            }
                            let item = &inventory[inv_idx];

                            let is_selected = i == picker_state.selected_index;
                            let item_color = object_class_color(&item.class);

                            let text = format!(
                                "{} - {}{}",
                                item.inv_letter,
                                item_name(item),
                                if item.quantity > 1 {
                                    format!(" (x{})", item.quantity)
                                } else {
                                    String::new()
                                }
                            );

                            let response = ui.selectable_label(
                                is_selected,
                                egui::RichText::new(&text).color(item_color),
                            );

                            // Allow clicking too
                            if response.clicked() {
                                // We can't easily trigger the confirm logic here due to ownership
                                // But we could store a "clicked_index" in state to handle in input system
                                // For now, just rely on keyboard or let it select and user presses enter
                            }
                        }
                    });
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Select with arrows/j/k, Confirm with Enter")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

// Helper functions duplicated from inventory.rs (could be shared in a common module)
fn item_name(item: &nh_core::object::Object) -> String {
    let objects = nh_core::data::objects::OBJECTS;
    if (item.object_type as usize) < objects.len() {
        let obj_def = &objects[item.object_type as usize];
        obj_def.name.to_string()
    } else {
        format!("{:?}", item.class)
    }
}

fn object_class_color(class: &nh_core::object::ObjectClass) -> egui::Color32 {
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
