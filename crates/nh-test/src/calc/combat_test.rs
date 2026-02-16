//! Combat logic comparison tests

use crate::ffi::CGameEngineSubprocess as CGameEngine;
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_base_damage_calculation() {
    let mut engine = CGameEngine::new();
    engine.init("Tourist", "Human", 0, 0).unwrap();

    // Test case 1: Long sword (id=1, placeholder) vs small monster
    // In our stub, it returns 4. In real NetHack, it depends on weapon.
    let damage = engine.calc_base_damage(1, true);
    assert_eq!(damage, 4, "Base damage stub should return 4");

    // In a real scenario, we would check against Rust implementation:
    // let rust_damage = rust_engine.calc_damage(Weapon::LongSword, MonsterSize::Small);
    // assert_eq!(rust_damage, damage);
}

#[test]
#[serial]
fn test_ac_access() {
    let mut engine = CGameEngine::new();
    engine.init("Tourist", "Human", 0, 0).unwrap();
    // Default AC should be 10
    assert_eq!(engine.ac(), 10);
}

#[test]
#[serial]
fn test_rng_access() {
    let mut engine = CGameEngine::new();
    engine.init("Tourist", "Human", 0, 0).unwrap();
    let val = engine.rng_rn2(10);
    assert!(val >= 0 && val < 10, "RNG value {} out of range", val);
}
