//! Direction selection UI for actions like open, close, kick

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use nh_core::magic::targeting::monsters_in_direction;

use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use crate::resources::GameStateResource;

pub struct DirectionPlugin;

impl Plugin for DirectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirectionSelectState>()
            .add_systems(Update, handle_direction_input.run_if(in_state(AppState::Playing)))
            .add_systems(
                EguiPrimaryContextPass,
                render_direction_ui
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// State for direction selection
#[derive(Resource, Default)]
pub struct DirectionSelectState {
    pub active: bool,
    pub action: Option<DirectionAction>,
    pub highlighted: Option<nh_core::action::Direction>,
}

#[derive(Clone, Copy, Debug)]
pub enum DirectionAction {
    Open,
    Close,
    Kick,
    Fight,
    Fire,
    Untrap,
    Force,
    Zap(char),
    Throw(char),
}

impl DirectionAction {
    pub fn name(&self) -> &'static str {
        match self {
            DirectionAction::Open => "open",
            DirectionAction::Close => "close",
            DirectionAction::Kick => "kick",
            DirectionAction::Fight => "fight",
            DirectionAction::Fire => "fire",
            DirectionAction::Untrap => "untrap",
            DirectionAction::Force => "force",
            DirectionAction::Zap(_) => "zap",
            DirectionAction::Throw(_) => "throw",
        }
    }

    pub fn to_command(&self, dir: nh_core::action::Direction) -> nh_core::action::Command {
        use nh_core::action::Command;
        match self {
            DirectionAction::Open => Command::Open(dir),
            DirectionAction::Close => Command::Close(dir),
            DirectionAction::Kick => Command::Kick(dir),
            DirectionAction::Fight => Command::Fight(dir),
            DirectionAction::Fire => Command::Fire(dir),
            DirectionAction::Untrap => Command::Untrap(dir),
            DirectionAction::Force => Command::Force(dir),
            DirectionAction::Zap(c) => Command::Zap(*c, dir),
            DirectionAction::Throw(c) => Command::Throw(*c, dir),
        }
    }
}

fn handle_direction_input(
    input: Res<ButtonInput<KeyCode>>,
    mut dir_state: ResMut<DirectionSelectState>,
    mut game_commands: MessageWriter<GameCommand>,
    mut monster_picker_state: ResMut<super::monster_picker::MonsterPickerState>,
) {
    if !dir_state.active {
        return;
    }

    // Cancel with Escape
    if input.just_pressed(KeyCode::Escape) {
        dir_state.active = false;
        dir_state.action = None;
        dir_state.highlighted = None;
        return;
    }

    // Switch to monster picker with Tab
    if input.just_pressed(KeyCode::Tab) {
        monster_picker_state.active = true;
        // Copy action type if applicable
        if let Some(DirectionAction::Zap(c)) = dir_state.action {
            monster_picker_state.action = Some(super::monster_picker::TargetAction::Zap(c));
        } else if let Some(DirectionAction::Throw(c)) = dir_state.action {
            monster_picker_state.action = Some(super::monster_picker::TargetAction::Throw(c));
        }
        dir_state.active = false;
        return;
    }

    // Direction keys
    use nh_core::action::Direction;

    let direction = if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Numpad7) {
        Some(Direction::NorthWest)
    } else if input.just_pressed(KeyCode::KeyK)
        || input.just_pressed(KeyCode::Numpad8)
        || input.just_pressed(KeyCode::ArrowUp)
    {
        Some(Direction::North)
    } else if input.just_pressed(KeyCode::KeyU) || input.just_pressed(KeyCode::Numpad9) {
        Some(Direction::NorthEast)
    } else if input.just_pressed(KeyCode::KeyH)
        || input.just_pressed(KeyCode::Numpad4)
        || input.just_pressed(KeyCode::ArrowLeft)
    {
        Some(Direction::West)
    } else if input.just_pressed(KeyCode::Period) || input.just_pressed(KeyCode::Numpad5) {
        Some(Direction::Self_)
    } else if input.just_pressed(KeyCode::KeyL)
        || input.just_pressed(KeyCode::Numpad6)
        || input.just_pressed(KeyCode::ArrowRight)
    {
        Some(Direction::East)
    } else if input.just_pressed(KeyCode::KeyB) || input.just_pressed(KeyCode::Numpad1) {
        Some(Direction::SouthWest)
    } else if input.just_pressed(KeyCode::KeyJ)
        || input.just_pressed(KeyCode::Numpad2)
        || input.just_pressed(KeyCode::ArrowDown)
    {
        Some(Direction::South)
    } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Numpad3) {
        Some(Direction::SouthEast)
    } else {
        None
    };

    if let Some(dir) = direction {
        if let Some(action) = &dir_state.action {
            // Send the command
            let command = action.to_command(dir);
            game_commands.write(GameCommand(command));
        }

        // Clear state
        dir_state.active = false;
        dir_state.action = None;
        dir_state.highlighted = None;
    }
}

