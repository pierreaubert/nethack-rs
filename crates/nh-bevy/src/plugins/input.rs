//! Input handling plugin - keyboard to game commands

use bevy::prelude::*;

use crate::components::CameraMode;
use crate::plugins::game::AppState;
use crate::plugins::ui::direction::{DirectionAction, DirectionSelectState};
use crate::plugins::ui::item_picker::{ItemPickerState, PickerAction};
use crate::plugins::ui::messages::MessageHistory;
use crate::plugins::ui::{DiscoveriesState, InventoryState};
use crate::resources::GameStateResource;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GameCommand>().add_systems(
            Update,
            (keyboard_to_command, process_game_command)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Game command message
#[derive(Message)]
pub struct GameCommand(pub nh_core::action::Command);

fn keyboard_to_command(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: MessageWriter<GameCommand>,
    inv_state: Res<InventoryState>,
    mut dir_state: ResMut<DirectionSelectState>,
    mut picker_state: ResMut<ItemPickerState>,
    discoveries_state: Res<DiscoveriesState>,
    msg_history: Res<MessageHistory>,
    game_state: Res<GameStateResource>,
    camera_mode: Res<State<CameraMode>>,
) {
    // Don't process game input when UI panels are active
    if inv_state.open
        || dir_state.active
        || picker_state.active
        || discoveries_state.open
        || msg_history.show_full_log
    {
        return;
    }

    use nh_core::action::{Command, Direction};

    // Helper to open item picker
    let mut open_picker = |action: PickerAction| {
        let inventory = &game_state.0.inventory;
        let filtered: Vec<usize> = inventory
            .iter()
            .enumerate()
            .filter(|(_, item)| action.filter(item))
            .map(|(i, _)| i)
            .collect();

        // If no items match, we could show a message, but for now just open empty or don't open?
        // Standard NetHack behavior: "You don't have anything to eat."
        // We open it anyway so user can see "No applicable items found" or use contextual Esc (e.g. fountain dip)
        
        picker_state.active = true;
        picker_state.action = Some(action);
        picker_state.selected_index = 0;
        picker_state.filtered_indices = filtered;
    };

    // Transform direction based on camera mode for intuitive 3D movement.
    // TopDown/Isometric: North is at screen top (no swap needed).
    // ThirdPerson/FirstPerson: camera faces +Z (South), so swap N↔S for arrow keys.
    let transform_for_camera = |dir: Direction| -> Direction {
        match camera_mode.get() {
            CameraMode::TopDown | CameraMode::Isometric => dir,
            CameraMode::ThirdPerson | CameraMode::FirstPerson => match dir {
                Direction::North => Direction::South,
                Direction::South => Direction::North,
                Direction::NorthEast => Direction::SouthEast,
                Direction::NorthWest => Direction::SouthWest,
                Direction::SouthEast => Direction::NorthEast,
                Direction::SouthWest => Direction::NorthWest,
                other => other,
            },
        }
    };

    // Vi-keys movement (hjklyubn) - keep original mapping for roguelike purists
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
    // Arrow keys - transform for camera-relative movement
    else if input.just_pressed(KeyCode::ArrowUp) {
        Some(transform_for_camera(Direction::North))
    } else if input.just_pressed(KeyCode::ArrowDown) {
        Some(transform_for_camera(Direction::South))
    } else if input.just_pressed(KeyCode::ArrowLeft) {
        Some(Direction::West)
    } else if input.just_pressed(KeyCode::ArrowRight) {
        Some(Direction::East)
    } else {
        None
    };

    if let Some(dir) = direction {
        if input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight) {
            commands.write(GameCommand(Command::Run(dir)));
        } else {
            commands.write(GameCommand(Command::Move(dir)));
        }
        return;
    }

    // Other commands
    if input.just_pressed(KeyCode::Period) && !input.pressed(KeyCode::ShiftLeft) {
        // '.' - rest/wait
        commands.write(GameCommand(Command::Rest));
    } else if input.just_pressed(KeyCode::Comma) && !input.pressed(KeyCode::ShiftLeft) {
        // ',' - pickup
        commands.write(GameCommand(Command::Pickup));
    } else if input.just_pressed(KeyCode::KeyS)
        && !input.pressed(KeyCode::ShiftLeft)
        && !input.pressed(KeyCode::ShiftRight)
    {
        // 's' - search
        commands.write(GameCommand(Command::Search));
    } else if input.just_pressed(KeyCode::Semicolon) && input.pressed(KeyCode::ShiftLeft) {
        // ':' - look at floor
        commands.write(GameCommand(Command::WhatsHere));
    } else if input.just_pressed(KeyCode::Comma) && input.pressed(KeyCode::ShiftLeft) {
        // '<' - go up stairs
        commands.write(GameCommand(Command::GoUp));
    } else if input.just_pressed(KeyCode::Period) && input.pressed(KeyCode::ShiftLeft) {
        // '>' - go down stairs
        commands.write(GameCommand(Command::GoDown));
    } else if input.just_pressed(KeyCode::KeyO) {
        // 'o' - open (needs direction)
        dir_state.active = true;
        dir_state.action = Some(DirectionAction::Open);
    } else if input.just_pressed(KeyCode::KeyC) && !input.pressed(KeyCode::ShiftLeft) {
        // 'c' - close (needs direction)
        dir_state.active = true;
        dir_state.action = Some(DirectionAction::Close);
    } else if input.just_pressed(KeyCode::KeyD) && input.pressed(KeyCode::ControlLeft) {
        // Ctrl+D - kick (needs direction)
        dir_state.active = true;
        dir_state.action = Some(DirectionAction::Kick);
    } else if input.just_pressed(KeyCode::KeyF)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
    {
        // 'F' - fight (needs direction)
        dir_state.active = true;
        dir_state.action = Some(DirectionAction::Fight);
    } else if input.just_pressed(KeyCode::KeyF)
        && !input.pressed(KeyCode::ShiftLeft)
        && !input.pressed(KeyCode::ShiftRight)
    {
        // 'f' - fire from quiver (needs direction)
        dir_state.active = true;
        dir_state.action = Some(DirectionAction::Fire);
    // '\' - discoveries: handled directly in discoveries.rs
    } else if input.just_pressed(KeyCode::KeyV)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
    {
        // 'V' - history
        commands.write(GameCommand(Command::History));
    }
    // Simple action keys (no extra input needed)
    else if input.just_pressed(KeyCode::KeyP)
        && !input.pressed(KeyCode::ShiftLeft)
        && !input.pressed(KeyCode::ShiftRight)
        && !input.pressed(KeyCode::ControlLeft)
    {
        // 'p' - pay shopkeeper
        commands.write(GameCommand(Command::Pay));
    } else if input.just_pressed(KeyCode::KeyX)
        && !input.pressed(KeyCode::ShiftLeft)
        && !input.pressed(KeyCode::ShiftRight)
        && !input.pressed(KeyCode::ControlLeft)
    {
        // 'x' - swap weapons
        commands.write(GameCommand(Command::SwapWeapon));
    } else if input.just_pressed(KeyCode::KeyX)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
    {
        // 'X' - two-weapon mode
        commands.write(GameCommand(Command::TwoWeapon));
    } else if input.just_pressed(KeyCode::KeyZ)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
    {
        // 'Z' - cast spell
        commands.write(GameCommand(Command::ShowSpells));
    } else if input.just_pressed(KeyCode::Digit4) && input.pressed(KeyCode::ShiftLeft) {
        // '$' - count gold
        commands.write(GameCommand(Command::CountGold));
    } else if input.just_pressed(KeyCode::Minus) {
        // '_' on most keyboards is Shift+Minus, but crossterm/bevy may vary
        if input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight) {
            // '_' - travel
            commands.write(GameCommand(Command::Travel));
        }
    } else if input.just_pressed(KeyCode::Equal)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
    {
        // '+' - enhance weapon skill
        commands.write(GameCommand(Command::EnhanceSkill));
    } else if input.just_pressed(KeyCode::KeyS)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
    {
        // 'S' - save game
        commands.write(GameCommand(Command::Save));
    }
    // Ctrl key combos
    else if input.just_pressed(KeyCode::KeyP) && input.pressed(KeyCode::ControlLeft) {
        // Ctrl+P - message history
        commands.write(GameCommand(Command::History));
    } else if input.just_pressed(KeyCode::KeyR) && input.pressed(KeyCode::ControlLeft) {
        // Ctrl+R - redraw
        commands.write(GameCommand(Command::Redraw));
    } else if input.just_pressed(KeyCode::KeyX) && input.pressed(KeyCode::ControlLeft) {
        // Ctrl+X - show attributes
        commands.write(GameCommand(Command::ShowAttributes));
    }
    // Item commands
    else if input.just_pressed(KeyCode::KeyE) && !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight) {
        // Check if any food on floor
        use nh_core::object::ObjectClass;
        let pos = game_state.0.player.pos;
        let has_food_on_floor = game_state.0.current_level.objects_at(pos.x, pos.y).iter().any(|o| o.class == ObjectClass::Food);
        if has_food_on_floor {
            commands.write(GameCommand(Command::Eat(None)));
        } else {
            open_picker(PickerAction::Eat);
        }
    } else if input.just_pressed(KeyCode::KeyQ) && !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight) {
        // Check if standing on fountain/sink
        use nh_core::dungeon::CellType;
        let pos = game_state.0.player.pos;
        let cell_type = game_state.0.current_level.cell(pos.x as usize, pos.y as usize).typ;
        if matches!(cell_type, CellType::Fountain | CellType::Sink) {
            commands.write(GameCommand(Command::Quaff(None)));
        } else {
            open_picker(PickerAction::Quaff);
        }
    } else if input.just_pressed(KeyCode::KeyR) && !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight) {
        // Check if standing on throne/statue
        use nh_core::dungeon::CellType;
        let pos = game_state.0.player.pos;
        let cell_type = game_state.0.current_level.cell(pos.x as usize, pos.y as usize).typ;
        if matches!(cell_type, CellType::Throne) {
            commands.write(GameCommand(Command::Read(None)));
        } else {
            open_picker(PickerAction::Read);
        }
    } else if input.just_pressed(KeyCode::KeyZ) && !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight) {
        open_picker(PickerAction::Zap);
    } else if input.just_pressed(KeyCode::KeyA) && !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight) {
        open_picker(PickerAction::Apply);
    } else if input.just_pressed(KeyCode::KeyW) && !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight) {
        open_picker(PickerAction::Wield);
    } else if input.just_pressed(KeyCode::KeyW) && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) {
        open_picker(PickerAction::Wear);
    } else if input.just_pressed(KeyCode::KeyT) && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) {
        open_picker(PickerAction::TakeOff);
    } else if input.just_pressed(KeyCode::KeyP) && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) {
        open_picker(PickerAction::PutOn);
    } else if input.just_pressed(KeyCode::KeyR) && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) {
        open_picker(PickerAction::Remove);
    } else if input.just_pressed(KeyCode::KeyD)
        && !input.pressed(KeyCode::ShiftLeft)
        && !input.pressed(KeyCode::ShiftRight)
        && !input.pressed(KeyCode::ControlLeft)
    {
        open_picker(PickerAction::Drop);
    } else if input.just_pressed(KeyCode::KeyT) && !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight) {
        open_picker(PickerAction::Throw);
    }
    // Actions
    else if input.just_pressed(KeyCode::KeyP) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+P - pray
        commands.write(GameCommand(Command::Pray));
    } else if input.just_pressed(KeyCode::KeyO) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+O - offer
        commands.write(GameCommand(Command::Offer));
    } else if input.just_pressed(KeyCode::KeyC) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+C - chat
        commands.write(GameCommand(Command::Chat));
    } else if input.just_pressed(KeyCode::KeyS) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+S - sit
        commands.write(GameCommand(Command::Sit));
    } else if input.just_pressed(KeyCode::KeyJ) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+J - jump
        commands.write(GameCommand(Command::Jump));
    } else if input.just_pressed(KeyCode::KeyI) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+I - invoke
        commands.write(GameCommand(Command::Invoke));
    } else if input.just_pressed(KeyCode::KeyL) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+L - loot
        commands.write(GameCommand(Command::Loot));
    } else if input.just_pressed(KeyCode::KeyT) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+T - turn undead
        commands.write(GameCommand(Command::TurnUndead));
    } else if input.just_pressed(KeyCode::KeyA) && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) {
        // Shift+A - monster ability
        commands.write(GameCommand(Command::MonsterAbility));
    } else if input.just_pressed(KeyCode::KeyR) && (input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)) {
        // Alt+R - ride
        commands.write(GameCommand(Command::Ride));
    }
}

