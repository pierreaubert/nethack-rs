//! Player attacks monster combat (uhitm.c)
//!
//! Handles all combat initiated by the player against monsters.
//! Includes weapon damage calculation, silver/blessed bonuses,
//! artifact integration, erosion, two-weapon fighting, and cleaving.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use super::artifact::{artifact_for_object, artifact_hit, spec_abon, spec_dbon, Artifact};
use super::{
    AmmunitionCount, ArmorProficiency, ArmorType, CombatEffect, CombatEncounter, CombatModifier,
    CombatResult, CombatSpell, CriticalHitType, DamageType, DefenseCalculation, DifficultyRating,
    DodgeSkill, EncounterState, Formation, GroupTactic, LootDrop, LootGenerator, RangedAttack,
    RangedCombatResult, RangedWeaponType, SkillLevel, SpecialCombatEffect, SpellCastResult,
    StatusEffect, StatusEffectTracker, TreasureHoard, WeaponSkill, apply_armor_penetration,
    apply_combat_modifiers, apply_damage_reduction, apply_encounter_modifiers,
    apply_special_effect, apply_status_effect, attempt_dodge, award_loot_to_player,
    award_monster_xp, award_player_xp, calculate_armor_damage_reduction,
    calculate_attribute_damage_bonus, calculate_encounter_xp, calculate_monster_xp_reward,
    calculate_skill_enhanced_damage, calculate_skill_enhanced_to_hit, calculate_status_damage,
    can_player_cast_spell, cast_combat_spell, check_flanking, determine_critical_hit,
    effect_severity_from_skill, execute_ranged_attack, flanking_damage_bonus, roll_special_effect,
    select_monster_target, should_trigger_special_effect, weapon_vs_armor_bonus,
};
use crate::monster::{Monster, MonsterFlags};
use crate::object::{Material, Object};
use crate::player::{AlignmentType, You};
use crate::rng::GameRng;

// ============================================================================
// Attack source flags (hmon type parameter)
// ============================================================================

/// How the attack was delivered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackSource {
    /// Melee (hand-to-hand)
    Melee,
    /// Thrown projectile
    Thrown,
    /// Applied (polearm, etc.)
    Applied,
    /// Kicked
    Kicked,
}

// ============================================================================
// Full hit result with messages
// ============================================================================

/// Full result of a combat hit, including messages and effects
#[derive(Debug, Clone)]
pub struct HitResult {
    /// Whether the attack connected
    pub hit: bool,
    /// Whether the defender died
    pub defender_died: bool,
    /// Whether the attacker died
    pub attacker_died: bool,
    /// Total damage dealt
    pub damage: i32,
    /// Messages to display
    pub messages: Vec<String>,
    /// Special effects triggered
    pub effects: Vec<CombatEffect>,
    /// Whether artifact gave a special message
    pub artifact_messaged: bool,
}

impl HitResult {
    pub fn miss() -> Self {
        Self {
            hit: false,
            defender_died: false,
            attacker_died: false,
            damage: 0,
            messages: Vec::new(),
            effects: Vec::new(),
            artifact_messaged: false,
        }
    }

    /// Convert to simple CombatResult
    pub fn to_combat_result(&self) -> CombatResult {
        CombatResult {
            hit: self.hit,
            defender_died: self.defender_died,
            attacker_died: self.attacker_died,
            damage: self.damage,
            special_effect: self.effects.first().copied(),
        }
    }
}

// ============================================================================
// Erosion helpers
// ============================================================================

/// Get the greatest erosion level on an object (max of rust/corrosion)
///
/// Based on greatest_erosion() macro in obj.h
pub fn greatest_erosion(obj: &Object) -> u8 {
    obj.erosion1.max(obj.erosion2)
}

/// Maximum erosion level
pub const MAX_ERODE: u8 = 3;

/// Try to erode a weapon from combat.
///
/// Returns true if the weapon was further eroded.
/// erosion_type: 0 = rust, 1 = corrode/acid
pub fn maybe_erode_weapon(weapon: &mut Object, erosion_type: u8, rng: &mut GameRng) -> bool {
    if weapon.erosion_proof || weapon.greased {
        return false;
    }

    // 1 in 10 chance per hit
    if rng.rn2(10) != 0 {
        return false;
    }

    let erosion = if erosion_type == 0 {
        &mut weapon.erosion1
    } else {
        &mut weapon.erosion2
    };

    if *erosion < MAX_ERODE {
        *erosion += 1;
        true
    } else {
        false
    }
}

// ============================================================================
// Silver damage
// ============================================================================

/// Check if a monster hates silver (takes extra damage from silver weapons).
///
/// Based on mon_hates_silver() / hates_silver() in mondata.c.
/// Werewolves, vampires, demons, shades, and imps are vulnerable.
pub fn mon_hates_silver(target: &Monster) -> bool {
    // Werewolves
    if target.flags.contains(MonsterFlags::WERE) {
        return true;
    }
    // Demons
    if target.flags.contains(MonsterFlags::DEMON) {
        return true;
    }
    // Undead (includes vampires and shades in NetHack's classification)
    if target.flags.contains(MonsterFlags::UNDEAD) {
        return true;
    }
    false
}

/// Calculate silver damage bonus against a monster.
///
/// Returns bonus damage (0 if target is not silver-vulnerable).
/// Based on silver damage logic in hmon_hitmon() and dmgval().
pub fn silver_damage(target: &Monster, rng: &mut GameRng) -> i32 {
    if mon_hates_silver(target) {
        rng.rnd(20) as i32
    } else {
        0
    }
}

// ============================================================================
// Blessed vs undead/demon
// ============================================================================

/// Calculate blessed weapon damage bonus vs undead/demon.
///
/// Based on dmgval() bonus logic in weapon.c.
/// Blessed weapons deal +1d4 vs undead and demons.
/// Cursed weapons deal no bonus.
pub fn buc_damage_bonus(weapon: &Object, target: &Monster, rng: &mut GameRng) -> i32 {
    if weapon.is_blessed() && (target.is_undead() || target.is_demon()) {
        rng.rnd(4) as i32
    } else {
        0
    }
}

// ============================================================================
// Weapon damage calculation (dmgval)
// ============================================================================

/// Calculate base weapon damage against a target monster.
///
/// Based on dmgval() in weapon.c.
/// Includes base dice, enchantment, erosion penalty, blessed bonus,
/// and silver bonus. Does NOT include strength, skill, or artifact bonuses.
///
/// Parameters:
/// - `weapon`: The weapon being used
/// - `weapon_material`: Material of the weapon (from ObjClassDef)
/// - `target`: The target monster
/// - `is_large`: Whether to use large monster damage (based on MonsterSize)
/// - `rng`: Random number generator
pub fn dmgval(
    weapon: &Object,
    weapon_material: Material,
    target: &Monster,
    _is_large: bool,
    rng: &mut GameRng,
) -> i32 {
    // Base weapon damage from dice
    let dice_num = if weapon.damage_dice > 0 {
        weapon.damage_dice
    } else {
        1
    };
    let dice_sides = if weapon.damage_sides > 0 {
        weapon.damage_sides
    } else {
        6
    };

    let mut tmp = rng.dice(dice_num as u32, dice_sides as u32) as i32;

    // Enchantment bonus (only positive enchantment for damage)
    if weapon.enchantment > 0 {
        tmp += weapon.enchantment as i32;
    }

    // Type-specific bonuses
    let mut bonus = 0;

    // Blessed vs undead/demon: +1d4
    if weapon.is_blessed() && (target.is_undead() || target.is_demon()) {
        bonus += rng.rnd(4) as i32;
    }

    // Silver weapon vs silver-hating monster: +1d20
    if weapon_material == Material::Silver && mon_hates_silver(target) {
        bonus += rng.rnd(20) as i32;
    }

    tmp += bonus;

    // Erosion penalty
    let erosion = greatest_erosion(weapon) as i32;
    tmp -= erosion;

    // Thick-skinned resistance: leather/soft materials do nothing
    if target.flags.contains(MonsterFlags::THICK_HIDE) && !weapon_material.is_metallic() {
        // Only block if material is softer than leather
        match weapon_material {
            Material::Leather | Material::Cloth | Material::Paper | Material::Veggy => {
                tmp = 0;
            }
            _ => {}
        }
    }

    // Minimum 1 damage on a hit
    tmp.max(1)
}

/// Calculate bare-hand damage.
///
/// Monks get enhanced unarmed damage based on level.
pub fn bare_hand_damage(player: &You, rng: &mut GameRng) -> i32 {
    if player.role == crate::player::Role::Monk {
        let sides = ((player.exp_level / 2) + 1).clamp(2, 16) as u32;
        rng.dice(1, sides) as i32
    } else {
        rng.dice(1, 2) as i32
    }
}

