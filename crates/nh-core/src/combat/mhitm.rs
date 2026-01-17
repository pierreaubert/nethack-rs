//! Monster attacks monster combat (mhitm.c)
//!
//! Handles all combat between monsters (including pets).

use super::{Attack, CombatEffect, CombatResult, DamageType};
use crate::monster::Monster;
use crate::rng::GameRng;

/// Calculate attacker monster's to-hit bonus
fn calculate_monster_to_hit(attacker: &Monster, defender: &Monster) -> i32 {
    // Base is attacker's level
    let mut to_hit = attacker.level as i32;

    // Attacker state penalties
    if attacker.state.confused {
        to_hit -= 2;
    }
    if attacker.state.stunned {
        to_hit -= 2;
    }
    if attacker.state.blinded {
        to_hit -= 2;
    }

    // Bonus vs disabled defender
    if defender.state.sleeping {
        to_hit += 2;
    }
    if defender.state.stunned || defender.state.confused || defender.state.blinded || defender.state.paralyzed {
        to_hit += 4;
    }
    if defender.state.fleeing {
        to_hit += 2;
    }

    to_hit
}

/// Check if attack hits based on to-hit and defender AC
fn attack_hits(to_hit: i32, defender_ac: i8, rng: &mut GameRng) -> bool {
    let roll = rng.rnd(20) as i32;
    // Same formula as player combat: roll + to_hit > 10 - AC
    roll + to_hit > 10 - defender_ac as i32
}