fn render_direction_ui(
    mut contexts: EguiContexts,
    dir_state: Res<DirectionSelectState>,
    game_state: Res<GameStateResource>,
) {
    if !dir_state.active {
        return ;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let action_name = dir_state
        .action
        .as_ref()
        .map(|a| a.name())
        .unwrap_or("select direction");

    egui::Area::new(egui::Id::new("direction_select"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220))
                .inner_margin(egui::Margin::same(16))
                .corner_radius(egui::CornerRadius::same(8))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("Which direction to {}?", action_name))
                            .color(egui::Color32::WHITE)
                            .size(16.0),
                    );

                    ui.add_space(12.0);

                    // Direction grid
                    let cell_size = 40.0;
                    let grid_start = ui.cursor().min;

                    // Direction labels with vi-keys
                    let directions = [
                        ("y", "NW", 0, 0),
                        ("k", "N", 1, 0),
                        ("u", "NE", 2, 0),
                        ("h", "W", 0, 1),
                        (".", "@", 1, 1),
                        ("l", "E", 2, 1),
                        ("b", "SW", 0, 2),
                        ("j", "S", 1, 2),
                        ("n", "SE", 2, 2),
                    ];

                    for (key, label, col, row) in directions {
                        let pos = egui::pos2(
                            grid_start.x + col as f32 * cell_size,
                            grid_start.y + row as f32 * cell_size,
                        );
                        let rect = egui::Rect::from_min_size(pos, egui::vec2(cell_size - 2.0, cell_size - 2.0));

                        let is_center = col == 1 && row == 1;
                        let bg_color = if is_center {
                            egui::Color32::from_rgb(60, 60, 100)
                        } else {
                            egui::Color32::from_rgb(50, 50, 50)
                        };

                        ui.painter().rect_filled(rect, 4.0, bg_color);
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Inside);

                        // Key label
                        ui.painter().text(
                            rect.center() - egui::vec2(0.0, 6.0),
                            egui::Align2::CENTER_CENTER,
                            key,
                            egui::FontId::monospace(14.0),
                            egui::Color32::YELLOW,
                        );

                        // Direction label
                        ui.painter().text(
                            rect.center() + egui::vec2(0.0, 8.0),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(10.0),
                            egui::Color32::LIGHT_GRAY,
                        );
                    }

                    // Reserve space for the grid
                    ui.add_space(cell_size * 3.0 + 8.0);

                    // Show monsters in range per direction (for targeting spells/wands)
                    ui.label(
                        egui::RichText::new("Monsters in each direction:")
                            .color(egui::Color32::LIGHT_GREEN)
                            .size(12.0),
                    );

                    // Check each cardinal direction for monsters
                    let directions_to_check = [
                        ((0, -1), "North"),
                        ((1, -1), "Northeast"),
                        ((1, 0), "East"),
                        ((1, 1), "Southeast"),
                        ((0, 1), "South"),
                        ((-1, 1), "Southwest"),
                        ((-1, 0), "West"),
                        ((-1, -1), "Northwest"),
                    ];

                    for ((dx, dy), dir_name) in directions_to_check {
                        let monsters = monsters_in_direction(
                            &game_state.0.player,
                            (dx, dy),
                            &game_state.0.current_level,
                            20,
                        );

                        if !monsters.is_empty() {
                            let monster_names = monsters
                                .iter()
                                .take(2)
                                .map(|m| format!("{} ({})", m.name, m.distance))
                                .collect::<Vec<_>>()
                                .join(", ");

                            ui.label(
                                egui::RichText::new(format!("{}: {}", dir_name, monster_names))
                                    .color(egui::Color32::YELLOW)
                                    .size(10.0),
                            );
                        }
                    }

                    ui.separator();

                    ui.label(
                        egui::RichText::new("Press Tab for monster picker, direction key to select, or Esc to cancel")
                            .color(egui::Color32::GRAY)
                            .small(),
                    );
                });
        });

    
}
