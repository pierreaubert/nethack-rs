//! Help system UI panel
//!
//! Provides:
//! - In-game key reference
//! - Command explanations
//! - Toggle with '?' key

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::plugins::game::AppState;

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HelpState>().add_systems(
            Update,
            (toggle_help, render_help).run_if(in_state(AppState::Playing)),
        );
    }
}

/// Help panel display state
#[derive(Resource, Default)]
pub struct HelpState {
    pub open: bool,
    pub tab: HelpTab,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum HelpTab {
    #[default]
    Movement,
    Actions,
    Interface,
}

/// Toggle help with ? key
fn toggle_help(input: Res<ButtonInput<KeyCode>>, mut state: ResMut<HelpState>) {
    // ? is Shift+/
    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
        && input.just_pressed(KeyCode::Slash)
    {
        state.open = !state.open;
    }
    // Also F1 for help
    if input.just_pressed(KeyCode::F1) {
        state.open = !state.open;
    }
    // Close on Escape
    if input.just_pressed(KeyCode::Escape) && state.open {
        state.open = false;
    }
}

/// Render the help panel
fn render_help(mut contexts: EguiContexts, mut help_state: ResMut<HelpState>) {
    if !help_state.open {
        return;
    }

    egui::Window::new("Help")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .min_width(640.0)
        .show(contexts.ctx_mut().unwrap(), |ui| {
            ui.label(
                egui::RichText::new("NetHack-RS Help")
                    .size(20.0)
                    .strong()
                    .color(egui::Color32::GOLD),
            );

            ui.separator();

            // Tab selection
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(help_state.tab == HelpTab::Movement, "Movement")
                    .clicked()
                {
                    help_state.tab = HelpTab::Movement;
                }
                if ui
                    .selectable_label(help_state.tab == HelpTab::Actions, "Actions")
                    .clicked()
                {
                    help_state.tab = HelpTab::Actions;
                }
                if ui
                    .selectable_label(help_state.tab == HelpTab::Interface, "Interface")
                    .clicked()
                {
                    help_state.tab = HelpTab::Interface;
                }
            });

            ui.separator();
            ui.add_space(5.0);

            match help_state.tab {
                HelpTab::Movement => render_movement_help(ui),
                HelpTab::Actions => render_actions_help(ui),
                HelpTab::Interface => render_interface_help(ui),
            }

            ui.add_space(10.0);

            // Close button
            ui.vertical_centered(|ui| {
                if ui.button("Close (? or Esc)").clicked() {
                    help_state.open = false;
                }
            });
        });
}

/// Render a 2-column key reference section
fn render_two_column_keys(
    ui: &mut egui::Ui,
    grid_id: &str,
    keys: &[(&str, &str)],
    color: egui::Color32,
) {
    egui::Grid::new(grid_id)
        .num_columns(4)
        .spacing([10.0, 3.0])
        .min_col_width(40.0)
        .show(ui, |ui| {
            let half = keys.len().div_ceil(2);
            for i in 0..half {
                // Left column
                let (key, desc) = keys[i];
                ui.colored_label(color, key);
                ui.label(desc);
                // Right column
                if let Some((key2, desc2)) = keys.get(half + i) {
                    ui.colored_label(color, *key2);
                    ui.label(*desc2);
                }
                ui.end_row();
            }
        });
}

/// Render movement help tab
fn render_movement_help(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("VI Keys").strong());
    ui.add_space(3.0);

    let vi_keys: &[(&str, &str)] = &[
        ("y", "Move NW"),
        ("k", "Move N"),
        ("u", "Move NE"),
        ("h", "Move W"),
        ("l", "Move E"),
        ("b", "Move SW"),
        ("j", "Move S"),
        ("n", "Move SE"),
    ];
    render_two_column_keys(ui, "vi_grid", vi_keys, egui::Color32::LIGHT_BLUE);

    ui.add_space(6.0);
    ui.label(egui::RichText::new("Arrow Keys & Other Movement").strong());
    ui.add_space(3.0);

    let other_keys: &[(&str, &str)] = &[
        ("Arrows", "Move in direction"),
        ("Shift+dir", "Run in direction"),
        (".", "Wait / rest one turn"),
        (",", "Pick up items"),
        ("<", "Go up stairs"),
        (">", "Go down stairs"),
        ("s", "Search for hidden things"),
        ("Click", "Auto-navigate to tile"),
    ];
    render_two_column_keys(ui, "other_move_grid", other_keys, egui::Color32::LIGHT_BLUE);
}

/// Render actions help tab
fn render_actions_help(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Item Commands").strong());
    ui.add_space(3.0);

    let item_actions: &[(&str, &str)] = &[
        ("a", "Apply (use) a tool"),
        ("d", "Drop an item"),
        ("e", "Eat food"),
        ("q", "Quaff (drink) potion"),
        ("r", "Read scroll / spellbook"),
        ("t", "Throw an item"),
        ("w", "Wield a weapon"),
        ("z", "Zap a wand"),
        ("W", "Wear armor"),
        ("T", "Take off armor"),
        ("P", "Put on ring / amulet"),
        ("R", "Remove ring / amulet"),
    ];
    render_two_column_keys(ui, "item_actions_grid", item_actions, egui::Color32::LIGHT_GREEN);

    ui.add_space(6.0);
    ui.label(egui::RichText::new("World Commands").strong());
    ui.add_space(3.0);

    let world_actions: &[(&str, &str)] = &[
        ("o", "Open a door"),
        ("c", "Close a door"),
        ("Ctrl+D", "Kick"),
        ("F", "Fight in a direction"),
        (":", "Look at what's here"),
        ("i", "Open inventory"),
    ];
    render_two_column_keys(ui, "world_actions_grid", world_actions, egui::Color32::LIGHT_GREEN);
}

/// Render interface help tab
fn render_interface_help(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("UI Panels").strong());
    ui.add_space(3.0);

    let panels: &[(&str, &str)] = &[
        ("i", "Inventory"),
        ("@", "Character sheet"),
        ("\\", "Discoveries"),
        ("#", "Extended commands"),
        ("M", "Toggle minimap"),
        ("V", "Message history"),
        ("?  F1", "This help screen"),
        ("Esc", "Close panel / pause menu"),
    ];
    render_two_column_keys(ui, "panels_grid", panels, egui::Color32::LIGHT_YELLOW);

    ui.add_space(6.0);
    ui.label(egui::RichText::new("Camera Controls").strong());
    ui.add_space(3.0);

    let camera: &[(&str, &str)] = &[
        ("F2", "Top-down camera"),
        ("F3", "Isometric camera"),
        ("F4", "Third-person camera"),
        ("F5", "First-person camera"),
        ("Scroll", "Zoom in / out"),
        ("L-Drag", "Orbit camera"),
        ("R-Drag", "Pan camera"),
        ("Home", "Reset camera"),
    ];
    render_two_column_keys(ui, "camera_grid", camera, egui::Color32::LIGHT_YELLOW);

    ui.add_space(6.0);
    ui.label(egui::RichText::new("Tips").strong());
    ui.add_space(3.0);

    ui.label("Left-click on tiles to auto-navigate (stops at monsters).");
    ui.label("Minimap: red = hostile, green = pet, yellow = peaceful.");
    ui.label("Lower armor class (AC) is better.");
}