/// Calculate damage multiplier based on defender monster's resistances
/// Returns (multiplier_num, multiplier_den) where damage = damage * num / den
fn damage_multiplier_for_monster_resistance(damage_type: DamageType, defender: &Monster) -> (i32, i32) {
    match damage_type {
        DamageType::Fire => {
            if defender.resists_fire() {
                (0, 1) // No damage
            } else {
                (1, 1) // Full damage
            }
        }
        DamageType::Cold => {
            if defender.resists_cold() {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::Electric => {
            if defender.resists_elec() {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::Acid => {
            if defender.resists_acid() {
                (1, 2) // Half damage with acid resistance
            } else {
                (1, 1)
            }
        }
        DamageType::Disintegrate => {
            if defender.resists_disint() {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        _ => (1, 1), // Full damage for non-elemental
    }
}

/// Apply special effects from monster attacks to defender monster
fn apply_monster_damage_effect(
    damage_type: DamageType,
    defender: &mut Monster,
    rng: &mut GameRng,
) -> Option<CombatEffect> {
    match damage_type {
        DamageType::Physical => None,

        DamageType::Fire => {
            // Fire resistance blocks fire effects
            if defender.resists_fire() {
                None
            } else {
                None // No special effect beyond damage
            }
        }

        DamageType::Cold => {
            // Cold resistance blocks cold effects
            if defender.resists_cold() {
                None
            } else {
                None
            }
        }

        DamageType::Electric => {
            // Electric resistance blocks shock effects
            if defender.resists_elec() {
                None
            } else {
                None
            }
        }

        DamageType::Acid => {
            // Acid resistance reduces but doesn't fully block
            None
        }

        DamageType::Sleep => {
            // Sleep resistance protects against sleep attacks
            if defender.resists_sleep() {
                None
            } else if rng.one_in(3) {
                let duration = rng.rnd(10) as u16 + 5;
                defender.sleep_timeout = defender.sleep_timeout.saturating_add(duration);
                defender.state.sleeping = true;
                Some(CombatEffect::Paralyzed)
            } else {
                None
            }
        }

        DamageType::Stone => {
            // Stone resistance protects against petrification
            if defender.resists_stone() {
                None
            } else {
                Some(CombatEffect::Petrifying)
            }
        }

        DamageType::Disintegrate => {
            // Disintegration resistance protects completely
            if defender.resists_disint() {
                None
            } else {
                Some(CombatEffect::Petrifying) // Instant death effect
            }
        }

        DamageType::Confuse => {
            let duration = rng.rnd(10) as u16 + 10;
            defender.confused_timeout = defender.confused_timeout.saturating_add(duration);
            defender.state.confused = true;
            Some(CombatEffect::Confused)
        }

        DamageType::Stun => {
            let duration = rng.rnd(5) as u16 + 5;
            defender.frozen_timeout = defender.frozen_timeout.saturating_add(duration);
            defender.state.stunned = true;
            Some(CombatEffect::Stunned)
        }

        DamageType::Blind => {
            let duration = rng.rnd(100) as u16 + 20;
            defender.blinded_timeout = defender.blinded_timeout.saturating_add(duration);
            defender.state.blinded = true;
            Some(CombatEffect::Blinded)
        }

        DamageType::Paralyze => {
            let duration = rng.rnd(5) as u16 + 3;
            defender.frozen_timeout = defender.frozen_timeout.saturating_add(duration);
            defender.state.paralyzed = true;
            Some(CombatEffect::Paralyzed)
        }

        DamageType::DrainLife => {
            // Drain one level (no drain resistance check for monsters currently)
            if defender.level > 0 {
                defender.level -= 1;
                defender.hp_max = (defender.hp_max - rng.rnd(5) as i32).max(1);
                defender.hp = defender.hp.min(defender.hp_max);
                Some(CombatEffect::Drained)
            } else {
                None
            }
        }

        DamageType::DrainStrength => {
            // Poison resistance protects against poison effects
            if defender.resists_poison() {
                None
            } else {
                // Monsters don't have attribute stats, but we can note it happened
                Some(CombatEffect::Poisoned)
            }
        }

        DamageType::Disease => {
            // Poison resistance protects against disease
            if defender.resists_poison() {
                None
            } else {
                Some(CombatEffect::Poisoned)
            }
        }

        DamageType::Digest => Some(CombatEffect::Engulfed),

        DamageType::Wrap | DamageType::Stick => Some(CombatEffect::Grabbed),

        _ => None,
    }
}

/// Monster melee attack against another monster
pub fn monster_attack_monster(
    attacker: &mut Monster,
    defender: &mut Monster,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // Calculate to-hit
    let to_hit = calculate_monster_to_hit(attacker, defender);

    // Use defender's AC
    let defender_ac = defender.ac;

    // Check if attack hits
    if !attack_hits(to_hit, defender_ac, rng) {
        return CombatResult::MISS;
    }

    // Calculate base damage from attack dice
    let mut damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Apply resistance-based damage reduction
    let (mult_num, mult_den) = damage_multiplier_for_monster_resistance(attack.damage_type, defender);
    damage = damage * mult_num / mult_den;

    // Ensure minimum 1 damage on hit (unless fully immune)
    if mult_num > 0 {
        damage = damage.max(1);
    }

    // Apply damage to defender
    defender.hp -= damage;

    // Apply special effects
    let special_effect = apply_monster_damage_effect(attack.damage_type, defender, rng);

    // Check for attacker death (cockatrice, etc.)
    let attacker_died = if special_effect == Some(CombatEffect::Petrifying) {
        // If defender was petrifying, attacker might die from touching stone
        // TODO: Check if attacker has petrification resistance
        false
    } else {
        false
    };

    CombatResult {
        hit: true,
        defender_died: defender.hp <= 0,
        attacker_died,
        damage,
        special_effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::AttackType;
    use crate::monster::MonsterId;

    fn test_monster(level: u8, ac: i8) -> Monster {
        let mut monster = Monster::new(MonsterId(1), level as i16, 5, 5);
        monster.level = level;
        monster.ac = ac;
        monster.hp = 50;
        monster.hp_max = 50;
        monster
    }

    #[test]
    fn test_monster_to_hit_base() {
        let attacker = test_monster(5, 10);
        let defender = test_monster(3, 8);

        // Level 5 attacker has to-hit of 5
        let to_hit = calculate_monster_to_hit(&attacker, &defender);
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_monster_to_hit_confused_attacker() {
        let mut attacker = test_monster(5, 10);
        attacker.state.confused = true;
        let defender = test_monster(3, 8);

        // Confused: 5 - 2 = 3
        let to_hit = calculate_monster_to_hit(&attacker, &defender);
        assert_eq!(to_hit, 3);
    }

    #[test]
    fn test_monster_to_hit_vs_sleeping_defender() {
        let attacker = test_monster(5, 10);
        let mut defender = test_monster(3, 8);
        defender.state.sleeping = true;

        // Sleeping defender: 5 + 2 = 7
        let to_hit = calculate_monster_to_hit(&attacker, &defender);
        assert_eq!(to_hit, 7);
    }

    #[test]
    fn test_monster_to_hit_vs_stunned_defender() {
        let attacker = test_monster(5, 10);
        let mut defender = test_monster(3, 8);
        defender.state.stunned = true;

        // Stunned defender: 5 + 4 = 9
        let to_hit = calculate_monster_to_hit(&attacker, &defender);
        assert_eq!(to_hit, 9);
    }

    #[test]
    fn test_attack_hits_high_to_hit() {
        let mut rng = GameRng::new(42);

        // High to-hit vs average AC should always hit
        let mut hits = 0;
        for _ in 0..100 {
            if attack_hits(15, 10, &mut rng) {
                hits += 1;
            }
        }
        // 15 + roll > 10 - 10 = 0, always hits
        assert_eq!(hits, 100);
    }

    #[test]
    fn test_attack_hits_low_to_hit() {
        let mut rng = GameRng::new(42);

        // Low to-hit vs good AC should rarely hit
        let mut hits = 0;
        for _ in 0..1000 {
            if attack_hits(1, -5, &mut rng) {
                hits += 1;
            }
        }
        // 1 + roll > 10 - (-5) = 15, need roll > 14, hit on 15-20 (30% chance)
        assert!(hits > 200 && hits < 400, "Expected ~30% hits, got {}", hits);
    }

    #[test]
    fn test_monster_attack_damages() {
        let mut attacker = test_monster(10, 5);
        let mut defender = test_monster(3, 10);
        defender.hp = 100;
        let mut rng = GameRng::new(42);

        let attack = Attack::new(AttackType::Claw, DamageType::Physical, 2, 6);

        // Level 10 vs AC 10 should hit reliably
        let result = monster_attack_monster(&mut attacker, &mut defender, &attack, &mut rng);

        // Should hit with 2d6 damage (2-12, min 1)
        if result.hit {
            assert!(result.damage >= 1 && result.damage <= 12);
            assert_eq!(defender.hp, 100 - result.damage);
        }
    }

    #[test]
    fn test_monster_confuse_effect() {
        let mut defender = test_monster(3, 10);
        let mut rng = GameRng::new(42);

        assert!(!defender.state.confused);
        assert_eq!(defender.confused_timeout, 0);

        let effect = apply_monster_damage_effect(DamageType::Confuse, &mut defender, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Confused));
        assert!(defender.state.confused);
        assert!(defender.confused_timeout >= 10);
    }

    #[test]
    fn test_monster_drain_level() {
        let mut defender = test_monster(5, 10);
        defender.hp_max = 50;
        defender.hp = 50;
        let mut rng = GameRng::new(42);

        let effect = apply_monster_damage_effect(DamageType::DrainLife, &mut defender, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Drained));
        assert_eq!(defender.level, 4, "Should lose 1 level");
        assert!(defender.hp_max < 50, "Max HP should be reduced");
    }

    #[test]
    fn test_monster_vs_monster_ac() {
        let mut rng = GameRng::new(42);

        // Monster with good AC (low is better)
        let attacker = test_monster(5, 5);
        let mut defender_good_ac = test_monster(3, -3);
        defender_good_ac.hp = 1000;

        // Monster with poor AC
        let mut defender_poor_ac = test_monster(3, 10);
        defender_poor_ac.hp = 1000;

        let attack = Attack::new(AttackType::Claw, DamageType::Physical, 1, 4);

        let mut hits_good_ac = 0;
        let mut hits_poor_ac = 0;

        for _ in 0..1000 {
            defender_good_ac.hp = 1000;
            defender_poor_ac.hp = 1000;

            let result = monster_attack_monster(&mut attacker.clone(), &mut defender_good_ac, &attack, &mut rng);
            if result.hit {
                hits_good_ac += 1;
            }

            let result = monster_attack_monster(&mut attacker.clone(), &mut defender_poor_ac, &attack, &mut rng);
            if result.hit {
                hits_poor_ac += 1;
            }
        }

        // Should hit poor AC more often
        assert!(
            hits_poor_ac > hits_good_ac,
            "Should hit AC 10 more than AC -3: {} vs {}",
            hits_poor_ac,
            hits_good_ac
        );
    }

    // Resistance tests

    #[test]
    fn test_fire_resistance_blocks_damage() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::FIRE;
        defender.hp = 100;

        let (mult_num, mult_den) = damage_multiplier_for_monster_resistance(DamageType::Fire, &defender);
        assert_eq!((mult_num, mult_den), (0, 1), "Fire resistance should block all fire damage");
    }

    #[test]
    fn test_cold_resistance_blocks_damage() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::COLD;

        let (mult_num, mult_den) = damage_multiplier_for_monster_resistance(DamageType::Cold, &defender);
        assert_eq!((mult_num, mult_den), (0, 1), "Cold resistance should block all cold damage");
    }

    #[test]
    fn test_elec_resistance_blocks_damage() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::ELEC;

        let (mult_num, mult_den) = damage_multiplier_for_monster_resistance(DamageType::Electric, &defender);
        assert_eq!((mult_num, mult_den), (0, 1), "Electric resistance should block all shock damage");
    }

    #[test]
    fn test_acid_resistance_halves_damage() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::ACID;

        let (mult_num, mult_den) = damage_multiplier_for_monster_resistance(DamageType::Acid, &defender);
        assert_eq!((mult_num, mult_den), (1, 2), "Acid resistance should halve acid damage");
    }

    #[test]
    fn test_disint_resistance_blocks_damage() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::DISINT;

        let (mult_num, mult_den) = damage_multiplier_for_monster_resistance(DamageType::Disintegrate, &defender);
        assert_eq!((mult_num, mult_den), (0, 1), "Disintegration resistance should block disintegration damage");
    }

    #[test]
    fn test_sleep_resistance_blocks_effect() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::SLEEP;
        let mut rng = GameRng::new(42);

        // Run multiple times since sleep has a 1/3 chance
        for _ in 0..100 {
            defender.state.sleeping = false;
            defender.sleep_timeout = 0;
            let effect = apply_monster_damage_effect(DamageType::Sleep, &mut defender, &mut rng);
            assert_eq!(effect, None, "Sleep resistance should block sleep effect");
            assert!(!defender.state.sleeping, "Monster should not be put to sleep");
        }
    }

    #[test]
    fn test_stone_resistance_blocks_petrification() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::STONE;
        let mut rng = GameRng::new(42);

        let effect = apply_monster_damage_effect(DamageType::Stone, &mut defender, &mut rng);
        assert_eq!(effect, None, "Stone resistance should block petrification");
    }

    #[test]
    fn test_poison_resistance_blocks_strength_drain() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::POISON;
        let mut rng = GameRng::new(42);

        let effect = apply_monster_damage_effect(DamageType::DrainStrength, &mut defender, &mut rng);
        assert_eq!(effect, None, "Poison resistance should block strength drain");
    }

    #[test]
    fn test_poison_resistance_blocks_disease() {
        use crate::monster::MonsterResistances;

        let mut defender = test_monster(5, 10);
        defender.resistances = MonsterResistances::POISON;
        let mut rng = GameRng::new(42);

        let effect = apply_monster_damage_effect(DamageType::Disease, &mut defender, &mut rng);
        assert_eq!(effect, None, "Poison resistance should block disease");
    }

    #[test]
    fn test_fire_attack_no_damage_with_resistance() {
        use crate::monster::MonsterResistances;

        let attacker = test_monster(10, 5);
        let mut defender = test_monster(3, 10);
        defender.resistances = MonsterResistances::FIRE;
        defender.hp = 100;
        let mut rng = GameRng::new(42);

        let attack = Attack::new(AttackType::Claw, DamageType::Fire, 2, 6);

        // Run multiple times to ensure hits happen
        let mut hit_count = 0;
        for _ in 0..100 {
            defender.hp = 100;
            let result = monster_attack_monster(&mut attacker.clone(), &mut defender, &attack, &mut rng);
            if result.hit {
                hit_count += 1;
                assert_eq!(result.damage, 0, "Fire attack should deal 0 damage to fire-resistant monster");
                assert_eq!(defender.hp, 100, "Fire-resistant monster HP should not change");
            }
        }
        assert!(hit_count > 0, "Should have hit at least once");
    }

    #[test]
    fn test_acid_attack_half_damage_with_resistance() {
        use crate::monster::MonsterResistances;

        let attacker = test_monster(10, 5);
        let mut defender = test_monster(3, 10);
        defender.resistances = MonsterResistances::ACID;
        defender.hp = 1000;
        let mut rng = GameRng::new(42);

        // 2d6 normally = 2-12, halved = 1-6
        let attack = Attack::new(AttackType::Claw, DamageType::Acid, 2, 6);

        let mut hit_count = 0;
        for _ in 0..100 {
            defender.hp = 1000;
            let result = monster_attack_monster(&mut attacker.clone(), &mut defender, &attack, &mut rng);
            if result.hit {
                hit_count += 1;
                // Halved damage from 2d6: min 1 (2/2), max 6 (12/2)
                assert!(result.damage >= 1 && result.damage <= 6,
                    "Acid damage {} should be halved (1-6)", result.damage);
            }
        }
        assert!(hit_count > 0, "Should have hit at least once");
    }
}
