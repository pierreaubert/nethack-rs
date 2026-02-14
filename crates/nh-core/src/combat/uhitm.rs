//! Player attacks monster combat (uhitm.c)
//!
//! Handles all combat initiated by the player against monsters.
//! Includes weapon damage calculation, silver/blessed bonuses,
//! artifact integration, erosion, two-weapon fighting, and cleaving.

use super::artifact::{artifact_for_object, artifact_hit, spec_abon, spec_dbon, Artifact};
use super::{CombatEffect, CombatResult};
use crate::monster::{Monster, MonsterFlags};
use crate::object::{Material, Object};
use crate::player::You;
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::artifact::{
        Artifact, ArtifactAlignment, ArtifactFlags, InvokeProperty,
    };
    use crate::combat::{Attack, AttackType, DamageType};
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

        let mut total_damage = 0;
        for _ in 0..100 {
            monster.hp = 100;
            let result = player_attack_monster(&mut player, &mut monster, None, &mut rng);
            if result.hit {
                assert!(
                    result.damage >= 3 && result.damage <= 4,
                    "Damage {} not in expected range 3-4 for str 17 unarmed",
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
}
