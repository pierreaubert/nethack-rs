//! Combat logic comparison tests

use crate::c_interface_ffi::FfiGameEngine;

#[test]
fn test_base_damage_calculation() {
    let engine = FfiGameEngine::new();
    
    // Test case 1: Long sword (id=1, placeholder) vs small monster
    // In our stub, it returns 4. In real NetHack, it depends on weapon.
    let damage = engine.calc_base_damage(1, true);
    assert_eq!(damage, 4, "Base damage stub should return 4");
    
    // In a real scenario, we would check against Rust implementation:
    // let rust_damage = rust_engine.calc_damage(Weapon::LongSword, MonsterSize::Small);
    // assert_eq!(rust_damage, damage);
}

#[test]
fn test_ac_access() {
    let engine = FfiGameEngine::new();
    // Default stub AC is 10
    assert_eq!(engine.ac(), 10);
}

#[test]
fn test_rng_access() {
    let engine = FfiGameEngine::new();
    // Stub rng_rn2 returns 0
    assert_eq!(engine.rng_rn2(10), 0);
}
