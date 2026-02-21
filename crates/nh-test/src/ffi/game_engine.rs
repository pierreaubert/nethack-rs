//! Rust FFI bindings for the C NetHack implementation.
//!
//! Provides extern fn declarations and a safe `CGameEngine` wrapper
//! for comparison testing with the Rust nethack-rs implementation.

use libc::{c_char, c_int, c_long, c_ulong, c_void};
use std::ffi::{CStr, CString};

// ============================================================================
// C Structures (must match C definitions)
// ============================================================================

/// Max inventory items
pub const NH_MAX_INVENTORY: usize = 52;

/// Max monsters on level
pub const NH_MAX_MONSTERS: usize = 100;

/// Max dungeon levels
pub const NH_MAX_DUNGEON_LEVELS: usize = 30;

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
    pub recharged: c_int,
    pub poisoned: c_int,
    pub otyp: c_int,
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
    pub strategy: c_ulong,
}

// ============================================================================
// FFI Function Declarations
// ============================================================================

unsafe extern "C" {
    // Initialization and Cleanup
    pub fn nh_ffi_init(
        role: *const c_char,
        race: *const c_char,
        gender: c_int,
        alignment: c_int,
    ) -> c_int;
    pub fn nh_ffi_free();
    pub fn nh_ffi_reset(seed: c_ulong) -> c_int;
    pub fn nh_ffi_reset_rng(seed: c_ulong);
    pub fn nh_ffi_generate_level() -> c_int;
    pub fn nh_ffi_generate_maze();

    // Command Execution
    pub fn nh_ffi_exec_cmd(cmd: c_char) -> c_int;
    pub fn nh_ffi_exec_cmd_dir(cmd: c_char, dx: c_int, dy: c_int) -> c_int;

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
    pub fn nh_ffi_set_dlevel(dnum: c_int, dlevel: c_int);

    // Character info
    pub fn nh_ffi_get_role() -> *const c_char;
    pub fn nh_ffi_get_race() -> *const c_char;
    pub fn nh_ffi_get_gender() -> c_int;
    pub fn nh_ffi_get_alignment() -> c_int;

    // State Manipulation
    pub fn nh_ffi_set_state(hp: c_int, hpmax: c_int, x: c_int, y: c_int, ac: c_int, moves: c_long);

    // State Serialization
    pub fn nh_ffi_get_state_json() -> *mut c_char;
    pub fn nh_ffi_get_map_json() -> *mut c_char;
    pub fn nh_ffi_free_string(ptr: *mut c_void);

    // Message Log
    pub fn nh_ffi_get_last_message() -> *mut c_char;

    // Objects
    pub fn nh_ffi_get_object_table_json() -> *mut c_char;

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

    // Logic/Calculation Wrappers
    pub fn nh_ffi_rng_rn2(limit: c_int) -> c_int;
    pub fn nh_ffi_calc_base_damage(weapon_id: c_int, small_monster: c_int) -> c_int;
    pub fn nh_ffi_get_ac() -> c_int;
    pub fn nh_ffi_test_setup_status(hp: c_int, max_hp: c_int, level: c_int, ac: c_int);
    pub fn nh_ffi_wear_item(item_id: c_int) -> c_int;
    pub fn nh_ffi_add_item_to_inv(item_id: c_int, weight: c_int) -> c_int;
    pub fn nh_ffi_get_weight() -> c_int;
    pub fn nh_ffi_set_wizard_mode(enable: c_int);

    // Extended state queries (convergence framework)
    pub fn nh_ffi_get_nutrition() -> c_int;
    pub fn nh_ffi_get_attributes_json() -> *mut c_char;
    pub fn nh_ffi_export_level() -> *mut c_char;

    // RNG tracing (convergence framework)
    pub fn nh_ffi_enable_rng_tracing();
    pub fn nh_ffi_disable_rng_tracing();
    pub fn nh_ffi_get_rng_trace() -> *mut c_char;
    pub fn nh_ffi_clear_rng_trace();

    // Function-level isolation testing (Phase 1)
    pub fn nh_ffi_test_finddpos(xl: c_int, yl: c_int, xh: c_int, yh: c_int, out_x: *mut c_int, out_y: *mut c_int);
    pub fn nh_ffi_test_dig_corridor(sx: c_int, sy: c_int, dx: c_int, dy: c_int, nxcor: c_int) -> c_int;
    pub fn nh_ffi_test_makecorridors();
    pub fn nh_ffi_test_join(a: c_int, b: c_int, nxcor: c_int);
    pub fn nh_ffi_get_smeq() -> *mut c_char;
    pub fn nh_ffi_get_doorindex() -> c_int;
    pub fn nh_ffi_get_cell_region(x1: c_int, y1: c_int, x2: c_int, y2: c_int) -> *mut c_char;
    pub fn nh_ffi_set_cell(x: c_int, y: c_int, typ: c_int);
    pub fn nh_ffi_clear_level();
    pub fn nh_ffi_add_room(lx: c_int, ly: c_int, hx: c_int, hy: c_int, rtype: c_int) -> c_int;
    pub fn nh_ffi_carve_room(lx: c_int, ly: c_int, hx: c_int, hy: c_int);
    pub fn nh_ffi_get_rect_json() -> *mut c_char;
}

