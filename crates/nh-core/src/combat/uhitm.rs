//! Player attacks monster combat (uhitm.c)
//!
//! Handles all combat initiated by the player against monsters.

use super::CombatResult;
use crate::monster::Monster;
use crate::object::Object;
use crate::player::You;
use crate::rng::GameRng;

/// Calculate the player's to-hit bonus
///
/// Based on find_roll_to_hit() in uhitm.c
pub fn calculate_to_hit(player: &You, target: &Monster, weapon: Option<&Object>) -> i32 {
    let mut to_hit: i32 = 1; // base

    // Add player experience level
    to_hit += player.exp_level;

    // Add strength to-hit bonus
    to_hit += player.attr_current.strength_to_hit_bonus() as i32;

    // Add dexterity to-hit bonus
    to_hit += player.attr_current.dexterity_to_hit_bonus() as i32;

    // Add luck
    to_hit += player.luck as i32;

    // Add player's intrinsic hit bonus (from items, spells, etc.)
    to_hit += player.hit_bonus as i32;

    // Add weapon bonuses
    if let Some(w) = weapon {
        // Weapon enchantment adds to-hit
        to_hit += w.enchantment as i32;

        // Weapon's base to-hit bonus (from ObjClassDef.bonus)
        to_hit += w.weapon_tohit as i32;
    }

    // Target state modifiers (easier to hit disabled targets)
    if target.state.sleeping {
        to_hit += 2;
    }
    if target.state.stunned || target.state.confused || target.state.blinded || target.state.paralyzed {
        to_hit += 4;
    }
    if target.state.fleeing {
        to_hit += 2;
    }

    // Encumbrance penalty
    let encumbrance = player.encumbrance();
    match encumbrance {
        crate::player::Encumbrance::Burdened => to_hit -= 1,
        crate::player::Encumbrance::Stressed => to_hit -= 3,
        crate::player::Encumbrance::Strained => to_hit -= 5,
        crate::player::Encumbrance::Overtaxed => to_hit -= 7,
        crate::player::Encumbrance::Overloaded => to_hit -= 9,
        crate::player::Encumbrance::Unencumbered => {}
    }

    // Status effect penalties on attacker
    if player.is_confused() {
        to_hit -= 2;
    }
    if player.is_stunned() {
        to_hit -= 2;
    }
    if player.is_blind() {
        to_hit -= 2;
    }

    to_hit
}

/// Roll to hit a monster
///
/// Returns true if the attack hits
pub fn attack_hits(to_hit: i32, target_ac: i8, rng: &mut GameRng) -> bool {
    let roll = rng.rnd(20) as i32;
    roll + to_hit > 10 - target_ac as i32
}

