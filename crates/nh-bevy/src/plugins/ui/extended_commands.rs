//! Extended command browser UI
//! Provides a menu for browsing and executing extended commands (#-commands)

use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use crate::plugins::ui::direction::{DirectionAction, DirectionSelectState};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use nh_core::action::Command;

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
    mut dir_state: ResMut<DirectionSelectState>,
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
            let cmd_name = &state.filtered_commands[state.selected_index].name;
            dispatch_extended_command(cmd_name, &mut commands, &mut dir_state);
            state.open = false;
        }
    }
}

/// Dispatch an extended command by name to the proper Command enum variant
fn dispatch_extended_command(
    name: &str,
    commands: &mut Commands,
    dir_state: &mut ResMut<DirectionSelectState>,
) {
    let lower = name.to_lowercase();
    // Simple commands (no extra input)
    let cmd = match lower.as_str() {
        "pray" => Some(Command::Pray),
        "offer" => Some(Command::Offer),
        "sit" => Some(Command::Sit),
        "chat" => Some(Command::Chat),
        "pay" => Some(Command::Pay),
        "dip" => Some(Command::Dip),
        "jump" => Some(Command::Jump),
        "ride" => Some(Command::Ride),
        "wipe" => Some(Command::Wipe),
        "invoke" => Some(Command::Invoke),
        "turn" => Some(Command::TurnUndead),
        "monster" => Some(Command::MonsterAbility),
        "enhance" => Some(Command::EnhanceSkill),
        "loot" => Some(Command::Loot),
        "travel" => Some(Command::Travel),
        "twoweapon" => Some(Command::TwoWeapon),
        "swap" => Some(Command::SwapWeapon),
        "search" => Some(Command::Search),
        "save" => Some(Command::Save),
        "quit" => Some(Command::Quit),
        "discoveries" | "known" => Some(Command::Discoveries),
        "history" => Some(Command::History),
        "attributes" => Some(Command::ShowAttributes),
        "conduct" => Some(Command::ShowConduct),
        "overview" => Some(Command::DungeonOverview),
        "spells" => Some(Command::ShowSpells),
        "equipment" => Some(Command::ShowEquipment),
        "vanquished" => Some(Command::Vanquished),
        "redraw" => Some(Command::Redraw),
        "gold" => Some(Command::CountGold),
        "help" | "version" => Some(Command::Help),
        _ => None,
    };

    if let Some(cmd) = cmd {
        commands.send_event(GameCommand(cmd));
        return;
    }

    // Direction-needing commands: open direction select UI
    let dir_action = match lower.as_str() {
        "untrap" => Some(DirectionAction::Untrap),
        "force" => Some(DirectionAction::Force),
        "fight" => Some(DirectionAction::Fight),
        "kick" => Some(DirectionAction::Kick),
        "open" => Some(DirectionAction::Open),
        "close" => Some(DirectionAction::Close),
        _ => None,
    };

    if let Some(action) = dir_action {
        dir_state.active = true;
        dir_state.action = Some(action);
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
            name: "help".to_string(),
            description: "Display help and command reference".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "version".to_string(),
            description: "Show version information and build details".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "history".to_string(),
            description: "Show game history and message log".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "discoveries".to_string(),
            description: "View all discovered item types".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "attributes".to_string(),
            description: "Show character attributes and stats".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "conduct".to_string(),
            description: "Show current conducts maintained".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "overview".to_string(),
            description: "Show dungeon overview".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "spells".to_string(),
            description: "Show known spells".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "equipment".to_string(),
            description: "Show worn and wielded equipment".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "vanquished".to_string(),
            description: "Show list of vanquished monsters".to_string(),
            category: CommandCategory::Meta,
        },
        CommandInfo {
            name: "gold".to_string(),
            description: "Count gold pieces".to_string(),
            category: CommandCategory::Meta,
        },
        // Gameplay: religious
        CommandInfo {
            name: "pray".to_string(),
            description: "Pray to your deity for help".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "offer".to_string(),
            description: "Offer a sacrifice to your deity".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "turn".to_string(),
            description: "Turn undead (clerics)".to_string(),
            category: CommandCategory::Gameplay,
        },
        // Gameplay: interaction
        CommandInfo {
            name: "chat".to_string(),
            description: "Chat with a nearby creature".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "pay".to_string(),
            description: "Pay a shopkeeper".to_string(),
            category: CommandCategory::Gameplay,
        },
        // Gameplay: actions
        CommandInfo {
            name: "sit".to_string(),
            description: "Sit down on the floor or a throne".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "dip".to_string(),
            description: "Dip an object into something".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "jump".to_string(),
            description: "Jump to a nearby location".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "ride".to_string(),
            description: "Ride or dismount a steed".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "wipe".to_string(),
            description: "Wipe off your face".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "invoke".to_string(),
            description: "Invoke a special power of an artifact".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "monster".to_string(),
            description: "Use a monster ability".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "loot".to_string(),
            description: "Loot a container on the floor".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "enhance".to_string(),
            description: "Enhance weapon skills".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "travel".to_string(),
            description: "Travel to a location on the map".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "twoweapon".to_string(),
            description: "Toggle two-weapon fighting".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "swap".to_string(),
            description: "Swap primary and secondary weapons".to_string(),
            category: CommandCategory::Gameplay,
        },
        // Gameplay: direction-needing
        CommandInfo {
            name: "untrap".to_string(),
            description: "Untrap a trap or chest".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "force".to_string(),
            description: "Force a lock open".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "kick".to_string(),
            description: "Kick something in a direction".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "fight".to_string(),
            description: "Force attack in a direction".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "open".to_string(),
            description: "Open a door or container".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "close".to_string(),
            description: "Close a door".to_string(),
            category: CommandCategory::Gameplay,
        },
        // Gameplay: meta
        CommandInfo {
            name: "adjust".to_string(),
            description: "Adjust inventory letters for items".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "save".to_string(),
            description: "Save and quit the game".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "quit".to_string(),
            description: "Quit the game (no save)".to_string(),
            category: CommandCategory::Gameplay,
        },
        CommandInfo {
            name: "redraw".to_string(),
            description: "Redraw the screen".to_string(),
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
