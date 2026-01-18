//! Rust FFI bindings for the C NetHack implementation
//!
//! These bindings allow Rust code to call into the C NetHack implementation
//! for comparison testing.

use libc::{c_char, c_int, c_void, c_ulong};
use std::ffi::{CStr, CString};

/// C game state structure size
pub const NH_GAME_STATE_SIZE: usize = std::mem::size_of::<CGameState>();

/// Max inventory items
pub const NH_MAX_INVENTORY: usize = 52;

/// Max monsters on level
pub const NH_MAX_MONSTERS: usize = 100;

/// Max dungeon levels
pub const NH_MAX_DUNGEON_LEVELS: usize = 30;

// ============================================================================
// C Structures (must match C definitions)
// ============================================================================

#[repr(C)]
pub struct CObject {
    pub name: [c_char; 64],
    pub class_: c_char,
    pub weight: c_int,
    pub value: c_int,
    pub quantity: c_int,
    pub enchantment: c_int,
    pub cursed: c_int,
    pub blessed: c_int,
    pub armor_class: c_int,
    pub damage: c_int,
    pub inv_letter: c_char,
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
pub struct CMonster {
    pub name: [c_char; 64],
    pub symbol: c_char,
    pub level: c_int,
    pub hp: c_int,
    pub max_hp: c_int,
    pub armor_class: c_int,
    pub x: c_int,
    pub y: c_int,
    pub asleep: c_int,
    pub peaceful: c_int,
}

#[repr(C)]
pub struct CPlayer {
    pub role: [c_char; 32],
    pub race: [c_char; 32],
    pub gender: c_int,
    pub alignment: c_int,
    pub hp: c_int,
    pub max_hp: c_int,
    pub energy: c_int,
    pub max_energy: c_int,
    pub x: c_int,
    pub y: c_int,
    pub level: c_int,
    pub experience_level: c_int,
    pub armor_class: c_int,
    pub gold: c_int,
    pub strength: c_int,
    pub dexterity: c_int,
    pub constitution: c_int,
    pub intelligence: c_int,
    pub wisdom: c_int,
    pub charisma: c_int,
    pub is_dead: c_int,
    pub hunger_state: c_int,
    pub confusion_timeout: c_int,
    pub stun_timeout: c_int,
    pub blindness_timeout: c_int,
}

#[repr(C)]
pub struct CGameState {
    pub player: CPlayer,
    pub inventory: [*mut CObject; NH_MAX_INVENTORY],
    pub inventory_count: c_int,
    pub monsters: [CMonster; NH_MAX_MONSTERS],
    pub monster_count: c_int,
    pub current_level: c_int,
    pub dungeon_depth: c_int,
    pub dungeon_visited: [c_int; NH_MAX_DUNGEON_LEVELS],
    pub turn_count: c_ulong,
    pub hunger_state: c_int,
    pub last_message: [c_char; 256],
}

// ============================================================================
// FFI Function Declarations
// ============================================================================

unsafe extern "C" {
    // Game Initialization
    pub fn nh_ffi_init(role: *const c_char, race: *const c_char, gender: c_int, alignment: c_int) -> c_int;
    pub fn nh_ffi_reset(seed: c_ulong) -> c_int;
    pub fn nh_ffi_free();

    // Command Execution
    pub fn nh_ffi_exec_cmd(cmd: c_char) -> c_int;
    pub fn nh_ffi_exec_cmd_dir(cmd: c_char, dx: c_int, dy: c_int) -> c_int;

    // State Access
    pub fn nh_ffi_get_hp() -> c_int;
    pub fn nh_ffi_get_max_hp() -> c_int;
    pub fn nh_ffi_get_energy() -> c_int;
    pub fn nh_ffi_get_max_energy() -> c_int;
    pub fn nh_ffi_get_position(x: *mut c_int, y: *mut c_int);
    pub fn nh_ffi_set_state(x: c_int, y: c_int, hp: c_int, max_hp: c_int, level: c_int, ac: c_int);
    pub fn nh_ffi_get_armor_class() -> c_int;
    pub fn nh_ffi_get_gold() -> c_int;
    pub fn nh_ffi_get_experience_level() -> c_int;
    pub fn nh_ffi_get_current_level() -> c_int;
    pub fn nh_ffi_get_dungeon_depth() -> c_int;
    pub fn nh_ffi_get_turn_count() -> c_ulong;
    pub fn nh_ffi_is_player_dead() -> c_int;

    // State Serialization
    pub fn nh_ffi_get_state_json() -> *mut c_char;

    // Message Log
    pub fn nh_ffi_get_last_message() -> *mut c_char;

    // Inventory
    pub fn nh_ffi_get_inventory_count() -> c_int;
    pub fn nh_ffi_get_inventory_json() -> *mut c_char;

    // Monsters
    pub fn nh_ffi_get_nearby_monsters_json() -> *mut c_char;
    pub fn nh_ffi_count_monsters() -> c_int;

    // Game Status
    pub fn nh_ffi_is_game_over() -> c_int;
    pub fn nh_ffi_is_game_won() -> c_int;
    pub fn nh_ffi_get_result_message() -> *mut c_char;

    // Memory Management
    pub fn nh_ffi_free_string(ptr: *mut c_void);
}

// ============================================================================
// Safe Rust Wrapper
// ============================================================================

/// Safe wrapper for the C NetHack game
pub struct CGameEngine {
    initialized: bool,
}

impl CGameEngine {
    /// Create a new C game engine
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the game with character parameters
    pub fn init(&mut self, role: &str, race: &str, gender: i32, alignment: i32) -> Result<(), String> {
        let role_c = CString::new(role).map_err(|e| format!("Invalid role: {}", e))?;
        let race_c = CString::new(race).map_err(|e| format!("Invalid race: {}", e))?;

        let result = unsafe {
            nh_ffi_init(role_c.as_ptr(), race_c.as_ptr(), gender as c_int, alignment as c_int)
        };

        if result < 0 {
            return Err("Failed to initialize C NetHack".to_string());
        }

        self.initialized = true;
        Ok(())
    }

