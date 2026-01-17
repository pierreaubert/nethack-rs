//! Monster attacks player combat (mhitu.c)
//!
//! Handles all combat initiated by monsters against the player.

use super::{Attack, CombatEffect, CombatResult, DamageType};
use crate::monster::Monster;
use crate::player::You;
use crate::rng::GameRng;

/// Calculate monster's to-hit bonus
///
/// Based on find_roll_to_hit() in mhitu.c
fn calculate_monster_to_hit(attacker: &Monster, player: &You) -> i32 {
    // Base is monster level
    let mut to_hit = attacker.level as i32;

    // Monster state penalties
    if attacker.state.confused {
        to_hit -= 2;
    }
    if attacker.state.stunned {
        to_hit -= 2;
    }
    if attacker.state.blinded {
        to_hit -= 2;
    }

    // Bonus vs disabled player
    if player.is_stunned() {
        to_hit += 2;
    }
    if player.is_confused() {
        to_hit += 2;
    }
    if player.is_blind() {
        to_hit += 2;
    }
    if player.sleeping_timeout > 0 {
        to_hit += 4;
    }
    if player.paralyzed_timeout > 0 {
        to_hit += 4;
    }

    to_hit
}

/// Calculate damage multiplier based on player's elemental resistances
/// Returns (multiplier_num, multiplier_den) where damage = damage * num / den
fn damage_multiplier_for_resistance(damage_type: DamageType, player: &You) -> (i32, i32) {
    use crate::player::Property;

    match damage_type {
        DamageType::Fire => {
            if player.properties.has(Property::FireResistance) {
                (0, 1) // No damage
            } else {
                (1, 1) // Full damage
            }
        }
        DamageType::Cold => {
            if player.properties.has(Property::ColdResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::Electric => {
            if player.properties.has(Property::ShockResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::Acid => {
            if player.properties.has(Property::AcidResistance) {
                (1, 2) // Half damage with acid resistance
            } else {
                (1, 1)
            }
        }
        DamageType::Disintegrate => {
            if player.properties.has(Property::DisintResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::MagicMissile => {
            if player.properties.has(Property::MagicResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        _ => {
            // Check for half physical damage for physical attacks
            if damage_type == DamageType::Physical
                && player.properties.has(Property::HalfPhysDamage)
            {
                (1, 2)
            } else {
                (1, 1)
            }
        }
    }
}

/// Monster melee attack against player
pub fn monster_attack_player(
    attacker: &Monster,
    player: &mut You,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // TODO: Check if monster can reach player (distance, engulfed, etc.)

    // Calculate to-hit
    let to_hit = calculate_monster_to_hit(attacker, player);

    // Roll to hit
    // Formula: roll + to_hit > 10 - AC means hit
    // With AC 10 (no armor), need roll + to_hit > 0 (always hits with any to_hit > -19)
    // With AC -10 (good armor), need roll + to_hit > 20 (harder to hit)
    let roll = rng.rnd(20) as i32;
    if roll + to_hit <= 10 - player.armor_class as i32 {
        return CombatResult::MISS;
    }

    // Calculate base damage
    let mut damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Apply resistance-based damage reduction
    let (mult_num, mult_den) = damage_multiplier_for_resistance(attack.damage_type, player);
    damage = damage * mult_num / mult_den;

    // Apply special damage effects based on damage type
    let special_effect = apply_damage_effect(attack.damage_type, player, damage, rng);

    // Apply damage to player (minimum 0 after resistance)
    if damage > 0 {
        player.hp -= damage;
    }

    CombatResult {
        hit: true,
        defender_died: player.hp <= 0,
        attacker_died: false,
        damage,
        special_effect,
    }
}

/// Apply special effects based on damage type
/// Returns (effect, damage_multiplier) where damage_multiplier adjusts the base damage
fn apply_damage_effect(
    damage_type: DamageType,
    player: &mut You,
    _damage: i32,
    rng: &mut GameRng,
) -> Option<CombatEffect> {
    use crate::player::{Attribute, Property};

    match damage_type {
        DamageType::Physical => None,

        DamageType::Fire => {
            // Fire resistance negates fire damage effects
            if player.properties.has(Property::FireResistance) {
                // With resistance, 1/20 chance to still burn items
                if rng.one_in(20) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            } else {
                // Without resistance, 1/3 chance to burn scrolls/potions
                if rng.one_in(3) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            }
        }

        DamageType::Cold => {
            // Cold resistance negates cold damage effects
            if player.properties.has(Property::ColdResistance) {
                None
            } else {
                // 1/3 chance to freeze and shatter potions
                if rng.one_in(3) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            }
        }

        DamageType::Electric => {
            // Shock resistance negates electric damage effects
            if player.properties.has(Property::ShockResistance) {
                None
            } else {
                // 1/3 chance to destroy rings or wands
                if rng.one_in(3) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            }
        }

        DamageType::Sleep => {
            // Sleep resistance protects against sleep attacks
            if player.properties.has(Property::SleepResistance) {
                None
            } else if rng.one_in(3) {
                // Put player to sleep for 5-14 turns
                let duration = rng.rnd(10) as u16 + 5;
                player.sleeping_timeout = player.sleeping_timeout.saturating_add(duration);
                Some(CombatEffect::Paralyzed)
            } else {
                None
            }
        }

        DamageType::DrainLife => {
            // Drain resistance protects against level drain
            if player.properties.has(Property::DrainResistance) {
                None
            } else if player.exp_level > 1 {
                // Drain one experience level (minimum 1)
                player.exp_level -= 1;
                // Also reduce max HP slightly
                player.hp_max = (player.hp_max - rng.rnd(5) as i32).max(1);
                player.hp = player.hp.min(player.hp_max);
                Some(CombatEffect::Drained)
            } else {
                None
            }
        }

        DamageType::Stone => {
            // Stone resistance protects against petrification
            if player.properties.has(Property::StoneResistance) {
                None
            } else {
                // Petrification is usually instant death if not resisted
                Some(CombatEffect::Petrifying)
            }
        }

        DamageType::DrainStrength => {
            // Poison resistance protects against strength drain
            if player.properties.has(Property::PoisonResistance) {
                None
            } else {
                // Drain 1 point of strength
                let current_str = player.attr_current.get(Attribute::Strength);
                if current_str > 3 {
                    player.attr_current.modify(Attribute::Strength, -1);
                    Some(CombatEffect::Poisoned)
                } else {
                    None
                }
            }
        }

        DamageType::DrainDexterity => {
            // Poison resistance protects against dexterity drain
            if player.properties.has(Property::PoisonResistance) {
                None
            } else {
                let current_dex = player.attr_current.get(Attribute::Dexterity);
                if current_dex > 3 {
                    player.attr_current.modify(Attribute::Dexterity, -1);
                    Some(CombatEffect::Poisoned)
                } else {
                    None
                }
            }
        }

        DamageType::DrainConstitution => {
            // Poison resistance protects against constitution drain
            if player.properties.has(Property::PoisonResistance) {
                None
            } else {
                let current_con = player.attr_current.get(Attribute::Constitution);
                if current_con > 3 {
                    player.attr_current.modify(Attribute::Constitution, -1);
                    Some(CombatEffect::Poisoned)
                } else {
                    None
                }
            }
        }

        DamageType::Disease => {
            // Sick resistance protects against disease
            if player.properties.has(Property::SickResistance) {
                None
            } else {
                // Apply sickness - drain constitution over time
                let current_con = player.attr_current.get(Attribute::Constitution);
                if current_con > 3 {
                    player.attr_current.modify(Attribute::Constitution, -1);
                }
                Some(CombatEffect::Diseased)
            }
        }

        DamageType::Acid => {
            // Acid resistance negates acid damage effects
            if player.properties.has(Property::AcidResistance) {
                None
            } else {
                // Corrode armor - reduce AC temporarily
                // In real NetHack this would erode specific armor pieces
                if rng.one_in(3) {
                    player.armor_class = player.armor_class.saturating_add(1);
                    Some(CombatEffect::ArmorCorroded)
                } else {
                    None
                }
            }
        }

        DamageType::Disintegrate => {
            // Disintegration resistance protects completely
            if player.properties.has(Property::DisintResistance) {
                None
            } else {
                // Disintegration is usually instant death
                Some(CombatEffect::Petrifying) // Reusing for instant death effect
            }
        }

        DamageType::Confuse => {
            // No direct resistance, but half spell damage might help
            let duration = rng.rnd(10) as u16 + 10;
            player.confused_timeout = player.confused_timeout.saturating_add(duration);
            Some(CombatEffect::Confused)
        }

        DamageType::Stun => {
            // Stun player for 5-9 turns
            let duration = rng.rnd(5) as u16 + 5;
            player.stunned_timeout = player.stunned_timeout.saturating_add(duration);
            Some(CombatEffect::Stunned)
        }

        DamageType::Blind => {
            // Blind player for 20-119 turns
            let duration = rng.rnd(100) as u16 + 20;
            player.blinded_timeout = player.blinded_timeout.saturating_add(duration);
            Some(CombatEffect::Blinded)
        }

        DamageType::Paralyze => {
            // Free action protects against paralysis
            if player.properties.has(Property::FreeAction) {
                None
            } else {
                // Paralyze player for 3-7 turns
                let duration = rng.rnd(5) as u16 + 3;
                player.paralyzed_timeout = player.paralyzed_timeout.saturating_add(duration);
                Some(CombatEffect::Paralyzed)
            }
        }

        DamageType::StealGold => {
            // Steal some gold (10-50%)
            if player.gold > 0 {
                let steal_percent = rng.rnd(40) as i32 + 10;
                let stolen = (player.gold * steal_percent) / 100;
                player.gold -= stolen.max(1);
                Some(CombatEffect::GoldStolen)
            } else {
                None
            }
        }

        DamageType::StealItem => {
            // TODO: Actually steal item from inventory
            // This requires inventory access which we don't have here
            Some(CombatEffect::ItemStolen)
        }

        DamageType::Teleport => {
            // TODO: Actually teleport player
            // This requires level access which we don't have here
            Some(CombatEffect::Teleported)
        }

        DamageType::Digest => {
            player.swallowed = true;
            Some(CombatEffect::Engulfed)
        }

        DamageType::Wrap | DamageType::Stick => {
            // TODO: Set grabbed_by to attacker's MonsterId
            // This requires attacker info which we don't have here
            Some(CombatEffect::Grabbed)
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::MonsterId;
    use crate::player::Attribute;

    fn test_player() -> You {
        let mut player = You::default();
        player.attr_current.set(Attribute::Dexterity, 10); // Neutral AC bonus
        player
    }

    fn test_monster(level: u8) -> Monster {
        let mut monster = Monster::new(MonsterId(1), level as i16, 5, 5);
        monster.level = level;
        monster
    }

    #[test]
    fn test_monster_to_hit_base() {
        let player = test_player();
        let monster = test_monster(5);

        // Level 5 monster has to-hit of 5
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_monster_to_hit_high_level() {
        let player = test_player();
        let monster = test_monster(15);

        // Level 15 monster has to-hit of 15
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 15);
    }

    #[test]
    fn test_monster_confused_penalty() {
        let player = test_player();
        let mut monster = test_monster(5);
        monster.state.confused = true;

        // Level 5 monster confused: 5 - 2 = 3
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 3);
    }

    #[test]
    fn test_monster_vs_stunned_player() {
        let mut player = test_player();
        player.stunned_timeout = 10;
        let monster = test_monster(5);

        // Level 5 monster vs stunned player: 5 + 2 = 7
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 7);
    }

    #[test]
    fn test_monster_vs_sleeping_player() {
        let mut player = test_player();
        player.sleeping_timeout = 10;
        let monster = test_monster(5);

        // Level 5 monster vs sleeping player: 5 + 4 = 9
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 9);
    }

    #[test]
    fn test_monster_attack_hits_with_ac() {
        let mut player = test_player();
        let monster = test_monster(10);
        let mut rng = GameRng::new(42);

        // Player with AC 10 (no armor)
        player.armor_class = 10;

        let attack = Attack::new(
            crate::combat::AttackType::Claw,
            DamageType::Physical,
            1,
            6,
        );

        // Level 10 monster vs AC 10 player
        // Roll + 10 > 10 - 10 = 0, so need roll > -10, always hits
        let mut hits = 0;
        for _ in 0..100 {
            player.hp = 100;
            let result = monster_attack_player(&monster, &mut player, &attack, &mut rng);
            if result.hit {
                hits += 1;
            }
        }
        assert_eq!(hits, 100, "Level 10 monster should always hit AC 10");
    }

    #[test]
    fn test_monster_attack_misses_good_ac() {
        let mut player = test_player();
        let monster = test_monster(1);
        let mut rng = GameRng::new(42);

        // Player with AC -10 (very good armor)
        player.armor_class = -10;

        let attack = Attack::new(
            crate::combat::AttackType::Claw,
            DamageType::Physical,
            1,
            6,
        );

        // Level 1 monster vs AC -10 player
        // Roll + 1 > 10 - (-10) = 20, so need roll > 19, only roll of 20 hits (5% chance)
        let mut hits = 0;
        for _ in 0..1000 {
            player.hp = 100;
            let result = monster_attack_player(&monster, &mut player, &attack, &mut rng);
            if result.hit {
                hits += 1;
            }
        }
        // Should hit about 5% of the time (1 in 20)
        assert!(
            hits > 20 && hits < 100,
            "Level 1 monster vs AC -10 should hit about 5%, got {}",
            hits
        );
    }

    #[test]
    fn test_confuse_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.confused_timeout, 0);

        let effect = apply_damage_effect(DamageType::Confuse, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Confused));
        assert!(player.confused_timeout >= 10, "Should be confused for at least 10 turns");
        assert!(player.confused_timeout <= 19, "Should be confused for at most 19 turns");
    }

    #[test]
    fn test_stun_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.stunned_timeout, 0);

        let effect = apply_damage_effect(DamageType::Stun, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Stunned));
        assert!(player.stunned_timeout >= 5, "Should be stunned for at least 5 turns");
        assert!(player.stunned_timeout <= 9, "Should be stunned for at most 9 turns");
    }

    #[test]
    fn test_blind_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.blinded_timeout, 0);

        let effect = apply_damage_effect(DamageType::Blind, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Blinded));
        assert!(player.blinded_timeout >= 20, "Should be blinded for at least 20 turns");
        assert!(player.blinded_timeout <= 119, "Should be blinded for at most 119 turns");
    }

    #[test]
    fn test_paralyze_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.paralyzed_timeout, 0);

        let effect = apply_damage_effect(DamageType::Paralyze, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Paralyzed));
        assert!(player.paralyzed_timeout >= 3, "Should be paralyzed for at least 3 turns");
        assert!(player.paralyzed_timeout <= 7, "Should be paralyzed for at most 7 turns");
    }

    #[test]
    fn test_drain_life_effect() {
        let mut player = test_player();
        player.exp_level = 5;
        player.hp_max = 50;
        player.hp = 50;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainLife, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Drained));
        assert_eq!(player.exp_level, 4, "Should lose one experience level");
        assert!(player.hp_max < 50, "Max HP should be reduced");
    }

    #[test]
    fn test_drain_life_at_level_1() {
        let mut player = test_player();
        player.exp_level = 1;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainLife, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Should not drain below level 1");
        assert_eq!(player.exp_level, 1, "Should stay at level 1");
    }

    #[test]
    fn test_drain_strength_effect() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 16);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainStrength, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Poisoned));
        assert_eq!(player.attr_current.get(Attribute::Strength), 15, "Should lose 1 strength");
    }

    #[test]
    fn test_drain_strength_at_minimum() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 3);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainStrength, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Should not drain below 3 strength");
        assert_eq!(player.attr_current.get(Attribute::Strength), 3);
    }

    #[test]
    fn test_steal_gold_effect() {
        let mut player = test_player();
        player.gold = 1000;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::StealGold, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::GoldStolen));
        assert!(player.gold < 1000, "Should have lost some gold");
        assert!(player.gold >= 500, "Should have lost at most 50%");
    }

    #[test]
    fn test_steal_gold_no_gold() {
        let mut player = test_player();
        player.gold = 0;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::StealGold, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Should not steal if no gold");
    }

    #[test]
    fn test_engulf_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert!(!player.swallowed);

        let effect = apply_damage_effect(DamageType::Digest, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Engulfed));
        assert!(player.swallowed, "Player should be swallowed");
    }

    // Resistance tests
    use crate::player::Property;

    #[test]
    fn test_sleep_resistance() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::SleepResistance);
        let mut rng = GameRng::new(42);

        // Try many times - with resistance, should never sleep
        for _ in 0..100 {
            let effect = apply_damage_effect(DamageType::Sleep, &mut player, 0, &mut rng);
            assert_eq!(effect, None, "Sleep resistance should protect");
        }
        assert_eq!(player.sleeping_timeout, 0);
    }

    #[test]
    fn test_drain_resistance() {
        let mut player = test_player();
        player.exp_level = 5;
        player.properties.grant_intrinsic(Property::DrainResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainLife, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Drain resistance should protect");
        assert_eq!(player.exp_level, 5, "Level should not change");
    }

    #[test]
    fn test_stone_resistance() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::StoneResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Stone, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Stone resistance should protect from petrification");
    }

    #[test]
    fn test_poison_resistance_blocks_strength_drain() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 16);
        player.properties.grant_intrinsic(Property::PoisonResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainStrength, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Poison resistance should protect from strength drain");
        assert_eq!(player.attr_current.get(Attribute::Strength), 16, "Strength should not change");
    }

    #[test]
    fn test_free_action_blocks_paralysis() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::FreeAction);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Paralyze, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Free action should protect from paralysis");
        assert_eq!(player.paralyzed_timeout, 0);
    }

    #[test]
    fn test_disintegration_resistance() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::DisintResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Disintegrate, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Disintegration resistance should protect");
    }

    #[test]
    fn test_acid_resistance() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::AcidResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Acid, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Acid resistance should protect from acid effects");
    }

    // Damage reduction tests
    #[test]
    fn test_fire_resistance_reduces_damage() {
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Fire, &test_player());
        assert_eq!((mult_num, mult_den), (1, 1), "No resistance = full damage");

        let mut player = test_player();
        player.properties.grant_intrinsic(Property::FireResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Fire, &player);
        assert_eq!((mult_num, mult_den), (0, 1), "Fire resistance = no damage");
    }

    #[test]
    fn test_cold_resistance_reduces_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::ColdResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Cold, &player);
        assert_eq!((mult_num, mult_den), (0, 1), "Cold resistance = no damage");
    }

    #[test]
    fn test_shock_resistance_reduces_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::ShockResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Electric, &player);
        assert_eq!((mult_num, mult_den), (0, 1), "Shock resistance = no damage");
    }

    #[test]
    fn test_acid_resistance_halves_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::AcidResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Acid, &player);
        assert_eq!((mult_num, mult_den), (1, 2), "Acid resistance = half damage");
    }

    #[test]
    fn test_half_physical_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::HalfPhysDamage);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Physical, &player);
        assert_eq!((mult_num, mult_den), (1, 2), "Half physical damage property");
    }

    #[test]
    fn test_fire_attack_with_resistance() {
        let mut player = test_player();
        player.hp = 100;
        player.armor_class = 10;
        player.properties.grant_intrinsic(Property::FireResistance);
        let monster = test_monster(10);
        let mut rng = GameRng::new(42);

        let attack = Attack::new(
            crate::combat::AttackType::Breath,
            DamageType::Fire,
            3,
            6,
        );

        let result = monster_attack_player(&monster, &mut player, &attack, &mut rng);

        assert!(result.hit, "Should still hit");
        assert_eq!(result.damage, 0, "Fire damage should be reduced to 0");
        assert_eq!(player.hp, 100, "HP should not change with fire resistance");
    }
}
