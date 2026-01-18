//! Rust FFI bindings for the NetHack FFI implementation
//!
//! These bindings provide access to the NetHack FFI interface for
//! comparison testing with the Rust nethack-rs implementation.

use libc::{c_char, c_int, c_void, c_ulong};
use std::ffi::{CStr, CString};

// ============================================================================
// FFI Function Declarations
// ============================================================================

unsafe extern "C" {
    // Initialization and Cleanup
    pub fn nh_ffi_init(role: *const c_char, race: *const c_char, gender: c_int, alignment: c_int) -> c_int;
    pub fn nh_ffi_free();
    pub fn nh_ffi_reset(seed: c_ulong) -> c_int;

    // State Queries
    pub fn nh_ffi_get_hp() -> c_int;
    pub fn nh_ffi_get_max_hp() -> c_int;
    pub fn nh_ffi_get_energy() -> c_int;
    pub fn nh_ffi_get_max_energy() -> c_int;
    pub fn nh_ffi_get_position(x: *mut c_int, y: *mut c_int);
    pub fn nh_ffi_get_armor_class() -> c_int;
    pub fn nh_ffi_get_gold() -> c_int;
    pub fn nh_ffi_get_experience_level() -> c_int;
    pub fn nh_ffi_get_current_level() -> c_int;
    pub fn nh_ffi_get_dungeon_depth() -> c_int;
    pub fn nh_ffi_get_turn_count() -> c_ulong;
    pub fn nh_ffi_is_player_dead() -> c_int;

    pub fn nh_ffi_get_role() -> *const c_char;
    pub fn nh_ffi_get_race() -> *const c_char;
    pub fn nh_ffi_get_gender() -> c_int;
    pub fn nh_ffi_get_alignment() -> c_int;

    // Command Execution
    pub fn nh_ffi_exec_cmd(cmd: c_char) -> c_int;
    pub fn nh_ffi_exec_cmd_dir(cmd: c_char, dx: c_int, dy: c_int) -> c_int;

    // State Serialization
    pub fn nh_ffi_get_state_json() -> *mut c_char;
    pub fn nh_ffi_free_string(ptr: *mut c_void);

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
}

// ============================================================================
// Safe Rust Wrapper
// ============================================================================

/// Safe wrapper for the NetHack FFI game engine
pub struct FfiGameEngine {
    initialized: bool,
}

impl FfiGameEngine {
    /// Create a new FFI game engine
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
            return Err("Failed to initialize NetHack FFI".to_string());
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

    /// Get player max energy
    pub fn max_energy(&self) -> i32 {
        unsafe { nh_ffi_get_max_energy() as i32 }
    }

    /// Get player position
    pub fn position(&self) -> (i32, i32) {
        let mut x: c_int = 0;
        let mut y: c_int = 0;
        unsafe { nh_ffi_get_position(&mut x, &mut y) };
        (x as i32, y as i32)
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

    /// Check if game is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        unsafe { nh_ffi_is_game_over() != 0 }
    }

    /// Check if game is won
    pub fn is_won(&self) -> bool {
        unsafe { nh_ffi_is_game_won() != 0 }
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

    /// Get nearby monsters as JSON
    pub fn nearby_monsters_json(&self) -> String {
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

    /// Get monster count
    pub fn monster_count(&self) -> i32 {
        unsafe { nh_ffi_count_monsters() as i32 }
    }

    /// Get role as string
    pub fn role(&self) -> String {
        let ptr = unsafe { nh_ffi_get_role() };
        if ptr.is_null() {
            return "Unknown".to_string();
        }
        unsafe {
            CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Get race as string
    pub fn race(&self) -> String {
        let ptr = unsafe { nh_ffi_get_race() };
        if ptr.is_null() {
            return "Unknown".to_string();
        }
        unsafe {
            CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Get gender as formatted string
    pub fn gender_string(&self) -> String {
        match unsafe { nh_ffi_get_gender() } {
            0 => "Male".to_string(),
            _ => "Female".to_string(),
        }
    }

    /// Get alignment as formatted string
    pub fn alignment_string(&self) -> String {
        match unsafe { nh_ffi_get_alignment() } {
            -1 => "Chaotic".to_string(),
            0 => "Neutral".to_string(),
            _ => "Lawful".to_string(),
        }
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
}

impl Drop for FfiGameEngine {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { nh_ffi_free() };
            self.initialized = false;
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
    fn test_ffi_game_engine_lifecycle() {
        let mut engine = FfiGameEngine::new();
        assert!(!engine.is_initialized());

        assert!(engine.init("Tourist", "Human", 0, 0).is_ok());
        assert!(engine.is_initialized());

        assert!(engine.hp() > 0);
        assert!(engine.max_hp() > 0);
        assert!(!engine.is_dead());
        assert!(!engine.is_game_over());

        // Execute some commands
        assert!(engine.exec_cmd_dir('h', -1, 0).is_ok());
        assert!(engine.exec_cmd_dir('j', 0, 1).is_ok());

        // Check turn count
        assert!(engine.turn_count() >= 2);
    }

    #[test]
    fn test_state_json() {
        let mut engine = FfiGameEngine::new();
        engine.init("Wizard", "Elf", 1, 1).unwrap();

        let json = engine.state_json();
        assert!(json.contains("turn"));
        assert!(json.contains("hp"));
        assert!(json.contains("x"));
        assert!(json.contains("y"));
    }

    #[test]
    fn test_unknown_command() {
        let mut engine = FfiGameEngine::new();
        engine.init("Priest", "Dwarf", 0, 0).unwrap();

        assert!(engine.exec_cmd('@').is_err());
    }
}