// ============================================================================
// Safe Rust Wrapper
// ============================================================================

pub struct CGameEngine {
    initialized: bool,
}

impl CGameEngine {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn init(
        &mut self,
        role: &str,
        race: &str,
        gender: i32,
        alignment: i32,
    ) -> Result<(), String> {
        let role_c = CString::new(role).map_err(|e| format!("Invalid role: {}", e))?;
        let race_c = CString::new(race).map_err(|e| format!("Invalid race: {}", e))?;

        let result = unsafe {
            nh_ffi_init(
                role_c.as_ptr(),
                race_c.as_ptr(),
                gender as c_int,
                alignment as c_int,
            )
        };

        if result < 0 {
            return Err("Failed to initialize C NetHack".to_string());
        }

        self.initialized = true;
        Ok(())
    }

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

    pub fn reset_rng(&self, seed: u64) -> Result<(), String> {
        if !self.initialized {
            return Err("Game not initialized".to_string());
        }

        unsafe { nh_ffi_reset_rng(seed as c_ulong) };
        Ok(())
    }

    pub fn generate_level(&self) -> Result<(), String> {
        if !self.initialized {
            return Err("Game not initialized".to_string());
        }

        let result = unsafe { nh_ffi_generate_level() };
        if result < 0 {
            return Err("Failed to generate level".to_string());
        }

        Ok(())
    }

    pub fn generate_maze(&self) -> Result<(), String> {
        if !self.initialized {
            return Err("Game not initialized".to_string());
        }

        unsafe { nh_ffi_generate_maze() };
        Ok(())
    }

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

    pub fn hp(&self) -> i32 {
        unsafe { nh_ffi_get_hp() as i32 }
    }

    pub fn max_hp(&self) -> i32 {
        unsafe { nh_ffi_get_max_hp() as i32 }
    }

    pub fn energy(&self) -> i32 {
        unsafe { nh_ffi_get_energy() as i32 }
    }

    pub fn max_energy(&self) -> i32 {
        unsafe { nh_ffi_get_max_energy() as i32 }
    }

    pub fn position(&self) -> (i32, i32) {
        let mut x: c_int = 0;
        let mut y: c_int = 0;
        unsafe { nh_ffi_get_position(&mut x, &mut y) };
        (x as i32, y as i32)
    }

    pub fn set_state(&self, hp: i32, hpmax: i32, x: i32, y: i32, ac: i32, moves: i64) {
        unsafe {
            nh_ffi_set_state(
                hp as c_int,
                hpmax as c_int,
                x as c_int,
                y as c_int,
                ac as c_int,
                moves as c_long,
            )
        };
    }

    pub fn armor_class(&self) -> i32 {
        unsafe { nh_ffi_get_armor_class() as i32 }
    }

