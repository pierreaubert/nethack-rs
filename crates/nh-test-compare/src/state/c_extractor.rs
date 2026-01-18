//! Extract unified game state from the C implementation.
//!
//! This module provides functionality to convert the C game state
//! into a UnifiedGameState for comparison with the Rust implementation.

use crate::c_interface::CGameEngine;
use crate::state::common::*;

/// Wrapper for C game engine state extraction
pub struct CGameWrapper<'a> {
    engine: &'a mut CGameEngine,
}

impl<'a> CGameWrapper<'a> {
    /// Create a new C game wrapper
    pub fn new(engine: &'a mut CGameEngine) -> Self {
        Self { engine }
    }

    /// Set C engine state from Rust state
    pub fn set_state(&mut self, x: i32, y: i32, hp: i32, max_hp: i32, experience_level: i32, armor_class: i32) {
        self.engine.set_state(x, y, hp, max_hp, experience_level, armor_class);
    }

    /// Extract unified state from C implementation
    pub fn extract_state(&self) -> UnifiedGameState {
        // Parse JSON state from C engine
        let json = self.engine.state_json();
        let json_value: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|_| serde_json::json!({}));

        let player_obj = &json_value["player"];

        UnifiedGameState {
            player: UnifiedPlayer {
                name: "Player".to_string(),
                role: c_char_to_string(&json_value["role"]),
                race: c_char_to_string(&json_value["race"]),
                gender: match json_value["gender"].as_i64().unwrap_or(0) {
                    0 => "Male".to_string(),
                    _ => "Female".to_string(),
                },
                alignment: match json_value["alignment"].as_i64().unwrap_or(0) {
                    -1 => "Chaotic".to_string(),
                    0 => "Neutral".to_string(),
                    _ => "Lawful".to_string(),
                },
            },
            position: (
                player_obj["x"].as_i64().unwrap_or(40) as i32,
                player_obj["y"].as_i64().unwrap_or(10) as i32,
            ),
            hp: player_obj["hp"].as_i64().unwrap_or(10) as i32,
            max_hp: player_obj["max_hp"].as_i64().unwrap_or(10) as i32,
            energy: player_obj["energy"].as_i64().unwrap_or(10) as i32,
            max_energy: player_obj["max_energy"].as_i64().unwrap_or(10) as i32,
            armor_class: player_obj["armor_class"].as_i64().unwrap_or(10) as i32,
            gold: player_obj["gold"].as_i64().unwrap_or(0) as i32,
            experience_level: player_obj["experience_level"].as_i64().unwrap_or(1) as i32,
            strength: player_obj["strength"].as_i64().unwrap_or(10) as i32,
            dexterity: player_obj["dexterity"].as_i64().unwrap_or(10) as i32,
            constitution: player_obj["constitution"].as_i64().unwrap_or(10) as i32,
            intelligence: player_obj["intelligence"].as_i64().unwrap_or(10) as i32,
            wisdom: player_obj["wisdom"].as_i64().unwrap_or(10) as i32,
            charisma: player_obj["charisma"].as_i64().unwrap_or(10) as i32,
            current_level: json_value["current_level"].as_i64().unwrap_or(1) as i32,
            dungeon_depth: json_value["dungeon_depth"].as_i64().unwrap_or(1) as i32,
            dungeon_visited: vec![1], // Simplified - not tracked in C wrapper yet
            has_amulet: false, // Not tracked in C wrapper yet
            turn: json_value["turn"].as_u64().unwrap_or(0),
            hunger: HungerState::NotHungry,
            status_effects: Vec::new(),
            inventory: extract_inventory(self.engine),
            nearby_monsters: extract_c_monsters(self.engine),
            conduct: ConductState::default(),
            is_dead: self.engine.is_dead(),
            death_message: if self.engine.is_dead() {
                Some("Killed in the C implementation".to_string())
            } else {
                None
            },
            is_won: false, // Not tracked in C wrapper yet
        }
    }

    /// Execute an action on the C engine
    pub fn step(&mut self, action: &GameAction) -> (f64, String) {
        // Execute the command
        if let Some((cmd, dx, dy)) = action_to_command(action) {
            if dx == 0 && dy == 0 {
                let _ = self.engine.exec_cmd(cmd);
            } else {
                let _ = self.engine.exec_cmd_dir(cmd, dx, dy);
            }
        }

        let message = self.engine.last_message();

        // Calculate reward
        let reward = calculate_c_reward(self.engine);

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

/// Convert C char array to String
fn c_char_to_string(value: &serde_json::Value) -> String {
    value.as_str()
        .unwrap_or("")
        .to_string()
}

/// Extract inventory from C engine
fn extract_inventory(engine: &CGameEngine) -> Vec<UnifiedObject> {
    let json = engine.inventory_json();
    let json_value: serde_json::Value = serde_json::from_str(&json)
        .unwrap_or_else(|_| serde_json::json!([]));

    json_value.as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|item| UnifiedObject {
            name: item["name"].as_str().unwrap_or("").to_string(),
            class: item["class"].as_str().unwrap_or("?").to_string(),
            quantity: item["qty"].as_i64().unwrap_or(1) as i32,
            enchantment: 0,
            cursed: false,
            blessed: false,
            armor_class: 0,
            damage: 0,
            weight: 0,
            value: 0,
        })
        .collect()
}

