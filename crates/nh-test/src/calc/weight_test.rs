//! Weight calculation tests using FFI
//!
//! Verifies inventory weight tracking matches C implementation.

use crate::ffi::CGameEngineSubprocess as CGameEngine;
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_inventory_weight_accumulation() {
    let mut engine = CGameEngine::new();
    engine.init("Tourist", "Human", 0, 0).unwrap();

    // Initial weight should be 0 (for stub purposes, real game has starting inventory)
    assert_eq!(engine.carrying_weight(), 0, "Initial weight should be 0");

    // Add item with weight 10
    engine.add_item_to_inv(1, 10).unwrap();
    assert_eq!(engine.carrying_weight(), 10, "Weight should be 10");

    // Add another item with weight 5
    engine.add_item_to_inv(2, 5).unwrap();
    assert_eq!(engine.carrying_weight(), 15, "Weight should be 15");
}