    pub fn gold(&self) -> i32 {
        unsafe { nh_ffi_get_gold() as i32 }
    }

    pub fn experience_level(&self) -> i32 {
        unsafe { nh_ffi_get_experience_level() as i32 }
    }

    pub fn current_level(&self) -> i32 {
        unsafe { nh_ffi_get_current_level() as i32 }
    }

    pub fn dungeon_depth(&self) -> i32 {
        unsafe { nh_ffi_get_dungeon_depth() as i32 }
    }

    pub fn turn_count(&self) -> u64 {
        unsafe { nh_ffi_get_turn_count() as u64 }
    }

    pub fn is_dead(&self) -> bool {
        unsafe { nh_ffi_is_player_dead() != 0 }
    }

    pub fn set_dlevel(&self, dnum: i32, dlevel: i32) {
        unsafe { nh_ffi_set_dlevel(dnum, dlevel) }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn is_game_over(&self) -> bool {
        unsafe { nh_ffi_is_game_over() != 0 }
    }

    pub fn is_won(&self) -> bool {
        unsafe { nh_ffi_is_game_won() != 0 }
    }

    pub fn state_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_state_json() };
        if json_ptr.is_null() {
            return "{}".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn map_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_map_json() };
        if json_ptr.is_null() {
            return "{}".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn last_message(&self) -> String {
        let msg_ptr = unsafe { nh_ffi_get_last_message() };
        if msg_ptr.is_null() {
            return "No message".to_string();
        }
        let result = unsafe { CStr::from_ptr(msg_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(msg_ptr as *mut c_void) };
        result
    }

    pub fn inventory_count(&self) -> i32 {
        unsafe { nh_ffi_get_inventory_count() as i32 }
    }

    pub fn inventory_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_inventory_json() };
        if json_ptr.is_null() {
            return "[]".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn object_table_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_object_table_json() };
        if json_ptr.is_null() {
            return "[]".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn monsters_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_nearby_monsters_json() };
        if json_ptr.is_null() {
            return "[]".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn monster_count(&self) -> i32 {
        unsafe { nh_ffi_count_monsters() as i32 }
    }

    pub fn role(&self) -> String {
        let ptr = unsafe { nh_ffi_get_role() };
        if ptr.is_null() {
            return "Unknown".to_string();
        }
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }

    pub fn race(&self) -> String {
        let ptr = unsafe { nh_ffi_get_race() };
        if ptr.is_null() {
            return "Unknown".to_string();
        }
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }

    pub fn gender_string(&self) -> String {
        match unsafe { nh_ffi_get_gender() } {
            0 => "Male".to_string(),
            _ => "Female".to_string(),
        }
    }

    pub fn alignment_string(&self) -> String {
        match unsafe { nh_ffi_get_alignment() } {
            -1 => "Chaotic".to_string(),
            0 => "Neutral".to_string(),
            _ => "Lawful".to_string(),
        }
    }

    pub fn result_message(&self) -> String {
        let msg_ptr = unsafe { nh_ffi_get_result_message() };
        if msg_ptr.is_null() {
            return "Unknown".to_string();
        }
        let result = unsafe { CStr::from_ptr(msg_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(msg_ptr as *mut c_void) };
        result
    }

    pub fn rng_rn2(&self, limit: i32) -> i32 {
        unsafe { nh_ffi_rng_rn2(limit as c_int) as i32 }
    }

    pub fn calc_base_damage(&self, weapon_id: i32, small_monster: bool) -> i32 {
        unsafe { nh_ffi_calc_base_damage(weapon_id as c_int, small_monster as c_int) as i32 }
    }

    pub fn ac(&self) -> i32 {
        unsafe { nh_ffi_get_ac() as i32 }
    }

    pub fn test_setup_status(&self, hp: i32, max_hp: i32, level: i32, ac: i32) {
        unsafe {
            nh_ffi_test_setup_status(hp as c_int, max_hp as c_int, level as c_int, ac as c_int)
        };
    }

    pub fn wear_item(&self, item_id: i32) -> Result<(), String> {
        let res = unsafe { nh_ffi_wear_item(item_id as c_int) };
        if res < 0 {
            Err("Failed to wear item".to_string())
        } else {
            Ok(())
        }
    }

    pub fn add_item_to_inv(&self, item_id: i32, weight: i32) -> Result<(), String> {
        let res = unsafe { nh_ffi_add_item_to_inv(item_id as c_int, weight as c_int) };
        if res < 0 {
            Err("Failed to add item".to_string())
        } else {
            Ok(())
        }
    }

    pub fn carrying_weight(&self) -> i32 {
        unsafe { nh_ffi_get_weight() as i32 }
    }

    pub fn set_wizard_mode(&self, enable: bool) {
        unsafe { nh_ffi_set_wizard_mode(if enable { 1 } else { 0 }) };
    }

    pub fn nutrition(&self) -> i32 {
        unsafe { nh_ffi_get_nutrition() as i32 }
    }

    pub fn attributes_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_attributes_json() };
        if json_ptr.is_null() {
            return "{}".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn export_level(&self) -> String {
        let json_ptr = unsafe { nh_ffi_export_level() };
        if json_ptr.is_null() {
            return "{}".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn enable_rng_tracing(&self) {
        unsafe { nh_ffi_enable_rng_tracing() };
    }

    pub fn disable_rng_tracing(&self) {
        unsafe { nh_ffi_disable_rng_tracing() };
    }

    pub fn rng_trace_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_rng_trace() };
        if json_ptr.is_null() {
            return "[]".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    pub fn clear_rng_trace(&self) {
        unsafe { nh_ffi_clear_rng_trace() };
    }

    // ========================================================================
    // Function-level isolation testing (Phase 1)
    // ========================================================================

    /// Test C's finddpos() in isolation. Returns (x, y) chosen position.
    pub fn test_finddpos(&self, xl: i32, yl: i32, xh: i32, yh: i32) -> (i32, i32) {
        let mut out_x: c_int = 0;
        let mut out_y: c_int = 0;
        unsafe {
            nh_ffi_test_finddpos(xl as c_int, yl as c_int, xh as c_int, yh as c_int, &mut out_x, &mut out_y);
        }
        (out_x as i32, out_y as i32)
    }

    /// Test C's dig_corridor() in isolation. Returns true if corridor was dug.
    pub fn test_dig_corridor(&self, sx: i32, sy: i32, dx: i32, dy: i32, nxcor: bool) -> bool {
        unsafe { nh_ffi_test_dig_corridor(sx as c_int, sy as c_int, dx as c_int, dy as c_int, nxcor as c_int) != 0 }
    }

    /// Test C's makecorridors() in isolation.
    pub fn test_makecorridors(&self) {
        unsafe { nh_ffi_test_makecorridors() }
    }

    /// Test C's join() in isolation (re-implemented in FFI using public functions).
    pub fn test_join(&self, a: i32, b: i32, nxcor: bool) {
        unsafe { nh_ffi_test_join(a as c_int, b as c_int, nxcor as c_int) }
    }

    /// Get C's smeq[] connectivity array as JSON.
    pub fn get_smeq(&self) -> String {
        let ptr = unsafe { nh_ffi_get_smeq() };
        if ptr.is_null() { return "[]".to_string(); }
        let result = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(ptr as *mut c_void) };
        result
    }

    /// Get C's doorindex value.
    pub fn get_doorindex(&self) -> i32 {
        unsafe { nh_ffi_get_doorindex() as i32 }
    }

    /// Get a rectangular region of level cells as a JSON array of type IDs.
    pub fn get_cell_region(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> String {
        let json_ptr = unsafe { nh_ffi_get_cell_region(x1 as c_int, y1 as c_int, x2 as c_int, y2 as c_int) };
        if json_ptr.is_null() {
            return "[]".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    /// Set a single cell type on the current C level.
    pub fn set_cell(&self, x: i32, y: i32, typ: i32) {
        unsafe { nh_ffi_set_cell(x as c_int, y as c_int, typ as c_int) }
    }

    /// Clear the entire C level to STONE.
    pub fn clear_level(&self) {
        unsafe { nh_ffi_clear_level() }
    }

    /// Add a room to C's rooms[] array. Returns room index or -1.
    pub fn add_room(&self, lx: i32, ly: i32, hx: i32, hy: i32, rtype: i32) -> i32 {
        unsafe { nh_ffi_add_room(lx as c_int, ly as c_int, hx as c_int, hy as c_int, rtype as c_int) as i32 }
    }

    /// Carve a room's interior and walls on the C level.
    pub fn carve_room(&self, lx: i32, ly: i32, hx: i32, hy: i32) {
        unsafe { nh_ffi_carve_room(lx as c_int, ly as c_int, hx as c_int, hy as c_int) }
    }

    /// Get C's rectangle list as JSON.
    pub fn rect_json(&self) -> String {
        let json_ptr = unsafe { nh_ffi_get_rect_json() };
        if json_ptr.is_null() {
            return "{\"count\":0,\"rects\":[]}".to_string();
        }
        let result = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
        unsafe { nh_ffi_free_string(json_ptr as *mut c_void) };
        result
    }

    /// Enable RNG tracing for the C engine.
    pub fn start_tracing(&mut self) {
        self.enable_rng_tracing();
    }

    /// Get RNG trace
    pub fn get_trace(&self) -> Vec<nh_rng::RngTraceEntry> {
        Vec::new() // Full trace requires JSON parsing; use rng_trace_json() instead
    }
}

impl Drop for CGameEngine {
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
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_c_game_engine_lifecycle() {
        let mut engine = CGameEngine::new();
        assert!(!engine.is_initialized());

        assert!(engine.init("Tourist", "Human", 0, 0).is_ok());
        assert!(engine.is_initialized());

        let hp = engine.hp();
        assert!(hp > 0, "HP should be positive, got {}", hp);
        assert!(engine.max_hp() > 0);
        assert!(!engine.is_dead());
        assert!(!engine.is_game_over());
        
        #[cfg(not(real_nethack))]
        assert_eq!(engine.position(), (40, 10));
        #[cfg(real_nethack)]
        assert_eq!(engine.position(), (0, 0));

        let (x_start, y_start) = engine.position();
        let _ = engine.exec_cmd_dir('h', -1, 0);
        assert_eq!(engine.position(), (x_start - 1, y_start));

        let _ = engine.exec_cmd_dir('j', 0, 1);
        assert_eq!(engine.position(), (x_start - 1, y_start + 1));

        let turns = engine.turn_count();
        #[cfg(not(real_nethack))]
        assert!(turns >= 2, "Turn count should be at least 2, got {}", turns);
        #[cfg(real_nethack)]
        assert!(turns >= 3, "Turn count should be at least 3, got {}", turns);
    }

    #[test]
    #[serial]
    fn test_state_json() {
        let mut engine = CGameEngine::new();
        engine.init("Wizard", "Elf", 1, 1).unwrap();

        let json = engine.state_json();
        assert!(json.contains("hp"));
        assert!(json.contains("x"));
        assert!(json.contains("y"));
    }

    #[test]
    #[serial]
    fn test_reset() {
        let mut engine = CGameEngine::new();
        engine.init("Rogue", "Gnome", 0, 0).unwrap();

        let (x_start, y_start) = engine.position();
        let _ = engine.exec_cmd_dir('l', 1, 0);
        let _ = engine.exec_cmd_dir('l', 1, 0);

        engine.reset(12345).unwrap();

        assert_eq!(engine.position(), (x_start, y_start));
        #[cfg(not(real_nethack))]
        assert_eq!(engine.turn_count(), 0);
        #[cfg(real_nethack)]
        assert_eq!(engine.turn_count(), 1);
    }

    #[test]
    #[serial]
    fn test_unknown_command() {
        let mut engine = CGameEngine::new();
        engine.init("Priest", "Dwarf", 0, 0).unwrap();

        assert!(engine.exec_cmd('@').is_err());
    }
}