// ============================================================================
// To-hit calculation
// ============================================================================

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
    if target.state.stunned
        || target.state.confused
        || target.state.blinded
        || target.state.paralyzed
    {
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

// ============================================================================
// Full hit resolution (hmon)
// ============================================================================

/// Full melee hit resolution with all damage modifiers and effects.
///
/// Based on hmon_hitmon() in uhitm.c.
/// Calculates weapon damage, applies enchantment, strength, blessed/silver
/// bonuses, artifact effects, erosion penalties, and poison.
///
/// This is the comprehensive version that produces messages.
#[allow(clippy::too_many_arguments)]
pub fn hmon(
    player: &mut You,
    target: &mut Monster,
    mut weapon: Option<&mut Object>,
    weapon_material: Option<Material>,
    source: AttackSource,
    dieroll: i32,
    artifacts: &[Artifact],
    rng: &mut GameRng,
) -> HitResult {
    let mut result = HitResult {
        hit: true,
        defender_died: false,
        attacker_died: false,
        damage: 0,
        messages: Vec::new(),
        effects: Vec::new(),
        artifact_messaged: false,
    };

    let mut damage: i32;
    let mut is_silver = false;

    match weapon {
        Some(ref w) => {
            let mat = weapon_material.unwrap_or(Material::Iron);

            if source == AttackSource::Melee {
                // Full melee weapon damage
                damage = dmgval(w, mat, target, target_is_large(target), rng);
            } else {
                // Thrown/applied: simpler damage
                damage = rng.rnd(2) as i32;
                // Silver projectile bonus
                if mat == Material::Silver && mon_hates_silver(target) {
                    let silver_bonus = if damage > 0 {
                        rng.rnd(20) as i32
                    } else {
                        rng.rnd(10) as i32
                    };
                    damage += silver_bonus;
                    is_silver = true;
                }
            }

            // Artifact damage integration
            if w.artifact != 0
                && let Some(art) = artifact_for_object(w, artifacts)
            {
                // Artifact to-hit bonus is already included in calculate_to_hit
                let (spec_bonus, _applies) = spec_dbon(w, art, target, damage, rng);
                damage += spec_bonus;

                // Artifact special effects (beheading, drain, elemental)
                let art_result =
                    artifact_hit(w, target, &mut damage, dieroll, artifacts, rng);
                if art_result.had_effect {
                    result.artifact_messaged = true;
                    result.messages.extend(art_result.messages);
                    result.effects.extend(art_result.effects);
                    if art_result.instant_kill {
                        result.defender_died = true;
                    }
                }
            }

            // Silver melee weapon message
            if source == AttackSource::Melee
                && mat == Material::Silver
                && mon_hates_silver(target)
                && !is_silver
            {
                result.messages.push(format!(
                    "Your silver weapon sears {}'s flesh!",
                    target.name
                ));
            }

            // Poison check
            if w.poisoned && source != AttackSource::Kicked && !target.resists_poison() {
                // 1/10 chance of instakill, otherwise 1d6 extra
                if rng.rn2(10) == 0 {
                    damage = target.hp + 200; // Fatal
                    result
                        .messages
                        .push(format!("The poison was deadly for {}!", target.name));
                    result.effects.push(CombatEffect::Poisoned);
                } else {
                    let poison_dmg = rng.rnd(6) as i32;
                    damage += poison_dmg;
                    result.effects.push(CombatEffect::Poisoned);
                }
            }
        }
        None => {
            // Bare-hand attack
            damage = bare_hand_damage(player, rng);
        }
    }

    // Strength damage bonus (melee and thrown)
    if source == AttackSource::Melee || source == AttackSource::Thrown {
        damage += player.attr_current.strength_damage_bonus() as i32;
    }

    // Player's intrinsic damage bonus
    damage += player.damage_bonus as i32;

    // Weapon enchantment to damage (for non-dmgval paths like thrown)
    if source != AttackSource::Melee
        && let Some(ref w) = weapon
        && w.enchantment > 0
    {
        damage += w.enchantment as i32;
    }

    // Minimum 1 damage on a hit
    damage = damage.max(1);

    // Apply damage to monster
    target.hp -= damage;
    result.damage = damage;

    if target.hp <= 0 {
        result.defender_died = true;
    } else {
        // Fleeing check: monsters may flee when badly wounded
        if !target.state.fleeing && target.hp < target.hp_max / 4 && rng.rn2(4) == 0 {
            target.state.fleeing = true;
            target.flee_timeout = rng.rnd(10) as u16 + 5;
        }

        // Wake sleeping monster
        if target.state.sleeping {
            target.state.sleeping = false;
            target.sleep_timeout = 0;
        }
    }

    // Weapon erosion from acid monsters
    if target.resists_acid()
        && source == AttackSource::Melee
        && let Some(ref mut w) = weapon
        && maybe_erode_weapon(w, 1, rng)
    {
        result
            .messages
            .push("Your weapon is corroded by acid!".to_string());
    }

    result
}

/// Check if a monster should use large-monster damage dice.
///
/// Based on bigmonst() macro.
fn target_is_large(target: &Monster) -> bool {
    // Simplified: use monster level as proxy for size
    // In full implementation, look up PerMonst.size >= Large
    target.level >= 6
}

// ============================================================================
// Two-weapon fighting
// ============================================================================

/// Resolve a two-weapon fighting attack (primary + secondary).
///
/// Based on hitum() two-weapon logic in uhitm.c.
/// Attacks with primary weapon first, then secondary if primary hits.
#[allow(clippy::too_many_arguments)]
pub fn two_weapon_hit(
    player: &mut You,
    target: &mut Monster,
    primary: &mut Object,
    primary_material: Material,
    secondary: &mut Object,
    secondary_material: Material,
    artifacts: &[Artifact],
    rng: &mut GameRng,
) -> (HitResult, Option<HitResult>) {
    let to_hit = calculate_to_hit(player, target, Some(primary));
    let target_ac = target.ac;

    if !attack_hits(to_hit, target_ac, rng) {
        return (HitResult::miss(), None);
    }

    let dieroll = rng.rnd(20) as i32;

    // Primary weapon attack
    let primary_result = hmon(
        player,
        target,
        Some(primary),
        Some(primary_material),
        AttackSource::Melee,
        dieroll,
        artifacts,
        rng,
    );

    // Secondary only if primary hit AND target still alive AND still adjacent
    if !primary_result.hit || primary_result.defender_died {
        return (primary_result, None);
    }

    // Secondary weapon to-hit (recalculate with off-hand penalties)
    let mut sec_to_hit = calculate_to_hit(player, target, Some(secondary));
    sec_to_hit -= 2; // Off-hand penalty

    if !attack_hits(sec_to_hit, target_ac, rng) {
        return (primary_result, Some(HitResult::miss()));
    }

    let sec_dieroll = rng.rnd(20) as i32;

    let secondary_result = hmon(
        player,
        target,
        Some(secondary),
        Some(secondary_material),
        AttackSource::Melee,
        sec_dieroll,
        artifacts,
        rng,
    );

    (primary_result, Some(secondary_result))
}

// ============================================================================
// Cleaving (Cleaver artifact)
// ============================================================================

/// Direction offsets for 8 directions (N, NE, E, SE, S, SW, W, NW)
const DIR_X: [i8; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
const DIR_Y: [i8; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];

/// Get direction index (0-7) from dx, dy
fn direction_index(dx: i8, dy: i8) -> usize {
    match (dx, dy) {
        (0, -1) => 0,  // N
        (1, -1) => 1,  // NE
        (1, 0) => 2,   // E
        (1, 1) => 3,   // SE
        (0, 1) => 4,   // S
        (-1, 1) => 5,  // SW
        (-1, 0) => 6,  // W
        (-1, -1) => 7, // NW
        _ => 0,
    }
}

/// Position + direction info for cleave targets
pub struct CleaveTarget {
    pub x: i8,
    pub y: i8,
    pub dir_idx: usize,
}

/// Calculate three cleave target positions (left, center, right).
///
/// Based on hitum_cleave() in uhitm.c.
/// The Cleaver artifact hits in 3 adjacent directions, rotating
/// clockwise or counterclockwise based on a counter.
pub fn cleave_targets(
    player_x: i8,
    player_y: i8,
    target_x: i8,
    target_y: i8,
    clockwise: bool,
) -> [CleaveTarget; 3] {
    let dx = (target_x - player_x).signum();
    let dy = (target_y - player_y).signum();
    let center_dir = direction_index(dx, dy);

    let (left_dir, right_dir) = if clockwise {
        (
            (center_dir + 7) % 8, // -1 mod 8
            (center_dir + 1) % 8,
        )
    } else {
        (
            (center_dir + 1) % 8,
            (center_dir + 7) % 8,
        )
    };

    [
        CleaveTarget {
            x: player_x + DIR_X[left_dir],
            y: player_y + DIR_Y[left_dir],
            dir_idx: left_dir,
        },
        CleaveTarget {
            x: target_x,
            y: target_y,
            dir_idx: center_dir,
        },
        CleaveTarget {
            x: player_x + DIR_X[right_dir],
            y: player_y + DIR_Y[right_dir],
            dir_idx: right_dir,
        },
    ]
}

// ============================================================================
// Creature vulnerability
// ============================================================================

/// Vulnerability multiplier for a monster against a damage type.
///
/// Returns a multiplier (1.0 = normal, 2.0 = double, 0.0 = immune).
pub fn creature_vulnerability(
    target: &Monster,
    weapon_material: Material,
) -> f32 {
    let mut mult = 1.0;

    // Silver vulnerability
    if weapon_material == Material::Silver && mon_hates_silver(target) {
        mult += 0.5; // +50% (silver bonus is already calculated separately as flat damage)
    }

    // Thick hide reduces damage from soft materials
    if target.flags.contains(MonsterFlags::THICK_HIDE) && !weapon_material.is_metallic() {
        mult *= 0.5;
    }

    mult
}

// ============================================================================
// Special weapon effects
// ============================================================================

/// Determine special effects from a weapon hit.
///
/// Returns list of combat effects triggered by the weapon.
pub fn special_weapon_effects(
    weapon: &Object,
    weapon_material: Material,
    target: &Monster,
) -> Vec<CombatEffect> {
    let mut effects = Vec::new();

    // Poisoned weapon
    if weapon.poisoned {
        effects.push(CombatEffect::Poisoned);
    }

    // Silver searing
    if weapon_material == Material::Silver && mon_hates_silver(target) {
        // Silver causes extra pain (tracked as damage, not as separate effect)
    }

    // Acid resistance on target can corrode attacker's weapon (ArmorCorroded used here)
    if target.resists_acid() {
        effects.push(CombatEffect::ArmorCorroded);
    }

    effects
}

// ============================================================================
// Artifact to-hit bonus
// ============================================================================

/// Get artifact-based to-hit bonus.
///
/// Wraps spec_abon from artifact.rs.
pub fn artifact_to_hit_bonus(
    weapon: &Object,
    target: &Monster,
    artifacts: &[Artifact],
    rng: &mut GameRng,
) -> i32 {
    if weapon.artifact == 0 {
        return 0;
    }
    if let Some(art) = artifact_for_object(weapon, artifacts) {
        spec_abon(art, target, rng)
    } else {
        0
    }
}

// ============================================================================
// Simple attack (legacy interface)
// ============================================================================

/// Player melee attack against monster (simple interface).
///
/// This is the original simplified interface that doesn't use artifacts
/// or generate messages. Use `hmon()` for the full combat system.
pub fn player_attack_monster(
    player: &mut You,
    target: &mut Monster,
    weapon: Option<&Object>,
    rng: &mut GameRng,
) -> CombatResult {
    // Route through hmonas for polymorphed unarmed attacks
    if player.monster_num.is_some() && weapon.is_none() {
        return hmonas(player, target, rng);
    }

    // Phase 13: Check if player is incapacitated by status effects
    if player.status_effects.is_incapacitated() {
        // Apply passive damage from status effects to player
        let status_damage = calculate_status_damage(&player.status_effects);
        player.hp = (player.hp - status_damage).max(0);
        return CombatResult::MISS;
    }

    // Determine weapon skill type being used
    let weapon_skill = get_weapon_skill(weapon);

    // Get player's skill level with this weapon type
    let skill_level = get_player_weapon_skill(player, weapon_skill);

    // Calculate base to-hit
    let base_to_hit = calculate_to_hit(player, target, weapon);

    // Phase 13: Apply status effect penalties to to-hit
    let status_to_hit_penalty = player.status_effects.attack_roll_penalty();
    let base_to_hit_with_effects = base_to_hit - status_to_hit_penalty;

    // Enhance to-hit with skill level and armor penetration
    let armor_penetration = skill_level.armor_penetration();
    let enhanced_to_hit =
        calculate_skill_enhanced_to_hit(base_to_hit_with_effects, skill_level, armor_penetration);

    // Get target AC and apply armor penetration
    let target_ac = target.ac;
    let effective_ac = apply_armor_penetration(target_ac, armor_penetration);

    // Roll to hit with enhanced to-hit bonus
    if !attack_hits(enhanced_to_hit, effective_ac, rng) {
        // Record miss for weapon proficiency tracking
        update_weapon_proficiency(player, weapon_skill, false, false);
        return CombatResult::MISS;
    }

    // Roll for critical hit
    let base_roll = rng.rnd(20) as i32;
    let critical = determine_critical_hit(base_roll, skill_level, rng);

    // Calculate base damage from weapon or bare hands
    let base_damage = match weapon {
        Some(w) => {
            // Use weapon's damage dice fields
            // If not set (0), default to 1d6
            let dice_num = if w.damage_dice > 0 { w.damage_dice } else { 1 };
            let dice_sides = if w.damage_sides > 0 {
                w.damage_sides
            } else {
                6
            };
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

    // Apply attribute-based damage bonuses based on weapon type
    let strength = player.attr_current.get(crate::player::Attribute::Strength) as u8;
    let dexterity = player.attr_current.get(crate::player::Attribute::Dexterity) as u8;
    let attr_bonus = calculate_attribute_damage_bonus(strength, dexterity, weapon_skill);

    let mut damage = base_damage + attr_bonus;

    // Add weapon enchantment to damage
    if let Some(w) = weapon {
        damage += w.enchantment as i32;
    }

    // Add weapon vs armor effectiveness bonus
    let armor_bonus = weapon_vs_armor_bonus(weapon_skill, target.ac);
    damage += armor_bonus;

    // Add player's intrinsic damage bonus
    damage += player.damage_bonus as i32;

    // Apply skill-enhanced damage with critical multiplier
    damage = calculate_skill_enhanced_damage(damage, skill_level, critical);

    // Ensure minimum 1 damage on a hit
    damage = damage.max(1);

    // Handle instant kill
    let target_died = if critical == CriticalHitType::InstantKill {
        target.hp = 0;
        true
    } else {
        target.hp -= damage;
        target.hp <= 0
    };

    // Determine special combat effects
    let mut special_effect = None;

    // Phase 13: On critical hit, potentially trigger status effects
    if critical.is_critical() && skill_level as u8 >= SkillLevel::Skilled as u8 {
        let effect_severity = effect_severity_from_skill(&skill_level);

        // Try to trigger stun effect
        if should_trigger_special_effect(&SpecialCombatEffect::Stun, &skill_level, rng) {
            apply_special_effect(
                &SpecialCombatEffect::Stun,
                &mut target.status_effects,
                "player critical hit",
                effect_severity,
            );
            special_effect = Some(super::CombatEffect::Stunned);
            target.state.stunned = true;
        }

        // Try to trigger trip effect
        if should_trigger_special_effect(&SpecialCombatEffect::Trip, &skill_level, rng) {
            apply_special_effect(
                &SpecialCombatEffect::Trip,
                &mut target.status_effects,
                "player trip",
                effect_severity,
            );
        }

        // Try to trigger disarm effect
        if should_trigger_special_effect(&SpecialCombatEffect::Disarm, &skill_level, rng) {
            apply_special_effect(
                &SpecialCombatEffect::Disarm,
                &mut target.status_effects,
                "player disarm",
                effect_severity,
            );
        }
    }

    // Phase 13: Apply passive damage to player from status effects
    let player_status_damage = calculate_status_damage(&player.status_effects);
    if player_status_damage > 0 {
        player.hp = (player.hp - player_status_damage).max(0);
    }

    // Update weapon proficiency based on this attack
    update_weapon_proficiency(player, weapon_skill, true, critical.is_critical());

    // Phase 18: Record player hit in monster's morale and threat assessment
    // Note: This is called before the monster actually dies, so morale is updated
    // before the monster is removed from the level
    if !target_died || damage > 0 {
        // We can't directly modify the Level here, so this will be called
        // from the caller (the combat coordinator)
        // This is just a marker that tells the caller to update the monster
    }

    CombatResult {
        hit: true,
        defender_died: target_died,
        attacker_died: false,
        damage,
        special_effect,
    }
}

// ============================================================================
// Phase 14: Experience & Leveling Integration
// ============================================================================

/// Process experience rewards after player defeats a monster
///
/// Awards XP based on monster level, difficulty, and player health.
/// Returns true if player leveled up.
pub fn process_player_xp_reward(
    player: &mut You,
    monster: &Monster,
    damage_dealt: i32,
    player_hp_before: i32,
) -> bool {
    // Calculate XP reward based on monster level
    let reward = calculate_monster_xp_reward(monster);

    // Calculate player health percentage (used for difficulty multiplier)
    let health_percent = (player.hp as f32 / player.hp_max as f32).clamp(0.0, 1.0);

    // Get total XP to award
    let xp_to_award =
        reward.calculate_total(player.exp_level, monster.level as i32, health_percent);

    // Award the XP and check for level up
    award_player_xp(player, xp_to_award)
}

// ============================================================================
// Phase 15: Spellcasting Combat Integration
// ============================================================================

/// Check if player can cast a spell in combat
pub fn can_player_cast_in_combat(player: &You, spell: CombatSpell) -> bool {
    can_player_cast_spell(player, spell)
}

/// Player casts a combat spell at target
pub fn player_cast_spell(
    player: &mut You,
    target: &mut Monster,
    spell: CombatSpell,
    rng: &mut GameRng,
) -> SpellCastResult {
    cast_combat_spell(player, target, spell, rng)
}

/// Get available combat spells for player's role
pub fn get_player_combat_spells(player: &You) -> &'static [CombatSpell] {
    super::CombatSpellList::get_spells_for_role(player.role)
}

// ============================================================================
// Phase 16: Loot & Treasure Integration
// ============================================================================

/// Generate loot drop when player defeats a monster
pub fn generate_monster_loot(monster: &Monster, rng: &mut GameRng) -> Option<Vec<LootDrop>> {
    // Check if monster is magical/special for better loot
    let is_magical = monster.level >= 8; // High level monsters are magical

    // Try to generate loot
    if let Some(drop) = LootGenerator::generate_loot(monster.level, is_magical, rng) {
        Some(vec![drop])
    } else {
        None
    }
}

/// Award all loot to player after defeating monster
pub fn award_monster_loot(player: &mut You, monster: &Monster, rng: &mut GameRng) -> i32 {
    // Try to generate loot
    if let Some(mut loot) = generate_monster_loot(monster, rng) {
        // Add bonus gold equal to monster level
        let bonus_gold = LootGenerator::generate_gold(monster.level, rng);

        // Add gold as a separate drop
        loot.push(LootDrop {
            category: super::LootCategory::Gold,
            rarity: super::ItemRarity::Common,
            item_type: format!("{} gold coins", bonus_gold),
            value: bonus_gold,
            gold_bonus: 0,
        });

        // Award loot to player and return total value
        award_loot_to_player(player, &loot)
    } else {
        // At least drop some gold
        let gold = LootGenerator::generate_gold(monster.level, rng);
        player.gold += gold;
        gold
    }
}

/// Calculate gold dropped by a dead monster (C: mkgold amount in mondead).
///
/// Uses the same formula as `LootGenerator::generate_gold()` but is called
/// from the death handler to place a physical gold pile on the floor.
pub fn calculate_monster_gold(monster: &Monster, rng: &mut GameRng) -> i32 {
    LootGenerator::generate_gold(monster.level, rng)
}

/// Check if monster drops treasure hoard (for boss/unique monsters)
pub fn should_drop_hoard(monster_level: u8) -> bool {
    // Boss-type monsters (level 15+) may have hoards
    monster_level >= 15
}

/// Generate and award a treasure hoard
pub fn award_boss_hoard(player: &mut You, monster: &Monster, rng: &mut GameRng) -> i32 {
    if !should_drop_hoard(monster.level) {
        return 0;
    }

    let hoard = TreasureHoard::generate_boss_hoard(monster.level, rng);
    award_loot_to_player(player, &hoard)
}

// ============================================================================
// Phase 17: Multi-Monster Encounter Integration
// ============================================================================

/// Create a combat encounter with multiple monsters
pub fn create_encounter(monster_ids: Vec<crate::monster::MonsterId>) -> CombatEncounter {
    let mut encounter = CombatEncounter::new(monster_ids);
    encounter.update_formation_for_count();
    encounter
}

/// Calculate encounter difficulty rating
pub fn calculate_encounter_difficulty(monsters: &[Monster], formation: Formation) -> i32 {
    DifficultyRating::calculate_total_difficulty(monsters, formation)
}

/// Get difficulty label for display
pub fn get_difficulty_label(difficulty: i32) -> &'static str {
    DifficultyRating::difficulty_label(difficulty)
}

/// Initialize encounter state
pub fn init_encounter_state(monster_ids: Vec<crate::monster::MonsterId>) -> EncounterState {
    EncounterState::new(monster_ids)
}

/// Check if any monsters in group are flanking the player
pub fn are_monsters_flanking(
    monsters: &[Monster],
    formation: Formation,
    player_pos: &crate::player::Position,
) -> bool {
    check_flanking(monsters, formation, player_pos)
}

/// Get flanking damage bonus multiplier
pub fn get_flanking_bonus(monsters: &[Monster], formation: Formation) -> f32 {
    flanking_damage_bonus(monsters, formation)
}

/// Apply encounter modifiers to a monster's stats
pub fn apply_encounter_effects(monster: &mut Monster, encounter_state: &EncounterState) {
    apply_encounter_modifiers(
        monster,
        encounter_state,
        encounter_state.encounter.formation,
    );
}

/// Process end of combat round for encounter
pub fn process_encounter_round(encounter_state: &mut EncounterState) {
    encounter_state.end_round();
}

/// Calculate XP reward for defeating entire encounter
pub fn get_encounter_victory_xp(
    encounter_state: &EncounterState,
    player_hp_remaining: i32,
    player_hp_max: i32,
) -> u32 {
    calculate_encounter_xp(
        &encounter_state.encounter,
        encounter_state.encounter.difficulty,
        player_hp_remaining,
        player_hp_max,
    )
}

// ============================================================================
// Weapon combat helper functions (weapon.c, uhitm.c)
// ============================================================================

/// Determine weapon skill type from a weapon object
///
/// Maps weapon types to the appropriate skill category for progression tracking.
/// Returns WeaponSkill::Bare for non-weapons.
pub fn get_weapon_skill(weapon: Option<&Object>) -> WeaponSkill {
    match weapon {
        None => WeaponSkill::Bare,
        Some(w) => {
            // Classify based on weapon name patterns; ideally uses weapon class constants
            // from object definitions when ObjClassDef integration is complete

            // Provisional classification based on common weapon names
            let name_lower = w
                .name
                .as_ref()
                .map(|n| n.to_lowercase())
                .unwrap_or_default();

            if name_lower.contains("dagger") || name_lower.contains("knife") {
                WeaponSkill::Dagger
            } else if name_lower.contains("sword")
                || name_lower.contains("scimitar")
                || name_lower.contains("saber")
            {
                WeaponSkill::Sword
            } else if name_lower.contains("axe") || name_lower.contains("hatchet") {
                WeaponSkill::Axe
            } else if name_lower.contains("spear") || name_lower.contains("pike") {
                WeaponSkill::Spear
            } else if name_lower.contains("polearm") || name_lower.contains("halberd") {
                WeaponSkill::Polearm
            } else if name_lower.contains("bow") {
                WeaponSkill::Bow
            } else if name_lower.contains("crossbow") {
                WeaponSkill::Crossbow
            } else if name_lower.contains("sling") {
                WeaponSkill::Sling
            } else if name_lower.contains("whip") {
                WeaponSkill::Whip
            } else if name_lower.contains("staff") || name_lower.contains("quarterstaff") {
                WeaponSkill::Staff
            } else if name_lower.contains("club")
                || name_lower.contains("mace")
                || name_lower.contains("hammer")
            {
                WeaponSkill::Blunt
            } else if name_lower.contains("flail") {
                WeaponSkill::Flail
            } else {
                // Default to generic sword skill
                WeaponSkill::Sword
            }
        }
    }
}

/// Calculate weapon hit value against a specific monster (hitval equivalent)
/// This is a "quality" rating of how good the weapon is against this target.
pub fn hitval(weapon: &Object, _target: &Monster) -> i32 {
    let mut hit = 0;

    // Base weapon to-hit bonus
    hit += weapon.weapon_tohit as i32;

    // Enchantment bonus
    hit += weapon.enchantment as i32;

    // Silver bonus: requires ObjClassDef material lookup on the weapon
    // (mon_hates_silver check is available but weapon material is not on Object)

    // Blessed weapons vs undead/demons
    if weapon.is_blessed() && (_target.is_undead() || _target.is_demon()) {
        hit += 2;
    }

    hit
}

/// Get weapon's to-hit bonus (weapon_hit_bonus equivalent)
/// Returns the total to-hit modifier from the weapon.
pub fn weapon_hit_bonus(weapon: &Object) -> i32 {
    let mut bonus = 0;

    // Base to-hit bonus from weapon type
    bonus += weapon.weapon_tohit as i32;

    // Enchantment adds to hit
    bonus += weapon.enchantment as i32;

    // Erosion can reduce to-hit (each level of erosion = -1)
    bonus -= weapon.erosion() as i32;

    bonus
}

/// Get weapon's damage bonus (weapon_dam_bonus equivalent)
/// Returns the total damage modifier from the weapon.
pub fn weapon_dam_bonus(weapon: &Object) -> i32 {
    let mut bonus = 0;

    // Enchantment adds to damage
    bonus += weapon.enchantment as i32;

    // Erosion reduces damage
    bonus -= weapon.erosion() as i32;

    bonus
}

/// Calculate complete roll to hit value (find_roll_to_hit equivalent)
/// This is the full to-hit calculation for a player attacking.
pub fn find_roll_to_hit(player: &You, target: &Monster, weapon: Option<&Object>) -> i32 {
    // Use the existing calculate_to_hit function
    calculate_to_hit(player, target, weapon)
}

/// Check if a weapon type is appropriate for throwing (throwing_weapon equivalent)
pub fn throwing_weapon(weapon: &Object) -> bool {
    // Daggers, darts, shuriken, boomerangs are good for throwing
    // Check object type or class
    matches!(weapon.class, crate::object::ObjectClass::Weapon) && weapon.thrown // Specifically marked as throwable
}

/// Check if weapon can be used for ranged attacks
pub fn is_ranged_weapon(weapon: &Object) -> bool {
    // Bows, crossbows, slings are ranged weapons
    // Would check specific object types
    // For now, check if it's a launcher type
    matches!(
        weapon.object_type,
        57..=62 // Bow types typically
    )
}

/// Map WeaponSkill (combat category) to SkillType (player tracking)
///
/// Bridges the two skill systems: WeaponSkill is used in combat calculations,
/// SkillType is used in player proficiency tracking. Maps 13 weapon categories
/// to the appropriate player skill type for advancement.
fn weapon_skill_to_skill_type(weapon_skill: WeaponSkill) -> crate::player::SkillType {
    use crate::player::SkillType;

    match weapon_skill {
        WeaponSkill::Bare => SkillType::BareHanded,
        WeaponSkill::Dagger => SkillType::Dagger,
        WeaponSkill::Sword => SkillType::BroadSword, // Generic sword maps to broad sword
        WeaponSkill::Axe => SkillType::Axe,
        WeaponSkill::Polearm => SkillType::Polearms,
        WeaponSkill::Bow => SkillType::Bow,
        WeaponSkill::Crossbow => SkillType::Crossbow,
        WeaponSkill::Sling => SkillType::Sling,
        WeaponSkill::Whip => SkillType::Whip,
        WeaponSkill::Staff => SkillType::Quarterstaff,
        WeaponSkill::Blunt => SkillType::Mace,
        WeaponSkill::Flail => SkillType::Flail,
        WeaponSkill::Spear => SkillType::Spear,
    }
}

/// Convert player SkillLevel (7 levels) to combat SkillLevel (5 levels)
///
/// Maps the player proficiency system (Restricted to GrandMaster)
/// to the simplified combat system (Unskilled to Master).
fn player_skill_level_to_combat(level: crate::player::SkillLevel) -> SkillLevel {
    use crate::player::SkillLevel as PlayerSkillLevel;

    match level {
        PlayerSkillLevel::Restricted | PlayerSkillLevel::Unskilled => SkillLevel::Unskilled,
        PlayerSkillLevel::Basic => SkillLevel::Basic,
        PlayerSkillLevel::Skilled => SkillLevel::Skilled,
        PlayerSkillLevel::Expert => SkillLevel::Expert,
        PlayerSkillLevel::Master | PlayerSkillLevel::GrandMaster => SkillLevel::Master,
    }
}

/// Get player's skill level with a specific weapon
///
/// Queries the player's proficiency tracking system to determine the
/// actual skill level for a weapon category. Maps from WeaponSkill
/// (combat system) to SkillType (player tracking) and returns the
/// corresponding player skill level, converted to combat SkillLevel.
pub fn get_player_weapon_skill(player: &You, weapon_skill: WeaponSkill) -> SkillLevel {
    let skill_type = weapon_skill_to_skill_type(weapon_skill);
    let player_level = player.skills.get(skill_type).level;
    player_skill_level_to_combat(player_level)
}

/// Update weapon proficiency after an attack
///
/// Tracks weapon usage and updates player proficiency based on whether
/// the attack hit, missed, or was critical. Awards experience points
/// and checks for skill advancement. This is called after every weapon
/// attack to maintain accurate proficiency tracking.
pub fn update_weapon_proficiency(
    player: &mut You,
    weapon_skill: WeaponSkill,
    hit: bool,
    is_critical: bool,
) {
    let skill_type = weapon_skill_to_skill_type(weapon_skill);

    // Award experience based on attack result
    {
        let skill = player.skills.get_mut(skill_type);
        if hit {
            // Hits award more experience, especially critical hits
            if is_critical {
                skill.add_practice(15); // Critical hit: 15 XP
            } else {
                skill.add_practice(10); // Normal hit: 10 XP
            }
        } else {
            // Misses still award minimal experience
            skill.add_practice(1);
        }
    }

    // Check if the skill can advance (if practice threshold met)
    // Reborrow after the previous scope ends
    let can_advance = player.skills.get(skill_type).can_advance();
    if can_advance && player.skills.slots > 0 {
        player.skills.get_mut(skill_type).advance();
        player.skills.slots -= 1;
    }
}

/// Phase 14: Advanced skill advancement tracking
///
/// Updates skill experience based on combat performance metrics
pub fn advance_weapon_skill_from_combat(
    player: &mut You,
    weapon_skill: WeaponSkill,
    hits: u32,
    misses: u32,
    critical_hits: u32,
) {
    let skill_type = weapon_skill_to_skill_type(weapon_skill);

    // Award practice points based on combat results
    {
        let skill = player.skills.get_mut(skill_type);
        let practice_points = (hits * 10) + (critical_hits * 5);
        skill.add_practice(practice_points as u16);

        // Penalize for misses but not heavily
        let miss_penalty = (misses / 3).min(5) as u16; // 1 point penalty per 3 misses, cap at 5
        skill.remove_practice(miss_penalty);
    }

    // Check for advancement - reborrow after the previous scope ends
    let can_advance = player.skills.get(skill_type).can_advance();
    if can_advance && player.skills.slots > 0 {
        player.skills.get_mut(skill_type).advance();
        player.skills.slots -= 1;
    }
}

/// Get player's armor proficiency level
///
/// Based on experience and class
pub fn get_player_armor_proficiency(player: &You) -> ArmorProficiency {
    match player.exp_level {
        0..=3 => ArmorProficiency::Untrained,
        4..=8 => ArmorProficiency::Novice,
        9..=15 => ArmorProficiency::Trained,
        16..=25 => ArmorProficiency::Expert,
        _ => ArmorProficiency::Master,
    }
}

/// Get player's dodge/evasion skill level
///
/// Based on dexterity and experience
pub fn get_player_dodge_skill(player: &You) -> DodgeSkill {
    let dex_bonus = (player.attr_current.get(crate::player::Attribute::Dexterity) as i32 - 10) / 2;
    let base_level = match player.exp_level {
        0..=3 => 0,
        4..=8 => 1,
        9..=15 => 2,
        16..=25 => 3,
        _ => 4,
    };

    let adjusted = base_level + dex_bonus;
    match adjusted {
        ..=0 => DodgeSkill::Untrained,
        1..=2 => DodgeSkill::Basic,
        3..=4 => DodgeSkill::Practiced,
        5..=6 => DodgeSkill::Expert,
        _ => DodgeSkill::Master,
    }
}

/// Calculate player defense against incoming attack
pub fn calculate_player_defense(player: &You) -> DefenseCalculation {
    let armor_prof = get_player_armor_proficiency(player);
    let dodge_skill = get_player_dodge_skill(player);

    // Base AC from armor class
    let base_ac = player.armor_class as i32;

    // For now, assume no armor degradation
    let degradation = super::ArmorDegradation::new(10);

    DefenseCalculation::calculate(base_ac, armor_prof, dodge_skill, degradation)
}

pub fn ammo_for_launcher(ammo: &Object, launcher: &Object) -> bool {
    // Check if ammo matches launcher type
    // Arrows for bows, bolts for crossbows, etc.
    match launcher.object_type {
        57 | 58 => matches!(ammo.object_type, 50..=56), // Bow + arrows
        59 | 60 => matches!(ammo.object_type, 63..=66), // Crossbow + bolts
        61 | 62 => matches!(ammo.object_type, 67..=70), // Sling + stones
        _ => false,
    }
}

/// Calculate damage for a thrown weapon (throw_damage equivalent concept)
pub fn throw_damage(weapon: &Object, player: &You, rng: &mut GameRng) -> i32 {
    let mut damage = if weapon.damage_dice > 0 && weapon.damage_sides > 0 {
        rng.dice(weapon.damage_dice as u32, weapon.damage_sides as u32) as i32
    } else {
        rng.dice(1, 4) as i32 // Default thrown damage
    };

    // Add enchantment
    damage += weapon.enchantment as i32;

    // Add strength bonus (half for thrown weapons)
    damage += player.attr_current.strength_damage_bonus() as i32 / 2;

    damage.max(1)
}

/// Comprehensive combat evaluation combining all factors
///
/// Evaluates a complete combat scenario including skill, modifiers, and special effects.
/// Returns detailed breakdown of combat calculation for debugging/analysis.
pub fn evaluate_comprehensive_combat(
    player: &You,
    target: &Monster,
    weapon: Option<&Object>,
    modifiers: &[CombatModifier],
    _rng: &mut GameRng,
) -> String {
    let weapon_skill = get_weapon_skill(weapon);
    let skill_level = get_player_weapon_skill(player, weapon_skill);
    let base_to_hit = calculate_to_hit(player, target, weapon);

    let armor_penetration = skill_level.armor_penetration();
    let enhanced_to_hit =
        calculate_skill_enhanced_to_hit(base_to_hit, skill_level, armor_penetration);

    let target_ac = target.ac;
    let effective_ac = apply_armor_penetration(target_ac, armor_penetration);

    let (mod_to_hit, mod_damage) = apply_combat_modifiers(modifiers);
    let final_to_hit = enhanced_to_hit + mod_to_hit;

    // Format comprehensive combat info
    format!(
        "Combat: {} vs {} (AC {})\n  Weapon: {:?}, Skill: {:?}\n  Base ToHit: {}, Enhanced: {}, Final: {}\n  Modifiers: +{} to-hit, +{} damage\n  EffectiveAC: {}",
        player.name,
        "Monster",
        target.ac,
        weapon_skill,
        skill_level,
        base_to_hit,
        enhanced_to_hit,
        final_to_hit,
        mod_to_hit,
        mod_damage,
        effective_ac
    )
}

// ============================================================================
// Ranged Combat System (Phase 11 Integration)
// ============================================================================

/// Determine ranged weapon type from launcher object
pub fn get_ranged_weapon_type(launcher: Option<&Object>) -> Option<RangedWeaponType> {
    match launcher {
        None => None,
        Some(w) => {
            let name_lower = w
                .name
                .as_ref()
                .map(|n| n.to_lowercase())
                .unwrap_or_default();

            if name_lower.contains("bow") {
                Some(RangedWeaponType::Bow)
            } else if name_lower.contains("crossbow") {
                Some(RangedWeaponType::Crossbow)
            } else if name_lower.contains("sling") {
                Some(RangedWeaponType::Sling)
            } else if w.thrown {
                Some(RangedWeaponType::Thrown)
            } else {
                None
            }
        }
    }
}

/// Calculate distance between attacker and target (simplified 2D distance)
pub fn calculate_distance(attacker_x: i32, attacker_y: i32, target_x: i32, target_y: i32) -> i32 {
    // Chebyshev distance (max of absolute differences) for grid-based combat
    let dx = (attacker_x - target_x).abs();
    let dy = (attacker_y - target_y).abs();
    dx.max(dy)
}

/// Player ranged attack against monster
pub fn player_ranged_attack(
    player: &mut You,
    target: &mut Monster,
    launcher: Option<&Object>,
    distance: i32,
    rng: &mut GameRng,
) -> CombatResult {
    // Determine ranged weapon type
    let Some(weapon_type) = get_ranged_weapon_type(launcher) else {
        // Not a valid ranged weapon
        return CombatResult::MISS;
    };

    // Get player's skill with ranged weapons (bow, crossbow, or sling)
    let weapon_skill = match weapon_type {
        RangedWeaponType::Bow => WeaponSkill::Bow,
        RangedWeaponType::Crossbow => WeaponSkill::Crossbow,
        RangedWeaponType::Sling => WeaponSkill::Sling,
        RangedWeaponType::Thrown => WeaponSkill::Dagger, // Use dagger skill for thrown
    };

    let skill_level = get_player_weapon_skill(player, weapon_skill);

    // Get base to-hit (without distance penalty)
    let strength = player.attr_current.get(crate::player::Attribute::Strength) as u8;
    let dexterity = player.attr_current.get(crate::player::Attribute::Dexterity) as u8;

    // Ranged weapons heavily favor dexterity
    let mut base_to_hit = (dexterity as i32 - 10) / 2;
    base_to_hit += player.exp_level;
    base_to_hit += player.luck as i32;

    // Add launcher enchantment
    if let Some(w) = launcher {
        base_to_hit += w.enchantment as i32;
        base_to_hit += w.weapon_tohit as i32;
    }

    // Create ranged attack info
    let ranged_attack = RangedAttack {
        weapon_type,
        distance,
        skill_level,
        base_to_hit,
    };

    // Check if in range
    if !ranged_attack.in_range() {
        return CombatResult::MISS;
    }

    // Line-of-sight check requires Level access; attack is validated at the caller level

    // Execute ranged attack
    let mut ranged_result = execute_ranged_attack(&ranged_attack, target.ac, rng);

    if !ranged_result.hit {
        return CombatResult::MISS;
    }

    // Calculate base damage from ammo/launcher
    let base_damage = if let Some(w) = launcher {
        if w.damage_dice > 0 && w.damage_sides > 0 {
            rng.dice(w.damage_dice as u32, w.damage_sides as u32) as i32
        } else {
            rng.dice(1, 6) as i32
        }
    } else {
        rng.dice(1, 4) as i32
    };

    // Calculate ranged damage with distance scaling
    let damage = ranged_attack.calculate_damage(base_damage, ranged_result.critical);

    // Handle instant kill
    let target_died = if ranged_result.critical == CriticalHitType::InstantKill {
        target.hp = 0;
        true
    } else {
        target.hp -= damage;
        target.hp <= 0
    };

    // Apply special effects for ranged weapons
    let mut special_effect = None;

    // On critical hit, chance for special effect
    if ranged_result.critical.is_critical() && skill_level as u8 >= SkillLevel::Skilled as u8 {
        if rng.one_in(4) {
            special_effect = Some(super::CombatEffect::ItemDestroyed); // Arrow breaks/pierces armor
        }
    }

    // Update weapon proficiency
    update_weapon_proficiency(
        player,
        weapon_skill,
        true,
        ranged_result.critical.is_critical(),
    );

    CombatResult {
        hit: true,
        defender_died: target_died,
        attacker_died: false,
        damage,
        special_effect,
    }
}

/// Consume ammunition for a ranged attack
///
/// Reduces ammunition count by one if available. Should be called after a
/// ranged attack succeeds or fails, to deduct the arrow/bolt/stone used.
///
/// Returns true if ammunition was successfully consumed, false if out of ammo
/// (which shouldn't happen if checks were done correctly upstream).
pub fn consume_ammunition(ammunition: &mut AmmunitionCount) -> bool {
    ammunition.consume()
}

/// Recover ammunition after a ranged attack
///
/// Attempts to pick up the projectile after an attack. In NetHack, arrows
/// can be recovered with some probability depending on what they hit.
/// Success rate should be higher for misses, lower for kills.
///
/// For now, this implements a simple recovery chance based on what happened.
pub fn try_recover_ammunition(
    ammunition: &mut AmmunitionCount,
    hit: bool,
    critical: CriticalHitType,
    rng: &mut crate::rng::GameRng,
) {
    // Recovery chance depends on whether projectile hit and how hard
    let recovery_chance = match (hit, critical) {
        (false, _) => 95,                    // 95% chance to recover on miss
        (true, CriticalHitType::None) => 75, // 75% on normal hit
        (true, CriticalHitType::Graze | CriticalHitType::Critical) => 50, // 50% on critical
        (true, CriticalHitType::Devastating) => 25, // 25% on devastating
        (true, CriticalHitType::InstantKill) => 0, // 0% on instant kill
    };

    if rng.rnd(100) < recovery_chance {
        ammunition.recover(1); // Recover one projectile
    }
}

/// Get ammunition requirements for a launcher
///
/// Determines what type and how much ammunition a launcher needs.
/// Returns (ammunition_type, ideal_count) where type is weapon class.
pub fn ammunition_requirement_for_launcher(launcher: &Object) -> Option<(u16, i32)> {
    // Bow requires arrows (50-56), ideal 20
    if launcher.object_type >= 57 && launcher.object_type <= 58 {
        return Some((50, 20)); // Arrows
    }

    // Crossbow requires bolts (63-66), ideal 15
    if launcher.object_type >= 59 && launcher.object_type <= 60 {
        return Some((63, 15)); // Bolts
    }

    // Sling requires stones (67-70), ideal 30
    if launcher.object_type >= 61 && launcher.object_type <= 62 {
        return Some((67, 30)); // Stones
    }

    None
}

/// Check if a target monster is friendly to the player
///
/// A monster is considered friendly if:
/// - It's tame (pet)
/// - It's peaceful and the player is not hostile
/// - It's an NPC or allied creature
///
/// Returns true if the monster should NOT be attacked, false if hostile/enemy.
pub fn is_friendly_target(target: &Monster, player: &You) -> bool {
    // Tame monsters are always friendly (pets)
    if target.state.tame {
        return true;
    }

    // Peaceful monsters that are co-aligned are friendly
    if target.state.peaceful {
        // If both player and monster are aligned the same way (or both neutral),
        // they're friendly. Co-alignment check:
        let player_align_type = player.alignment.typ;
        let monster_align_type = crate::player::AlignmentType::from_value(target.alignment);

        let coaligned = if player_align_type == crate::player::AlignmentType::Neutral {
            // Neutral player doesn't have alignment-based friendliness
            false
        } else {
            player_align_type == monster_align_type
        };

        if coaligned {
            return true; // Same alignment = friendly
        }
    }

    // All other monsters are hostile/enemy
    false
}

/// Friendly fire check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FriendlyFireResult {
    /// Safe to attack - no friendly fire risk
    Safe,
    /// Target is friendly - would cause friendly fire
    TargetFriendly,
    /// Friendly unit in projectile path - would cause collateral damage
    CollateralRisk,
}

/// Check if a ranged attack would cause friendly fire
///
/// Validates that the attack target and projectile path don't hit friendly units.
/// Returns Safe if the attack is clear, or describes the friendly fire risk.
pub fn check_friendly_fire(
    attacker: &You,
    target: &Monster,
    level: &crate::dungeon::Level,
) -> FriendlyFireResult {
    // First check: is the target itself friendly?
    if is_friendly_target(target, attacker) {
        return FriendlyFireResult::TargetFriendly;
    }

    // Second check: scan projectile path for friendly units
    // (This would require scanning all monsters in level, which isn't available
    // in this function signature without Level access to monster list)
    // For now, we implement basic checks above

    FriendlyFireResult::Safe
}

/// Get friendly fire warning message
///
/// Returns a descriptive message about the friendly fire risk that can be
/// shown to the player before confirming the attack.
pub fn friendly_fire_warning_message(result: FriendlyFireResult, target: &Monster) -> String {
    match result {
        FriendlyFireResult::Safe => "Attack is clear - no friendly units at risk.".to_string(),
        FriendlyFireResult::TargetFriendly => {
            format!(
                "{} is friendly! {} won't attack your ally!",
                &target.name,
                if target.state.tame { "You" } else { "You" }
            )
        }
        FriendlyFireResult::CollateralRisk => {
            "Warning: A friendly unit may be in the projectile path!".to_string()
        }
    }
}

/// Attempt ranged attack with friendly fire prevention
///
/// Checks for friendly fire before executing the attack.
/// Returns true if attack proceeds, false if blocked by friendly fire.
pub fn can_attack_ranged_safely(
    attacker: &You,
    target: &Monster,
    level: &crate::dungeon::Level,
) -> Result<(), String> {
    let ff_result = check_friendly_fire(attacker, target, level);

    match ff_result {
        FriendlyFireResult::Safe => Ok(()),
        FriendlyFireResult::TargetFriendly => Err(format!(
            "{} is your ally - you can't attack them!",
            &target.name
        )),
        FriendlyFireResult::CollateralRisk => Err("Friendly units are in the way!".to_string()),
    }
}

/// Monster ranged attack against player
pub fn monster_ranged_attack(
    attacker: &Monster,
    player: &mut You,
    distance: i32,
    rng: &mut GameRng,
) -> CombatResult {
    // Simple monster ranged attack (if monster has ranged capability)
    // Most monsters use melee, but some can throw rocks, breathe, etc.

    // Get monster skill level
    let skill_level = match attacker.level {
        0..=2 => SkillLevel::Unskilled,
        3..=6 => SkillLevel::Basic,
        7..=12 => SkillLevel::Skilled,
        13..=20 => SkillLevel::Expert,
        _ => SkillLevel::Master,
    };

    // Assume thrown rock/projectile attack
    let base_to_hit = attacker.level as i32;

    let ranged_attack = RangedAttack {
        weapon_type: RangedWeaponType::Thrown,
        distance,
        skill_level,
        base_to_hit,
    };

    if !ranged_attack.in_range() {
        return CombatResult::MISS;
    }

    let ranged_result = execute_ranged_attack(&ranged_attack, player.armor_class, rng);

    if !ranged_result.hit {
        return CombatResult::MISS;
    }

    // Calculate damage
    let base_damage = rng.dice(1, 4) as i32;
    let damage = ranged_attack.calculate_damage(base_damage, ranged_result.critical);

    let player_died = if ranged_result.critical == CriticalHitType::InstantKill {
        player.hp = 0;
        true
    } else {
        player.hp -= damage;
        player.hp <= 0
    };

    CombatResult {
        hit: true,
        defender_died: player_died,
        attacker_died: false,
        damage,
        special_effect: None,
    }
}

// ============================================================================
// Player Defense Application (Phase 12 Integration)
// ============================================================================

/// Apply player defense to incoming damage
///
/// Calculates damage reduction from armor, applies dodge saves,
/// and returns final damage taken
pub fn apply_player_defense(
    player: &You,
    incoming_damage: i32,
    damage_type: DamageType,
    rng: &mut GameRng,
) -> i32 {
    // Calculate player defense
    let defense = calculate_player_defense(player);

    // Try to dodge (dodge skill scales with dexterity)
    let dodge_skill = get_player_dodge_skill(player);
    if attempt_dodge(dodge_skill, 0, rng) {
        // Dodged! Take 1 damage or none
        return 0;
    }

    // Calculate armor damage reduction
    let armor_type = ArmorType::Light; // Worn armor type detection requires per-slot equipment tracking
    let reduction = calculate_armor_damage_reduction(
        defense.base_ac,
        defense.proficiency,
        damage_type,
        armor_type,
    );

    // Apply damage reduction
    apply_damage_reduction(incoming_damage, reduction)
}

/// Check if player can dodge an attack
pub fn can_player_dodge(player: &You, attacker_accuracy: i32, rng: &mut GameRng) -> bool {
    let dodge_skill = get_player_dodge_skill(player);
    attempt_dodge(dodge_skill, attacker_accuracy, rng)
}

/// Check if player is wearing armor of a specific type
pub fn player_wearing_armor_type(player: &You, armor_type: ArmorType) -> bool {
    // Per-slot worn armor tracking not yet implemented; defaults to no armor
    let _ = (player, armor_type);
    false
}

/// Check if weapon can penetrate player armor
pub fn weapon_penetrates_armor(weapon: Option<&Object>, armor_type: ArmorType) -> bool {
    match weapon {
        None => true, // Bare hands can't penetrate armor
        Some(w) => {
            // Enchanted/special weapons can penetrate better
            w.enchantment > 0 || matches!(armor_type, ArmorType::Light)
        }
    }
}

/// Check if weapon is wielded two-handed
pub fn is_two_handed(weapon: &Object) -> bool {
    // Two-handed weapons are typically heavy (>=100) or specific types
    weapon.weight >= 100
        || matches!(
            weapon.object_type,
            20..=25 // Two-handed sword types
        )
}

/// Get skill type for a weapon (uwep_skill_type equivalent concept)
pub fn weapon_skill_type(weapon: &Object) -> crate::player::SkillType {
    use crate::player::SkillType;

    // Map weapon object type to skill type
    // This would normally use a lookup table from the object definition
    match weapon.object_type {
        // Daggers
        0..=5 => SkillType::Dagger,
        // Short swords
        6..=10 => SkillType::ShortSword,
        // Broadswords
        11..=15 => SkillType::BroadSword,
        // Long swords
        16..=19 => SkillType::LongSword,
        // Two-handed swords
        20..=25 => SkillType::TwoHandedSword,
        // Axes
        30..=35 => SkillType::Axe,
        // Maces/clubs
        40..=45 => SkillType::Mace,
        // Polearms
        46..=49 => SkillType::Polearms,
        // Bows
        57..=58 => SkillType::Bow,
        // Crossbows
        59..=60 => SkillType::Crossbow,
        // Slings
        61..=62 => SkillType::Sling,
        // Whips
        70..=72 => SkillType::Whip,
        // Default to bare handed
        _ => SkillType::BareHanded,
    }
}

/// Get skill to-hit bonus for weapon use (weapon.c:weapon_hit_bonus)
///
/// Handles three cases per C source:
/// - Normal weapon: -4 (unskilled) to +3 (expert)
/// - Two-weapon combat: uses min of weapon skill and two-weapon skill, -9 to -3
/// - Bare-handed: scales with martial arts (monks get double bonus)
pub fn skill_hit_bonus(player: &You, weapon: Option<&Object>, is_twoweap: bool) -> i32 {
    use crate::player::SkillLevel;

    let wep_type = match weapon {
        Some(w) => weapon_skill_type(w),
        None => crate::player::SkillType::BareHanded,
    };

    if wep_type == crate::player::SkillType::BareHanded {
        // Bare-handed/martial arts (weapon.c:1452-1465)
        //   b.h. m.a.
        //   unskl: +1  n/a
        //   basic: +1   +3
        //   skild: +2   +4
        //   exprt: +2   +5
        //   mastr: +3   +6
        //   grand: +3   +7
        let raw = player.skills.get(wep_type).level.as_int();
        let raw = raw.max(1) - 1; // unskilled => 0
        let is_martial = martial_bonus(player);
        return ((raw + 2) * if is_martial { 2 } else { 1 }) / 2;
    }

    if is_twoweap {
        // Two-weapon combat (weapon.c:1431-1451)
        // Use minimum of weapon skill and two-weapon skill
        let tw_level = player
            .skills
            .get(crate::player::SkillType::TwoWeapon)
            .level;
        let wep_level = player.skills.get(wep_type).level;
        let effective = if wep_level.as_int() < tw_level.as_int() {
            wep_level
        } else {
            tw_level
        };
        return match effective {
            SkillLevel::Restricted | SkillLevel::Unskilled => -9,
            SkillLevel::Basic => -7,
            SkillLevel::Skilled => -5,
            SkillLevel::Expert => -3,
            SkillLevel::Master => -2,
            SkillLevel::GrandMaster => -1,
        };
    }

    // Normal weapon skill (weapon.c:1413-1430)
    let skill_level = player.skills.get(wep_type).level;
    match skill_level {
        SkillLevel::Restricted | SkillLevel::Unskilled => -4,
        SkillLevel::Basic => 0,
        SkillLevel::Skilled => 2,
        SkillLevel::Expert => 3,
        SkillLevel::Master => 4,
        SkillLevel::GrandMaster => 5,
    }
}

/// Get skill damage bonus for weapon use (weapon.c:weapon_dam_bonus)
///
/// Handles three cases per C source:
/// - Normal weapon: -2 (unskilled) to +2 (expert)
/// - Two-weapon combat: uses min of weapon skill and two-weapon skill, -3 to +1
/// - Bare-handed: scales with martial arts (monks get triple bonus)
pub fn skill_dam_bonus(player: &You, weapon: Option<&Object>, is_twoweap: bool) -> i32 {
    use crate::player::SkillLevel;

    let wep_type = match weapon {
        Some(w) => weapon_skill_type(w),
        None => crate::player::SkillType::BareHanded,
    };

    if wep_type == crate::player::SkillType::BareHanded {
        // Bare-handed/martial arts (weapon.c:1546-1558)
        //   b.h. m.a.
        //   unskl:  0   n/a
        //   basic: +1   +3
        //   skild: +1   +4
        //   exprt: +2   +6
        //   mastr: +2   +7
        //   grand: +3   +9
        let raw = player.skills.get(wep_type).level.as_int();
        let raw = raw.max(1) - 1; // unskilled => 0
        let is_martial = martial_bonus(player);
        return ((raw + 1) * if is_martial { 3 } else { 1 }) / 2;
    }

    if is_twoweap {
        // Two-weapon combat (weapon.c:1526-1545)
        let tw_level = player
            .skills
            .get(crate::player::SkillType::TwoWeapon)
            .level;
        let wep_level = player.skills.get(wep_type).level;
        let effective = if wep_level.as_int() < tw_level.as_int() {
            wep_level
        } else {
            tw_level
        };
        return match effective {
            SkillLevel::Restricted | SkillLevel::Unskilled => -3,
            SkillLevel::Basic => -1,
            SkillLevel::Skilled => 0,
            SkillLevel::Expert => 1,
            SkillLevel::Master => 2,
            SkillLevel::GrandMaster => 3,
        };
    }

    // Normal weapon skill (weapon.c:1506-1525)
    let skill_level = player.skills.get(wep_type).level;
    match skill_level {
        SkillLevel::Restricted | SkillLevel::Unskilled => -2,
        SkillLevel::Basic => 0,
        SkillLevel::Skilled => 1,
        SkillLevel::Expert => 2,
        SkillLevel::Master => 3,
        SkillLevel::GrandMaster => 4,
    }
}

/// Check if player gets martial arts bonus (monks in human form)
///
/// Port of C's martial_bonus() macro.
pub fn martial_bonus(player: &You) -> bool {
    player.role == crate::player::Role::Monk
}

// Skill practice tracking is in player/skills.rs via use_skill()

// ============================================================================
// Pre-attack validation and main attack function
// ============================================================================

/// Perform pre-attack validation checks.
///
/// Checks if attack should be blocked due to invisible monsters, mimics,
/// peaceful creatures, or other special conditions.
///
/// # Arguments
/// * `target` - The monster being attacked
/// * `weapon` - The weapon being used (if any)
/// * `can_see_target` - Whether player can see the target
/// * `is_peaceful` - Whether target is peaceful
/// * `force_fight` - Whether player is force-fighting
/// * `rng` - Random number generator
///
/// # Returns
/// true if attack should be blocked, false if allowed
pub fn attack_checks(
    target: &mut Monster,
    weapon: Option<&Object>,
    can_see_target: bool,
    is_peaceful: bool,
    force_fight: bool,
    _rng: &mut GameRng,
) -> bool {
    // Wake up waiting monsters
    target.state.sleeping = false;

    // Force-fight bypasses most checks
    if force_fight {
        return false;
    }

    // Invisible monster detection
    if !can_see_target && target.state.invisible {
        // Block attack, but mark as detected
        target.state.invisible = false;
        return true;
    }

    // Mimic detection
    if target.state.hiding {
        // Reveal hidden monster
        target.state.hiding = false;
        return true;
    }

    // Peaceful confirmation
    if is_peaceful && can_see_target {
        // Check for Stormbringer special case
        let is_stormbringer = weapon.map_or(false, |_w| {
            // Would check for artifact "Stormbringer" - for now stub
            false
        });
        if !is_stormbringer {
            // In full implementation, would ask for confirmation
            // For now, just allow but mark for future UI
            return false;
        }
    }

    false // Attack allowed
}

/// Main player melee attack against monster.
///
/// Handles safe pets, attack validation, special cases like leprechaun evasion,
/// and delegates to actual combat calculation.
///
/// # Arguments
/// * `player` - The player attacking
/// * `target` - The monster being attacked
/// * `weapon` - The weapon being used (if any)
/// * `force_fight` - Whether player is force-fighting
/// * `rng` - Random number generator
///
/// # Returns
/// The result of the combat action
pub fn attack(
    player: &mut You,
    target: &mut Monster,
    weapon: Option<&Object>,
    force_fight: bool,
    rng: &mut GameRng,
) -> CombatResult {
    // Pre-attack validation
    let can_see = true; // Would check player's vision
    if attack_checks(
        target,
        weapon,
        can_see,
        target.state.peaceful,
        force_fight,
        rng,
    ) {
        return CombatResult::MISS;
    }

    // Check if player can attack
    if matches!(player.monster_num, Some(m) if is_player_pacifist_form(m)) {
        return CombatResult::MISS;
    }

    // Capacity/overexertion check
    if matches!(player.encumbrance(), crate::player::Encumbrance::Overloaded) {
        return CombatResult::MISS;
    }

    // Execute actual attack
    player_attack_monster(player, target, weapon, rng)
}

/// Check if polymorphed player has pacifist form
fn is_player_pacifist_form(_monster_type: i16) -> bool {
    // Would check if polymorphed form has no attacks
    false
}

// ============================================================================
// Damage bonus functions (dbon, special_dmgval)
// ============================================================================

/// Calculate strength-based damage bonus (dbon in C).
///
/// Returns the damage bonus based on player's current strength.
/// This is separate from the to-hit bonus.
pub fn dbon(player: &You) -> i32 {
    // If polymorphed, no strength bonus
    if player.monster_num.is_some() {
        return 0;
    }

    let str = player.attr_current.get(crate::player::Attribute::Strength);

    if str < 6 {
        -1
    } else if str < 16 {
        0
    } else if str < 18 {
        1
    } else if str == 18 {
        2 // up to 18
    } else if str <= 93 {
        // 18/01 to 18/75 (stored as 19-93)
        3
    } else if str <= 108 {
        // 18/76 to 18/90 (stored as 94-108)
        4
    } else if str < 118 {
        // 18/91 to 18/99 (stored as 109-117)
        5
    } else {
        // 18/100 or higher (stored as 118+)
        6
    }
}

/// Calculate special damage value for weapon against monster (special_dmgval in C).
///
/// Returns bonus damage for special weapon properties like silver, blessed vs undead, etc.
///
/// # Arguments
/// * `weapon` - The weapon being used
/// * `is_silver_weapon` - Whether the weapon is made of silver
/// * `target_hates_silver` - Whether the target is vulnerable to silver
/// * `target_is_undead_or_demon` - Whether the target is undead or a demon
/// * `rng` - Random number generator
pub fn special_dmgval(
    weapon: &Object,
    is_silver_weapon: bool,
    target_hates_silver: bool,
    target_is_undead_or_demon: bool,
    rng: &mut GameRng,
) -> i32 {
    let mut bonus = 0;

    // Silver weapons deal extra damage to silver-hating monsters
    if is_silver_weapon && target_hates_silver {
        bonus += rng.rnd(20) as i32;
    }

    // Blessed weapons deal extra damage to undead and demons
    if weapon.is_blessed() && target_is_undead_or_demon {
        bonus += rng.rnd(4) as i32;
    }

    bonus
}

// ============================================================================
// Jousting (joust)
// ============================================================================

/// Result of a joust attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoustResult {
    /// No joust bonus (ordinary hit)
    NoJoust,
    /// Successful joust (extra damage)
    Success,
    /// Joust hit but lance breaks
    LanceBreaks,
}

