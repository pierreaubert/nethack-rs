//! Monster picker UI for selecting targets for spells and wands

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use nh_core::action::{Command, Direction};
use nh_core::magic::targeting::{TargetInfo, find_monsters_in_range};

use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use crate::resources::GameStateResource;

pub struct MonsterPickerPlugin;

impl Plugin for MonsterPickerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MonsterPickerState>().add_systems(
            Update,
            (handle_picker_input, update_monster_list)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
        app.add_systems(
            EguiPrimaryContextPass,
            render_picker
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Actions that use monster targeting
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TargetAction {
    Zap(char),
    Throw(char),
    Spell,
}

impl TargetAction {
    pub fn name(&self) -> &'static str {
        match self {
            TargetAction::Zap(_) => "zap",
            TargetAction::Throw(_) => "throw at",
            TargetAction::Spell => "cast spell at",
        }
    }
}

/// Monster information for the picker
#[derive(Clone, Debug)]
pub struct PickerMonsterInfo {
    pub index: usize,
    pub target: TargetInfo,
    pub health_bar: String,
}

impl PickerMonsterInfo {
    fn new(index: usize, target: TargetInfo) -> Self {
        let health_percent = target.health_percent();
        let filled = (health_percent / 10).max(1).min(10) as usize;
        let empty = 10 - filled;
        let health_bar = format!(
            "[{}{}] {}%",
            "=".repeat(filled),
            " ".repeat(empty),
            health_percent
        );

        Self {
            index,
            target,
            health_bar,
        }
    }
}

#[derive(Resource)]
pub struct MonsterPickerState {
    pub active: bool,
    pub action: Option<TargetAction>,
    pub selected_index: usize,
    pub monsters: Vec<PickerMonsterInfo>,
    pub last_refresh: u32,
}

impl Default for MonsterPickerState {
    fn default() -> Self {
        Self {
            active: false,
            action: None,
            selected_index: 0,
            monsters: Vec::new(),
            last_refresh: 0,
        }
    }
}

/// Update the monster list (refresh every frame when active)
fn update_monster_list(
    mut picker_state: ResMut<MonsterPickerState>,
    game_state: Res<GameStateResource>,
) {
    if !picker_state.active {
        return;
    }

    picker_state.last_refresh += 1;

    // Get all monsters in range (max 20 tiles)
    let targets = find_monsters_in_range(&game_state.0.player, &game_state.0.current_level, 20);

    picker_state.monsters = targets
        .into_iter()
        .enumerate()
        .map(|(idx, target)| PickerMonsterInfo::new(idx, target))
        .collect();

    // Reset selection if it's out of bounds
    if picker_state.selected_index >= picker_state.monsters.len() {
        picker_state.selected_index = picker_state.monsters.len().saturating_sub(1);
    }
}

/// Convert a (dx, dy) vector to a Direction enum
fn vector_to_direction(dx: i8, dy: i8) -> Direction {
    match (dx.signum(), dy.signum()) {
        (0, -1) => Direction::North,
        (0, 1) => Direction::South,
        (1, 0) => Direction::East,
        (-1, 0) => Direction::West,
        (1, -1) => Direction::NorthEast,
        (-1, -1) => Direction::NorthWest,
        (1, 1) => Direction::SouthEast,
        (-1, 1) => Direction::SouthWest,
        (0, 0) => Direction::Self_,
        _ => Direction::Self_, // Shouldn't happen, but default to self
    }
}

fn handle_picker_input(
    input: Res<ButtonInput<KeyCode>>,
    mut picker_state: ResMut<MonsterPickerState>,
    game_state: Res<GameStateResource>,
    mut game_commands: MessageWriter<GameCommand>,
) {
    if !picker_state.active {
        return;
    }

    // Close with Escape
    if input.just_pressed(KeyCode::Escape) {
        picker_state.active = false;
        picker_state.action = None;
        picker_state.monsters.clear();
        return;
    }

    let list_len = picker_state.monsters.len();

    if list_len > 0 {
        // Navigation
        if input.just_pressed(KeyCode::KeyJ) || input.just_pressed(KeyCode::ArrowDown) {
            picker_state.selected_index = (picker_state.selected_index + 1) % list_len;
        }
        if input.just_pressed(KeyCode::KeyK) || input.just_pressed(KeyCode::ArrowUp) {
            picker_state.selected_index = (picker_state.selected_index + list_len - 1) % list_len;
        }

        // Selection with Enter or Space - trigger spell/zap with calculated direction
        if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space) {
            if let Some(monster_info) = picker_state.monsters.get(picker_state.selected_index) {
                if let Some(action) = picker_state.action {
                    // Get the target position
                    let target = &monster_info.target;

                    // Calculate direction from player to target
                    let dx = (target.x - game_state.0.player.pos.x).signum() as i8;
                    let dy = (target.y - game_state.0.player.pos.y).signum() as i8;
                    let direction = vector_to_direction(dx, dy);

                    // Send the appropriate command
                    let command = match action {
                        TargetAction::Zap(c) => Command::Zap(c, direction),
                        TargetAction::Throw(c) => Command::Throw(c, direction),
                        TargetAction::Spell => {
                            // Spell targeting would be handled differently
                            // For now, treat like zap
                            Command::Zap('?', direction)
                        }
                    };

                    game_commands.write(GameCommand(command));
                }
            }

            // Clear state
            picker_state.active = false;
            picker_state.action = None;
            picker_state.monsters.clear();
        }
    }
}

fn render_picker(
    mut contexts: EguiContexts,
    picker_state: Res<MonsterPickerState>,
) {
    if !picker_state.active {
        return ;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let action_name = picker_state.action.map(|a| a.name()).unwrap_or("target");

    egui::Window::new(format!("Select monster to {}", action_name))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.set_min_width(400.0);

            if picker_state.monsters.is_empty() {
                ui.label(
                    egui::RichText::new("No monsters visible in range.")
                        .italics()
                        .color(egui::Color32::GRAY),
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for monster_info in picker_state.monsters.iter() {
                            let is_selected = monster_info.index == picker_state.selected_index;
                            let text_color = if monster_info.target.is_hostile {
                                egui::Color32::RED
                            } else {
                                egui::Color32::YELLOW
                            };

                            let text = format!(
                                "{:>2}. {} ({} away) - {}",
                                monster_info.index + 1,
                                monster_info.target.name,
                                monster_info.target.distance,
                                monster_info.health_bar,
                            );

                            let _response = ui.selectable_label(
                                is_selected,
                                egui::RichText::new(&text).color(text_color).monospace(),
                            );
                        }
                    });

                ui.separator();

                // Show details of selected monster
                if picker_state.selected_index < picker_state.monsters.len() {
                    let selected = &picker_state.monsters[picker_state.selected_index];
                    let target = &selected.target;
                    let hp_info = format!(
                        "{} HP / {} HP ({}%)",
                        target.hp,
                        target.hp_max,
                        target.health_percent()
                    );

                    ui.label(egui::RichText::new(&format!("Selected: {}", target.name)).strong());
                    ui.label(format!("Distance: {}", target.distance));
                    ui.label(format!("Health: {}", hp_info));
                }
            }

            ui.separator();
            ui.label(
                egui::RichText::new(
                    "Navigate with arrows/j/k, Confirm with Enter, Cancel with Esc",
                )
                .small()
                .color(egui::Color32::GRAY),
            );
        });

    
}
