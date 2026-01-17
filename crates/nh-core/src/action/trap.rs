//! Trap mechanics (trap.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;

/// Trap types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    ArrowTrap,
    DartTrap,
    Pit,
    SpikedPit,
    TeleportTrap,
    BearTrap,
    SleepingGas,
    FireTrap,
}

/// Check for and trigger traps at a position
pub fn check_trap(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    let _trap = state.current_level.trap_at(x, y);
    ActionResult::NoTime
}

/// Trigger a specific trap
pub fn trigger_trap(state: &mut GameState, trap_type: TrapType) -> ActionResult {
    match trap_type {
        TrapType::ArrowTrap => {
            state.message("An arrow shoots out at you!");
            let damage = state.rng.rnd(6) as i32;
            state.player.take_damage(damage);
        }
        TrapType::Pit => {
            state.message("You fall into a pit!");
            let damage = state.rng.rnd(6) as i32;
            state.player.take_damage(damage);
        }
        TrapType::TeleportTrap => {
            state.message("You feel a strange vibration...");
        }
        _ => {
            state.message("You trigger a trap!");
        }
    }

    if state.player.is_dead() {
        return ActionResult::Died("killed by a trap".to_string());
    }

    ActionResult::Success
}

/// Search for traps
pub fn do_search(state: &mut GameState) -> ActionResult {
    state.message("You search for traps and secret doors.");
    ActionResult::Success
}
