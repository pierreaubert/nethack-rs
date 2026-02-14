//! Extract unified game state from the NetHack FFI implementation.
//!
//! This module provides functionality to convert the NetHack FFI game state
//! into a UnifiedGameState for comparison with the Rust implementation.

use crate::c_interface_ffi::FfiGameEngine;
use crate::state::common::*;

/// Wrapper for FFI game engine state extraction
pub struct FfiGameWrapper<'a> {
    engine: &'a mut FfiGameEngine,
}

impl<'a> FfiGameWrapper<'a> {
    /// Create a new FFI game wrapper
    pub fn new(engine: &'a mut FfiGameEngine) -> Self {
        Self { engine }
    }

    /// Extract unified state from FFI implementation
    pub fn extract_state(&self) -> UnifiedGameState {
        // Parse JSON state from FFI engine
        let json = self.engine.state_json();
        let json_value: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|_| serde_json::json!({}));

        let player_info = &json_value["player"];

        UnifiedGameState {
            player: UnifiedPlayer {
                name: "Player".to_string(),
                role: self.engine.role(),
                race: self.engine.race(),
                gender: self.engine.gender_string(),
                alignment: self.engine.alignment_string(),
            },
            position: self.engine.position(),
            hp: self.engine.hp(),
            max_hp: self.engine.max_hp(),
            energy: self.engine.energy(),
            max_energy: self.engine.max_energy(),
            armor_class: self.engine.armor_class(),
            gold: self.engine.gold(),
            experience_level: self.engine.experience_level(),
            strength: 10, // Simplified - not in FFI
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            current_level: self.engine.current_level(),
            dungeon_depth: self.engine.dungeon_depth(),
            dungeon_visited: vec![1],
            has_amulet: false,
            turn: self.engine.turn_count(),
            hunger: HungerState::NotHungry,
            status_effects: Vec::new(),
            inventory: extract_inventory(self.engine),
            nearby_monsters: extract_ffi_monsters(self.engine),
            conduct: ConductState::default(),
            is_dead: self.engine.is_dead(),
            death_message: if self.engine.is_dead() {
                Some("Killed in the FFI implementation".to_string())
            } else {
                None
            },
            is_won: self.engine.is_won(),
        }
    }

    /// Execute an action on the FFI engine
    pub fn step(&mut self, action: &GameAction) -> (f64, String) {
        if let Some((cmd, dx, dy)) = action_to_command(action) {
            if dx == 0 && dy == 0 {
                let _ = self.engine.exec_cmd(cmd);
            } else {
                let _ = self.engine.exec_cmd_dir(cmd, dx, dy);
            }
        }

        let message = self.engine.last_message();
        let reward = calculate_ffi_reward(self.engine);

        (reward, message)
    }

    /// Get messages from last turn
    pub fn last_messages(&self) -> Vec<String> {
        vec![self.engine.last_message()]
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        self.engine.is_game_over()
    }
}

/// Convert action to FFI command
fn action_to_command(action: &GameAction) -> Option<(char, i32, i32)> {
    match action {
        GameAction::MoveNorth => Some(('k', 0, -1)),
        GameAction::MoveSouth => Some(('j', 0, 1)),
        GameAction::MoveEast => Some(('l', 1, 0)),
        GameAction::MoveWest => Some(('h', -1, 0)),
        GameAction::MoveNorthWest => Some(('y', -1, -1)),
        GameAction::MoveNorthEast => Some(('u', 1, -1)),
        GameAction::MoveSouthWest => Some(('b', -1, 1)),
        GameAction::MoveSouthEast => Some(('n', 1, 1)),
        GameAction::Wait => Some(('.', 0, 0)),
        GameAction::Pickup => Some((',', 0, 0)),
        GameAction::GoUp => Some(('<', 0, 0)),
        GameAction::GoDown => Some(('>', 0, 0)),
        GameAction::Inventory => Some(('i', 0, 0)),
        GameAction::Look => Some(('/', 0, 0)),
        GameAction::History => Some(('\\', 0, 0)),
        GameAction::Help => Some(('?', 0, 0)),
        GameAction::Save => Some(('S', 0, 0)),
        GameAction::Quit => Some(('Q', 0, 0)),
        _ => Some(('.', 0, 0)),
    }
}

