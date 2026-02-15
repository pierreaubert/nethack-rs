//! Stub C game engine for the virtual player system.
//!
//! The real C FFI bindings live in `nh-test`. This module provides a
//! compile-time placeholder so that the orchestrator and c_extractor
//! code can reference `CGameEngine` without pulling in the full C
//! NetHack link dependencies.  When actually running comparison sessions
//! the caller should use the real engine from `nh_test::ffi`.

/// Stub wrapper for the C NetHack game engine.
///
/// All methods panic at runtime — they exist only so that the crate
/// compiles.  Use `nh_test::ffi::CGameEngine` for real C interaction.
pub struct CGameEngine {
    _private: (),
}

impl CGameEngine {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn init(
        &mut self,
        _role: &str,
        _race: &str,
        _gender: i32,
        _alignment: i32,
    ) -> Result<(), String> {
        Err("CGameEngine stub — use nh_test::ffi::CGameEngine for real C FFI".into())
    }

    pub fn reset(&mut self, _seed: u64) -> Result<(), String> {
        Err("CGameEngine stub".into())
    }

    pub fn exec_cmd(&self, _cmd: char) -> Result<(), String> {
        Err("CGameEngine stub".into())
    }

    pub fn exec_cmd_dir(&self, _cmd: char, _dx: i32, _dy: i32) -> Result<(), String> {
        Err("CGameEngine stub".into())
    }

    pub fn hp(&self) -> i32 { 0 }
    pub fn max_hp(&self) -> i32 { 0 }
    pub fn energy(&self) -> i32 { 0 }
    pub fn max_energy(&self) -> i32 { 0 }
    pub fn position(&self) -> (i32, i32) { (0, 0) }
    pub fn set_state(&self, _x: i32, _y: i32, _hp: i32, _max_hp: i32, _level: i32, _ac: i32) {}
    pub fn armor_class(&self) -> i32 { 10 }
    pub fn gold(&self) -> i32 { 0 }
    pub fn experience_level(&self) -> i32 { 1 }
    pub fn current_level(&self) -> i32 { 1 }
    pub fn dungeon_depth(&self) -> i32 { 1 }
    pub fn turn_count(&self) -> u64 { 0 }
    pub fn is_dead(&self) -> bool { false }
    pub fn is_game_over(&self) -> bool { false }
    pub fn is_won(&self) -> bool { false }
    pub fn state_json(&self) -> String { "{}".into() }
    pub fn last_message(&self) -> String { String::new() }
    pub fn inventory_json(&self) -> String { "[]".into() }
    pub fn monsters_json(&self) -> String { "[]".into() }
    pub fn role(&self) -> String { "Unknown".into() }
    pub fn race(&self) -> String { "Unknown".into() }
    pub fn gender_string(&self) -> String { "Unknown".into() }
    pub fn alignment_string(&self) -> String { "Unknown".into() }
}