/// Check for jousting when hitting a monster with a lance while mounted (joust in C).
///
/// # Arguments
/// * `player` - The player
/// * `is_mounted` - Whether player is mounted
/// * `is_fumbling` - Whether player is fumbling
/// * `is_lance` - Whether weapon is a lance
/// * `target_is_solid` - Whether target is solid (not unsolid/incorporeal)
/// * `weapon` - The lance being used
/// * `rng` - Random number generator
///
/// # Returns
/// `JoustResult` indicating the outcome
pub fn joust(
    player: &You,
    is_mounted: bool,
    is_fumbling: bool,
    is_lance: bool,
    target_is_solid: bool,
    weapon: &Object,
    rng: &mut GameRng,
) -> JoustResult {
    // Must be mounted to joust
    if !is_mounted {
        return JoustResult::NoJoust;
    }

    // Can't joust if fumbling or stunned
    if is_fumbling || player.is_stunned() {
        return JoustResult::NoJoust;
    }

    // Weapon must be a lance
    if !is_lance {
        return JoustResult::NoJoust;
    }

    // Get skill level for lance
    let skill_type = weapon_skill_type(weapon);
    let skill_level = player.skills.get(skill_type).level;

    use crate::player::SkillLevel;
    let skill_rating = match skill_level {
        SkillLevel::Restricted | SkillLevel::Unskilled => 1,
        SkillLevel::Basic => 2,
        SkillLevel::Skilled => 3,
        SkillLevel::Expert => 4,
        SkillLevel::Master | SkillLevel::GrandMaster => 5,
    };

    // Odds to joust: expert:80%, skilled:60%, basic:40%, unskilled:20%
    let joust_roll = rng.rn2(5);
    if joust_roll < skill_rating as u32 {
        // Check for lance breaking (rare) - use luck 0 for simplicity
        if joust_roll == 0 && rng.rnl(50, 0) == 49 && target_is_solid {
            return JoustResult::LanceBreaks;
        }
        return JoustResult::Success;
    }

    JoustResult::NoJoust
}