/// Player melee attack against monster
pub fn player_attack_monster(
    player: &mut You,
    target: &mut Monster,
    weapon: Option<&Object>,
    rng: &mut GameRng,
) -> CombatResult {
    let to_hit = calculate_to_hit(player, target, weapon);

    // Get target AC from monster (set from PerMonst when monster is created)
    let target_ac = target.ac;

    if !attack_hits(to_hit, target_ac, rng) {
        return CombatResult::MISS;
    }

    // Calculate base damage from weapon or bare hands
    let base_damage = match weapon {
        Some(w) => {
            // Use weapon's damage dice fields
            // If not set (0), default to 1d6
            let dice_num = if w.damage_dice > 0 { w.damage_dice } else { 1 };
            let dice_sides = if w.damage_sides > 0 { w.damage_sides } else { 6 };
            rng.dice(dice_num as u32, dice_sides as u32) as i32
        }
        None => {
            // Bare hands - Monks get better unarmed damage based on level
            if player.role == crate::player::Role::Monk {
                // Monks deal 1d(level/2 + 1) damage, minimum 1d2, maximum 1d16
                let sides = ((player.exp_level / 2) + 1).clamp(2, 16) as u32;
                rng.dice(1, sides) as i32
            } else {
                // Non-monks deal 1d2 bare-handed
                rng.dice(1, 2) as i32
            }
        }
    };

    // Apply damage modifiers
    let mut damage = base_damage;

    // Add strength damage bonus
    damage += player.attr_current.strength_damage_bonus() as i32;

    // Add weapon enchantment to damage
    if let Some(w) = weapon {
        damage += w.enchantment as i32;
    }

    // Add player's intrinsic damage bonus
    damage += player.damage_bonus as i32;

    // Ensure minimum 1 damage on a hit
    damage = damage.max(1);

    // Apply damage to monster
    target.hp -= damage;

    CombatResult {
        hit: true,
        defender_died: target.hp <= 0,
        attacker_died: false,
        damage,
        special_effect: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::MonsterId;
    use crate::player::Attribute;

    fn test_player() -> You {
        let mut player = You::default();
        // Set sensible defaults for testing
        player.exp_level = 1;
        player.attr_current.set(Attribute::Strength, 10);
        player.attr_current.set(Attribute::Dexterity, 10);
        player.luck = 0;
        player
    }

    fn test_monster() -> Monster {
        Monster::new(MonsterId(1), 10, 5, 5)
    }

    #[test]
    fn test_base_to_hit() {
        let player = test_player();
        let monster = test_monster();

        // Base to-hit for level 1 player with average stats: 1 + 1 (level) = 2
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 2);
    }

    #[test]
    fn test_to_hit_with_level() {
        let mut player = test_player();
        let monster = test_monster();

        player.exp_level = 10;
        // Base 1 + level 10 = 11
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 11);
    }

    #[test]
    fn test_to_hit_with_strength_bonus() {
        let mut player = test_player();
        let monster = test_monster();

        // Strength 17 gives +1 to-hit
        player.attr_current.set(Attribute::Strength, 17);
        // Base 1 + level 1 + str bonus 1 = 3
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 3);

        // Low strength (5) gives -2 to-hit
        player.attr_current.set(Attribute::Strength, 5);
        // Base 1 + level 1 + str bonus -2 = 0
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 0);
    }

    #[test]
    fn test_to_hit_with_dexterity_bonus() {
        let mut player = test_player();
        let monster = test_monster();

        // High dexterity (18) gives +3 to-hit
        player.attr_current.set(Attribute::Dexterity, 18);
        // Base 1 + level 1 + dex bonus 3 = 5
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_to_hit_with_luck() {
        let mut player = test_player();
        let monster = test_monster();

        player.luck = 5;
        // Base 1 + level 1 + luck 5 = 7
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 7);
    }

    #[test]
    fn test_to_hit_vs_sleeping_monster() {
        let player = test_player();
        let mut monster = test_monster();

        monster.state.sleeping = true;
        // Base 2 + sleeping bonus 2 = 4
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 4);
    }

    #[test]
    fn test_to_hit_vs_stunned_monster() {
        let player = test_player();
        let mut monster = test_monster();

        monster.state.stunned = true;
        // Base 2 + stunned bonus 4 = 6
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 6);
    }

    #[test]
    fn test_to_hit_vs_fleeing_monster() {
        let player = test_player();
        let mut monster = test_monster();

        monster.state.fleeing = true;
        // Base 2 + fleeing bonus 2 = 4
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 4);
    }

    #[test]
    fn test_to_hit_confused_player() {
        let mut player = test_player();
        let monster = test_monster();

        player.confused_timeout = 10;
        // Base 2 - confused penalty 2 = 0
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 0);
    }

    #[test]
    fn test_to_hit_with_enchanted_weapon() {
        let player = test_player();
        let monster = test_monster();

        let mut weapon = Object::new(crate::object::ObjectId(1), 0, crate::object::ObjectClass::Weapon);
        weapon.enchantment = 3;

        // Base 2 + weapon enchant 3 = 5
        let to_hit = calculate_to_hit(&player, &monster, Some(&weapon));
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_attack_hits_mechanics() {
        let mut rng = GameRng::new(42);

        // With high to-hit, should hit AC 10 most of the time
        let high_to_hit = 10;
        let mut hits = 0;
        for _ in 0..100 {
            if attack_hits(high_to_hit, 10, &mut rng) {
                hits += 1;
            }
        }
        // 10 + roll > 10 - 10 = 0, so need roll > -10, which is always true
        // Actually: roll + 10 > 0, roll is 1-20, so always hits
        assert_eq!(hits, 100);

        // With low to-hit against good AC, should hit less often
        let low_to_hit = -5;
        hits = 0;
        for _ in 0..100 {
            if attack_hits(low_to_hit, -5, &mut rng) {
                hits += 1;
            }
        }
        // roll - 5 > 10 - (-5) = 15, so need roll > 20, which never happens
        // Actually: roll - 5 > 15, so roll > 20, impossible with d20
        assert_eq!(hits, 0);
    }

    #[test]
    fn test_damage_with_strength_bonus() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100; // High HP to not die
        let mut rng = GameRng::new(42);

        // Strength 17 gives +2 damage
        player.attr_current.set(Attribute::Strength, 17);

        // Run several attacks and check damage
        let mut total_damage = 0;
        for _ in 0..100 {
            monster.hp = 100;
            let result = player_attack_monster(&mut player, &mut monster, None, &mut rng);
            if result.hit {
                // Bare hands 1d2 + str bonus 2 = 3-4 damage
                assert!(
                    result.damage >= 3 && result.damage <= 4,
                    "Damage {} not in expected range 3-4 for str 17 unarmed",
                    result.damage
                );
                total_damage += result.damage;
            }
        }

        // Should have hit at least some times
        assert!(total_damage > 0, "Should have dealt some damage");
    }

    #[test]
    fn test_damage_with_weapon() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        let mut rng = GameRng::new(42);

        // Create a weapon: long sword (1d8 damage), +2 enchantment
        let mut weapon = Object::new(crate::object::ObjectId(1), 0, crate::object::ObjectClass::Weapon);
        weapon.damage_dice = 1;
        weapon.damage_sides = 8;
        weapon.enchantment = 2;

        // With str 10 (0 bonus), enchant +2: damage = 1d8 + 0 + 2 = 3-10
        for _ in 0..100 {
            monster.hp = 100;
            let result = player_attack_monster(&mut player, &mut monster, Some(&weapon), &mut rng);
            if result.hit {
                assert!(
                    result.damage >= 3 && result.damage <= 10,
                    "Damage {} not in expected range 3-10 for 1d8+2 weapon",
                    result.damage
                );
            }
        }
    }

    #[test]
    fn test_damage_minimum() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        let mut rng = GameRng::new(42);

        // Set low strength for negative bonus
        player.attr_current.set(Attribute::Strength, 5); // -1 damage bonus

        // Bare hands 1d2 - 1 = 0-1, but minimum is 1
        for _ in 0..100 {
            monster.hp = 100;
            let result = player_attack_monster(&mut player, &mut monster, None, &mut rng);
            if result.hit {
                assert!(
                    result.damage >= 1,
                    "Damage {} should be at least 1",
                    result.damage
                );
            }
        }
    }

    #[test]
    fn test_to_hit_with_weapon_bonus() {
        let player = test_player();
        let monster = test_monster();

        // Create a weapon with base to-hit bonus
        let mut weapon = Object::new(crate::object::ObjectId(1), 0, crate::object::ObjectClass::Weapon);
        weapon.weapon_tohit = 2;
        weapon.enchantment = 1;

        // Base 2 + weapon to-hit 2 + enchant 1 = 5
        let to_hit = calculate_to_hit(&player, &monster, Some(&weapon));
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_player_vs_monster_ac() {
        let player = test_player();
        let mut rng = GameRng::new(42);

        // Monster with good AC (low is better)
        let mut monster_good_ac = test_monster();
        monster_good_ac.ac = -5;
        monster_good_ac.hp = 100;

        // Monster with poor AC
        let mut monster_poor_ac = test_monster();
        monster_poor_ac.ac = 10;
        monster_poor_ac.hp = 100;

        // Count hits against each
        let mut hits_good_ac = 0;
        let mut hits_poor_ac = 0;

        for _ in 0..1000 {
            monster_good_ac.hp = 100;
            monster_poor_ac.hp = 100;

            let result = player_attack_monster(&mut player.clone(), &mut monster_good_ac, None, &mut rng);
            if result.hit {
                hits_good_ac += 1;
            }

            let result = player_attack_monster(&mut player.clone(), &mut monster_poor_ac, None, &mut rng);
            if result.hit {
                hits_poor_ac += 1;
            }
        }

        // Should hit the poor AC monster more often
        assert!(
            hits_poor_ac > hits_good_ac,
            "Should hit AC 10 more than AC -5: {} vs {}",
            hits_poor_ac,
            hits_good_ac
        );

        // Level 1 player vs AC -5: need roll + 2 > 10 - (-5) = 15, so roll > 13 (35% chance)
        // Level 1 player vs AC 10: need roll + 2 > 10 - 10 = 0, always hits
        assert_eq!(hits_poor_ac, 1000, "Should always hit AC 10");
    }
}
