//! Input handling plugin - keyboard to game commands

use bevy::prelude::*;

use crate::plugins::game::AppState;
use crate::plugins::ui::direction::{DirectionAction, DirectionSelectState};
use crate::plugins::ui::InventoryState;
use crate::resources::GameStateResource;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameCommand>().add_systems(
            Update,
            (keyboard_to_command, process_game_command)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Game command event
#[derive(Event)]
pub struct GameCommand(pub nh_core::action::Command);

fn keyboard_to_command(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: EventWriter<GameCommand>,
    inv_state: Res<InventoryState>,
    dir_state: Res<DirectionSelectState>,
    mut dir_state_mut: ResMut<DirectionSelectState>,
) {
    // Don't process game input when UI panels are active
    if inv_state.open || dir_state.active {
        return;
    }

    use nh_core::action::{Command, Direction};

    // Vi-keys movement (hjklyubn)
    let direction = if input.just_pressed(KeyCode::KeyH) {
        Some(Direction::West)
    } else if input.just_pressed(KeyCode::KeyJ) {
        Some(Direction::South)
    } else if input.just_pressed(KeyCode::KeyK) {
        Some(Direction::North)
    } else if input.just_pressed(KeyCode::KeyL) {
        Some(Direction::East)
    } else if input.just_pressed(KeyCode::KeyY) {
        Some(Direction::NorthWest)
    } else if input.just_pressed(KeyCode::KeyU) {
        Some(Direction::NorthEast)
    } else if input.just_pressed(KeyCode::KeyB) {
        Some(Direction::SouthWest)
    } else if input.just_pressed(KeyCode::KeyN) {
        Some(Direction::SouthEast)
    }
    // Arrow keys
    else if input.just_pressed(KeyCode::ArrowUp) {
        Some(Direction::North)
    } else if input.just_pressed(KeyCode::ArrowDown) {
        Some(Direction::South)
    } else if input.just_pressed(KeyCode::ArrowLeft) {
        Some(Direction::West)
    } else if input.just_pressed(KeyCode::ArrowRight) {
        Some(Direction::East)
    } else {
        None
    };

    if let Some(dir) = direction {
        commands.send(GameCommand(Command::Move(dir)));
        return;
    }

    // Other commands
    if input.just_pressed(KeyCode::Period) && !input.pressed(KeyCode::ShiftLeft) {
        // '.' - rest/wait
        commands.send(GameCommand(Command::Rest));
    } else if input.just_pressed(KeyCode::Comma) && !input.pressed(KeyCode::ShiftLeft) {
        // ',' - pickup
        commands.send(GameCommand(Command::Pickup));
    } else if input.just_pressed(KeyCode::KeyS) {
        // 's' - search
        commands.send(GameCommand(Command::Search));
    } else if input.just_pressed(KeyCode::Semicolon) && input.pressed(KeyCode::ShiftLeft) {
        // ':' - look at floor
        commands.send(GameCommand(Command::WhatsHere));
    } else if input.just_pressed(KeyCode::Comma) && input.pressed(KeyCode::ShiftLeft) {
        // '<' - go up stairs
        commands.send(GameCommand(Command::GoUp));
    } else if input.just_pressed(KeyCode::Period) && input.pressed(KeyCode::ShiftLeft) {
        // '>' - go down stairs
        commands.send(GameCommand(Command::GoDown));
    } else if input.just_pressed(KeyCode::KeyO) {
        // 'o' - open (needs direction)
        dir_state_mut.active = true;
        dir_state_mut.action = Some(DirectionAction::Open);
    } else if input.just_pressed(KeyCode::KeyC) && !input.pressed(KeyCode::ShiftLeft) {
        // 'c' - close (needs direction)
        dir_state_mut.active = true;
        dir_state_mut.action = Some(DirectionAction::Close);
    } else if input.just_pressed(KeyCode::KeyD) && input.pressed(KeyCode::ControlLeft) {
        // Ctrl+D - kick (needs direction)
        dir_state_mut.active = true;
        dir_state_mut.action = Some(DirectionAction::Kick);
    }
    // Note: 'i' for inventory is handled by the inventory plugin itself
    // Note: 'e' for eat would need item selection UI
}

fn process_game_command(
    mut commands: EventReader<GameCommand>,
    mut game_state: ResMut<GameStateResource>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for GameCommand(command) in commands.read() {
        // Clear previous messages
        game_state.0.clear_messages();

        // Create game loop and execute command
        let state = std::mem::take(&mut game_state.0);
        let mut game_loop = nh_core::GameLoop::new(state);
        let result = game_loop.tick(command.clone());

        // Get state back
        game_state.0 = std::mem::take(game_loop.state_mut());

        // Handle result
        match result {
            nh_core::GameLoopResult::Continue => {
                // Messages will be displayed by the messages UI
            }
            nh_core::GameLoopResult::PlayerDied(msg) => {
                info!("GAME OVER: {}", msg);
                game_state.0.message(format!("You died: {}", msg));
                next_app_state.set(AppState::GameOver);
            }
            nh_core::GameLoopResult::PlayerQuit => {
                info!("Player quit");
            }
            nh_core::GameLoopResult::PlayerWon => {
                info!("You win!");
                game_state.0.message("You ascend to demigod-hood!");
            }
            nh_core::GameLoopResult::SaveAndQuit => {
                info!("Game saved");
            }
        }
    }
}
