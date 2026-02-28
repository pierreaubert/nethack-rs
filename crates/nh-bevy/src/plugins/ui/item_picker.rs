//! Item picker UI for selecting items for actions (eat, drop, etc.)

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use nh_core::action::Command;
use nh_core::object::{Object, ObjectClass};

use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use crate::plugins::ui::direction::{DirectionAction, DirectionSelectState};
use crate::resources::GameStateResource;

pub struct ItemPickerPlugin;

impl Plugin for ItemPickerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemPickerState>()
            .add_systems(Update, handle_picker_input.run_if(in_state(AppState::Playing)))
            .add_systems(
                EguiPrimaryContextPass,
                render_picker
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
    Dip,
    DipItem(char),
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
            PickerAction::Dip | PickerAction::DipItem(_) => "dip",
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
            PickerAction::Dip => true,   // Can dip most things
            PickerAction::DipItem(_) => obj.class == ObjectClass::Potion,
        }
    }
}

fn handle_picker_input(
    input: Res<ButtonInput<KeyCode>>,
    mut picker_state: ResMut<ItemPickerState>,
    mut commands: MessageWriter<GameCommand>,
    mut dir_state: ResMut<DirectionSelectState>,
    game_state: Res<GameStateResource>,
) {
    if !picker_state.active {
        return;
    }

    // Close with Escape
    if input.just_pressed(KeyCode::Escape) {
        if let Some(PickerAction::DipItem(target)) = picker_state.action {
            // Check if standing on fountain
            use nh_core::dungeon::CellType;
            let pos = game_state.0.player.pos;
            let cell_type = game_state.0.current_level.cell(pos.x as usize, pos.y as usize).typ;
            if cell_type == CellType::Fountain {
                commands.write(GameCommand(Command::Dip(target, None)));
            }
        }
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
    commands: &mut MessageWriter<GameCommand>,
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
                commands.write(GameCommand(Command::Eat(Some(item_char))));
            }
            PickerAction::Quaff => {
                commands.write(GameCommand(Command::Quaff(Some(item_char))));
            }
            PickerAction::Read => {
                commands.write(GameCommand(Command::Read(Some(item_char))));
            }
            PickerAction::Wield => {
                commands.write(GameCommand(Command::Wield(Some(item_char))));
            }
            PickerAction::Wear => {
                commands.write(GameCommand(Command::Wear(item_char)));
            }
            PickerAction::TakeOff => {
                commands.write(GameCommand(Command::TakeOff(item_char)));
            }
            PickerAction::PutOn => {
                commands.write(GameCommand(Command::PutOn(item_char)));
            }
            PickerAction::Remove => {
                commands.write(GameCommand(Command::Remove(item_char)));
            }
            PickerAction::Drop => {
                commands.write(GameCommand(Command::Drop(item_char)));
            }
            PickerAction::Apply => {
                commands.write(GameCommand(Command::Apply(item_char)));
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
            PickerAction::Dip => {
                // Next step: select what to dip into (potion)
                picker_state.action = Some(PickerAction::DipItem(item_char));
                picker_state.selected_index = 0;
                // Note: filtered_indices will be updated by handle_item_picker_input
                return;
            }
            PickerAction::DipItem(target) => {
                commands.write(GameCommand(Command::Dip(target, Some(item_char))));
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
        return ;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let action_name = picker_state.action.map(|a| a.name()).unwrap_or("select");
    let inventory = &game_state.0.inventory;

    let mut title = format!("Select item to {}", action_name);
    if let Some(PickerAction::DipItem(_)) = picker_state.action {
        use nh_core::dungeon::CellType;
        let pos = game_state.0.player.pos;
        let cell_type = game_state.0.current_level.cell(pos.x as usize, pos.y as usize).typ;
        if cell_type == CellType::Fountain {
            title += " (Esc for fountain)";
        }
    }

    egui::Window::new(title)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
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

use super::{item_name, object_class_color};
