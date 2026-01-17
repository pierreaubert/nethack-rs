//! Help system UI panel
//!
//! Provides:
//! - In-game key reference
//! - Command explanations
//! - Toggle with '?' key

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::plugins::game::AppState;

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HelpState>()
            .add_systems(Update, (toggle_help, render_help).run_if(in_state(AppState::Playing)));
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
        .min_width(500.0)
        .show(contexts.ctx_mut(), |ui| {
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

/// Render movement help tab
fn render_movement_help(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Movement Keys").strong());
    ui.add_space(5.0);

    egui::Grid::new("movement_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
            // VI keys
            ui.label(egui::RichText::new("VI Keys:").strong());
            ui.end_row();

            let vi_keys = [
                ("y", "Move diagonally up-left"),
                ("k", "Move up"),
                ("u", "Move diagonally up-right"),
                ("h", "Move left"),
                (".", "Wait (rest)"),
                ("l", "Move right"),
                ("b", "Move diagonally down-left"),
                ("j", "Move down"),
                ("n", "Move diagonally down-right"),
            ];

            for (key, desc) in vi_keys {
                ui.colored_label(egui::Color32::LIGHT_BLUE, key);
                ui.label(desc);
                ui.end_row();
            }

            ui.end_row();
            ui.label(egui::RichText::new("Arrow Keys:").strong());
            ui.end_row();

            let arrow_keys = [
                ("Up Arrow", "Move up"),
                ("Down Arrow", "Move down"),
                ("Left Arrow", "Move left"),
                ("Right Arrow", "Move right"),
            ];

            for (key, desc) in arrow_keys {
                ui.colored_label(egui::Color32::LIGHT_BLUE, key);
                ui.label(desc);
                ui.end_row();
            }

            ui.end_row();
            ui.label(egui::RichText::new("Other Movement:").strong());
            ui.end_row();

            let other_movement = [
                ("<", "Climb stairs up / use ladder up"),
                (">", "Climb stairs down / use ladder down"),
                ("g + direction", "Run in direction"),
            ];

            for (key, desc) in other_movement {
                ui.colored_label(egui::Color32::LIGHT_BLUE, key);
                ui.label(desc);
                ui.end_row();
            }
        });
}

/// Render actions help tab
fn render_actions_help(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Action Keys").strong());
    ui.add_space(5.0);

    egui::Grid::new("actions_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
            let actions = [
                ("a", "Apply (use) an item"),
                ("c", "Close a door"),
                ("d", "Drop an item"),
                ("e", "Eat something"),
                ("f", "Fire a projectile"),
                ("i", "Open inventory"),
                ("o", "Open a door"),
                ("p", "Pay shopkeeper"),
                ("q", "Quaff (drink) a potion"),
                ("r", "Read a scroll or book"),
                ("s", "Search for hidden things"),
                ("t", "Throw an item"),
                ("w", "Wield a weapon"),
                ("z", "Zap a wand"),
                ("D", "Drop multiple items"),
                ("E", "Engrave on the floor"),
                ("P", "Put on accessory"),
                ("Q", "Select ammunition"),
                ("R", "Remove accessory"),
                ("T", "Take off armor"),
                ("W", "Wear armor"),
                (",", "Pick up items"),
                (":", "Look at what's here"),
                (";", "Look at a position"),
            ];

            for (key, desc) in actions {
                ui.colored_label(egui::Color32::LIGHT_GREEN, key);
                ui.label(desc);
                ui.end_row();
            }
        });
}

/// Render interface help tab
fn render_interface_help(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Interface Keys").strong());
    ui.add_space(5.0);

    egui::Grid::new("interface_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
            let interface_keys = [
                ("Left Click", "Navigate to clicked tile (auto-path)"),
                ("Right Drag", "Pan camera"),
                ("Scroll", "Zoom camera"),
                ("Escape", "Open pause menu / cancel"),
                ("i", "Open inventory"),
                ("C", "Character sheet"),
                ("@", "Character sheet (alternate)"),
                ("M", "Toggle minimap"),
                ("?", "This help screen"),
                ("F1", "Help (alternate)"),
                ("F2", "Top-down camera"),
                ("F3", "Isometric camera"),
                ("F4", "Third-person camera"),
                ("F5", "First-person camera"),
                ("Home", "Reset camera zoom/pan"),
            ];

            for (key, desc) in interface_keys {
                ui.colored_label(egui::Color32::LIGHT_YELLOW, key);
                ui.label(desc);
                ui.end_row();
            }
        });

    ui.add_space(10.0);

    ui.label(egui::RichText::new("Tips").strong());
    ui.add_space(5.0);

    ui.label("- Left-click on tiles to auto-navigate using pathfinding");
    ui.label("- Navigation stops when encountering monsters");
    ui.label("- The minimap shows explored areas, monsters, and stairs");
    ui.label("- Red dots are hostile monsters, green are pets, yellow are peaceful");
    ui.label("- Lower armor class (AC) is better");
}