    /// Reset the game to initial state
    pub fn reset(&mut self, seed: u64) -> Result<(), String> {
        if !self.initialized {
            return Err("Game not initialized".to_string());
        }

        let result = unsafe { nh_ffi_reset(seed as c_ulong) };
        if result < 0 {
            return Err("Failed to reset game".to_string());
        }

        Ok(())
    }

    /// Execute a command
    pub fn exec_cmd(&self, cmd: char) -> Result<(), String> {
        if !self.initialized {
            return Err("Game not initialized".to_string());
        }

        let result = unsafe { nh_ffi_exec_cmd(cmd as c_char) };
        if result == -2 {
            return Err(format!("Unknown command: {}", cmd));
        }
        if result < 0 {
            return Err("Command failed".to_string());
        }

        Ok(())
    }

    /// Execute a directional command
    pub fn exec_cmd_dir(&self, cmd: char, dx: i32, dy: i32) -> Result<(), String> {
        if !self.initialized {
            return Err("Game not initialized".to_string());
        }

        let result = unsafe { nh_ffi_exec_cmd_dir(cmd as c_char, dx as c_int, dy as c_int) };
        if result < 0 {
            return Err("Directional command failed".to_string());
        }

        Ok(())
    }

    /// Get player HP
    pub fn hp(&self) -> i32 {
        unsafe { nh_ffi_get_hp() as i32 }
    }

    /// Get player max HP
    pub fn max_hp(&self) -> i32 {
        unsafe { nh_ffi_get_max_hp() as i32 }
    }

    /// Get player energy
    pub fn energy(&self) -> i32 {
        unsafe { nh_ffi_get_energy() as i32 }
    }

    /// Get player position
    pub fn position(&self) -> (i32, i32) {
        let mut x: c_int = 0;
        let mut y: c_int = 0;
        unsafe { nh_ffi_get_position(&mut x, &mut y) };
        (x as i32, y as i32)
    }

    /// Set exact game state (synchronization)
    pub fn set_state(&self, x: i32, y: i32, hp: i32, max_hp: i32, level: i32, ac: i32) {
        unsafe { nh_ffi_set_state(x as c_int, y as c_int, hp as c_int, max_hp as c_int, level as c_int, ac as c_int) };
    }

    /// Get armor class
    pub fn armor_class(&self) -> i32 {
        unsafe { nh_ffi_get_armor_class() as i32 }
    }

    /// Get gold
    pub fn gold(&self) -> i32 {
        unsafe { nh_ffi_get_gold() as i32 }
    }

    /// Get experience level
    pub fn experience_level(&self) -> i32 {
        unsafe { nh_ffi_get_experience_level() as i32 }
    }

