//! Stub C game engine for the virtual player system.
//!
//! The real C FFI bindings live in `nh-test`. This module provides a
//! compile-time placeholder so that the orchestrator and c_extractor
//! code can reference `CGameEngine` without pulling in the full C
//! NetHack link dependencies.  When actually running comparison sessions
//! the caller should use the real engine from `nh_test::ffi`.

use nh_core::CGameEngineTrait;

/// Stub wrapper for the C NetHack game engine.
pub struct CGameEngine {
    _private: (),
}

impl CGameEngineTrait for CGameEngine {
    fn init(
        &mut self,
        _role: &str,
        _race: &str,
        _gender: i32,
        _alignment: i32,
    ) -> Result<(), String> {
        Err("CGameEngine stub — use nh_test::ffi::CGameEngine for real C FFI".into())
    }

    fn reset(&mut self, _seed: u64) -> Result<(), String> {
        Err("CGameEngine stub".into())
    }

    fn generate_and_place(&self) -> Result<(), String> {
        Err("CGameEngine stub".into())
    }

    fn export_level(&self) -> String {
        "{}".into()
    }

    fn exec_cmd(&self, _cmd: char) -> Result<(), String> {
        Err("CGameEngine stub".into())
    }

    fn exec_cmd_dir(&self, _cmd: char, _dx: i32, _dy: i32) -> Result<(), String> {
        Err("CGameEngine stub".into())
    }

    fn hp(&self) -> i32 { 0 }
    fn max_hp(&self) -> i32 { 0 }
    fn energy(&self) -> i32 { 0 }
    fn max_energy(&self) -> i32 { 0 }
    fn position(&self) -> (i32, i32) { (0, 0) }
    fn set_state(&self, _hp: i32, _hpmax: i32, _x: i32, _y: i32, _ac: i32, _moves: i64) {}
    fn armor_class(&self) -> i32 { 10 }
    fn gold(&self) -> i32 { 0 }
    fn experience_level(&self) -> i32 { 1 }
    fn current_level(&self) -> i32 { 1 }
    fn dungeon_depth(&self) -> i32 { 1 }
    fn turn_count(&self) -> u64 { 0 }
    fn is_dead(&self) -> bool { false }
    fn is_game_over(&self) -> bool { false }
    fn is_won(&self) -> bool { false }
    fn state_json(&self) -> String { "{}".into() }
    fn last_message(&self) -> String { String::new() }
    fn inventory_json(&self) -> String { "[]".into() }
    fn monsters_json(&self) -> String { "[]".into() }
    fn role(&self) -> String { "Unknown".into() }
    fn race(&self) -> String { "Unknown".into() }
    fn gender_string(&self) -> String { "Unknown".into() }
    fn alignment_string(&self) -> String { "Unknown".into() }
}

impl CGameEngine {
    pub fn new() -> Self {
        Self { _private: () }
    }
}
