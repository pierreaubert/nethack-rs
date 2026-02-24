//! Key bindings display UI
//! Shows all available key bindings and controls in the game

use crate::plugins::game::AppState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub struct KeyBindingsPlugin;

impl Plugin for KeyBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindingsState>().add_systems(
            Update,
            (handle_key_bindings_input, render_key_bindings)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Key bindings display state
#[derive(Resource, Default)]
pub struct KeyBindingsState {
    pub open: bool,
    pub active_tab: KeyBindingTab,
}

/// Available tabs for key binding display
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyBindingTab {
    #[default]
    Movement,
    Actions,
    Equipment,
    Magic,
    System,
}

/// Handle input for key bindings display
fn handle_key_bindings_input(
    input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<KeyBindingsState>,
) {
    // Open with Alt+? or Alt+Slash
    if (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight))
        && input.just_pressed(KeyCode::Slash)
    {
        state.open = !state.open;
    }

    if input.just_pressed(KeyCode::Escape) && state.open {
        state.open = false;
    }

    if state.open {
        // Tab navigation
        if input.just_pressed(KeyCode::KeyH) || input.just_pressed(KeyCode::KeyL) {
            // Cycle tabs with h/l keys
            state.active_tab = match state.active_tab {
                KeyBindingTab::Movement => KeyBindingTab::Actions,
                KeyBindingTab::Actions => KeyBindingTab::Equipment,
                KeyBindingTab::Equipment => KeyBindingTab::Magic,
                KeyBindingTab::Magic => KeyBindingTab::System,
                KeyBindingTab::System => KeyBindingTab::Movement,
            };
        }
    }
}

/// Render the key bindings display
fn render_key_bindings(mut contexts: EguiContexts, mut state: ResMut<KeyBindingsState>) {
    if !state.open {
        return;
    }

    egui::Window::new("Key Bindings")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(true)
        .default_width(700.0)
        .default_height(500.0)
        .show(contexts.ctx_mut().unwrap(), |ui| {
            ui.label(
                egui::RichText::new("Key Bindings Reference")
                    .strong()
                    .size(18.0),
            );
            ui.separator();

            // Tab selection
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(state.active_tab == KeyBindingTab::Movement, "Movement")
                    .clicked()
                {
                    state.active_tab = KeyBindingTab::Movement;
                }
                if ui
                    .selectable_label(state.active_tab == KeyBindingTab::Actions, "Actions")
                    .clicked()
                {
                    state.active_tab = KeyBindingTab::Actions;
                }
                if ui
                    .selectable_label(state.active_tab == KeyBindingTab::Equipment, "Equipment")
                    .clicked()
                {
                    state.active_tab = KeyBindingTab::Equipment;
                }
                if ui
                    .selectable_label(state.active_tab == KeyBindingTab::Magic, "Magic")
                    .clicked()
                {
                    state.active_tab = KeyBindingTab::Magic;
                }
                if ui
                    .selectable_label(state.active_tab == KeyBindingTab::System, "System")
                    .clicked()
                {
                    state.active_tab = KeyBindingTab::System;
                }
            });

            ui.separator();

            // Content based on selected tab
            egui::ScrollArea::vertical()
                .max_height(350.0)
                .show(ui, |ui| match state.active_tab {
                    KeyBindingTab::Movement => render_movement_keys(ui),
                    KeyBindingTab::Actions => render_action_keys(ui),
                    KeyBindingTab::Equipment => render_equipment_keys(ui),
                    KeyBindingTab::Magic => render_magic_keys(ui),
                    KeyBindingTab::System => render_system_keys(ui),
                });

            ui.separator();
            ui.label(
                egui::RichText::new("h/l to switch tabs • Esc to close")
                    .color(egui::Color32::GRAY)
                    .small(),
            );
        });
}

/// Render movement key bindings
fn render_movement_keys(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Movement Keys").strong());
    ui.separator();

    egui::Grid::new("movement_grid")
        .num_columns(3)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            key_row(ui, "hjklyubn", "VI-keys for 8-direction movement");
            key_row(ui, "hjkl", "Move left/down/up/right");
            key_row(ui, "yubn", "Diagonal movement");
            key_row(ui, "Arrow keys", "Alternative movement");
            key_row(ui, ".", "Stay in place / rest");
            key_row(ui, "<", "Go up stairs");
            key_row(ui, ">", "Go down stairs");
            key_row(ui, "g", "Pick up item (same as ,)");
        });
}

/// Render action key bindings
fn render_action_keys(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Action Keys").strong());
    ui.separator();

    egui::Grid::new("action_grid")
        .num_columns(3)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            key_row(ui, ",", "Pick up item");
            key_row(ui, "d", "Drop item from inventory");
            key_row(ui, "e", "Eat food or corpse");
            key_row(ui, "a", "Apply tool or wand");
            key_row(ui, "q", "Drink potion");
            key_row(ui, "r", "Read scroll or spellbook");
            key_row(ui, "o", "Open door (then direction)");
            key_row(ui, "c", "Close door (then direction)");
            key_row(ui, "s", "Search for secret passages");
            key_row(ui, "p", "Pay at shop or toll");
            key_row(ui, "!", "Chat with NPC");
        });
}

/// Render equipment key bindings
fn render_equipment_keys(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Equipment Keys").strong());
    ui.separator();

    egui::Grid::new("equipment_grid")
        .num_columns(3)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            key_row(ui, "w", "Wear armor (from inventory)");
            key_row(ui, "W", "Take off armor");
            key_row(ui, "P", "Put on ring or amulet");
            key_row(ui, "R", "Remove ring or amulet");
            key_row(ui, "x", "Equip weapon");
            key_row(ui, "X", "Unequip weapon");
            key_row(ui, "i", "View inventory");
            key_row(ui, "'", "Show equipped items");
        });
}

/// Render magic key bindings
fn render_magic_keys(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Magic & Combat Keys").strong());
    ui.separator();

    egui::Grid::new("magic_grid")
        .num_columns(3)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            key_row(ui, "z", "Zap wand (then direction)");
            key_row(ui, "t", "Throw item at target");
            key_row(ui, "f", "Fire missile weapon");
            key_row(ui, "^", "Invoke spell or prayer");
            key_row(ui, "p", "Pray at altar");
            key_row(ui, "F", "Force-attack monster");
            key_row(ui, "m", "Move without attacking");
            key_row(ui, "v", "View current level");
        });
}

/// Render system key bindings
fn render_system_keys(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("System Keys").strong());
    ui.separator();

    egui::Grid::new("system_grid")
        .num_columns(3)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            key_row(ui, "?", "Show help");
            key_row(ui, "#", "Extended commands menu");
            key_row(ui, "\\", "Show discoveries");
            key_row(ui, "Ctrl-P", "Show message history");
            key_row(ui, "S", "Save game");
            key_row(ui, "q", "Quit game");
            key_row(ui, "Escape", "Cancel current action");
            key_row(ui, "`", "Show key bindings (this menu)");
        });
}

/// Helper to render a key binding row
fn key_row(ui: &mut egui::Ui, keys: &str, description: &str) {
    ui.colored_label(egui::Color32::LIGHT_BLUE, keys);
    ui.separator();
    ui.label(description);
    ui.end_row();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding_tabs() {
        assert_eq!(KeyBindingTab::Movement as u8, 0);
        // Verify all tabs exist
        let _tabs = [
            KeyBindingTab::Movement,
            KeyBindingTab::Actions,
            KeyBindingTab::Equipment,
            KeyBindingTab::Magic,
            KeyBindingTab::System,
        ];
    }
}