// ============================================================================
// Shade functions (shade_aware, shade_miss)
// ============================================================================

/// Check if an object can affect shades (shade_aware in C).
///
/// Returns true if the object either:
/// 1. Can affect shades directly
/// 2. Is handled properly by other routines for shades
///
/// # Arguments
/// * `weapon` - The weapon being used (if any)
/// * `is_silver` - Whether the weapon is made of silver
/// * `is_mirror` - Whether the weapon is a mirror
/// * `is_garlic` - Whether the weapon is garlic
pub fn shade_aware(
    weapon: Option<&Object>,
    is_silver: bool,
    is_mirror: bool,
    is_garlic: bool,
) -> bool {
    let Some(obj) = weapon else {
        return false;
    };

    // Silver objects affect shades
    if is_silver {
        return true;
    }

    // Heavy objects (boulder, iron ball, chain) affect shades
    if obj.weight >= 200 {
        return true;
    }

    // Mirrors (silver in reflective surface) affect shades
    if is_mirror {
        return true;
    }

    // Garlic causes shades to flee
    if is_garlic {
        return true;
    }

    false
}

/// Check if attack passes harmlessly through a shade (shade_miss in C).
///
/// Returns true if the attack misses because target is a shade and
/// the weapon can't affect shades.
///
/// # Arguments
/// * `target_is_shade` - Whether target is a shade
/// * `weapon` - The weapon being used (if any)
/// * `is_silver` - Whether the weapon is made of silver
/// * `is_mirror` - Whether the weapon is a mirror
/// * `is_garlic` - Whether the weapon is garlic
pub fn shade_miss(
    target_is_shade: bool,
    weapon: Option<&Object>,
    is_silver: bool,
    is_mirror: bool,
    is_garlic: bool,
) -> bool {
    // Only applies to shades
    if !target_is_shade {
        return false;
    }

    // Check if weapon can affect shades
    if shade_aware(weapon, is_silver, is_mirror, is_garlic) {
        return false;
    }

    // Attack passes through the shade
    true
}

/// Generate shade miss message
pub fn shade_miss_message(
    attacker_name: &str,
    weapon_name: Option<&str>,
    target_name: &str,
) -> String {
    match weapon_name {
        Some(name) => format!(
            "{}'s {} passes harmlessly through {}.",
            attacker_name, name, target_name
        ),
        None => format!(
            "{}'s attack passes harmlessly through {}.",
            attacker_name, target_name
        ),
    }
}

// ============================================================================
// Silver functions (hates_silver, mon_hates_silver, silver_sears)
// ============================================================================

/// Check if a monster type hates silver (hates_silver in C).
///
/// Returns true for werewolves, vampires, demons, shades, and most imps.
///
/// # Arguments
/// * `is_were` - Whether monster is a lycanthrope
/// * `is_vampire` - Whether monster is a vampire
/// * `is_demon` - Whether monster is a demon
/// * `is_shade` - Whether monster is a shade
/// * `is_imp` - Whether monster is an imp
/// * `is_tengu` - Whether monster is a tengu (exception among imps)
pub fn hates_silver_check(
    is_were: bool,
    is_vampire: bool,
    is_demon: bool,
    is_shade: bool,
    is_imp: bool,
    is_tengu: bool,
) -> bool {
    is_were || is_vampire || is_demon || is_shade || (is_imp && !is_tengu)
}

