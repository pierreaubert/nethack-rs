//! Inventory UI panel

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::plugins::game::AppState;
use crate::resources::{GameStateResource, AssetRegistryResource};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryState>()
            .add_systems(Update, handle_inventory_input.run_if(in_state(AppState::Playing)))
            .add_systems(
                EguiPrimaryContextPass,
                render_inventory
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Inventory UI state
#[derive(Resource, Default)]
pub struct InventoryState {
    pub open: bool,
    pub selected_index: usize,
    pub action_mode: Option<InventoryAction>,
}

#[derive(Clone, Copy, Debug)]
pub enum InventoryAction {
    Drop,
    Eat,
    Wear,
    Wield,
    Apply,
    Read,
    Quaff,
}

impl InventoryAction {
    pub fn name(&self) -> &'static str {
        match self {
            InventoryAction::Drop => "Drop",
            InventoryAction::Eat => "Eat",
            InventoryAction::Wear => "Wear",
            InventoryAction::Wield => "Wield",
            InventoryAction::Apply => "Apply",
            InventoryAction::Read => "Read",
            InventoryAction::Quaff => "Quaff",
        }
    }
}

fn handle_inventory_input(
    input: Res<ButtonInput<KeyCode>>,
    mut inv_state: ResMut<InventoryState>,
    game_state: Res<GameStateResource>,
) {
    // Toggle inventory with 'i'
    if input.just_pressed(KeyCode::KeyI) {
        inv_state.open = !inv_state.open;
        inv_state.action_mode = None;
    }

    // Close with Escape
    if input.just_pressed(KeyCode::Escape) && inv_state.open {
        inv_state.open = false;
        inv_state.action_mode = None;
    }

    // Navigate with j/k or up/down when open
    if inv_state.open {
        let item_count = game_state.0.inventory.len();
        if item_count > 0 {
            if input.just_pressed(KeyCode::KeyJ) || input.just_pressed(KeyCode::ArrowDown) {
                inv_state.selected_index = (inv_state.selected_index + 1) % item_count;
            }
            if input.just_pressed(KeyCode::KeyK) || input.just_pressed(KeyCode::ArrowUp) {
                inv_state.selected_index = (inv_state.selected_index + item_count - 1) % item_count;
            }
        }

        // Action shortcuts when inventory is open
        if input.just_pressed(KeyCode::KeyD) {
            inv_state.action_mode = Some(InventoryAction::Drop);
        }
        if input.just_pressed(KeyCode::KeyE) {
            inv_state.action_mode = Some(InventoryAction::Eat);
        }
        if input.just_pressed(KeyCode::KeyW) {
            inv_state.action_mode = Some(InventoryAction::Wear);
        }
    }
}

fn render_inventory(
    mut contexts: EguiContexts,
    inv_state: Res<InventoryState>,
    game_state: Res<GameStateResource>,
    asset_registry: Res<AssetRegistryResource>,
) {
    if !inv_state.open {
        return ;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let inventory = &game_state.0.inventory;

    egui::Window::new("Inventory")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.set_min_width(350.0);
            ui.set_min_height(300.0);

            // Header with weight
            ui.horizontal(|ui| {
                let weight = game_state.0.inventory_weight();
                let capacity = game_state.0.player.carrying_capacity;
                ui.label(
                    egui::RichText::new(format!("Weight: {}/{}", weight, capacity)).color(
                        if weight > capacity as u32 {
                            egui::Color32::RED
                        } else {
                            egui::Color32::GRAY
                        },
                    ),
                );

                if let Some(action) = &inv_state.action_mode {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Select item to {}", action.name()))
                            .color(egui::Color32::YELLOW),
                    );
                }
            });

            ui.separator();

            if inventory.is_empty() {
                ui.label(egui::RichText::new("Your inventory is empty.").italics());
            } else {
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for (idx, item) in inventory.iter().enumerate() {
                            let is_selected = idx == inv_state.selected_index;

                            let item_color = object_class_color(&item.class);

                            // Check for mapped icon
                            let mut prefix = String::new();
                            if let Ok(icon) = asset_registry.0.get_icon(item) {
                                prefix = format!("[{}] ", icon.tui_char);
                            }

                            let text = format!(
                                "{}{} - {}{}",
                                prefix,
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

                            if response.clicked() {
                                // Could trigger item action here
                            }
                        }
                    });
            }

            ui.separator();

            // Footer with controls
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("j/k:Navigate  d:Drop  e:Eat  w:Wear  Esc:Close")
                        .color(egui::Color32::GRAY)
                        .small(),
                );
            });
        });

    
}

use super::{item_name, object_class_color};