fn process_game_command(
    mut commands: MessageReader<GameCommand>,
    mut game_state: ResMut<GameStateResource>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut game_over_info: ResMut<crate::resources::GameOverInfo>,
    mut exit: MessageWriter<AppExit>,
) {
    for GameCommand(command) in commands.read() {
        // Clear previous messages
        game_state.0.clear_messages();

        // Create game loop and execute command
        let state = std::mem::take(&mut game_state.0);
        let mut game_loop = nh_core::GameLoop::new(state);
        
        let result = if let nh_core::action::Command::Quaff(obj_letter) = command {
            let action_result = nh_core::action::quaff::dodrink(game_loop.state_mut(), *obj_letter);
            match action_result {
                nh_core::action::ActionResult::Died(msg) => nh_core::GameLoopResult::PlayerDied(msg),
                nh_core::action::ActionResult::Quit => nh_core::GameLoopResult::PlayerQuit,
                nh_core::action::ActionResult::Save => nh_core::GameLoopResult::SaveAndQuit,
                _ => nh_core::GameLoopResult::Continue,
            }
        } else if let nh_core::action::Command::Eat(obj_letter) = command {
            let action_result = nh_core::action::eat::do_eat(game_loop.state_mut(), *obj_letter);
            match action_result {
                nh_core::action::ActionResult::Died(msg) => nh_core::GameLoopResult::PlayerDied(msg),
                nh_core::action::ActionResult::Quit => nh_core::GameLoopResult::PlayerQuit,
                nh_core::action::ActionResult::Save => nh_core::GameLoopResult::SaveAndQuit,
                _ => nh_core::GameLoopResult::Continue,
            }
        } else if let nh_core::action::Command::Read(obj_letter) = command {
            let action_result = nh_core::action::read::do_read(game_loop.state_mut(), *obj_letter);
            match action_result {
                nh_core::action::ActionResult::Died(msg) => nh_core::GameLoopResult::PlayerDied(msg),
                nh_core::action::ActionResult::Quit => nh_core::GameLoopResult::PlayerQuit,
                nh_core::action::ActionResult::Save => nh_core::GameLoopResult::SaveAndQuit,
                _ => nh_core::GameLoopResult::Continue,
            }
        } else if let nh_core::action::Command::Dip(item, potion) = command {
            let action_result = nh_core::action::quaff::dodip(game_loop.state_mut(), *item, *potion);
            match action_result {
                nh_core::action::ActionResult::Died(msg) => nh_core::GameLoopResult::PlayerDied(msg),
                nh_core::action::ActionResult::Quit => nh_core::GameLoopResult::PlayerQuit,
                nh_core::action::ActionResult::Save => nh_core::GameLoopResult::SaveAndQuit,
                _ => nh_core::GameLoopResult::Continue,
            }
        } else {
            game_loop.tick(command.clone())
        };

        // Get state back
        game_state.0 = std::mem::take(game_loop.state_mut());

        // Handle result
        match result {
            nh_core::GameLoopResult::Continue => {
                // Messages will be displayed by the messages UI
            }
            nh_core::GameLoopResult::PlayerDied(msg) => {
                info!("GAME OVER: {}", msg);
                game_over_info.cause_of_death = Some(msg);
                game_over_info.is_victory = false;
                // Delete save file (permadeath)
                let path =
                    nh_core::save::default_save_path(&game_state.0.player.name);
                let _ = nh_core::save::delete_save(&path);
                next_app_state.set(AppState::GameOver);
            }
            nh_core::GameLoopResult::PlayerQuit => {
                exit.write(AppExit::Success);
            }
            nh_core::GameLoopResult::PlayerWon => {
                info!("YOU ASCENDED!");
                game_over_info.cause_of_death = None;
                game_over_info.is_victory = true;
                // Delete save file (ascension is permanent too)
                let path =
                    nh_core::save::default_save_path(&game_state.0.player.name);
                let _ = nh_core::save::delete_save(&path);
                next_app_state.set(AppState::Victory);
            }
            nh_core::GameLoopResult::SaveAndQuit => {
                let path =
                    nh_core::save::default_save_path(&game_state.0.player.name);
                if let Err(e) = nh_core::save::save_game(&game_state.0, &path) {
                    error!("Failed to save game: {}", e);
                }
                exit.write(AppExit::Success);
            }
        }
    }
}