/// Calculate silver searing damage and return message if applicable.
///
/// Returns (extra_damage, optional_message)
///
/// # Arguments
/// * `is_silver_weapon` - Whether the weapon is made of silver
/// * `target_hates_silver` - Whether the target is vulnerable to silver
/// * `target_name` - Name of the target for the message
/// * `rng` - Random number generator
pub fn silver_sears(
    is_silver_weapon: bool,
    target_hates_silver: bool,
    target_name: &str,
    rng: &mut GameRng,
) -> (i32, Option<String>) {
    if !is_silver_weapon || !target_hates_silver {
        return (0, None);
    }

    let damage = rng.rnd(20) as i32;
    let msg = format!("The silver sears {}!", target_name);
    (damage, Some(msg))
}

// ============================================================================
// Sticking (sticks)
// ============================================================================

/// Check if attacker sticks to target (sticks in C).
///
/// Some monsters (like mimics) stick to what they attack.
///
/// # Arguments
/// * `is_mimic` - Whether attacker is a mimic
/// * `is_sticky` - Whether attacker has sticky attack
pub fn sticks(is_mimic: bool, is_sticky: bool) -> bool {
    is_mimic || is_sticky
}

// ============================================================================
// Object retouch functions (retouch_object, retouch_equipment)
// ============================================================================

/// Check if player should be hurt by touching an object.
///
/// Some objects (like silver for werewolves) hurt certain creatures on touch.
///
/// # Arguments
/// * `player_hates_silver` - Whether player is vulnerable to silver
/// * `is_silver_object` - Whether the object is made of silver
pub fn retouch_object(player_hates_silver: bool, is_silver_object: bool) -> Option<i32> {
    // Check if player hates silver and object is silver
    if player_hates_silver && is_silver_object {
        // Silver burns the player
        return Some(1); // Minimal damage for touch
    }

    None
}

/// Check all equipped items for retouch damage.
///
/// Returns total damage from all harmful equipped items.
///
/// # Arguments
/// * `player_hates_silver` - Whether player is vulnerable to silver
/// * `inventory` - Player's inventory
/// * `is_silver_fn` - Function to check if an object is silver
pub fn retouch_equipment<F>(player_hates_silver: bool, inventory: &[Object], is_silver_fn: F) -> i32
where
    F: Fn(&Object) -> bool,
{
    let mut total_damage = 0;

    for obj in inventory {
        // Only check worn/wielded items
        if obj.worn_mask == 0 {
            continue;
        }

        if let Some(damage) = retouch_object(player_hates_silver, is_silver_fn(obj)) {
            total_damage += damage;
        }
    }

    total_damage
}

// ============================================================================
// Additional NetHack C function equivalents
// ============================================================================

/// Main melee attack function (hitum in C).
///
/// Wrapper for the full attack sequence including checks and execution.
///
/// # Arguments
/// * `player` - The attacking player
/// * `target` - The monster being attacked
/// * `weapon` - The weapon being used
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result
pub fn hitum(
    player: &mut You,
    target: &mut Monster,
    weapon: Option<&Object>,
    rng: &mut GameRng,
) -> CombatResult {
    // Check if we can attack
    if target.state.peaceful && !player.properties.has(crate::player::Property::Conflict) {
        // Would need confirmation in full implementation
        return CombatResult::MISS;
    }

    // Execute the attack
    player_attack_monster(player, target, weapon, rng)
}

/// Cleaving attack - hit multiple adjacent targets (hitum_cleave in C).
///
/// When the player has a cleaving weapon, they can potentially hit
/// multiple monsters in one swing.
///
/// # Arguments
/// * `player` - The attacking player
/// * `targets` - List of adjacent monsters
/// * `weapon` - The cleaving weapon
/// * `rng` - Random number generator
///
/// # Returns
/// Vector of combat results for each target hit
pub fn hitum_cleave(
    player: &mut You,
    targets: &mut [&mut Monster],
    weapon: &Object,
    rng: &mut GameRng,
) -> Vec<CombatResult> {
    let mut results = Vec::new();

    // Cleaving can hit up to 3 adjacent targets
    let max_cleave = 3.min(targets.len());

    for target in targets.iter_mut().take(max_cleave) {
        let result = player_attack_monster(player, *target, Some(weapon), rng);
        results.push(result);

        // If we miss, the cleave chain ends
        if !result.hit {
            break;
        }
    }

    results
}

/// Attack a known (visible) monster (known_hitum in C).
///
/// This is used when the player explicitly targets a visible monster.
///
/// # Arguments
/// * `player` - The attacking player
/// * `target` - The visible monster
/// * `weapon` - The weapon being used
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result
pub fn known_hitum(
    player: &mut You,
    target: &mut Monster,
    weapon: Option<&Object>,
    rng: &mut GameRng,
) -> CombatResult {
    // Wake up the monster
    target.state.sleeping = false;

    // Mark as seen
    target.state.invisible = false;

    hitum(player, target, weapon, rng)
}

/// Hit a monster and apply damage - simplified version (hmon in C).
///
/// Core damage application function.
///
/// # Arguments
/// * `player` - The attacking player
/// * `target` - The monster being hit
/// * `weapon` - The weapon used
/// * `damage` - Base damage to apply
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result with damage applied
pub fn hmon_simple(
    _player: &mut You,
    target: &mut Monster,
    weapon: Option<&Object>,
    damage: i32,
    rng: &mut GameRng,
) -> CombatResult {
    let mut final_damage = damage;

    // Apply weapon special properties
    if let Some(w) = weapon {
        // Silver weapon bonus (check material if available)
        if mon_hates_silver(target) {
            // Would check w.material == Material::Silver in full implementation
            // For now, check if weapon name contains "silver"
            if w.name
                .as_ref()
                .map_or(false, |n| n.to_lowercase().contains("silver"))
            {
                final_damage += rng.rnd(20) as i32;
            }
        }

        // Blessed weapon vs undead/demon (simplified)
        if w.is_blessed() && target.level >= 10 {
            // Higher level monsters are more likely to be demons/undead
            final_damage += rng.rnd(4) as i32;
        }
    }

    // Apply damage
    target.hp -= final_damage;

    CombatResult {
        hit: true,
        defender_died: target.hp <= 0,
        attacker_died: false,
        damage: final_damage,
        special_effect: None,
    }
}

/// Bare-handed attack against monster (hmonas in C).
///
/// Used when player is polymorphed into a form with natural attacks.
///
/// # Arguments
/// * `player` - The attacking player
/// * `target` - The monster being attacked
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result
pub fn hmonas(player: &mut You, target: &mut Monster, rng: &mut GameRng) -> CombatResult {
    // Calculate damage based on polymorphed form
    let damage = if player.monster_num.is_some() {
        // Polymorphed - use monster form damage
        // Simplified: use level-based damage
        rng.dice(1, (player.exp_level as u32).max(2).min(10)) as i32
    } else if player.role == crate::player::Role::Monk {
        // Monks have martial arts
        let sides = ((player.exp_level / 2) + 1).clamp(2, 16) as u32;
        rng.dice(1, sides) as i32
    } else {
        // Regular bare hands
        rng.dice(1, 2) as i32
    };

    hmon_simple(player, target, None, damage, rng)
}

/// Object hits monster (thrown/launched object) (ohitmon in C).
///
/// Handles damage when a thrown or launched object hits a monster.
///
/// # Arguments
/// * `target` - The monster being hit
/// * `obj` - The object that hit
/// * `launcher_damage_bonus` - Bonus damage from launcher (if any)
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result
pub fn ohitmon(
    target: &mut Monster,
    obj: &Object,
    launcher_damage_bonus: i32,
    rng: &mut GameRng,
) -> CombatResult {
    // Calculate base damage from object
    let dice = if obj.damage_dice > 0 {
        obj.damage_dice
    } else {
        1
    };
    let sides = if obj.damage_sides > 0 {
        obj.damage_sides
    } else {
        4
    };
    let mut damage = rng.dice(dice as u32, sides as u32) as i32;

    // Add enchantment
    damage += obj.enchantment as i32;

    // Add launcher bonus
    damage += launcher_damage_bonus;

    // Silver object bonus (check name for "silver")
    if mon_hates_silver(target) {
        if obj
            .name
            .as_ref()
            .map_or(false, |n| n.to_lowercase().contains("silver"))
        {
            damage += rng.rnd(20) as i32;
        }
    }

    // Minimum damage
    damage = damage.max(1);

    target.hp -= damage;

    CombatResult {
        hit: true,
        defender_died: target.hp <= 0,
        attacker_died: false,
        damage,
        special_effect: None,
    }
}

/// Apply special damage effects to monster (damageum in C).
///
/// Handles special damage types like poison, disease, etc.
///
/// # Arguments
/// * `player` - The attacking player
/// * `target` - The monster being damaged
/// * `damage_type` - Type of special damage
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result with special effects
pub fn damageum(
    _player: &You,
    target: &mut Monster,
    damage_type: DamageType,
    rng: &mut GameRng,
) -> CombatResult {
    let mut damage = 0;
    let mut special_effect = None;

    match damage_type {
        DamageType::Fire => {
            if !target.resists_fire() {
                damage = rng.dice(2, 6) as i32;
                // No specific burning effect in CombatEffect, damage is enough
            }
        }
        DamageType::Cold => {
            if !target.resists_cold() {
                damage = rng.dice(2, 6) as i32;
                special_effect = Some(super::CombatEffect::Slowed);
            }
        }
        DamageType::Electric => {
            if !target.resists_elec() {
                damage = rng.dice(2, 6) as i32;
                special_effect = Some(super::CombatEffect::Stunned);
            }
        }
        DamageType::Poison => {
            if !target.resists_poison() {
                damage = rng.dice(1, 6) as i32;
                special_effect = Some(super::CombatEffect::Poisoned);
            }
        }
        DamageType::Acid => {
            if !target.resists_acid() {
                damage = rng.dice(1, 6) as i32;
            }
        }
        DamageType::DrainLife => {
            // Simplified: no drain resistance check, just apply damage
            damage = rng.dice(1, 4) as i32;
            // Reduce max HP
            target.hp_max = (target.hp_max - 1).max(1);
            special_effect = Some(super::CombatEffect::Drained);
        }
        DamageType::Stone => {
            if !target.resists_stone() {
                // Instant petrification
                target.hp = 0;
                special_effect = Some(super::CombatEffect::Petrifying);
            }
        }
        _ => {
            // Physical or unhandled type - no special damage
        }
    }

    if damage > 0 {
        target.hp -= damage;
    }

    CombatResult {
        hit: damage > 0 || special_effect.is_some(),
        defender_died: target.hp <= 0,
        attacker_died: false,
        damage,
        special_effect,
    }
}

/// Miss a monster (missum in C).
///
/// Called when player misses a monster - may have side effects.
///
/// # Arguments
/// * `player` - The attacking player
/// * `target` - The monster that was missed
/// * `weapon` - The weapon used
/// * `rng` - Random number generator
///
/// # Returns
/// Message about the miss
pub fn missum(
    _player: &You,
    target: &Monster,
    weapon: Option<&Object>,
    _rng: &mut GameRng,
) -> String {
    let weapon_name = weapon
        .map(|w| w.display_name())
        .unwrap_or_else(|| "bare hands".to_string());
    format!("You miss the {} with your {}.", target.name, weapon_name)
}

/// Use a mirror against a shade (shade_glare in C).
///
/// Mirrors can harm shades that look into them.
///
/// # Arguments
/// * `target` - The shade being affected
/// * `is_shade` - Whether the target is a shade (caller must determine)
/// * `rng` - Random number generator
///
/// # Returns
/// Whether the shade was affected and damage dealt
pub fn shade_glare(target: &mut Monster, is_shade: bool, rng: &mut GameRng) -> (bool, i32) {
    // Only shades are vulnerable
    if !is_shade {
        return (false, 0);
    }

    // Check if shade looks (random chance)
    if rng.one_in(2) {
        // Shade sees its own reflection and is damaged
        let damage = rng.dice(2, 6) as i32;
        target.hp -= damage;
        (true, damage)
    } else {
        // Shade averts its gaze
        (false, 0)
    }
}

/// Check if a monster hates silver by name heuristic (fallback).
///
/// Prefer the flags-based mon_hates_silver() above when MonsterFlags are available.
pub fn mon_hates_silver_by_name(monster: &Monster) -> bool {
    let name_lower = monster.name.to_lowercase();

    // Check for demon-like names
    let is_demon = name_lower.contains("demon")
        || name_lower.contains("devil")
        || name_lower.contains("incubus")
        || name_lower.contains("succubus");

    // Check for undead names
    let is_undead = name_lower.contains("zombie")
        || name_lower.contains("skeleton")
        || name_lower.contains("vampire")
        || name_lower.contains("wraith")
        || name_lower.contains("ghost")
        || name_lower.contains("shade")
        || name_lower.contains("lich")
        || name_lower.contains("mummy");

    // Check for werecreatures
    let is_were = name_lower.starts_with("were");

    is_demon || is_undead || is_were
}

