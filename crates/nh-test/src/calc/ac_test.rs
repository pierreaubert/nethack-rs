//! AC Calculation tests using FFI
//!
//! Verifies AC logic matches C implementation.

use crate::ffi::CGameEngine;
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_ac_update_on_wear() {
    let mut engine = CGameEngine::new();
    engine.init("Knight", "Human", 0, 0).unwrap();

    // Initial AC should be 10 (unarmored)
    engine.test_setup_status(10, 10, 1, 10);
    assert_eq!(engine.ac(), 10, "Initial AC should be 10");

    // Wear an item (id 1, mock)
    // In our stub, wearing anything reduces AC by 1
    engine.wear_item(1).unwrap();

    assert_eq!(engine.ac(), 9, "AC should decrease by 1 after wearing item");

    // Wear another item
    engine.wear_item(2).unwrap();
    assert_eq!(
        engine.ac(),
        8,
        "AC should decrease by 1 after wearing another item"
    );
}