/// Calculate reward for FFI engine
fn calculate_ffi_reward(engine: &FfiGameEngine) -> f64 {
    let mut reward = 0.0;
    reward += 0.01;
    if engine.is_dead() {
        reward -= 100.0;
    }
    reward
}

/// Extract inventory from FFI engine
fn extract_inventory(engine: &FfiGameEngine) -> Vec<UnifiedObject> {
    let json = engine.inventory_json();
    let json_value: serde_json::Value = serde_json::from_str(&json)
        .unwrap_or_else(|_| serde_json::json!([]));

    json_value.as_array()
        .map(|arr| arr.iter()
            .map(|item| UnifiedObject {
                name: item["name"].as_str().unwrap_or("Unknown").to_string(),
                class: item["class"].as_str().unwrap_or("?").to_string(),
                quantity: item["qty"].as_i64().unwrap_or(1) as i32,
                weight: 0,
                value: 0,
                enchantment: 0,
                cursed: false,
                blessed: false,
                armor_class: 0,
                damage: 0,
            })
            .collect()
        )
        .unwrap_or_default()
}

/// Extract monsters from FFI engine
fn extract_ffi_monsters(engine: &FfiGameEngine) -> Vec<UnifiedMonster> {
    let json = engine.nearby_monsters_json();
    let json_value: serde_json::Value = serde_json::from_str(&json)
        .unwrap_or_else(|_| serde_json::json!([]));

    json_value.as_array()
        .map(|arr| arr.iter()
            .map(|mon| UnifiedMonster {
                name: mon["name"].as_str().unwrap_or("Unknown").to_string(),
                symbol: mon["symbol"].as_str().unwrap_or("?").chars().next().unwrap_or('?'),
                level: mon["hp"].as_i64().unwrap_or(0) as i32,
                hp: mon["hp"].as_i64().unwrap_or(0) as i32,
                max_hp: mon["hp"].as_i64().unwrap_or(0) as i32,
                armor_class: 0,
                position: (
                    mon["x"].as_i64().unwrap_or(0) as i32,
                    mon["y"].as_i64().unwrap_or(0) as i32,
                ),
                asleep: false,
                peaceful: false,
            })
            .collect()
        )
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c_interface_ffi::FfiGameEngine;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_extract_ffi_state() {
        let mut engine = FfiGameEngine::new();
        engine.init("Tourist", "Human", 0, 0).unwrap();

        let wrapper = FfiGameWrapper::new(&mut engine);
        let state = wrapper.extract_state();

        assert!(!state.is_dead);
    }

    #[test]
    #[serial]
    fn test_ffi_state_step() {
        let mut engine = FfiGameEngine::new();
        engine.init("Wizard", "Elf", 1, 1).unwrap();

        let mut wrapper = FfiGameWrapper::new(&mut engine);

        let initial = wrapper.extract_state();
        assert_eq!(initial.position.0, 40);

        let (reward, message) = wrapper.step(&GameAction::MoveEast);
        assert!(reward >= 0.0);
        assert!(!message.is_empty());

        let state = wrapper.extract_state();
        assert_eq!(state.position.0, 41);
    }

    #[test]
    #[serial]
    fn test_ffi_inventory_extraction() {
        let mut engine = FfiGameEngine::new();
        engine.init("Rogue", "Gnome", 0, 0).unwrap();

        let wrapper = FfiGameWrapper::new(&mut engine);
        let state = wrapper.extract_state();

        assert_eq!(state.inventory.len(), 0);
    }
}