/// Check if something hates silver in general (hates_silver in C).
///
/// # Arguments
/// * `is_demon` - Whether the creature is a demon
/// * `is_undead` - Whether the creature is undead
/// * `is_were` - Whether the creature is a werecreature
///
/// # Returns
/// Whether the creature hates silver
pub fn hates_silver(is_demon: bool, is_undead: bool, is_were: bool) -> bool {
    is_demon || is_undead || is_were
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::artifact::{
        Artifact, ArtifactAlignment, ArtifactFlags, InvokeProperty,
    };
    use crate::combat::{AmmunitionCount, Attack, AttackType, DamageType, RangedAttack, RangedWeaponType};
    use crate::monster::{MonsterId, MonsterFlags, MonsterResistances};
    use crate::object::{BucStatus, ObjectClass, ObjectId};
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

    fn test_weapon() -> Object {
        let mut w = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        w.damage_dice = 1;
        w.damage_sides = 8;
        w
    }

    fn test_silver_weapon() -> Object {
        let mut w = Object::new(ObjectId(2), 0, ObjectClass::Weapon);
        w.damage_dice = 1;
        w.damage_sides = 6;
        w
    }

    fn test_blessed_weapon() -> Object {
        let mut w = Object::new(ObjectId(3), 0, ObjectClass::Weapon);
        w.damage_dice = 1;
        w.damage_sides = 8;
        w.buc = BucStatus::Blessed;
        w
    }

    fn test_undead_monster() -> Monster {
        let mut m = Monster::new(MonsterId(2), 50, 3, 3);
        m.name = "zombie".to_string();
        m.hp = 30;
        m.hp_max = 30;
        m.level = 3;
        m.flags = MonsterFlags::UNDEAD;
        m
    }

    fn test_demon_monster() -> Monster {
        let mut m = Monster::new(MonsterId(3), 60, 4, 4);
        m.name = "imp".to_string();
        m.hp = 40;
        m.hp_max = 40;
        m.level = 5;
        m.flags = MonsterFlags::DEMON;
        m
    }

    fn test_were_monster() -> Monster {
        let mut m = Monster::new(MonsterId(4), 70, 6, 6);
        m.name = "werewolf".to_string();
        m.hp = 25;
        m.hp_max = 25;
        m.level = 4;
        m.flags = MonsterFlags::WERE;
        m
    }

    fn test_artifacts() -> Vec<Artifact> {
        vec![
            Artifact {
                name: "Frost Brand",
                otyp: 10,
                spfx: ArtifactFlags::RESTR
                    .union(ArtifactFlags::ATTK)
                    .union(ArtifactFlags::DEFN),
                cspfx: ArtifactFlags::NONE,
                mtype: 0,
                attk: Attack::new(AttackType::None, DamageType::Cold, 5, 0),
                defn: Attack::new(AttackType::None, DamageType::Cold, 0, 0),
                cary: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
                inv_prop: InvokeProperty::None,
                alignment: ArtifactAlignment::None,
                role: -1,
                race: -1,
                cost: 3000,
                color: 0,
            },
            Artifact {
                name: "Stormbringer",
                otyp: 20,
                spfx: ArtifactFlags::RESTR
                    .union(ArtifactFlags::ATTK)
                    .union(ArtifactFlags::DEFN)
                    .union(ArtifactFlags::INTEL)
                    .union(ArtifactFlags::DRLI),
                cspfx: ArtifactFlags::NONE,
                mtype: 0,
                attk: Attack::new(AttackType::None, DamageType::DrainLife, 5, 2),
                defn: Attack::new(AttackType::None, DamageType::DrainLife, 0, 0),
                cary: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
                inv_prop: InvokeProperty::None,
                alignment: ArtifactAlignment::Chaotic,
                role: -1,
                race: -1,
                cost: 8000,
                color: 0,
            },
        ]
    }

    // ======== To-hit tests (preserved from original) ========

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
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 3);

        // Low strength (5) gives -2 to-hit
        player.attr_current.set(Attribute::Strength, 5);
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 0);
    }

    #[test]
    fn test_to_hit_with_dexterity_bonus() {
        let mut player = test_player();
        let monster = test_monster();

        player.attr_current.set(Attribute::Dexterity, 18);
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_to_hit_with_luck() {
        let mut player = test_player();
        let monster = test_monster();

        player.luck = 5;
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 7);
    }

    #[test]
    fn test_to_hit_vs_sleeping_monster() {
        let player = test_player();
        let mut monster = test_monster();

        monster.state.sleeping = true;
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 4);
    }

    #[test]
    fn test_to_hit_vs_stunned_monster() {
        let player = test_player();
        let mut monster = test_monster();

        monster.state.stunned = true;
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 6);
    }

    #[test]
    fn test_to_hit_vs_fleeing_monster() {
        let player = test_player();
        let mut monster = test_monster();

        monster.state.fleeing = true;
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 4);
    }

    #[test]
    fn test_to_hit_confused_player() {
        let mut player = test_player();
        let monster = test_monster();

        player.confused_timeout = 10;
        let to_hit = calculate_to_hit(&player, &monster, None);
        assert_eq!(to_hit, 0);
    }

    #[test]
    fn test_to_hit_with_enchanted_weapon() {
        let player = test_player();
        let monster = test_monster();

        let mut weapon = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        weapon.enchantment = 3;

        let to_hit = calculate_to_hit(&player, &monster, Some(&weapon));
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_attack_hits_mechanics() {
        let mut rng = GameRng::new(42);

        // High to-hit vs poor AC: always hits
        let high_to_hit = 10;
        let mut hits = 0;
        for _ in 0..100 {
            if attack_hits(high_to_hit, 10, &mut rng) {
                hits += 1;
            }
        }
        assert_eq!(hits, 100);

        // Low to-hit vs good AC: never hits
        let low_to_hit = -5;
        hits = 0;
        for _ in 0..100 {
            if attack_hits(low_to_hit, -5, &mut rng) {
                hits += 1;
            }
        }
        assert_eq!(hits, 0);
    }

    // ======== Simple attack tests (preserved from original) ========

    #[test]
    fn test_damage_with_strength_bonus() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        let mut rng = GameRng::new(42);

        player.attr_current.set(Attribute::Strength, 17);

        // Bare hands: 1d2 (1-2) + str attr bonus (str_bonus=3, dex_bonus=0, Bare: 3+0=3)
        // + weapon_vs_armor_bonus(Bare, 10) = -2
        // = 1-2 + 3 - 2 = 2-3
        // With Unskilled graze (0.5x on low rolls), min can be 1
        // skill_enhanced_damage adds 0 (Unskilled damage_bonus = 0)
        let mut total_damage = 0;
        for _ in 0..100 {
            monster.hp = 100;
            let result = player_attack_monster(&mut player, &mut monster, None, &mut rng);
            if result.hit {
                assert!(
                    result.damage >= 1 && result.damage <= 4,
                    "Damage {} not in expected range 1-4 for str 17 unarmed",
                    result.damage
                );
                total_damage += result.damage;
            }
        }
        assert!(total_damage > 0, "Should have dealt some damage");
    }

    #[test]
    fn test_damage_with_weapon() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        let mut rng = GameRng::new(42);

        // Create a weapon: long sword (1d8 damage), +2 enchantment
        // Defaults to WeaponSkill::Sword (no name set)
        // str=10,dex=10: attr_bonus = 0, weapon_vs_armor_bonus(Sword, 10) = 0
        // base: 1d8 (1-8) + 0 + enchant(2) + 0 + 0 = 3-10
        // With Unskilled graze (0.5x on low rolls), min can be 1
        let mut weapon = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        weapon.damage_dice = 1;
        weapon.damage_sides = 8;
        weapon.enchantment = 2;

        for _ in 0..100 {
            monster.hp = 100;
            let result =
                player_attack_monster(&mut player, &mut monster, Some(&weapon), &mut rng);
            if result.hit {
                assert!(
                    result.damage >= 1 && result.damage <= 10,
                    "Damage {} not in expected range 1-10 for 1d8+2 weapon",
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

        player.attr_current.set(Attribute::Strength, 5);

        for _ in 0..100 {
            monster.hp = 100;
            let result = player_attack_monster(&mut player, &mut monster, None, &mut rng);
            if result.hit {
                assert!(result.damage >= 1, "Damage {} should be at least 1", result.damage);
            }
        }
    }

    #[test]
    fn test_to_hit_with_weapon_bonus() {
        let player = test_player();
        let monster = test_monster();

        // Create a weapon with base to-hit bonus
        let mut weapon = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        weapon.weapon_tohit = 2;
        weapon.enchantment = 1;

        let to_hit = calculate_to_hit(&player, &monster, Some(&weapon));
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_player_vs_monster_ac() {
        let player = test_player();
        let mut rng = GameRng::new(42);

        let mut monster_good_ac = test_monster();
        monster_good_ac.ac = -5;
        monster_good_ac.hp = 100;

        let mut monster_poor_ac = test_monster();
        monster_poor_ac.ac = 10;
        monster_poor_ac.hp = 100;

        let mut hits_good_ac = 0;
        let mut hits_poor_ac = 0;

        for _ in 0..1000 {
            monster_good_ac.hp = 100;
            monster_poor_ac.hp = 100;

            let result =
                player_attack_monster(&mut player.clone(), &mut monster_good_ac, None, &mut rng);
            if result.hit {
                hits_good_ac += 1;
            }

            let result =
                player_attack_monster(&mut player.clone(), &mut monster_poor_ac, None, &mut rng);
            if result.hit {
                hits_poor_ac += 1;
            }
        }

        assert!(
            hits_poor_ac > hits_good_ac,
            "Should hit AC 10 more than AC -5: {} vs {}",
            hits_poor_ac, hits_good_ac
        );
        assert_eq!(hits_poor_ac, 1000, "Should always hit AC 10");
    }

    // ======== New tests for expanded combat ========

    #[test]
    fn test_greatest_erosion() {
        let mut obj = Object::default();
        obj.erosion1 = 0;
        obj.erosion2 = 0;
        assert_eq!(greatest_erosion(&obj), 0);

        obj.erosion1 = 2;
        obj.erosion2 = 1;
        assert_eq!(greatest_erosion(&obj), 2);

        obj.erosion1 = 1;
        obj.erosion2 = 3;
        assert_eq!(greatest_erosion(&obj), 3);
    }

    #[test]
    fn test_maybe_erode_weapon_protected() {
        let mut weapon = test_weapon();
        let mut rng = GameRng::new(42);

        // Erosion-proof weapon should never erode
        weapon.erosion_proof = true;
        for _ in 0..100 {
            assert!(!maybe_erode_weapon(&mut weapon, 0, &mut rng));
        }
        assert_eq!(weapon.erosion1, 0);

        // Greased weapon should never erode
        weapon.erosion_proof = false;
        weapon.greased = true;
        for _ in 0..100 {
            assert!(!maybe_erode_weapon(&mut weapon, 0, &mut rng));
        }
        assert_eq!(weapon.erosion1, 0);
    }

    #[test]
    fn test_maybe_erode_weapon_cap() {
        let mut weapon = test_weapon();
        let mut rng = GameRng::new(42);

        // Set to max erosion
        weapon.erosion1 = MAX_ERODE;
        for _ in 0..100 {
            assert!(!maybe_erode_weapon(&mut weapon, 0, &mut rng));
        }
        assert_eq!(weapon.erosion1, MAX_ERODE);
    }

    #[test]
    fn test_mon_hates_silver_undead() {
        let zombie = test_undead_monster();
        assert!(mon_hates_silver(&zombie));
    }

    #[test]
    fn test_mon_hates_silver_demon() {
        let demon = test_demon_monster();
        assert!(mon_hates_silver(&demon));
    }

    #[test]
    fn test_mon_hates_silver_were() {
        let were = test_were_monster();
        assert!(mon_hates_silver(&were));
    }

    #[test]
    fn test_mon_hates_silver_normal() {
        let m = test_monster();
        assert!(!mon_hates_silver(&m));
    }

    #[test]
    fn test_silver_damage_vs_vulnerable() {
        let mut rng = GameRng::new(42);
        let zombie = test_undead_monster();

        // Should deal 1-20 extra damage
        let dmg = silver_damage(&zombie, &mut rng);
        assert!(dmg >= 1 && dmg <= 20);
    }

    #[test]
    fn test_silver_damage_vs_normal() {
        let mut rng = GameRng::new(42);
        let m = test_monster();

        // Should deal 0 extra damage
        let dmg = silver_damage(&m, &mut rng);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_buc_damage_blessed_vs_undead() {
        let mut rng = GameRng::new(42);
        let weapon = test_blessed_weapon();
        let zombie = test_undead_monster();

        let bonus = buc_damage_bonus(&weapon, &zombie, &mut rng);
        assert!(bonus >= 1 && bonus <= 4);
    }

    #[test]
    fn test_buc_damage_blessed_vs_demon() {
        let mut rng = GameRng::new(42);
        let weapon = test_blessed_weapon();
        let demon = test_demon_monster();

        let bonus = buc_damage_bonus(&weapon, &demon, &mut rng);
        assert!(bonus >= 1 && bonus <= 4);
    }

    #[test]
    fn test_buc_damage_uncursed_vs_undead() {
        let mut rng = GameRng::new(42);
        let weapon = test_weapon(); // uncursed
        let zombie = test_undead_monster();

        // Uncursed gives no bonus
        let bonus = buc_damage_bonus(&weapon, &zombie, &mut rng);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_buc_damage_blessed_vs_normal() {
        let mut rng = GameRng::new(42);
        let weapon = test_blessed_weapon();
        let m = test_monster();

        // Normal monster gets no bonus from blessed
        let bonus = buc_damage_bonus(&weapon, &m, &mut rng);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_dmgval_basic() {
        let mut rng = GameRng::new(42);
        let weapon = test_weapon(); // 1d8

        let m = test_monster();

        // Should get 1-8 damage (no enchantment, no bonuses)
        for _ in 0..100 {
            let dmg = dmgval(&weapon, Material::Iron, &m, false, &mut rng);
            assert!(dmg >= 1 && dmg <= 8, "dmgval {} out of range", dmg);
        }
    }

    #[test]
    fn test_dmgval_with_enchantment() {
        let mut rng = GameRng::new(42);
        let mut weapon = test_weapon();
        weapon.enchantment = 3;

        let m = test_monster();

        // 1d8 + 3 = 4-11
        for _ in 0..100 {
            let dmg = dmgval(&weapon, Material::Iron, &m, false, &mut rng);
            assert!(dmg >= 4 && dmg <= 11, "dmgval {} out of range 4-11", dmg);
        }
    }

    #[test]
    fn test_dmgval_with_erosion() {
        let mut rng = GameRng::new(42);
        let mut weapon = test_weapon();
        weapon.erosion1 = 2; // rusty

        let m = test_monster();

        // 1d8 - 2 erosion = min 1
        for _ in 0..100 {
            let dmg = dmgval(&weapon, Material::Iron, &m, false, &mut rng);
            assert!(dmg >= 1, "dmgval {} should be at least 1", dmg);
        }
    }

    #[test]
    fn test_dmgval_silver_vs_undead() {
        let mut rng = GameRng::new(42);
        let weapon = test_silver_weapon(); // 1d6

        let zombie = test_undead_monster();

        // 1d6 + 1d20 silver = 2-26
        let mut got_high = false;
        for _ in 0..100 {
            let dmg = dmgval(&weapon, Material::Silver, &zombie, false, &mut rng);
            assert!(dmg >= 2, "dmgval {} should be >= 2", dmg);
            if dmg > 10 {
                got_high = true;
            }
        }
        assert!(got_high, "Silver should deal high damage to undead");
    }

    #[test]
    fn test_dmgval_blessed_vs_demon() {
        let mut rng = GameRng::new(42);
        let weapon = test_blessed_weapon(); // 1d8 blessed

        let demon = test_demon_monster();

        // 1d8 + 1d4 blessed = 2-12
        let mut got_blessed_bonus = false;
        for _ in 0..100 {
            let dmg = dmgval(&weapon, Material::Iron, &demon, false, &mut rng);
            assert!(dmg >= 2, "dmgval {} should be >= 2", dmg);
            if dmg > 8 {
                got_blessed_bonus = true;
            }
        }
        assert!(
            got_blessed_bonus,
            "Blessed should deal bonus damage to demon"
        );
    }

    #[test]
    fn test_dmgval_normal_monster() {
        let mut rng = GameRng::new(42);
        let weapon = test_blessed_weapon(); // 1d8 blessed

        let m = test_monster(); // Normal monster

        // 1d8 only (blessed bonus doesn't apply to normal monsters)
        for _ in 0..100 {
            let dmg = dmgval(&weapon, Material::Iron, &m, false, &mut rng);
            assert!(dmg >= 1 && dmg <= 8, "dmgval {} should be 1-8", dmg);
        }
    }

    #[test]
    fn test_hmon_basic_weapon() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        monster.hp_max = 100;
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut weapon = test_weapon();
        let result = hmon(
            &mut player,
            &mut monster,
            Some(&mut weapon),
            Some(Material::Iron),
            AttackSource::Melee,
            10,
            &artifacts,
            &mut rng,
        );

        assert!(result.hit);
        assert!(result.damage >= 1);
        assert!(monster.hp < 100);
    }

    #[test]
    fn test_hmon_bare_hand() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        monster.hp_max = 100;
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let result = hmon(
            &mut player,
            &mut monster,
            None,
            None,
            AttackSource::Melee,
            10,
            &artifacts,
            &mut rng,
        );

        assert!(result.hit);
        assert!(result.damage >= 1);
    }

    #[test]
    fn test_hmon_kills_monster() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 25);
        player.exp_level = 20;
        player.damage_bonus = 10;

        let mut monster = test_monster();
        monster.hp = 3;
        monster.hp_max = 3;
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut weapon = test_weapon();
        weapon.enchantment = 5;

        let result = hmon(
            &mut player,
            &mut monster,
            Some(&mut weapon),
            Some(Material::Iron),
            AttackSource::Melee,
            10,
            &artifacts,
            &mut rng,
        );

        assert!(result.hit);
        assert!(result.defender_died);
    }

    #[test]
    fn test_hmon_poisoned_weapon() {
        let mut player = test_player();
        let artifacts = test_artifacts();

        // Run many trials to test poison effects
        let mut got_poison = false;
        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            let mut monster = test_monster();
            monster.hp = 200;
            monster.hp_max = 200;

            let mut weapon = test_weapon();
            weapon.poisoned = true;

            let result = hmon(
                &mut player,
                &mut monster,
                Some(&mut weapon),
                Some(Material::Iron),
                AttackSource::Melee,
                10,
                &artifacts,
                &mut rng,
            );

            if result.effects.contains(&CombatEffect::Poisoned) {
                got_poison = true;
                break;
            }
        }
        assert!(got_poison, "Poison should trigger occasionally");
    }

    #[test]
    fn test_hmon_poison_resistant_monster() {
        let mut player = test_player();
        let artifacts = test_artifacts();

        // Poison-resistant monster should not take poison damage
        for seed in 0..50u64 {
            let mut rng = GameRng::new(seed);
            let mut monster = test_monster();
            monster.hp = 200;
            monster.hp_max = 200;
            monster.resistances = MonsterResistances::POISON;

            let mut weapon = test_weapon();
            weapon.poisoned = true;

            let result = hmon(
                &mut player,
                &mut monster,
                Some(&mut weapon),
                Some(Material::Iron),
                AttackSource::Melee,
                10,
                &artifacts,
                &mut rng,
            );

            assert!(
                !result.effects.contains(&CombatEffect::Poisoned),
                "Poison-resistant monster should not be poisoned"
            );
        }
    }

    #[test]
    fn test_hmon_wakes_sleeping_monster() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        monster.hp_max = 100;
        monster.state.sleeping = true;
        monster.sleep_timeout = 10;
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let result = hmon(
            &mut player,
            &mut monster,
            None,
            None,
            AttackSource::Melee,
            10,
            &artifacts,
            &mut rng,
        );

        assert!(result.hit);
        // Monster should wake up after being hit
        assert!(!monster.state.sleeping);
        assert_eq!(monster.sleep_timeout, 0);
    }

    #[test]
    fn test_hmon_with_artifact() {
        let mut player = test_player();
        let mut monster = test_monster();
        monster.hp = 100;
        monster.hp_max = 100;
        monster.name = "orc".to_string();
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        // Frost Brand artifact (index 1)
        let mut weapon = Object::new(ObjectId(10), 10, ObjectClass::Weapon);
        weapon.damage_dice = 1;
        weapon.damage_sides = 8;
        weapon.artifact = 1;
        weapon.name = Some("Frost Brand".to_string());

        let result = hmon(
            &mut player,
            &mut monster,
            Some(&mut weapon),
            Some(Material::Iron),
            AttackSource::Melee,
            10,
            &artifacts,
            &mut rng,
        );

        assert!(result.hit);
        // Artifact should produce messages
        assert!(result.artifact_messaged);
    }

    #[test]
    fn test_hmon_acid_erodes_weapon() {
        let mut player = test_player();
        let artifacts = test_artifacts();

        // Try many seeds to find one where acid erodes
        let mut got_erosion = false;
        for seed in 0..500u64 {
            let mut rng = GameRng::new(seed);
            let mut monster = test_monster();
            monster.hp = 200;
            monster.hp_max = 200;
            monster.resistances = MonsterResistances::ACID;

            let mut weapon = test_weapon();

            let result = hmon(
                &mut player,
                &mut monster,
                Some(&mut weapon),
                Some(Material::Iron),
                AttackSource::Melee,
                10,
                &artifacts,
                &mut rng,
            );

            if weapon.erosion2 > 0 {
                got_erosion = true;
                assert!(result
                    .messages
                    .iter()
                    .any(|m| m.contains("corroded")));
                break;
            }
        }
        assert!(got_erosion, "Acid monster should erode weapon eventually");
    }

    #[test]
    fn test_two_weapon_hit_basic() {
        let mut player = test_player();
        player.exp_level = 10; // Good to-hit
        player.attr_current.set(Attribute::Strength, 18);

        let mut monster = test_monster();
        monster.hp = 200;
        monster.hp_max = 200;
        monster.ac = 10; // Easy to hit

        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut primary = test_weapon();
        let mut secondary = test_weapon();

        let (primary_result, secondary_result) = two_weapon_hit(
            &mut player,
            &mut monster,
            &mut primary,
            Material::Iron,
            &mut secondary,
            Material::Iron,
            &artifacts,
            &mut rng,
        );

        assert!(primary_result.hit);
        // Secondary should also attempt (monster has easy AC)
        assert!(secondary_result.is_some());
    }

    #[test]
    fn test_two_weapon_no_secondary_on_miss() {
        let mut player = test_player();
        // Very low to-hit
        player.exp_level = 1;
        player.attr_current.set(Attribute::Strength, 3);
        player.attr_current.set(Attribute::Dexterity, 3);
        player.luck = -10;

        let mut monster = test_monster();
        monster.hp = 200;
        monster.hp_max = 200;
        monster.ac = -10; // Very hard to hit

        let artifacts = test_artifacts();

        // Find a seed where we miss
        let mut found_miss = false;
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let mut primary = test_weapon();
            let mut secondary = test_weapon();

            let (primary_result, secondary_result) = two_weapon_hit(
                &mut player,
                &mut monster,
                &mut primary,
                Material::Iron,
                &mut secondary,
                Material::Iron,
                &artifacts,
                &mut rng,
            );

            if !primary_result.hit {
                assert!(
                    secondary_result.is_none(),
                    "No secondary attack when primary misses"
                );
                found_miss = true;
                break;
            }
        }
        assert!(found_miss, "Should find a miss with bad stats");
    }

    #[test]
    fn test_cleave_targets_basic() {
        // Player at (5,5), target at (6,5) = East
        let targets = cleave_targets(5, 5, 6, 5, true);

        // Center should be the target
        assert_eq!(targets[1].x, 6);
        assert_eq!(targets[1].y, 5);

        // Left (clockwise: NE) and right (SE)
        assert_eq!(targets[0].x, 6);
        assert_eq!(targets[0].y, 4); // NE
        assert_eq!(targets[2].x, 6);
        assert_eq!(targets[2].y, 6); // SE
    }

    #[test]
    fn test_cleave_targets_north() {
        // Player at (5,5), target at (5,4) = North
        let targets = cleave_targets(5, 5, 5, 4, true);

        // Center: (5,4) = North
        assert_eq!(targets[1].x, 5);
        assert_eq!(targets[1].y, 4);

        // Clockwise: left = NW, right = NE
        assert_eq!(targets[0].x, 4);
        assert_eq!(targets[0].y, 4); // NW
        assert_eq!(targets[2].x, 6);
        assert_eq!(targets[2].y, 4); // NE
    }

    #[test]
    fn test_cleave_targets_counterclockwise() {
        // Same as above but counterclockwise
        let targets = cleave_targets(5, 5, 6, 5, false);

        // Center should be the target
        assert_eq!(targets[1].x, 6);
        assert_eq!(targets[1].y, 5);

        // Counterclockwise: left = SE, right = NE
        assert_eq!(targets[0].x, 6);
        assert_eq!(targets[0].y, 6); // SE
        assert_eq!(targets[2].x, 6);
        assert_eq!(targets[2].y, 4); // NE
    }

    #[test]
    fn test_creature_vulnerability_silver() {
        let zombie = test_undead_monster();
        let mult = creature_vulnerability(&zombie, Material::Silver);
        assert!(mult > 1.0);
    }

    #[test]
    fn test_creature_vulnerability_normal() {
        let m = test_monster();
        let mult = creature_vulnerability(&m, Material::Iron);
        assert!((mult - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_creature_vulnerability_thick_hide() {
        let mut m = test_monster();
        m.flags = MonsterFlags::THICK_HIDE;

        // Metallic weapon against thick hide: normal damage
        let mult = creature_vulnerability(&m, Material::Iron);
        assert!((mult - 1.0).abs() < f32::EPSILON);

        // Soft material against thick hide: reduced damage
        let mult = creature_vulnerability(&m, Material::Leather);
        assert!(mult < 1.0);
    }

    #[test]
    fn test_special_weapon_effects_poison() {
        let mut weapon = test_weapon();
        weapon.poisoned = true;

        let m = test_monster();
        let effects = special_weapon_effects(&weapon, Material::Iron, &m);
        assert!(effects.contains(&CombatEffect::Poisoned));
    }

    #[test]
    fn test_special_weapon_effects_acid_monster() {
        let weapon = test_weapon();

        let mut m = test_monster();
        m.resistances = MonsterResistances::ACID;

        let effects = special_weapon_effects(&weapon, Material::Iron, &m);
        assert!(effects.contains(&CombatEffect::ArmorCorroded));
    }

    #[test]
    fn test_bare_hand_damage_monk() {
        let mut player = test_player();
        player.role = crate::player::Role::Monk;
        player.exp_level = 10;
        let mut rng = GameRng::new(42);

        // Monk at level 10: 1d(10/2+1) = 1d6
        let mut min_seen = i32::MAX;
        let mut max_seen = i32::MIN;
        for _ in 0..100 {
            let dmg = bare_hand_damage(&player, &mut rng);
            min_seen = min_seen.min(dmg);
            max_seen = max_seen.max(dmg);
        }
        assert!(min_seen >= 1);
        assert!(max_seen <= 6);
    }

    #[test]
    fn test_bare_hand_damage_non_monk() {
        let player = test_player(); // Not monk
        let mut rng = GameRng::new(42);

        // Non-monks: 1d2
        for _ in 0..100 {
            let dmg = bare_hand_damage(&player, &mut rng);
            assert!(dmg >= 1 && dmg <= 2);
        }
    }

    #[test]
    fn test_direction_index_all_directions() {
        assert_eq!(direction_index(0, -1), 0); // N
        assert_eq!(direction_index(1, -1), 1); // NE
        assert_eq!(direction_index(1, 0), 2); // E
        assert_eq!(direction_index(1, 1), 3); // SE
        assert_eq!(direction_index(0, 1), 4); // S
        assert_eq!(direction_index(-1, 1), 5); // SW
        assert_eq!(direction_index(-1, 0), 6); // W
        assert_eq!(direction_index(-1, -1), 7); // NW
    }

    #[test]
    fn test_artifact_to_hit_bonus_no_artifact() {
        let weapon = test_weapon();
        let m = test_monster();
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let bonus = artifact_to_hit_bonus(&weapon, &m, &artifacts, &mut rng);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_hmon_fleeing_when_wounded() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 18);
        let artifacts = test_artifacts();

        // Run many trials, look for a case where monster starts fleeing
        let mut got_fleeing = false;
        for seed in 0..500u64 {
            let mut rng = GameRng::new(seed);
            let mut monster = test_monster();
            monster.hp = 5; // Low HP
            monster.hp_max = 20;
            monster.state.fleeing = false;

            let result = hmon(
                &mut player,
                &mut monster,
                None,
                None,
                AttackSource::Melee,
                10,
                &artifacts,
                &mut rng,
            );

            if !result.defender_died && monster.state.fleeing {
                got_fleeing = true;
                break;
            }
        }
        assert!(got_fleeing, "Wounded monster should flee sometimes");
    }

    #[test]
    fn test_hit_result_to_combat_result() {
        let result = HitResult {
            hit: true,
            defender_died: false,
            attacker_died: false,
            damage: 5,
            messages: vec!["test".to_string()],
            effects: vec![CombatEffect::Poisoned],
            artifact_messaged: false,
        };

        let combat_result = result.to_combat_result();
        assert!(combat_result.hit);
        assert_eq!(combat_result.damage, 5);
        assert_eq!(combat_result.special_effect, Some(CombatEffect::Poisoned));
    }

    // Tests for new uhitm functions

    #[test]
    fn test_dbon_low_strength() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 5);
        assert_eq!(dbon(&player), -1, "Strength 5 should give -1 damage bonus");
    }

    #[test]
    fn test_dbon_average_strength() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 10);
        assert_eq!(dbon(&player), 0, "Strength 10 should give 0 damage bonus");
    }

    #[test]
    fn test_dbon_high_strength() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 18);
        assert_eq!(dbon(&player), 2, "Strength 18 should give +2 damage bonus");
    }

    #[test]
    fn test_dbon_exceptional_strength() {
        let mut player = test_player();
        // 18/50 is stored as 18 + 50 = 68 in the attribute system
        player.attr_current.set(Attribute::Strength, 68);
        assert_eq!(
            dbon(&player),
            3,
            "Strength 18/50 should give +3 damage bonus"
        );
    }

    #[test]
    fn test_dbon_polymorphed() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 18);
        player.monster_num = Some(10); // Polymorphed
        assert_eq!(
            dbon(&player),
            0,
            "Polymorphed player should get 0 strength bonus"
        );
    }

    #[test]
    fn test_joust_not_mounted() {
        let player = test_player();
        let weapon = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        let mut rng = GameRng::new(42);

        let result = joust(&player, false, false, true, true, &weapon, &mut rng);
        assert_eq!(
            result,
            JoustResult::NoJoust,
            "Should not joust when not mounted"
        );
    }

    #[test]
    fn test_joust_fumbling() {
        let player = test_player();
        let weapon = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        let mut rng = GameRng::new(42);

        let result = joust(&player, true, true, true, true, &weapon, &mut rng);
        assert_eq!(
            result,
            JoustResult::NoJoust,
            "Should not joust when fumbling"
        );
    }

    #[test]
    fn test_joust_not_lance() {
        let player = test_player();
        let weapon = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        let mut rng = GameRng::new(42);

        let result = joust(&player, true, false, false, true, &weapon, &mut rng);
        assert_eq!(
            result,
            JoustResult::NoJoust,
            "Should not joust without lance"
        );
    }

    #[test]
    fn test_shade_aware_silver() {
        let weapon = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        assert!(
            shade_aware(Some(&weapon), true, false, false),
            "Silver should affect shades"
        );
    }

    #[test]
    fn test_shade_aware_heavy() {
        let mut weapon = Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Weapon,
        );
        weapon.weight = 250; // Heavy object
        assert!(
            shade_aware(Some(&weapon), false, false, false),
            "Heavy objects should affect shades"
        );
    }

    #[test]
    fn test_shade_aware_mirror() {
        let weapon = Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Tool,
        );
        assert!(
            shade_aware(Some(&weapon), false, true, false),
            "Mirrors should affect shades"
        );
    }

    #[test]
    fn test_shade_aware_garlic() {
        let weapon = Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Food,
        );
        assert!(
            shade_aware(Some(&weapon), false, false, true),
            "Garlic should affect shades"
        );
    }

    #[test]
    fn test_shade_miss_not_shade() {
        let weapon = Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Weapon,
        );
        assert!(
            !shade_miss(false, Some(&weapon), false, false, false),
            "Non-shade should not cause shade miss"
        );
    }

    #[test]
    fn test_shade_miss_with_silver() {
        let weapon = Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Weapon,
        );
        assert!(
            !shade_miss(true, Some(&weapon), true, false, false),
            "Silver should hit shades"
        );
    }

    #[test]
    fn test_shade_miss_normal_weapon() {
        let weapon = Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Weapon,
        );
        assert!(
            shade_miss(true, Some(&weapon), false, false, false),
            "Normal weapon should miss shades"
        );
    }

    #[test]
    fn test_hates_silver_check() {
        assert!(
            hates_silver_check(true, false, false, false, false, false),
            "Werewolves hate silver"
        );
        assert!(
            hates_silver_check(false, true, false, false, false, false),
            "Vampires hate silver"
        );
        assert!(
            hates_silver_check(false, false, true, false, false, false),
            "Demons hate silver"
        );
        assert!(
            hates_silver_check(false, false, false, true, false, false),
            "Shades hate silver"
        );
        assert!(
            hates_silver_check(false, false, false, false, true, false),
            "Imps hate silver"
        );
        assert!(
            !hates_silver_check(false, false, false, false, true, true),
            "Tengu don't hate silver"
        );
        assert!(
            !hates_silver_check(false, false, false, false, false, false),
            "Normal monsters don't hate silver"
        );
    }

    #[test]
    fn test_silver_sears() {
        let mut rng = GameRng::new(42);

        let (damage, msg) = silver_sears(true, true, "the vampire", &mut rng);
        assert!(
            damage > 0,
            "Silver should deal damage to silver-hating monster"
        );
        assert!(msg.is_some(), "Should have searing message");
        assert!(
            msg.unwrap().contains("sears"),
            "Message should mention searing"
        );
    }

    #[test]
    fn test_silver_sears_no_effect() {
        let mut rng = GameRng::new(42);

        let (damage, msg) = silver_sears(false, true, "the vampire", &mut rng);
        assert_eq!(damage, 0, "Non-silver should not deal extra damage");
        assert!(msg.is_none(), "Should have no message");

        let (damage2, msg2) = silver_sears(true, false, "the orc", &mut rng);
        assert_eq!(
            damage2, 0,
            "Silver should not affect non-vulnerable monster"
        );
        assert!(msg2.is_none(), "Should have no message");
    }

    #[test]
    fn test_sticks() {
        assert!(sticks(true, false), "Mimics should stick");
        assert!(sticks(false, true), "Sticky monsters should stick");
        assert!(sticks(true, true), "Sticky mimics should stick");
        assert!(!sticks(false, false), "Normal monsters should not stick");
    }

    #[test]
    fn test_retouch_object() {
        assert_eq!(
            retouch_object(true, true),
            Some(1),
            "Silver should hurt silver-hating player"
        );
        assert_eq!(
            retouch_object(true, false),
            None,
            "Non-silver should not hurt"
        );
        assert_eq!(
            retouch_object(false, true),
            None,
            "Silver should not hurt non-vulnerable player"
        );
    }

    #[test]
    fn test_shade_miss_message() {
        let msg = shade_miss_message("You", Some("sword"), "the shade");
        assert!(
            msg.contains("passes harmlessly through"),
            "Message should describe passing through"
        );
        assert!(msg.contains("sword"), "Message should mention weapon");

        let msg2 = shade_miss_message("You", None, "the shade");
        assert!(
            msg2.contains("attack"),
            "Message should mention attack when no weapon"
        );
    }

    // ========================================================================
    // WEAPON PROFICIENCY TESTS
    // ========================================================================

    #[test]
    fn test_weapon_skill_to_skill_type_mapping() {
        use crate::player::SkillType;

        // Test all weapon skill mappings
        assert_eq!(
            weapon_skill_to_skill_type(WeaponSkill::Bare),
            SkillType::BareHanded
        );
        assert_eq!(
            weapon_skill_to_skill_type(WeaponSkill::Dagger),
            SkillType::Dagger
        );
        assert_eq!(
            weapon_skill_to_skill_type(WeaponSkill::Sword),
            SkillType::BroadSword
        );
        assert_eq!(weapon_skill_to_skill_type(WeaponSkill::Axe), SkillType::Axe);
        assert_eq!(weapon_skill_to_skill_type(WeaponSkill::Bow), SkillType::Bow);
        assert_eq!(
            weapon_skill_to_skill_type(WeaponSkill::Crossbow),
            SkillType::Crossbow
        );
        assert_eq!(
            weapon_skill_to_skill_type(WeaponSkill::Sling),
            SkillType::Sling
        );
    }

    #[test]
    fn test_player_skill_level_conversion() {
        use crate::player::SkillLevel as PlayerSkillLevel;

        // Test conversion from player system to combat system
        assert_eq!(
            player_skill_level_to_combat(PlayerSkillLevel::Restricted),
            SkillLevel::Unskilled
        );
        assert_eq!(
            player_skill_level_to_combat(PlayerSkillLevel::Unskilled),
            SkillLevel::Unskilled
        );
        assert_eq!(
            player_skill_level_to_combat(PlayerSkillLevel::Basic),
            SkillLevel::Basic
        );
        assert_eq!(
            player_skill_level_to_combat(PlayerSkillLevel::Skilled),
            SkillLevel::Skilled
        );
        assert_eq!(
            player_skill_level_to_combat(PlayerSkillLevel::Expert),
            SkillLevel::Expert
        );
        assert_eq!(
            player_skill_level_to_combat(PlayerSkillLevel::Master),
            SkillLevel::Master
        );
        assert_eq!(
            player_skill_level_to_combat(PlayerSkillLevel::GrandMaster),
            SkillLevel::Master
        );
    }

    #[test]
    fn test_get_player_weapon_skill_basic() {
        let player = test_player();

        // Default player starts with Unskilled in all weapons
        let skill = get_player_weapon_skill(&player, WeaponSkill::Sword);
        assert_eq!(
            skill,
            SkillLevel::Unskilled,
            "New player should be unskilled with sword"
        );
    }

    #[test]
    fn test_get_player_weapon_skill_different_weapons() {
        let mut player = test_player();

        // Manually set different skill levels for different weapons
        use crate::player::{SkillLevel as PlayerSkillLevel, SkillType};

        player.skills.get_mut(SkillType::Dagger).level = PlayerSkillLevel::Skilled;
        player.skills.get_mut(SkillType::BroadSword).level = PlayerSkillLevel::Expert;

        // Test that get_player_weapon_skill returns correct levels
        let dagger_skill = get_player_weapon_skill(&player, WeaponSkill::Dagger);
        assert_eq!(
            dagger_skill,
            SkillLevel::Skilled,
            "Should have skilled dagger proficiency"
        );

        let sword_skill = get_player_weapon_skill(&player, WeaponSkill::Sword);
        assert_eq!(
            sword_skill,
            SkillLevel::Expert,
            "Should have expert sword proficiency"
        );
    }

    #[test]
    fn test_update_weapon_proficiency_on_hit() {
        let mut player = test_player();
        let initial_practice = player
            .skills
            .get(crate::player::SkillType::BroadSword)
            .practice;

        // Simulate a hit with broad sword
        update_weapon_proficiency(&mut player, WeaponSkill::Sword, true, false);

        let new_practice = player
            .skills
            .get(crate::player::SkillType::BroadSword)
            .practice;

        assert!(
            new_practice > initial_practice,
            "Practice should increase on hit"
        );
        assert_eq!(
            new_practice - initial_practice,
            10,
            "Hit should award 10 practice points"
        );
    }

    #[test]
    fn test_update_weapon_proficiency_on_critical_hit() {
        let mut player = test_player();
        let initial_practice = player
            .skills
            .get(crate::player::SkillType::BroadSword)
            .practice;

        // Simulate a critical hit
        update_weapon_proficiency(&mut player, WeaponSkill::Sword, true, true);

        let new_practice = player
            .skills
            .get(crate::player::SkillType::BroadSword)
            .practice;

        assert_eq!(
            new_practice - initial_practice,
            15,
            "Critical hit should award 15 practice points"
        );
    }

    #[test]
    fn test_update_weapon_proficiency_on_miss() {
        let mut player = test_player();
        let initial_practice = player.skills.get(crate::player::SkillType::Dagger).practice;

        // Simulate a miss with dagger
        update_weapon_proficiency(&mut player, WeaponSkill::Dagger, false, false);

        let new_practice = player.skills.get(crate::player::SkillType::Dagger).practice;

        assert_eq!(
            new_practice - initial_practice,
            1,
            "Miss should award 1 practice point"
        );
    }

    #[test]
    fn test_update_weapon_proficiency_different_weapons() {
        let mut player = test_player();

        use crate::player::SkillType;

        let initial_sword_practice = player.skills.get(SkillType::BroadSword).practice;
        let initial_axe_practice = player.skills.get(SkillType::Axe).practice;

        // Hit with sword
        update_weapon_proficiency(&mut player, WeaponSkill::Sword, true, false);

        // Hit with axe
        update_weapon_proficiency(&mut player, WeaponSkill::Axe, true, false);

        let final_sword_practice = player.skills.get(SkillType::BroadSword).practice;
        let final_axe_practice = player.skills.get(SkillType::Axe).practice;

        // Both should have advanced by 10
        assert_eq!(final_sword_practice - initial_sword_practice, 10);
        assert_eq!(final_axe_practice - initial_axe_practice, 10);
    }

    #[test]
    fn test_skill_advancement_with_slots() {
        let mut player = test_player();

        use crate::player::{SkillLevel, SkillType};

        // Allow BroadSword to advance to Expert and start at Unskilled
        player.skills.set_max(SkillType::BroadSword, SkillLevel::Expert);
        player.skills.get_mut(SkillType::BroadSword).level = SkillLevel::Unskilled;
        // Give the player advancement slots
        player.skills.slots = 1;

        // Get a skill to its advancement threshold
        let skill = player.skills.get_mut(SkillType::BroadSword);
        while !skill.can_advance() {
            skill.add_practice(5);
        }

        let initial_level = player.skills.get(SkillType::BroadSword).level;

        // Now update proficiency to trigger advancement
        update_weapon_proficiency(&mut player, WeaponSkill::Sword, true, false);

        let final_level = player.skills.get(SkillType::BroadSword).level;

        // The skill should have advanced
        assert!(
            (final_level as u8) > (initial_level as u8),
            "Skill should advance when slots available"
        );
        assert_eq!(
            player.skills.slots, 0,
            "Advancement slot should be consumed"
        );
    }

    #[test]
    fn test_no_skill_advancement_without_slots() {
        let mut player = test_player();

        use crate::player::{SkillLevel, SkillType};

        // Allow Dagger to advance to Basic and start at Unskilled
        player.skills.set_max(SkillType::Dagger, SkillLevel::Basic);
        player.skills.get_mut(SkillType::Dagger).level = SkillLevel::Unskilled;
        // No advancement slots
        player.skills.slots = 0;

        // Get a skill to its advancement threshold
        let skill = player.skills.get_mut(SkillType::Dagger);
        while !skill.can_advance() {
            skill.add_practice(5);
        }

        let initial_level = player.skills.get(SkillType::Dagger).level;

        // Try to advance without slots
        update_weapon_proficiency(&mut player, WeaponSkill::Dagger, true, false);

        let final_level = player.skills.get(SkillType::Dagger).level;

        // The skill should NOT have advanced
        assert_eq!(
            final_level, initial_level,
            "Skill should not advance without slots"
        );
    }

    #[test]
    fn test_bare_handed_skill_tracking() {
        let mut player = test_player();

        use crate::player::SkillType;

        let initial_practice = player.skills.get(SkillType::BareHanded).practice;

        // Unarmed hit (no weapon)
        update_weapon_proficiency(&mut player, WeaponSkill::Bare, true, false);

        let final_practice = player.skills.get(SkillType::BareHanded).practice;

        assert_eq!(
            final_practice - initial_practice,
            10,
            "Bare-handed skill should track unarmed attacks"
        );
    }

    // ========================================================================
    // RANGED COMBAT TESTS
    // ========================================================================

    #[test]
    fn test_ranged_attack_struct_creation() {
        let ranged = RangedAttack {
            weapon_type: RangedWeaponType::Bow,
            distance: 6,
            skill_level: SkillLevel::Skilled,
            base_to_hit: 5,
        };

        assert!(ranged.in_range(), "Distance 6 should be in range for bow");
        assert_eq!(
            ranged.distance_penalty(),
            0,
            "Within optimal range should have 0 penalty"
        );
    }

    #[test]
    fn test_ranged_weapon_ranges() {
        // Test max ranges for different weapon types
        assert_eq!(RangedWeaponType::Bow.max_range(), 12);
        assert_eq!(RangedWeaponType::Crossbow.max_range(), 15);
        assert_eq!(RangedWeaponType::Sling.max_range(), 8);
        assert_eq!(RangedWeaponType::Thrown.max_range(), 5);

        // Test optimal ranges
        assert_eq!(RangedWeaponType::Bow.optimal_range(), 6);
        assert_eq!(RangedWeaponType::Crossbow.optimal_range(), 10);
        assert_eq!(RangedWeaponType::Sling.optimal_range(), 4);
        assert_eq!(RangedWeaponType::Thrown.optimal_range(), 2);
    }

    #[test]
    fn test_ranged_distance_penalty() {
        let bow_attack = RangedAttack {
            weapon_type: RangedWeaponType::Bow,
            distance: 10, // 4 squares beyond optimal (6)
            skill_level: SkillLevel::Basic,
            base_to_hit: 5,
        };

        // Penalty should be -(4/2) = -2
        assert_eq!(bow_attack.distance_penalty(), -2);
    }

    #[test]
    fn test_ranged_out_of_range() {
        let bow_attack = RangedAttack {
            weapon_type: RangedWeaponType::Bow,
            distance: 15, // Beyond max range of 12
            skill_level: SkillLevel::Skilled,
            base_to_hit: 5,
        };

        assert!(
            !bow_attack.in_range(),
            "Distance 15 should be out of range for bow"
        );
        assert_eq!(
            bow_attack.distance_penalty(),
            -20,
            "Out of range should have -20 penalty"
        );
    }

    #[test]
    fn test_ammunition_count_basic() {
        let mut ammo = AmmunitionCount::new(20, 30);

        assert!(ammo.has_ammo(), "Should have ammunition");
        assert_eq!(ammo.capacity_percent(), 66, "20/30 = ~66%");
    }

    #[test]
    fn test_ammunition_consumption() {
        let mut ammo = AmmunitionCount::new(5, 20);

        assert!(ammo.consume(), "Should consume ammunition when available");
        assert_eq!(ammo.count, 4, "Count should decrease by 1");

        // Consume until empty
        for _ in 0..4 {
            ammo.consume();
        }

        assert!(!ammo.consume(), "Should fail to consume when empty");
        assert_eq!(ammo.count, 0, "Count should be 0");
    }

    #[test]
    fn test_ammunition_recovery() {
        let mut ammo = AmmunitionCount::new(5, 20);

        ammo.recover(10); // Try to recover 10
        assert_eq!(ammo.count, 15, "Should recover up to capacity");

        ammo.recover(10); // Try to recover more
        assert_eq!(ammo.count, 20, "Should cap at capacity");
    }

    #[test]
    fn test_ammunition_status_messages() {
        let full = AmmunitionCount::new(20, 20);
        assert!(full.status_message().contains("Full"));

        let low = AmmunitionCount::new(2, 20);
        assert!(low.status_message().contains("Low"));

        let out = AmmunitionCount::new(0, 20);
        assert!(out.status_message().contains("Out of ammo"));
    }

    #[test]
    fn test_ammunition_is_low() {
        // is_low() checks count < capacity / 4, i.e. count < 5 for capacity 20
        let ammo = AmmunitionCount::new(4, 20); // 20% = below 25%
        assert!(ammo.is_low(), "4/20 should be considered low");

        let ammo2 = AmmunitionCount::new(10, 20); // 50% = above 25%
        assert!(!ammo2.is_low(), "10/20 should not be considered low");

        let ammo3 = AmmunitionCount::new(5, 20); // 25% = exactly at threshold, not below
        assert!(!ammo3.is_low(), "5/20 should not be considered low (not strictly below 25%)");
    }

    #[test]
    fn test_ammunition_requirement_for_bow() {
        let mut bow = Object::default();
        bow.object_type = 57; // Bow

        let requirement = ammunition_requirement_for_launcher(&bow);
        assert_eq!(requirement, Some((50, 20)), "Bow should require arrows");
    }

    #[test]
    fn test_ammunition_requirement_for_crossbow() {
        let mut crossbow = Object::default();
        crossbow.object_type = 59; // Crossbow

        let requirement = ammunition_requirement_for_launcher(&crossbow);
        assert_eq!(requirement, Some((63, 15)), "Crossbow should require bolts");
    }

    #[test]
    fn test_ammunition_requirement_for_sling() {
        let mut sling = Object::default();
        sling.object_type = 61; // Sling

        let requirement = ammunition_requirement_for_launcher(&sling);
        assert_eq!(requirement, Some((67, 30)), "Sling should require stones");
    }

    #[test]
    fn test_ammunition_requirement_no_launcher() {
        let not_launcher = Object::default();

        let requirement = ammunition_requirement_for_launcher(&not_launcher);
        assert_eq!(requirement, None, "Non-launcher should return None");
    }

    #[test]
    fn test_consume_ammunition() {
        let mut ammo = AmmunitionCount::new(10, 20);

        let consumed = consume_ammunition(&mut ammo);
        assert!(consumed, "Should successfully consume");
        assert_eq!(ammo.count, 9, "Count should decrease");
    }

    #[test]
    fn test_try_recover_ammunition_on_miss() {
        let mut ammo = AmmunitionCount::new(10, 20);
        let mut rng = crate::rng::GameRng::new(42);

        try_recover_ammunition(&mut ammo, false, CriticalHitType::None, &mut rng);

        // On miss, should likely recover (95% chance)
        // With seed 42, we should get recovery
        assert!(ammo.count >= 10, "Should likely recover on miss");
    }

    #[test]
    fn test_try_recover_ammunition_on_critical() {
        let mut ammo = AmmunitionCount::new(10, 20);
        let mut rng = crate::rng::GameRng::new(100);

        // With seed 100, critical hit (25% recovery) less likely to recover
        try_recover_ammunition(&mut ammo, true, CriticalHitType::Devastating, &mut rng);

        // Might or might not recover depending on RNG
        // Just verify structure works
        assert!(ammo.count >= 10 && ammo.count <= 11);
    }

    #[test]
    fn test_ranged_weapon_base_damage_bonus() {
        assert_eq!(RangedWeaponType::Bow.base_damage_bonus(), 0);
        assert_eq!(RangedWeaponType::Crossbow.base_damage_bonus(), 1);
        assert_eq!(RangedWeaponType::Sling.base_damage_bonus(), 0);
        assert_eq!(RangedWeaponType::Thrown.base_damage_bonus(), -1);
    }

    #[test]
    fn test_ranged_weapon_names() {
        assert_eq!(RangedWeaponType::Bow.name(), "bow");
        assert_eq!(RangedWeaponType::Crossbow.name(), "crossbow");
        assert_eq!(RangedWeaponType::Sling.name(), "sling");
        assert_eq!(RangedWeaponType::Thrown.name(), "thrown weapon");
    }

    #[test]
    fn test_ranged_optimal_vs_max_distance() {
        for weapon in &[
            RangedWeaponType::Bow,
            RangedWeaponType::Crossbow,
            RangedWeaponType::Sling,
            RangedWeaponType::Thrown,
        ] {
            assert!(
                weapon.optimal_range() < weapon.max_range(),
                "{} optimal should be less than max",
                weapon.name()
            );
        }
    }

    // ========================================================================
    // FRIENDLY FIRE PREVENTION TESTS
    // ========================================================================

    #[test]
    fn test_is_friendly_target_tame_monster() {
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.tame = true; // Pet/familiar
        let player = test_player();

        assert!(
            is_friendly_target(&target, &player),
            "Tame monsters should be friendly"
        );
    }

    #[test]
    fn test_is_friendly_target_peaceful_coaligned() {
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.peaceful = true;
        target.alignment = 10; // Positive alignment = Lawful

        let mut player = test_player();
        // is_friendly_target checks player.alignment.typ, not record
        player.alignment.typ = crate::player::AlignmentType::Lawful;

        assert!(
            is_friendly_target(&target, &player),
            "Peaceful co-aligned monsters should be friendly"
        );
    }

    #[test]
    fn test_is_friendly_target_peaceful_not_coaligned() {
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.peaceful = true;
        target.alignment = 10; // Positive alignment

        let mut player = test_player();
        player.alignment.record = -5; // Negative (not co-aligned)

        assert!(
            !is_friendly_target(&target, &player),
            "Peaceful but non-aligned monsters should not be friendly"
        );
    }

    #[test]
    fn test_is_friendly_target_hostile_monster() {
        let target = Monster::new(MonsterId(1), 10, 5, 5);
        let player = test_player();

        assert!(
            !is_friendly_target(&target, &player),
            "Hostile monsters should not be friendly"
        );
    }

    #[test]
    fn test_is_friendly_target_neutral_player() {
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.peaceful = true;
        target.alignment = 10;

        let mut player = test_player();
        player.alignment.record = 0; // Neutral

        assert!(
            !is_friendly_target(&target, &player),
            "Neutral player doesn't have alignment-based friendliness"
        );
    }

    #[test]
    fn test_check_friendly_fire_safe_target() {
        let attacker = test_player();
        let target = Monster::new(MonsterId(1), 10, 5, 5); // Hostile
        let level = crate::dungeon::Level::new(crate::dungeon::DLevel::main_dungeon_start());

        let result = check_friendly_fire(&attacker, &target, &level);
        assert_eq!(
            result,
            FriendlyFireResult::Safe,
            "Attacking hostile monster should be safe"
        );
    }

    #[test]
    fn test_check_friendly_fire_friendly_target() {
        let mut attacker = test_player();
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.tame = true; // Pet

        let level = crate::dungeon::Level::new(crate::dungeon::DLevel::main_dungeon_start());

        let result = check_friendly_fire(&attacker, &target, &level);
        assert_eq!(
            result,
            FriendlyFireResult::TargetFriendly,
            "Attacking friendly monster should flag as target friendly"
        );
    }

    #[test]
    fn test_friendly_fire_warning_safe() {
        let target = Monster::new(MonsterId(1), 10, 5, 5);

        let msg = friendly_fire_warning_message(FriendlyFireResult::Safe, &target);
        assert!(
            msg.contains("clear") || msg.contains("safe"),
            "Safe message should mention clear/safe"
        );
    }

    #[test]
    fn test_friendly_fire_warning_target_friendly() {
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.name = "Fluffy the dog".to_string();

        let msg = friendly_fire_warning_message(FriendlyFireResult::TargetFriendly, &target);
        assert!(
            msg.contains("friendly"),
            "Should warn about friendly target"
        );
        assert!(msg.contains("Fluffy"), "Should include target name");
    }

    #[test]
    fn test_friendly_fire_warning_collateral() {
        let target = Monster::new(MonsterId(1), 10, 5, 5);

        let msg = friendly_fire_warning_message(FriendlyFireResult::CollateralRisk, &target);
        assert!(
            msg.contains("friendly") || msg.contains("path"),
            "Should warn about projectile path"
        );
    }

    #[test]
    fn test_can_attack_ranged_safely_ok() {
        let attacker = test_player();
        let target = Monster::new(MonsterId(1), 10, 5, 5); // Hostile
        let level = crate::dungeon::Level::new(crate::dungeon::DLevel::main_dungeon_start());

        let result = can_attack_ranged_safely(&attacker, &target, &level);
        assert!(
            result.is_ok(),
            "Attacking hostile monster should be allowed"
        );
    }

    #[test]
    fn test_can_attack_ranged_safely_blocked() {
        let attacker = test_player();
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.tame = true; // Pet

        let level = crate::dungeon::Level::new(crate::dungeon::DLevel::main_dungeon_start());

        let result = can_attack_ranged_safely(&attacker, &target, &level);
        assert!(
            result.is_err(),
            "Attacking friendly monster should be blocked"
        );
        assert!(
            result.unwrap_err().contains("ally"),
            "Error should mention ally"
        );
    }

    #[test]
    fn test_friendly_fire_prevention_multiple_alignments() {
        use crate::player::AlignmentType;

        // Test various alignment combinations
        // is_friendly_target uses player.alignment.typ (not record) and
        // AlignmentType::from_value(monster.alignment) for the monster.
        // Neutral players never get alignment-based friendliness.
        let alignments: Vec<(AlignmentType, i8, bool)> = vec![
            (AlignmentType::Lawful, 10, true),     // Both Lawful
            (AlignmentType::Chaotic, -10, true),    // Both Chaotic
            (AlignmentType::Neutral, 0, false),     // Both neutral (neutral player never friendly)
            (AlignmentType::Lawful, -10, false),    // Lawful vs Chaotic
            (AlignmentType::Lawful, 0, false),      // Lawful vs Neutral
        ];

        for (player_align_type, monster_align, expected_friendly) in alignments {
            let mut target = Monster::new(MonsterId(1), 10, 5, 5);
            target.state.peaceful = true;
            target.alignment = monster_align;

            let mut player = test_player();
            player.alignment.typ = player_align_type;

            let is_friendly = is_friendly_target(&target, &player);
            assert_eq!(
                is_friendly, expected_friendly,
                "Alignment ({:?}, {}) should result in friendly={}",
                player_align_type, monster_align, expected_friendly
            );
        }
    }

    #[test]
    fn test_tame_overrides_alignment() {
        // Tame monsters are friendly regardless of alignment
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.tame = true;
        target.alignment = -100; // Very hostile alignment

        let mut player = test_player();
        player.alignment.record = 100; // Very positive alignment

        assert!(
            is_friendly_target(&target, &player),
            "Tame status should override alignment"
        );
    }

    #[test]
    fn test_peaceful_requires_coalignment() {
        let mut target = Monster::new(MonsterId(1), 10, 5, 5);
        target.state.peaceful = true;
        target.state.tame = false; // Not tame

        let mut player = test_player();
        player.alignment.record = 10; // Positive
        target.alignment = -10; // Negative (not co-aligned)

        // Without tame or co-alignment, should not be friendly
        assert!(
            !is_friendly_target(&target, &player),
            "Peaceful without co-alignment should not be friendly"
        );
    }

    #[test]
    fn test_skill_hit_bonus_normal_weapon() {
        use crate::player::SkillLevel;
        let mut player = test_player();
        let weapon = test_weapon();

        // Default skill level is Restricted => -4
        assert_eq!(skill_hit_bonus(&player, Some(&weapon), false), -4);

        // Set to Expert
        let skill_type = weapon_skill_type(&weapon);
        player.skills.get_mut(skill_type).level = SkillLevel::Expert;
        assert_eq!(skill_hit_bonus(&player, Some(&weapon), false), 3);
    }

    #[test]
    fn test_skill_hit_bonus_two_weapon() {
        use crate::player::SkillLevel;
        let mut player = test_player();
        let weapon = test_weapon();

        // Two-weapon unskilled => -9
        assert_eq!(skill_hit_bonus(&player, Some(&weapon), true), -9);

        // Set two-weapon skill to Expert but weapon still Restricted
        // Uses min of the two, so still Restricted (-9)
        player
            .skills
            .get_mut(crate::player::SkillType::TwoWeapon)
            .level = SkillLevel::Expert;
        assert_eq!(skill_hit_bonus(&player, Some(&weapon), true), -9);

        // Now set weapon skill to Expert too
        let skill_type = weapon_skill_type(&weapon);
        player.skills.get_mut(skill_type).level = SkillLevel::Expert;
        assert_eq!(skill_hit_bonus(&player, Some(&weapon), true), -3);
    }

    #[test]
    fn test_skill_hit_bonus_bare_handed() {
        use crate::player::SkillLevel;
        let mut player = test_player();

        // Non-monk, unskilled bare-handed: (max(1,1)-1 + 2) * 1 / 2 = 1
        assert_eq!(skill_hit_bonus(&player, None, false), 1);

        // Monk at Expert: (max(4,1)-1 + 2) * 2 / 2 = 5
        player.role = crate::player::Role::Monk;
        player
            .skills
            .get_mut(crate::player::SkillType::BareHanded)
            .level = SkillLevel::Expert;
        assert_eq!(skill_hit_bonus(&player, None, false), 5);
    }

    #[test]
    fn test_skill_dam_bonus_normal_weapon() {
        use crate::player::SkillLevel;
        let mut player = test_player();
        let weapon = test_weapon();

        // Default Restricted => -2
        assert_eq!(skill_dam_bonus(&player, Some(&weapon), false), -2);

        // Expert => +2
        let skill_type = weapon_skill_type(&weapon);
        player.skills.get_mut(skill_type).level = SkillLevel::Expert;
        assert_eq!(skill_dam_bonus(&player, Some(&weapon), false), 2);
    }

    #[test]
    fn test_skill_dam_bonus_two_weapon() {
        use crate::player::SkillLevel;
        let mut player = test_player();
        let weapon = test_weapon();

        // Two-weapon unskilled => -3
        assert_eq!(skill_dam_bonus(&player, Some(&weapon), true), -3);

        // Both at Expert => +1
        player
            .skills
            .get_mut(crate::player::SkillType::TwoWeapon)
            .level = SkillLevel::Expert;
        let skill_type = weapon_skill_type(&weapon);
        player.skills.get_mut(skill_type).level = SkillLevel::Expert;
        assert_eq!(skill_dam_bonus(&player, Some(&weapon), true), 1);
    }

    #[test]
    fn test_martial_bonus() {
        let mut player = test_player();
        assert!(!martial_bonus(&player)); // Default Valkyrie
        player.role = crate::player::Role::Monk;
        assert!(martial_bonus(&player));
    }
}