    /// Get current level
    pub fn current_level(&self) -> i32 {
        unsafe { nh_ffi_get_current_level() as i32 }
    }

    /// Get dungeon depth
    pub fn dungeon_depth(&self) -> i32 {
        unsafe { nh_ffi_get_dungeon_depth() as i32 }
    }

    /// Get turn count
    pub fn turn_count(&self) -> u64 {
        unsafe { nh_ffi_get_turn_count() as u64 }
    }

    /// Check if player is dead
    pub fn is_dead(&self) -> bool {
        unsafe { nh_ffi_is_player_dead() != 0 }
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        unsafe { nh_ffi_is_game_over() != 0 }
    }

    /// Get state as JSON string
    pub fn state_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_state_json() };
        if json_ptr.is_null() {
            return "{}".to_string();
        }
        let result = unsafe {
            CStr::from_ptr(json_ptr)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    /// Get last message
    pub fn last_message(&self) -> String {
        let msg_ptr = unsafe { nh_ffi_get_last_message() };
        if msg_ptr.is_null() {
            return "No message".to_string();
        }
        let result = unsafe {
            CStr::from_ptr(msg_ptr)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { nh_ffi_free_string(msg_ptr as *mut c_void) };
        result
    }

    /// Get inventory count
    pub fn inventory_count(&self) -> i32 {
        unsafe { nh_ffi_get_inventory_count() as i32 }
    }

    /// Get inventory as JSON
    pub fn inventory_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_inventory_json() };
        if json_ptr.is_null() {
            return "[]".to_string();
        }
        let result = unsafe {
            CStr::from_ptr(json_ptr)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    /// Get monsters as JSON
    pub fn monsters_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_nearby_monsters_json() };
        if json_ptr.is_null() {
            return "[]".to_string();
        }
        let result = unsafe {
            CStr::from_ptr(json_ptr)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    /// Get result message
    pub fn result_message(&self) -> String {
        let msg_ptr = unsafe { nh_ffi_get_result_message() };
        if msg_ptr.is_null() {
            return "Unknown".to_string();
        }
        let result = unsafe {
            CStr::from_ptr(msg_ptr)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { nh_ffi_free_string(msg_ptr as *mut c_void) };
        result
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Drop for CGameEngine {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { nh_ffi_free() };
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_game_engine_lifecycle() {
        let mut engine = CGameEngine::new();
        assert!(!engine.is_initialized());

        // Initialize with default character
        assert!(engine.init("Tourist", "Human", 0, 0).is_ok());
        assert!(engine.is_initialized());

        // Check initial state - HP should be positive for a new game
        let hp = engine.hp();
        assert!(hp > 0, "HP should be positive, got {}", hp);
        assert!(engine.max_hp() > 0);
        assert!(!engine.is_dead());
        assert!(!engine.is_game_over());
        assert_eq!(engine.position(), (40, 10));

        // Execute some commands using directional to avoid interference
        let _ = engine.exec_cmd_dir('h', -1, 0);
        assert_eq!(engine.position(), (39, 10));

        let _ = engine.exec_cmd_dir('j', 0, 1);
        assert_eq!(engine.position(), (39, 11));

        // Check turn count
        let turns = engine.turn_count();
        assert!(turns >= 2, "Turn count should be at least 2, got {}", turns);
    }

    #[test]
    fn test_state_json() {
        let mut engine = CGameEngine::new();
        engine.init("Wizard", "Elf", 1, 1).unwrap();

        let json = engine.state_json();
        assert!(json.contains("hp"));
        assert!(json.contains("x"));
        assert!(json.contains("y"));
    }

    #[test]
    fn test_reset() {
        let mut engine = CGameEngine::new();
        engine.init("Rogue", "Gnome", 0, 0).unwrap();

        // Move somewhere - use directional command to avoid test interference
        let _ = engine.exec_cmd_dir('l', 1, 0);
        let _ = engine.exec_cmd_dir('l', 1, 0);

        // Reset
        engine.reset(12345).unwrap();

        // Should be back at start
        assert_eq!(engine.position(), (40, 10));
        assert_eq!(engine.turn_count(), 0);
    }

    #[test]
    fn test_unknown_command() {
        let mut engine = CGameEngine::new();
        engine.init("Priest", "Dwarf", 0, 0).unwrap();

        // Unknown command should return error (using '@' which is not a valid command)
        assert!(engine.exec_cmd('@').is_err());
    }
}