/// Extract monsters from C engine
fn extract_c_monsters(engine: &CGameEngine) -> Vec<UnifiedMonster> {
    let json = engine.monsters_json();
    let json_value: serde_json::Value = serde_json::from_str(&json)
        .unwrap_or_else(|_| serde_json::json!([]));

    json_value.as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|monster| UnifiedMonster {
            name: monster["name"].as_str().unwrap_or("").to_string(),
            symbol: monster["symbol"].as_str().unwrap_or("?").chars().next().unwrap_or('?'),
            level: monster["level"].as_i64().unwrap_or(1) as i32,
            hp: monster["hp"].as_i64().unwrap_or(1) as i32,
            max_hp: monster["hp"].as_i64().unwrap_or(1) as i32,
            armor_class: monster["armor_class"].as_i64().unwrap_or(10) as i32,
            position: (
                monster["x"].as_i64().unwrap_or(0) as i32,
                monster["y"].as_i64().unwrap_or(0) as i32,
            ),
            asleep: false,
            peaceful: false,
        })
        .collect()
}

/// Convert action to C command
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
        _ => Some(('.', 0, 0)), // Default to wait for unimplemented actions
    }
}

/// Calculate reward for C engine
fn calculate_c_reward(engine: &CGameEngine) -> f64 {
    let mut reward = 0.0;

    // Small reward for each turn
    reward += 0.01;

    // Check for death
    if engine.is_dead() {
        reward -= 100.0;
    }

    reward
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c_interface::CGameEngine;

    #[test]
    fn test_extract_c_state() {
        let mut engine = CGameEngine::new();
        engine.init("Tourist", "Human", 0, 0).unwrap();

        let wrapper = CGameWrapper::new(&mut engine);
        let state = wrapper.extract_state();

        // Check basic fields
        assert_eq!(state.player.role, "Tourist");
        assert_eq!(state.player.race, "Human");
        assert!(!state.is_dead);
    }

    #[test]
    fn test_c_state_step() {
        let mut engine = CGameEngine::new();
        engine.init("Wizard", "Elf", 1, 1).unwrap();

        let mut wrapper = CGameWrapper::new(&mut engine);

        // Check initial position
        let initial = wrapper.extract_state();
        assert_eq!(initial.position.0, 40, "Initial x should be 40");

        // Execute a move
        let (reward, message) = wrapper.step(&GameAction::MoveEast);

        // Check result
        assert!(reward >= 0.0, "Reward should be non-negative, got {}", reward);
        assert!(!message.is_empty(), "Message should not be empty");

        // Check position changed
        let state = wrapper.extract_state();
        assert_eq!(state.position.0, 41, "x should be 41 after moving east from 40, got {}", state.position.0);
    }

    #[test]
    fn test_c_inventory_extraction() {
        let mut engine = CGameEngine::new();
        engine.init("Rogue", "Gnome", 0, 0).unwrap();

        let wrapper = CGameWrapper::new(&mut engine);
        let state = wrapper.extract_state();

        // Should have empty inventory initially
        assert_eq!(state.inventory.len(), 0);
    }
}
