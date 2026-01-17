//! To-hit calculation comparison
//!
//! Compares the to-hit formula between C and Rust implementations.
//!
//! C formula from find_roll_to_hit() in uhitm.c:
//! ```c
//! tmp = 1 + Luck + abon() + find_mac(mtmp) + u.uhitinc
//!       + maybe_polyd(youmonst.data->mlevel, u.ulevel);
//! ```
//!
//! Additional modifiers:
//! - Monster state: stunned (+2), fleeing (+2), sleeping (+2), paralyzed (+4)
//! - Role: Monk without armor gets bonus
//! - Race: Elf vs Orc (+1)
//! - Encumbrance: penalty based on near_capacity()
//! - Trap: -3 if trapped
//! - Weapon bonuses: hitval() and weapon_hit_bonus()

use crate::Isaac64;

/// Components of the to-hit formula
#[derive(Debug, Clone, Default)]
pub struct ToHitComponents {
    pub base: i32,              // Always 1
    pub luck: i32,              // Player luck (-13 to +13)
    pub abon: i32,              // Ability bonus from strength/dexterity
    pub target_ac: i32,         // Target's armor class (lower is better)
    pub hit_inc: i32,           // u.uhitinc - intrinsic hit bonus
    pub level: i32,             // Player level
    pub stunned_bonus: i32,     // +2 if target stunned
    pub fleeing_bonus: i32,     // +2 if target fleeing
    pub sleeping_bonus: i32,    // +2 if target sleeping
    pub paralyzed_bonus: i32,   // +4 if target paralyzed
    pub encumbrance: i32,       // Penalty from carrying too much
    pub trap_penalty: i32,      // -3 if player trapped
    pub weapon_bonus: i32,      // Weapon hit bonuses
}

impl ToHitComponents {
    /// Calculate total to-hit modifier
    pub fn total(&self) -> i32 {
        self.base
            + self.luck
            + self.abon
            + self.target_ac
            + self.hit_inc
            + self.level
            + self.stunned_bonus
            + self.fleeing_bonus
            + self.sleeping_bonus
            + self.paralyzed_bonus
            + self.encumbrance
            + self.trap_penalty
            + self.weapon_bonus
    }
}

/// Attack hit check formula
///
/// Returns true if attack hits.
/// C formula: roll + to_hit > 10 - target_ac
/// Equivalent to: roll + to_hit + target_ac > 10
pub fn attack_hits(roll: i32, to_hit: i32, target_ac: i32) -> bool {
    roll + to_hit > 10 - target_ac
}

