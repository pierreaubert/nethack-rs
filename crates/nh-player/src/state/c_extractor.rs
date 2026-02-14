//! Extract unified game state from the C implementation.
//!
//! This module provides functionality to convert the C game state
//! into a UnifiedGameState for comparison with the Rust implementation.

use crate::ffi::CGameEngine;
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
    pub fn set_state(
        &mut self,
        x: i32,
        y: i32,
        hp: i32,
        max_hp: i32,
        experience_level: i32,
        armor_class: i32,
    ) {
        self.engine
            .set_state(x, y, hp, max_hp, experience_level, armor_class);
    }

    /// Extract unified state from C implementation
    pub fn extract_state(&self) -> UnifiedGameState {
        // Parse JSON for fields not available via direct methods
        let json = self.engine.state_json();
        let json_value: serde_json::Value =
            serde_json::from_str(&json).unwrap_or_else(|_| serde_json::json!({}));
        let player_obj = &json_value["player"];

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
            strength: player_obj["strength"].as_i64().unwrap_or(10) as i32,
            dexterity: player_obj["dexterity"].as_i64().unwrap_or(10) as i32,
            constitution: player_obj["constitution"].as_i64().unwrap_or(10) as i32,
            intelligence: player_obj["intelligence"].as_i64().unwrap_or(10) as i32,
            wisdom: player_obj["wisdom"].as_i64().unwrap_or(10) as i32,
            charisma: player_obj["charisma"].as_i64().unwrap_or(10) as i32,
            current_level: self.engine.current_level(),
            dungeon_depth: self.engine.dungeon_depth(),
            dungeon_visited: vec![1],
            has_amulet: false,
            turn: self.engine.turn_count(),
            hunger: HungerState::NotHungry,
            status_effects: Vec::new(),
            inventory: extract_inventory(self.engine),
            nearby_monsters: extract_monsters(self.engine),
            conduct: ConductState::default(),
            is_dead: self.engine.is_dead(),
            death_message: if self.engine.is_dead() {
                Some("Killed in the C implementation".to_string())
            } else {
                None
            },
            is_won: self.engine.is_won(),
        }
    }

    /// Execute an action on the C engine
    pub fn step(&mut self, action: &GameAction) -> (f64, String) {
        if let Some((cmd, dx, dy)) = action_to_command(action) {
            if dx == 0 && dy == 0 {
                let _ = self.engine.exec_cmd(cmd);
            } else {
                let _ = self.engine.exec_cmd_dir(cmd, dx, dy);
            }
        }

        let message = self.engine.last_message();
        let reward = calculate_reward(self.engine);

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

/// Extract inventory from C engine
fn extract_inventory(engine: &CGameEngine) -> Vec<UnifiedObject> {
    let json = engine.inventory_json();
    let json_value: serde_json::Value =
        serde_json::from_str(&json).unwrap_or_else(|_| serde_json::json!([]));

    json_value
        .as_array()
        .map(|arr| {
            arr.iter()
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
        })
        .unwrap_or_default()
}

/// Extract monsters from C engine
fn extract_monsters(engine: &CGameEngine) -> Vec<UnifiedMonster> {
    let json = engine.monsters_json();
    let json_value: serde_json::Value =
        serde_json::from_str(&json).unwrap_or_else(|_| serde_json::json!([]));

    json_value
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|monster| UnifiedMonster {
                    name: monster["name"].as_str().unwrap_or("").to_string(),
                    symbol: monster["symbol"]
                        .as_str()
                        .unwrap_or("?")
                        .chars()
                        .next()
                        .unwrap_or('?'),
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
        })
        .unwrap_or_default()
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
        _ => Some(('.', 0, 0)),
    }
}

/// Calculate reward for C engine
fn calculate_reward(engine: &CGameEngine) -> f64 {
    let mut reward = 0.01;
    if engine.is_dead() {
        reward -= 100.0;
    }
    reward
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::CGameEngine;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_extract_c_state() {
        let mut engine = CGameEngine::new();
        engine.init("Tourist", "Human", 0, 0).unwrap();

        let wrapper = CGameWrapper::new(&mut engine);
        let state = wrapper.extract_state();

        assert_eq!(state.player.role, "Tourist");
        assert_eq!(state.player.race, "Human");
        assert!(!state.is_dead);
    }

    #[test]
    #[serial]
    fn test_c_state_step() {
        let mut engine = CGameEngine::new();
        engine.init("Wizard", "Elf", 1, 1).unwrap();

        let mut wrapper = CGameWrapper::new(&mut engine);

        let initial = wrapper.extract_state();
        assert_eq!(initial.position.0, 40, "Initial x should be 40");

        let (reward, message) = wrapper.step(&GameAction::MoveEast);

        assert!(
            reward >= 0.0,
            "Reward should be non-negative, got {}",
            reward
        );
        assert!(!message.is_empty(), "Message should not be empty");

        let state = wrapper.extract_state();
        assert_eq!(
            state.position.0, 41,
            "x should be 41 after moving east from 40, got {}",
            state.position.0
        );
    }

    #[test]
    #[serial]
    fn test_c_inventory_extraction() {
        let mut engine = CGameEngine::new();
        engine.init("Rogue", "Gnome", 0, 0).unwrap();

        let wrapper = CGameWrapper::new(&mut engine);
        let state = wrapper.extract_state();

        assert_eq!(state.inventory.len(), 0);
    }
}
