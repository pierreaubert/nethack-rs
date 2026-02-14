//! Extended command browser UI
//! Provides a menu for browsing and executing extended commands (#-commands)

use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub struct ExtendedCommandsPlugin;

impl Plugin for ExtendedCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtendedCommandsState>().add_systems(
            Update,
            (handle_extended_commands_input, render_extended_commands)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Extended commands browser state
#[derive(Resource, Default)]
pub struct ExtendedCommandsState {
    pub open: bool,
    pub search_filter: String,
    pub selected_index: usize,
    pub filtered_commands: Vec<CommandInfo>,
}

/// Information about a single extended command
#[derive(Clone, Debug)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
}

/// Extended command categories
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandCategory {
    Meta,
    Gameplay,
    Wizard,
}

impl CommandCategory {
    pub fn name(&self) -> &'static str {
        match self {
            CommandCategory::Meta => "Meta",
            CommandCategory::Gameplay => "Gameplay",
            CommandCategory::Wizard => "Wizard",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            CommandCategory::Meta => egui::Color32::LIGHT_BLUE,
            CommandCategory::Gameplay => egui::Color32::LIGHT_GREEN,
            CommandCategory::Wizard => egui::Color32::from_rgb(200, 100, 255),
        }
    }
}

/// Handle input for extended commands browser
fn handle_extended_commands_input(
    input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ExtendedCommandsState>,
    mut commands: Commands,
) {
    // Toggle with '#' key (Shift+3 on most keyboards)
    let shift_held = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
    if shift_held && input.just_pressed(KeyCode::Digit3) {
        state.open = !state.open;
        if state.open {
            update_filtered_commands(&mut state);
        }
    }

    if input.just_pressed(KeyCode::Escape) && state.open {
        state.open = false;
    }

    if !state.open {
        return;
    }

    // Navigation when open
    let len = state.filtered_commands.len();
    if len > 0 {
        if input.just_pressed(KeyCode::KeyJ) || input.just_pressed(KeyCode::ArrowDown) {
            state.selected_index = (state.selected_index + 1) % len;
        }
        if input.just_pressed(KeyCode::KeyK) || input.just_pressed(KeyCode::ArrowUp) {
            state.selected_index = (state.selected_index + len.saturating_sub(1)) % len;
        }

        // Execute selected command with Enter
        if input.just_pressed(KeyCode::Enter)
            && state.selected_index < state.filtered_commands.len()
        {
            let cmd_name = state.filtered_commands[state.selected_index].name.clone();
            commands.send_event(GameCommand(nh_core::action::Command::ExtendedCommand(
                cmd_name,
            )));
            state.open = false;
        }
    }
}

/// Render the extended commands browser
fn render_extended_commands(
    mut contexts: EguiContexts,
    mut state: ResMut<ExtendedCommandsState>,
    prev_search: Local<String>,
) {
    if !state.open {
        return;
    }

    let mut needs_filter_update = false;

    egui::Window::new("Extended Commands (#)")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(true)
        .default_width(600.0)
        .default_height(450.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(
                egui::RichText::new("Extended Commands - Use # prefix to execute")
                    .strong()
                    .color(egui::Color32::LIGHT_BLUE),
            );
            ui.separator();

            // Search filter
            ui.horizontal(|ui| {
                ui.label("Search:");
                if ui.text_edit_singleline(&mut state.search_filter).changed() {
                    needs_filter_update = true;
                }
            });

            if needs_filter_update || state.search_filter != *prev_search {
                update_filtered_commands(&mut state);
            }

            // Commands list
            let commands_len = state.filtered_commands.len();
            if commands_len == 0 {
                ui.label(egui::RichText::new("No commands match your search").italics());
            } else {
                let mut clicked_index = None;
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (i, cmd) in state.filtered_commands.iter().enumerate() {
                            let selected = i == state.selected_index;

                            let response = ui.selectable_label(selected, "");

                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    cmd.category.color(),
                                    egui::RichText::new(&cmd.name).strong(),
                                );
                                ui.separator();
                                ui.label(egui::RichText::new(&cmd.description).small());
                            });

                            if response.clicked() {
                                clicked_index = Some(i);
                            }

                            ui.separator();
                        }
                    });
                if let Some(i) = clicked_index {
                    state.selected_index = i;
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("j/k")
                        .color(egui::Color32::YELLOW)
                        .monospace(),
                );
                ui.label("navigate");

                ui.separator();

                ui.label(
                    egui::RichText::new("Enter")
                        .color(egui::Color32::YELLOW)
                        .monospace(),
                );
                ui.label("execute");

                ui.separator();

                ui.label(
                    egui::RichText::new("#/Esc")
                        .color(egui::Color32::YELLOW)
                        .monospace(),
                );
                ui.label("close");
            });
        });
}

/// Update filtered commands based on search filter
fn update_filtered_commands(state: &mut ExtendedCommandsState) {
    let all_commands = get_all_commands();

    let search = state.search_filter.to_lowercase();
    state.filtered_commands = all_commands
        .into_iter()
        .filter(|cmd| {
            search.is_empty()
                || cmd.name.to_lowercase().contains(&search)
                || cmd.description.to_lowercase().contains(&search)
                || cmd.category.name().to_lowercase().contains(&search)
        })
        .collect();

    state.selected_index = 0;
}

/// Get all available extended commands
fn get_all_commands() -> Vec<CommandInfo> {
    vec![
        // Meta commands
        CommandInfo {
            name: "version".to_string(),
            description: "Show version information and build details".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "help".to_string(),
            description: "Display help and command reference".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "history".to_string(),
            description: "Show game history and background".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "key bindings".to_string(),
            description: "Display all key bindings and controls".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "menu controls".to_string(),
            description: "Show menu navigation controls".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "direction keys".to_string(),
            description: "Display directional movement keys".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "list commands".to_string(),
            description: "List all available extended commands".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "mode info".to_string(),
            description: "Show game mode information".to_string(),
            category: CommandCategory::Meta,
        },
        // Gameplay commands
        CommandInfo {
            name: "adjust".to_string(),
            description: "Adjust inventory letters for items".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "annotate".to_string(),
            description: "Add notes to the current level".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "discoveries".to_string(),
            description: "View all discovered item types".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "explore mode".to_string(),
            description: "Enter explore mode (no permadeath)".to_string(),
            category: CommandCategory::Gameplay,
        },
        // Wizard mode commands
        CommandInfo {
            name: "setwiz".to_string(),
            description: "Enable wizard mode (debug features)".to_string(),
            category: CommandCategory::Wizard,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_category_names() {
        assert_eq!(CommandCategory::Meta.name(), "Meta");
        assert_eq!(CommandCategory::Gameplay.name(), "Gameplay");
        assert_eq!(CommandCategory::Wizard.name(), "Wizard");
    }

    #[test]
    fn test_get_all_commands() {
        let commands = get_all_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name == "version"));
        assert!(commands.iter().any(|c| c.name == "help"));
    }
}
