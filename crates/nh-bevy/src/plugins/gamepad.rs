//! Gamepad input handling plugin

use bevy::prelude::*;
use nh_core::action::{Command, Direction};

use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use crate::plugins::ui::direction::{DirectionAction, DirectionSelectState};
use crate::plugins::ui::{InventoryState, ItemPickerState};
use crate::resources::GameStateResource;

pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (gamepad_input).run_if(in_state(AppState::Playing)));
    }
}

fn gamepad_input(
    gamepads: Query<(Entity, &Gamepad)>,
    mut commands: MessageWriter<GameCommand>,
    mut inv_state: ResMut<InventoryState>,
    mut dir_state: ResMut<DirectionSelectState>,
    picker_state: Res<ItemPickerState>,
    mut next_state: ResMut<NextState<AppState>>,
    _game_state: Res<GameStateResource>,
) {
    // Don't process game input when UI panels are active (except for their own navigation)
    // For now, we block if inventory or picker is open.
    if inv_state.open || dir_state.active || picker_state.active {
        // TODO: Map gamepad to UI navigation
        return;
    }

    for (_entity, gamepad) in &gamepads {
        // 1. Movement - D-pad
        let mut move_dir = None;

        if gamepad.just_pressed(GamepadButton::DPadUp) {
            move_dir = Some(Direction::North);
        } else if gamepad.just_pressed(GamepadButton::DPadDown) {
            move_dir = Some(Direction::South);
        } else if gamepad.just_pressed(GamepadButton::DPadLeft) {
            move_dir = Some(Direction::West);
        } else if gamepad.just_pressed(GamepadButton::DPadRight) {
            move_dir = Some(Direction::East);
        }

        // Analog Stick Movement (with deadzone)
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

        if move_dir.is_none() && (left_stick_x.abs() > 0.5 || left_stick_y.abs() > 0.5) {
            // Convert analog to 8-way discrete
            let angle = left_stick_y.atan2(left_stick_x);
            let octant = (angle / (std::f32::consts::PI / 4.0)).round() as i32;

            move_dir = match octant {
                0 => Some(Direction::East),
                1 => Some(Direction::NorthEast),
                2 => Some(Direction::North),
                3 => Some(Direction::NorthWest),
                -4 | 4 => Some(Direction::West),
                -3 => Some(Direction::SouthWest),
                -2 => Some(Direction::South),
                -1 => Some(Direction::SouthEast),
                _ => None,
            };
        }

        if let Some(dir) = move_dir {
            // Use Right Trigger or Button for Running
            if gamepad.pressed(GamepadButton::RightTrigger2)
                || gamepad.pressed(GamepadButton::RightTrigger)
            {
                commands.write(GameCommand(Command::Run(dir)));
            } else {
                commands.write(GameCommand(Command::Move(dir)));
            }
        }

        // 2. Buttons

        // A / Cross - Pickup
        if gamepad.just_pressed(GamepadButton::South) {
            commands.write(GameCommand(Command::Pickup));
        }

        // X / Square - Open Inventory
        if gamepad.just_pressed(GamepadButton::West) {
            inv_state.open = true;
        }

        // B / Circle - Rest / Wait
        if gamepad.just_pressed(GamepadButton::East) {
            commands.write(GameCommand(Command::Rest));
        }

        // Y / Triangle - Search
        if gamepad.just_pressed(GamepadButton::North) {
            commands.write(GameCommand(Command::Search));
        }

        // Start - Pause Menu
        if gamepad.just_pressed(GamepadButton::Start) {
            next_state.set(AppState::Paused);
        }

        // Select - Discoveries (placeholder for help)
        if gamepad.just_pressed(GamepadButton::Select) {
            commands.write(GameCommand(Command::Discoveries));
        }

        // L1 / LB - Open (needs direction)
        if gamepad.just_pressed(GamepadButton::LeftTrigger) {
            dir_state.active = true;
            dir_state.action = Some(DirectionAction::Open);
        }

        // R1 / RB - Kick (needs direction)
        if gamepad.just_pressed(GamepadButton::RightTrigger) {
            // If already used for running, this might conflict.
            // In NetHack, 'k' is kick. Let's use Right Bumper for Kick.
        }
    }
}