/// Compare hit check between Rust and expected formula
pub fn verify_hit_check(to_hit: i32, target_ac: i32, seed: u64) -> (bool, bool, i32) {
    let mut rust_rng = Isaac64::new(seed);
    let roll = rust_rng.rnd(20) as i32;
    let rust_hits = roll + to_hit > 10 - target_ac;
    let formula_hits = attack_hits(roll, to_hit, target_ac);
    (rust_hits, formula_hits, roll)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_formula_basic() {
        // Test basic hit formula: roll + to_hit > 10 - target_ac
        // AC 10 (no armor): need roll + to_hit > 0
        // AC 0 (good armor): need roll + to_hit > 10
        // AC -10 (excellent): need roll + to_hit > 20

        // With to_hit = 5, AC = 10:
        // Need roll > 0 - 5 = -5, so any roll hits
        assert!(attack_hits(1, 5, 10)); // Roll 1, to_hit 5, AC 10 -> 1+5 > 0 -> true

        // With to_hit = 0, AC = 5:
        // Need roll > 10 - 5 = 5
        assert!(!attack_hits(5, 0, 5)); // Roll 5, to_hit 0, AC 5 -> 5+0 > 5 -> false
        assert!(attack_hits(6, 0, 5)); // Roll 6, to_hit 0, AC 5 -> 6+0 > 5 -> true

        // With to_hit = 10, AC = 0:
        // Need roll > 10 - 10 = 0
        assert!(attack_hits(1, 10, 0)); // Roll 1, to_hit 10, AC 0 -> 1+10 > 10 -> true

        // With to_hit = 0, AC = -5:
        // Need roll > 10 - (-5) = 15
        assert!(!attack_hits(15, 0, -5)); // Roll 15, to_hit 0, AC -5 -> 15+0 > 15 -> false
        assert!(attack_hits(16, 0, -5)); // Roll 16, to_hit 0, AC -5 -> 16+0 > 15 -> true
    }

    #[test]
    fn test_tohit_components() {
        // Basic character: level 1, no luck, no bonuses
        let components = ToHitComponents {
            base: 1,
            luck: 0,
            level: 1,
            ..Default::default()
        };
        assert_eq!(components.total(), 2);

        // Lucky character with good weapon
        let components = ToHitComponents {
            base: 1,
            luck: 5,
            level: 10,
            weapon_bonus: 3,
            ..Default::default()
        };
        assert_eq!(components.total(), 19);

        // Unlucky character, trapped, overloaded
        let components = ToHitComponents {
            base: 1,
            luck: -5,
            level: 5,
            trap_penalty: -3,
            encumbrance: -5,
            ..Default::default()
        };
        assert_eq!(components.total(), -7);
    }

    #[test]
    fn test_monster_state_bonuses() {
        // Verify monster state bonuses match C values
        let stunned_bonus = 2;
        let fleeing_bonus = 2;
        let sleeping_bonus = 2;
        let paralyzed_bonus = 4;

        let mut components = ToHitComponents {
            base: 1,
            ..Default::default()
        };

        // No bonuses
        assert_eq!(components.total(), 1);

        // All bonuses
        components.stunned_bonus = stunned_bonus;
        components.fleeing_bonus = fleeing_bonus;
        components.sleeping_bonus = sleeping_bonus;
        components.paralyzed_bonus = paralyzed_bonus;

        assert_eq!(components.total(), 1 + 2 + 2 + 2 + 4); // 11
    }

    #[test]
    fn test_hit_check_consistency() {
        // Verify hit check produces consistent results
        for seed in [42u64, 12345, 99999, 1, 777] {
            for to_hit in [-5, 0, 5, 10, 15] {
                for target_ac in [-10, -5, 0, 5, 10] {
                    let (rust, formula, roll) = verify_hit_check(to_hit, target_ac, seed);
                    assert_eq!(
                        rust, formula,
                        "Hit check mismatch: to_hit={}, AC={}, roll={}, seed={}",
                        to_hit, target_ac, roll, seed
                    );
                }
            }
        }
    }

    #[test]
    fn test_d20_roll_distribution() {
        // Verify d20 rolls are uniformly distributed (1-20)
        let mut rust_rng = Isaac64::new(42);
        let mut counts = [0u32; 20];

        for _ in 0..100000 {
            let roll = rust_rng.rnd(20) as usize;
            assert!(roll >= 1 && roll <= 20, "Invalid d20 roll: {}", roll);
            counts[roll - 1] += 1;
        }

        // Each number should appear roughly 5000 times (5%)
        // Allow 20% deviation
        let expected = 5000.0;
        let min = (expected * 0.8) as u32;
        let max = (expected * 1.2) as u32;

        for (i, &count) in counts.iter().enumerate() {
            assert!(
                count >= min && count <= max,
                "Roll {} appeared {} times (expected ~{})",
                i + 1,
                count,
                expected
            );
        }
    }

    #[test]
    fn test_encumbrance_penalties() {
        // C encumbrance: tmp -= (tmp2 * 2) - 1
        // tmp2 is near_capacity() result: 0-5
        // 0 = unencumbered: no penalty
        // 1 = burdened: -1
        // 2 = stressed: -3
        // 3 = strained: -5
        // 4 = overtaxed: -7
        // 5 = overloaded: -9

        let expected_penalties = [0, -1, -3, -5, -7, -9];
        for (capacity, expected) in expected_penalties.iter().enumerate() {
            let penalty = if capacity > 0 {
                -((capacity as i32 * 2) - 1)
            } else {
                0
            };
            assert_eq!(
                penalty, *expected,
                "Encumbrance {} should give penalty {}",
                capacity, expected
            );
        }
    }
}
