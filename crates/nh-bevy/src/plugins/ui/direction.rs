//! Direction selection UI for actions like open, close, kick

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::plugins::input::GameCommand;

pub struct DirectionPlugin;

impl Plugin for DirectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirectionSelectState>()
            .add_systems(Update, (handle_direction_input, render_direction_ui).chain());
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
}

impl DirectionAction {
    pub fn name(&self) -> &'static str {
        match self {
            DirectionAction::Open => "open",
            DirectionAction::Close => "close",
            DirectionAction::Kick => "kick",
            DirectionAction::Fight => "fight",
        }
    }

    pub fn to_command(&self, dir: nh_core::action::Direction) -> nh_core::action::Command {
        use nh_core::action::Command;
        match self {
            DirectionAction::Open => Command::Open,
            DirectionAction::Close => Command::Close,
            DirectionAction::Kick => Command::Kick,
            DirectionAction::Fight => Command::Fight(dir),
        }
    }
}

fn handle_direction_input(
    input: Res<ButtonInput<KeyCode>>,
    mut dir_state: ResMut<DirectionSelectState>,
    mut game_commands: EventWriter<GameCommand>,
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

    // Direction keys
    use nh_core::action::Direction;

    let direction = if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Numpad7) {
        Some(Direction::NorthWest)
    } else if input.just_pressed(KeyCode::KeyK) || input.just_pressed(KeyCode::Numpad8) || input.just_pressed(KeyCode::ArrowUp) {
        Some(Direction::North)
    } else if input.just_pressed(KeyCode::KeyU) || input.just_pressed(KeyCode::Numpad9) {
        Some(Direction::NorthEast)
    } else if input.just_pressed(KeyCode::KeyH) || input.just_pressed(KeyCode::Numpad4) || input.just_pressed(KeyCode::ArrowLeft) {
        Some(Direction::West)
    } else if input.just_pressed(KeyCode::Period) || input.just_pressed(KeyCode::Numpad5) {
        Some(Direction::Self_)
    } else if input.just_pressed(KeyCode::KeyL) || input.just_pressed(KeyCode::Numpad6) || input.just_pressed(KeyCode::ArrowRight) {
        Some(Direction::East)
    } else if input.just_pressed(KeyCode::KeyB) || input.just_pressed(KeyCode::Numpad1) {
        Some(Direction::SouthWest)
    } else if input.just_pressed(KeyCode::KeyJ) || input.just_pressed(KeyCode::Numpad2) || input.just_pressed(KeyCode::ArrowDown) {
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
            game_commands.send(GameCommand(command));
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
) {
    if !dir_state.active {
        return;
    }

    let action_name = dir_state
        .action
        .as_ref()
        .map(|a| a.name())
        .unwrap_or("select direction");

    egui::Area::new(egui::Id::new("direction_select"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220))
                .inner_margin(egui::Margin::same(16.0))
                .rounding(egui::Rounding::same(8.0))
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
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

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

                    ui.label(
                        egui::RichText::new("Press a direction key or Esc to cancel")
                            .color(egui::Color32::GRAY)
                            .small(),
                    );
                });
        });
}
