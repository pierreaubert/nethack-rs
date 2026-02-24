//! Combat system
//!
//! Implements player-vs-monster, monster-vs-player, and monster-vs-monster combat.

pub mod artifact;
mod attack_type;
mod damage_type;
mod mhitm;
mod mhitu;
mod uhitm;

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::monster::Monster;

pub use attack_type::AttackType;
pub use damage_type::{
    DamageType,
    ElementalDamageResult,
    ErodeResult,
    ErosionType,
    Resistance,
    acid_damage,
    add_erosion_words,
    attack_has_dmgtype,
    can_erode_material,
    cold_damage,
    dmgtype_fromattack,
    dmgval,
    erode_armor,
    erode_obj,
    erode_obj_text,
    // Erosion functions
    erosion_matters,
    fire_damage,
    poison_damage,
    shock_damage,
    weapon_dmgval_bonus,
};
pub use mhitm::{
    MmResult, explmm, fightm, gazemm, gulpmm, hitmm, mattackm, mdamagem, mdisplacem, missmm,
    mm_aggression, mm_displacement, monster_attack_monster, mswingsm, passivemm,
};
pub use mhitu::{
    MonsterAttackResult, SeduceResult, apply_grab_damage, automiss, could_seduce,
    damage_effect_message, diseasemu, doseduce, expels, explmu, gazemu, gulp_blnd_check, gulpmu,
    hit_message, mattacku, miss_message, monster_attack_player, passiveum, resistance_message,
    steal_it, stealamulet, stealarm, stealgold, thitu, try_escape_grab, u_slow_down,
    wild_miss_message,
};
pub use uhitm::{
    AttackSource, CleaveTarget, HitResult, JoustResult,
    ammo_for_launcher, artifact_to_hit_bonus, attack, attack_checks, attack_hits,
    bare_hand_damage, buc_damage_bonus, calculate_to_hit, cleave_targets, creature_vulnerability,
    dbon, find_roll_to_hit, greatest_erosion, hates_silver_check, hitval, hmon,
    hates_silver, is_ranged_weapon, is_two_handed, joust, maybe_erode_weapon,
    mon_hates_silver, mon_hates_silver_by_name,
    advance_weapon_skill_from_combat,
    player_attack_monster, process_player_xp_reward, retouch_equipment, retouch_object, shade_aware, shade_miss,
    shade_miss_message, silver_damage, silver_sears, skill_dam_bonus, skill_hit_bonus,
    special_dmgval, special_weapon_effects, sticks, throw_damage, throwing_weapon,
    two_weapon_hit, weapon_dam_bonus, weapon_hit_bonus, weapon_skill_type,
    // Loot system
    award_monster_loot, award_boss_hoard, calculate_monster_gold, generate_monster_loot, should_drop_hoard,
    // Combat spells
    can_player_cast_in_combat, player_cast_spell, get_player_combat_spells,
    // Encounter system
    create_encounter, calculate_encounter_difficulty, get_difficulty_label,
    init_encounter_state, are_monsters_flanking, get_flanking_bonus,
    apply_encounter_effects, process_encounter_round, get_encounter_victory_xp,
    // Polymorph combat
    hmonas, player_wearing_armor_type,
};

// Phase 14: Experience & Leveling System exports
// (Defined inline in mod.rs, no separate module)

use crate::NATTK;
use serde::{Deserialize, Serialize};

/// A single attack definition (from struct attk in monattk.h)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attack {
    /// How the attack is delivered
    pub attack_type: AttackType,
    /// What kind of damage is dealt
    pub damage_type: DamageType,
    /// Number of damage dice
    pub dice_num: u8,
    /// Sides per damage die
    pub dice_sides: u8,
}

impl Attack {
    /// Create a new attack
    pub const fn new(
        attack_type: AttackType,
        damage_type: DamageType,
        dice_num: u8,
        dice_sides: u8,
    ) -> Self {
        Self {
            attack_type,
            damage_type,
            dice_num,
            dice_sides,
        }
    }

    /// Check if this is a valid/active attack
    pub const fn is_active(&self) -> bool {
        !matches!(self.attack_type, AttackType::None)
    }

    /// Get the average damage for this attack
    pub fn average_damage(&self) -> f32 {
        if self.dice_sides == 0 {
            return 0.0;
        }
        self.dice_num as f32 * (self.dice_sides as f32 + 1.0) / 2.0
    }
}

/// Attack set for a monster (6 attacks max)
pub type AttackSet = [Attack; NATTK];

/// Create an empty attack set
pub const fn empty_attacks() -> AttackSet {
    [Attack::new(AttackType::None, DamageType::Physical, 0, 0); NATTK]
}

/// Result of a combat action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatResult {
    /// Whether the attack connected
    pub hit: bool,
    /// Whether the defender died
    pub defender_died: bool,
    /// Whether the attacker died (e.g., from cockatrice corpse)
    pub attacker_died: bool,
    /// Damage dealt (before resistances)
    pub damage: i32,
    /// Special effect triggered
    pub special_effect: Option<CombatEffect>,
}

impl CombatResult {
    pub const MISS: Self = Self {
        hit: false,
        defender_died: false,
        attacker_died: false,
        damage: 0,
        special_effect: None,
    };

    pub const fn hit(damage: i32) -> Self {
        Self {
            hit: true,
            defender_died: false,
            attacker_died: false,
            damage,
            special_effect: None,
        }
    }
}

/// Special effects that can occur during combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEffect {
    Poisoned,
    Paralyzed,
    Slowed,
    Stunned,
    Confused,
    Blinded,
    Drained,
    Diseased,
    Petrifying,
    Teleported,
    ItemStolen,
    GoldStolen,
    Engulfed,
    Grabbed,
    ItemDestroyed,
    ArmorCorroded,
}

/// Critical hit types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalHitType {
    /// No critical hit
    None,
    /// Grazing hit (50% damage)
    Graze,
    /// Normal critical hit (150% damage)
    Critical,
    /// Devastating critical hit (200% damage)
    Devastating,
    /// Instant kill hit (one-shot)
    InstantKill,
}

impl CriticalHitType {
    /// Get damage multiplier for this critical hit
    pub const fn damage_multiplier(&self) -> f32 {
        match self {
            CriticalHitType::None => 1.0,
            CriticalHitType::Graze => 0.5,
            CriticalHitType::Critical => 1.5,
            CriticalHitType::Devastating => 2.0,
            CriticalHitType::InstantKill => f32::INFINITY,
        }
    }

    /// Check if this is a crit (for UI/messages)
    pub const fn is_critical(&self) -> bool {
        matches!(
            self,
            CriticalHitType::Critical | CriticalHitType::Devastating | CriticalHitType::InstantKill
        )
    }
}

/// Weapon skill types for progression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WeaponSkill {
    Bare,
    Dagger,
    Sword,
    Axe,
    Polearm,
    Bow,
    Crossbow,
    Sling,
    Whip,
    Staff,
    Blunt,
    Flail,
    Spear,
}

impl WeaponSkill {
    /// Get list of all weapon skills
    pub fn all() -> &'static [WeaponSkill] {
        &[
            WeaponSkill::Bare,
            WeaponSkill::Dagger,
            WeaponSkill::Sword,
            WeaponSkill::Axe,
            WeaponSkill::Polearm,
            WeaponSkill::Bow,
            WeaponSkill::Crossbow,
            WeaponSkill::Sling,
            WeaponSkill::Whip,
            WeaponSkill::Staff,
            WeaponSkill::Blunt,
            WeaponSkill::Flail,
            WeaponSkill::Spear,
        ]
    }

    /// Get skill name
    pub const fn name(&self) -> &'static str {
        match self {
            WeaponSkill::Bare => "bare hands",
            WeaponSkill::Dagger => "daggers",
            WeaponSkill::Sword => "swords",
            WeaponSkill::Axe => "axes",
            WeaponSkill::Polearm => "polearms",
            WeaponSkill::Bow => "bows",
            WeaponSkill::Crossbow => "crossbows",
            WeaponSkill::Sling => "slings",
            WeaponSkill::Whip => "whips",
            WeaponSkill::Staff => "staffs",
            WeaponSkill::Blunt => "blunt weapons",
            WeaponSkill::Flail => "flails",
            WeaponSkill::Spear => "spears",
        }
    }
}

/// Weapon skill proficiency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SkillLevel {
    /// Never used
    Unskilled = 0,
    /// Basic training
    Basic = 1,
    /// Intermediate proficiency
    Skilled = 2,
    /// High proficiency
    Expert = 3,
    /// Master level
    Master = 4,
}

impl SkillLevel {
    /// Get hit bonus for this skill level
    pub const fn hit_bonus(&self) -> i32 {
        match self {
            SkillLevel::Unskilled => 0,
            SkillLevel::Basic => 1,
            SkillLevel::Skilled => 2,
            SkillLevel::Expert => 3,
            SkillLevel::Master => 4,
        }
    }

    /// Get damage bonus for this skill level
    pub const fn damage_bonus(&self) -> i32 {
        match self {
            SkillLevel::Unskilled => 0,
            SkillLevel::Basic => 1,
            SkillLevel::Skilled => 2,
            SkillLevel::Expert => 3,
            SkillLevel::Master => 5,
        }
    }

    /// Get critical hit chance percent
    pub const fn crit_chance(&self) -> u8 {
        match self {
            SkillLevel::Unskilled => 0,
            SkillLevel::Basic => 5,
            SkillLevel::Skilled => 10,
            SkillLevel::Expert => 15,
            SkillLevel::Master => 20,
        }
    }

    /// Get armor penetration (reduces target AC)
    pub const fn armor_penetration(&self) -> i32 {
        match self {
            SkillLevel::Unskilled => 0,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 1,
            SkillLevel::Expert => 2,
            SkillLevel::Master => 3,
        }
    }

    /// Advance to next skill level
    pub fn advance(&self) -> SkillLevel {
        match self {
            SkillLevel::Unskilled => SkillLevel::Basic,
            SkillLevel::Basic => SkillLevel::Skilled,
            SkillLevel::Skilled => SkillLevel::Expert,
            SkillLevel::Expert => SkillLevel::Master,
            SkillLevel::Master => SkillLevel::Master,
        }
    }
}

// ============================================================================
// Armor protection mapping
// ============================================================================

/// Returns worn mask indicating which armor protects against an attack type.
///
/// The return value is a bitmask of body slots (W_ARMOR*) that provide protection
/// against this attack type. Special values:
/// - `!0` (all bits set) means attacks that don't need armor defense
/// - `0` means attacks with no armor defense available
///
/// # Arguments
/// * `attack_type` - The type of attack being delivered
///
/// # Returns
/// A u32 bitmask of worn_mask constants indicating which armor helps
pub fn attk_protection(attack_type: AttackType) -> u32 {
    use crate::action::wear::worn_mask::*;

    match attack_type {
        // Attacks that don't need armor defense (magic, ranged effects)
        AttackType::None
        | AttackType::Spit
        | AttackType::Explode
        | AttackType::ExplodeOnDeath
        | AttackType::Gaze
        | AttackType::Breath
        | AttackType::Magic => !0,

        // Claw/weapon attacks blocked by gloves
        AttackType::Claw | AttackType::Touch | AttackType::Weapon => W_ARMG,

        // Kick attacks blocked by boots
        AttackType::Kick => W_ARMF,

        // Head butt attacks blocked by helmet
        AttackType::Butt => W_ARMH,

        // Hugs need both cloak and gloves
        AttackType::Hug => W_ARMC | W_ARMG,

        // Bite/sting/engulf/tentacle - no armor defense available
        AttackType::Bite | AttackType::Sting | AttackType::Engulf | AttackType::Tentacle => 0,
    }
}

// ============================================================================
// Armor Class Calculation
// ============================================================================

/// Calculate monster's current armor class (find_mac in C).
///
/// Monster AC is calculated from:
/// - Base AC from monster type
/// - Worn armor bonuses
///
/// # Arguments
/// * `monster` - The monster to calculate AC for
///
/// # Returns
/// The calculated armor class
pub fn find_mac(monster: &crate::monster::Monster) -> i8 {
    let mut base = monster.ac as i32;

    // Subtract armor bonuses from worn items
    for obj in &monster.inventory {
        if obj.worn_mask != 0 {
            base -= armor_bonus(obj);
        }
    }

    base.clamp(-128, 127) as i8
}

/// Calculate armor bonus for an object (ARM_BONUS macro in C).
///
/// The bonus is the base armor value plus enchantment, minus erosion.
pub fn armor_bonus(obj: &crate::object::Object) -> i32 {
    let base = obj.base_ac as i32;
    let enchant = obj.enchantment as i32;
    let erosion = obj.erosion() as i32;

    (base + enchant - erosion).max(0)
}

/// Check if grease on an object protects against an effect.
///
/// Greased items have a chance to deflect certain attacks.
/// Each use has a chance to use up the grease.
///
/// # Arguments
/// * `obj` - The object to check
/// * `rng` - Random number generator
///
/// # Returns
/// true if grease protected, false otherwise
pub fn grease_protect(obj: &mut crate::object::Object, rng: &mut crate::rng::GameRng) -> bool {
    if !obj.greased {
        return false;
    }

    // Grease protects
    // 1 in 2 chance to use up the grease
    if rng.one_in(2) {
        obj.greased = false;
    }

    true
}

/// Check if a monster has any attacks (noattacks in C).
///
/// # Arguments
/// * `monster` - The monster to check
///
/// # Returns
/// true if monster has no active attacks
pub fn noattacks(monster: &crate::monster::Monster) -> bool {
    for attack in &monster.attacks {
        if attack.is_active() {
            return false;
        }
    }
    true
}

/// Get all active attacks for a monster (attacks in C).
///
/// # Arguments
/// * `monster` - The monster to get attacks for
///
/// # Returns
/// Iterator over active attacks
pub fn get_attacks(monster: &crate::monster::Monster) -> impl Iterator<Item = &Attack> {
    monster.attacks.iter().filter(|a| a.is_active())
}

// ============================================================================
// Magic resistance/negation
// ============================================================================

/// Calculate magic negation level (mc) for a creature.
///
/// Magic negation reduces incoming spell damage. Higher levels provide better protection.
/// The mc level can range from 0 (no protection) to 3+ (high protection).
///
/// # Arguments
/// * `is_player` - Whether the creature is the player
/// * `inventory` - The creature's inventory (worn items)
/// * `has_protection` - Whether creature has magical Protection
/// * `protection_level` - Level of intrinsic protection (0-3)
/// * `spell_protection` - Additional spell protection value
/// * `is_high_priest` - Whether creature is a priest (grants mc)
/// * `is_aligned_priest` - Whether creature is aligned with gods
/// * `is_minion` - Whether creature is a minion
///
/// # Returns
/// Magic cancellation level (0-3+)
pub fn magic_negation(
    _is_player: bool,
    inventory: &[crate::object::Object],
    has_protection: bool,
    protection_level: i8,
    spell_protection: i8,
    is_high_priest: bool,
    is_aligned_priest: bool,
    is_minion: bool,
) -> i8 {
    let mut mc = 0i8;
    let mut got_protect = has_protection;

    // Check worn armor for magic cancellation
    for obj in inventory {
        if obj.worn_mask & crate::action::wear::worn_mask::W_ARMOR != 0 {
            let armor_mc = obj.magic_cancellation();
            if armor_mc > mc {
                mc = armor_mc;
            }
        }

        // Check for artifact Protection if not already found
        if !got_protect {
            let being_worn = (obj.worn_mask
                & (crate::action::wear::worn_mask::W_ARMOR
                    | crate::action::wear::worn_mask::W_ACCESSORY
                    | crate::action::wear::worn_mask::W_WEP))
                != 0;
            if crate::object::protects(obj, being_worn) {
                got_protect = true;
            }
        }
    }

    // Extrinsic Protection increases mc by 1
    if got_protect && mc < 3 {
        mc += 1;
    } else if mc < 1 {
        // Intrinsic Protection grants minimum mc 1
        if (has_protection && protection_level > 0) || spell_protection > 0 {
            mc = 1;
        } else if is_high_priest || is_aligned_priest || is_minion {
            mc = 1;
        }
    }

    mc
}

// ============================================================================
// Enhanced Combat Calculation System (Phase 9)
// ============================================================================

/// Determine critical hit type based on hit roll and skill level
///
/// Higher skill levels increase critical hit chance. This function
/// rolls for critical hit and returns the appropriate type.
pub fn determine_critical_hit(
    base_roll: i32,
    skill_level: SkillLevel,
    rng: &mut crate::rng::GameRng,
) -> CriticalHitType {
    // Base critical chance from skill level
    let crit_chance = skill_level.crit_chance() as i32;

    // Roll for critical
    let crit_roll = rng.rnd(100) as i32;

    // If roll beats critical threshold, it's a critical
    if crit_roll < crit_chance {
        // Determine type of critical (basic, devastating, or instakill)
        // Higher base rolls increase severity
        if base_roll > 19 {
            // Instakill on natural 20+ with high skill
            if skill_level as u8 >= SkillLevel::Expert as u8 {
                return CriticalHitType::InstantKill;
            }
            CriticalHitType::Devastating
        } else if base_roll > 15 {
            CriticalHitType::Devastating
        } else {
            CriticalHitType::Critical
        }
    } else if base_roll < 5 && skill_level == SkillLevel::Unskilled {
        // Unskilled can have grazes on low rolls
        CriticalHitType::Graze
    } else {
        CriticalHitType::None
    }
}

/// Calculate enhanced hit bonus with skill modifiers
///
/// Integrates weapon skill level with the base to-hit calculation.
/// Skill level provides hit bonus and armor penetration.
pub fn calculate_skill_enhanced_to_hit(
    base_to_hit: i32,
    skill_level: SkillLevel,
    _armor_penetration: i32,
) -> i32 {
    base_to_hit + skill_level.hit_bonus()
}

/// Calculate enhanced damage with skill and critical modifiers
///
/// Applies skill level damage bonus and critical hit multiplier
/// to the base damage roll.
pub fn calculate_skill_enhanced_damage(
    base_damage: i32,
    skill_level: SkillLevel,
    critical: CriticalHitType,
) -> i32 {
    let mut damage = base_damage;

    // Add skill level damage bonus
    damage += skill_level.damage_bonus();

    // Apply critical multiplier
    if critical == CriticalHitType::InstantKill {
        // Instant kill - return massive damage
        i32::MAX
    } else {
        let multiplier = critical.damage_multiplier();
        (damage as f32 * multiplier) as i32
    }
}

/// Apply armor penetration modifier to effective AC
///
/// Higher skill levels can partially ignore target's AC protection.
pub fn apply_armor_penetration(target_ac: i8, armor_penetration: i32) -> i8 {
    // Each point of penetration reduces effective AC by 1
    (target_ac + armor_penetration as i8).min(127).max(-128)
}

/// Weapon proficiency tracking
///
/// Stores per-weapon skill progression and usage statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponProficiency {
    /// Current skill level with this weapon
    pub skill_level: SkillLevel,
    /// Number of times this weapon has hit
    pub hits: u32,
    /// Number of times this weapon has missed
    pub misses: u32,
    /// Number of critical hits achieved
    pub critical_hits: u32,
    /// Experience points toward next level
    pub experience: u32,
}

impl WeaponProficiency {
    /// Create new proficiency entry
    pub const fn new(skill_level: SkillLevel) -> Self {
        Self {
            skill_level,
            hits: 0,
            misses: 0,
            critical_hits: 0,
            experience: 0,
        }
    }

    /// Get current hit rate (hits / total attacks)
    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f32 / total as f32
        }
    }

    /// Record a hit with this weapon
    pub fn record_hit(&mut self, is_critical: bool) {
        self.hits += 1;
        if is_critical {
            self.critical_hits += 1;
        }
        // Gain experience on hit
        self.experience += 10;
    }

    /// Record a miss with this weapon
    pub fn record_miss(&mut self) {
        self.misses += 1;
        // Small experience gain on miss (encourages practice)
        self.experience += 1;
    }

    /// Attempt to advance to next skill level
    ///
    /// Returns true if skill was advanced
    pub fn try_advance(&mut self) -> bool {
        // Experience requirement increases with level
        let requirement = match self.skill_level {
            SkillLevel::Unskilled => 100,
            SkillLevel::Basic => 250,
            SkillLevel::Skilled => 500,
            SkillLevel::Expert => 1000,
            SkillLevel::Master => u32::MAX, // Can't advance beyond Master
        };

        if self.experience >= requirement {
            self.skill_level = self.skill_level.advance();
            self.experience = 0;
            true
        } else {
            false
        }
    }
}

impl Default for WeaponProficiency {
    fn default() -> Self {
        Self::new(SkillLevel::Unskilled)
    }
}

/// Calculate attribute-based damage bonuses
///
/// Different attributes affect damage based on weapon type
pub fn calculate_attribute_damage_bonus(
    strength: u8,
    dexterity: u8,
    weapon_type: WeaponSkill,
) -> i32 {
    let str_bonus = (strength as i32 - 10) / 2;
    let dex_bonus = (dexterity as i32 - 10) / 4; // Dex is less important for damage

    match weapon_type {
        // Melee weapons benefit heavily from strength
        WeaponSkill::Sword
        | WeaponSkill::Axe
        | WeaponSkill::Blunt
        | WeaponSkill::Flail
        | WeaponSkill::Spear => (str_bonus * 2 + dex_bonus).max(-5),

        // Polearms balance strength and dexterity
        WeaponSkill::Polearm | WeaponSkill::Staff => (str_bonus + dex_bonus).max(-3),

        // Ranged weapons benefit from dexterity
        WeaponSkill::Bow | WeaponSkill::Crossbow | WeaponSkill::Sling => {
            (dex_bonus * 2 + str_bonus).max(-5)
        }

        // Finesse weapons (dagger, whip) benefit from dexterity
        WeaponSkill::Dagger | WeaponSkill::Whip => (dex_bonus * 2 + str_bonus).max(-3),

        // Bare hands benefit from both
        WeaponSkill::Bare => (str_bonus + dex_bonus).max(-2),
    }
}

/// Calculate weapon type effectiveness vs target armor type
///
/// Some weapons are better against certain armor
pub fn weapon_vs_armor_bonus(weapon_type: WeaponSkill, _target_ac_type: i8) -> i32 {
    // Basic system: certain weapons are better at penetrating armor
    match weapon_type {
        // Piercing weapons (dagger, spear) penetrate armor well
        WeaponSkill::Dagger | WeaponSkill::Spear => 1,
        // Blunt weapons (blunt, flail, staff) are okay
        WeaponSkill::Blunt | WeaponSkill::Flail | WeaponSkill::Staff => 0,
        // Slashing weapons (sword, axe) are standard
        WeaponSkill::Sword | WeaponSkill::Axe => 0,
        // Polearms are balanced
        WeaponSkill::Polearm => 0,
        // Ranged weapons (bow, crossbow, sling) have limited armor penetration
        WeaponSkill::Bow | WeaponSkill::Crossbow | WeaponSkill::Sling => -1,
        // Whip has poor armor penetration
        WeaponSkill::Whip => -1,
        // Bare hands very poor
        WeaponSkill::Bare => -2,
    }
}

/// Combat situation modifiers
///
/// Different combat situations apply various bonuses/penalties
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatModifier {
    /// Flanking bonus (+2 to hit)
    Flanking,
    /// Fighting surrounded penalty (-2 to hit)
    Surrounded,
    /// High ground tactical bonus (+1 to hit)
    HighGround,
    /// Disarmed penalty (-4 to hit, -2 damage)
    Disarmed,
    /// Weapon broken penalty (-3 to hit, -1 damage)
    WeaponBroken,
    /// Exhausted/fatigued penalty (-2 to hit, -1 damage)
    Exhausted,
    /// Inspired/blessed bonus (+1 to hit, +1 damage)
    Inspired,
    /// Cursed weapon penalty (-1 to hit, -1 damage)
    Cursed,
}

impl CombatModifier {
    /// Get to-hit modifier
    pub const fn to_hit_modifier(&self) -> i32 {
        match self {
            CombatModifier::Flanking => 2,
            CombatModifier::Surrounded => -2,
            CombatModifier::HighGround => 1,
            CombatModifier::Disarmed => -4,
            CombatModifier::WeaponBroken => -3,
            CombatModifier::Exhausted => -2,
            CombatModifier::Inspired => 1,
            CombatModifier::Cursed => -1,
        }
    }

    /// Get damage modifier
    pub const fn damage_modifier(&self) -> i32 {
        match self {
            CombatModifier::Flanking => 0,
            CombatModifier::Surrounded => 0,
            CombatModifier::HighGround => 0,
            CombatModifier::Disarmed => -2,
            CombatModifier::WeaponBroken => -1,
            CombatModifier::Exhausted => -1,
            CombatModifier::Inspired => 1,
            CombatModifier::Cursed => -1,
        }
    }
}

/// Apply all active combat modifiers
///
/// Combines multiple combat situation modifiers
pub fn apply_combat_modifiers(modifiers: &[CombatModifier]) -> (i32, i32) {
    let to_hit: i32 = modifiers.iter().map(|m| m.to_hit_modifier()).sum();
    let damage: i32 = modifiers.iter().map(|m| m.damage_modifier()).sum();
    (to_hit, damage)
}

/// Special combat situations that grant additional effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialCombatEffect {
    /// Disarm: weapon knocked from target's hand
    Disarm,
    /// Trip: target knocked prone
    Trip,
    /// Stun: target dazed/confused
    Stun,
    /// Poison: weapon delivers poison
    Poison,
    /// Drain: target loses life energy
    LifeDrain,
    /// Disease: target infected
    Disease,
    /// Petrify: target turns to stone
    Petrify,
}

impl SpecialCombatEffect {
    /// Determine if attacker can attempt this effect
    pub fn can_attempt_with_skill(self, skill_level: SkillLevel) -> bool {
        match self {
            // Basic effects available at basic skill
            SpecialCombatEffect::Stun | SpecialCombatEffect::Poison => {
                skill_level as u8 >= SkillLevel::Basic as u8
            }
            // Disarm needs skilled
            SpecialCombatEffect::Disarm | SpecialCombatEffect::Trip => {
                skill_level as u8 >= SkillLevel::Skilled as u8
            }
            // Advanced effects need expert+
            SpecialCombatEffect::LifeDrain
            | SpecialCombatEffect::Disease
            | SpecialCombatEffect::Petrify => skill_level as u8 >= SkillLevel::Expert as u8,
        }
    }

    /// Get base success chance (0-100)
    pub fn base_success_chance(&self) -> u8 {
        match self {
            SpecialCombatEffect::Stun => 25,
            SpecialCombatEffect::Poison => 30,
            SpecialCombatEffect::Disarm => 20,
            SpecialCombatEffect::Trip => 25,
            SpecialCombatEffect::LifeDrain => 15,
            SpecialCombatEffect::Disease => 20,
            SpecialCombatEffect::Petrify => 10,
        }
    }
}

/// Determine if a special effect triggers
///
/// Returns true if effect triggers based on skill and chance
pub fn roll_special_effect(
    effect: SpecialCombatEffect,
    skill_level: SkillLevel,
    rng: &mut crate::rng::GameRng,
) -> bool {
    if !effect.can_attempt_with_skill(skill_level) {
        return false;
    }

    let base_chance = effect.base_success_chance() as i32;
    // Higher skill increases success chance
    let adjusted_chance = base_chance + (skill_level as u8 as i32 * 5);
    let adjusted_chance = adjusted_chance.min(95); // Cap at 95%

    let roll = rng.rnd(100) as i32;
    roll < adjusted_chance
}

/// Critical hit effect information
#[derive(Debug, Clone)]
pub struct CriticalHitEffect {
    /// Type of critical hit
    pub crit_type: CriticalHitType,
    /// Damage multiplier already applied
    pub damage_multiplier: f32,
    /// Special effect that triggers on crit
    pub special_effect: Option<SpecialCombatEffect>,
    /// Message to display
    pub message: String,
}

impl CriticalHitEffect {
    /// Create a critical hit effect from crit type
    pub fn new(crit_type: CriticalHitType) -> Self {
        let (damage_multiplier, message) = match crit_type {
            CriticalHitType::None => (1.0, "You hit.".to_string()),
            CriticalHitType::Graze => (0.5, "You graze the target!".to_string()),
            CriticalHitType::Critical => (1.5, "You hit critically!".to_string()),
            CriticalHitType::Devastating => (2.0, "You hit with devastating force!".to_string()),
            CriticalHitType::InstantKill => (f32::INFINITY, "You land a killing blow!".to_string()),
        };

        Self {
            crit_type,
            damage_multiplier,
            special_effect: None,
            message,
        }
    }

    /// Add special effect to critical
    pub fn with_effect(mut self, effect: SpecialCombatEffect) -> Self {
        self.special_effect = Some(effect);
        self
    }

    /// Update message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}

/// Combat status effect tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatStatusEffect {
    /// Temporarily knocked prone
    Prone,
    /// Weapon knocked away
    Disarmed,
    /// Briefly stunned
    Dazed,
    /// Poisoned (takes damage over time)
    Poisoned,
    /// Life drained
    Drained,
    /// Contract disease
    Diseased,
    /// Slowly turning to stone
    Petrifying,
}

impl CombatStatusEffect {
    /// Duration of effect in turns
    pub const fn duration_turns(&self) -> u32 {
        match self {
            CombatStatusEffect::Prone => 1,
            CombatStatusEffect::Disarmed => 2,
            CombatStatusEffect::Dazed => 1,
            CombatStatusEffect::Poisoned => 10,
            CombatStatusEffect::Drained => 1,
            CombatStatusEffect::Diseased => 50,
            CombatStatusEffect::Petrifying => 15,
        }
    }

    /// Damage per turn (if applicable)
    pub const fn damage_per_turn(&self) -> i32 {
        match self {
            CombatStatusEffect::Poisoned => 1,
            CombatStatusEffect::Diseased => 1,
            _ => 0,
        }
    }

    /// Message displayed when effect triggers
    pub const fn trigger_message(&self) -> &'static str {
        match self {
            CombatStatusEffect::Prone => "You are knocked down!",
            CombatStatusEffect::Disarmed => "Your weapon flies away!",
            CombatStatusEffect::Dazed => "You see stars!",
            CombatStatusEffect::Poisoned => "You are poisoned!",
            CombatStatusEffect::Drained => "You feel your life draining away!",
            CombatStatusEffect::Diseased => "You contract a disease!",
            CombatStatusEffect::Petrifying => "You begin to turn to stone!",
        }
    }
}

/// Comprehensive combat calculation wrapper
///
/// Combines all combat calculations into a single function
/// Returns full combat info: to-hit, damage, critical, effects
#[derive(Debug, Clone)]
pub struct ComprehensiveCombatCalc {
    pub base_to_hit: i32,
    pub skill_to_hit: i32,
    pub effective_ac: i8,
    pub base_damage: i32,
    pub critical: CriticalHitType,
    pub final_damage: i32,
    pub modifiers: Vec<CombatModifier>,
    pub special_effects: Vec<SpecialCombatEffect>,
}

/// Calculate comprehensive combat information
///
/// Integrates all combat systems for a complete attack calculation
pub fn calculate_comprehensive_combat(
    base_to_hit: i32,
    skill_level: SkillLevel,
    base_damage: i32,
    _weapon_type: WeaponSkill,
    armor_penetration: i32,
    critical: CriticalHitType,
    target_ac: i8,
    modifiers: &[CombatModifier],
    rng: &mut crate::rng::GameRng,
) -> ComprehensiveCombatCalc {
    // Calculate skill-enhanced to-hit
    let skill_to_hit = calculate_skill_enhanced_to_hit(base_to_hit, skill_level, armor_penetration);

    // Apply combat modifiers to to-hit
    let (mod_to_hit, mod_damage) = apply_combat_modifiers(modifiers);
    let final_to_hit = skill_to_hit + mod_to_hit;

    // Apply armor penetration to effective AC
    let effective_ac = apply_armor_penetration(target_ac, armor_penetration);

    // Calculate skill-enhanced damage
    let skill_damage = calculate_skill_enhanced_damage(base_damage, skill_level, critical);

    // Apply combat modifiers to damage
    let final_damage = (skill_damage + mod_damage).max(1);

    // Collect active special effects
    let mut special_effects = Vec::new();
    if skill_level as u8 >= SkillLevel::Expert as u8 && rng.rnd(100) < 20 {
        // Random chance for skilled fighters to trigger special effects
        special_effects.push(SpecialCombatEffect::Stun);
    }

    ComprehensiveCombatCalc {
        base_to_hit,
        skill_to_hit: final_to_hit,
        effective_ac,
        base_damage,
        critical,
        final_damage,
        modifiers: modifiers.to_vec(),
        special_effects,
    }
}

// ============================================================================
// Ranged Combat System (Phase 11)
// ============================================================================

/// Ranged weapon types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangedWeaponType {
    /// Bow - medium range, medium speed
    Bow,
    /// Crossbow - long range, slow fire rate
    Crossbow,
    /// Sling - short-medium range, fast
    Sling,
    /// Thrown weapon - short range
    Thrown,
}

impl RangedWeaponType {
    /// Get maximum range in squares
    pub const fn max_range(&self) -> i32 {
        match self {
            RangedWeaponType::Bow => 12,
            RangedWeaponType::Crossbow => 15,
            RangedWeaponType::Sling => 8,
            RangedWeaponType::Thrown => 5,
        }
    }

    /// Get optimal range (best accuracy)
    pub const fn optimal_range(&self) -> i32 {
        match self {
            RangedWeaponType::Bow => 6,
            RangedWeaponType::Crossbow => 10,
            RangedWeaponType::Sling => 4,
            RangedWeaponType::Thrown => 2,
        }
    }

    /// Get base damage bonus for this weapon type
    pub const fn base_damage_bonus(&self) -> i32 {
        match self {
            RangedWeaponType::Bow => 0,
            RangedWeaponType::Crossbow => 1,
            RangedWeaponType::Sling => 0,
            RangedWeaponType::Thrown => -1,
        }
    }

    /// Get name for display
    pub const fn name(&self) -> &'static str {
        match self {
            RangedWeaponType::Bow => "bow",
            RangedWeaponType::Crossbow => "crossbow",
            RangedWeaponType::Sling => "sling",
            RangedWeaponType::Thrown => "thrown weapon",
        }
    }
}

/// Ranged attack information
#[derive(Debug, Clone)]
pub struct RangedAttack {
    /// Type of ranged weapon
    pub weapon_type: RangedWeaponType,
    /// Distance to target (in squares)
    pub distance: i32,
    /// Skill level of attacker
    pub skill_level: SkillLevel,
    /// Base to-hit bonus
    pub base_to_hit: i32,
}

impl RangedAttack {
    /// Calculate accuracy penalty based on distance
    ///
    /// Returns negative modifier for increased distance
    pub fn distance_penalty(&self) -> i32 {
        let optimal = self.weapon_type.optimal_range();
        let max = self.weapon_type.max_range();

        if self.distance > max {
            // Out of range - massive penalty
            -20
        } else if self.distance <= optimal {
            // Within optimal range - no penalty
            0
        } else {
            // Beyond optimal - penalty increases with distance
            let excess = self.distance - optimal;
            -(excess / 2)
        }
    }

    /// Check if target is in range
    pub fn in_range(&self) -> bool {
        self.distance <= self.weapon_type.max_range() && self.distance > 0
    }

    /// Check if projectile can reach target (with line-of-sight)
    ///
    /// Validates that the projectile path is not blocked by walls or obstacles.
    /// This should be called before executing a ranged attack to ensure
    /// the shot is possible.
    pub fn has_clear_line_of_fire(
        &self,
        from_x: i8,
        from_y: i8,
        to_x: i8,
        to_y: i8,
        level: &crate::dungeon::Level,
    ) -> bool {
        // Use Bresenham's line algorithm to trace projectile path
        trace_projectile_path(from_x, from_y, to_x, to_y, level)
    }

    /// Calculate final ranged to-hit bonus
    pub fn calculate_to_hit(&self) -> i32 {
        if !self.in_range() {
            return -99; // Impossible hit
        }

        // Start with base to-hit
        let mut to_hit = self.base_to_hit;

        // Add skill level bonus
        to_hit += self.skill_level.hit_bonus();

        // Apply distance penalty
        to_hit += self.distance_penalty();

        // Ranged weapons benefit more from dexterity than strength
        // (This should be added by caller based on actual attributes)

        to_hit
    }

    /// Calculate ranged damage with distance scaling
    ///
    /// Damage decreases slightly with distance
    pub fn calculate_damage(&self, base_damage: i32, critical: CriticalHitType) -> i32 {
        if !self.in_range() {
            return 0;
        }

        let mut damage = base_damage;

        // Add weapon type bonus
        damage += self.weapon_type.base_damage_bonus();

        // Add skill level bonus
        damage += self.skill_level.damage_bonus();

        // Apply critical multiplier
        let multiplier = critical.damage_multiplier();
        damage = (damage as f32 * multiplier) as i32;

        // Distance degrades damage slightly
        let optimal = self.weapon_type.optimal_range();
        if self.distance > optimal {
            let excess = self.distance - optimal;
            let degradation = 1 + (excess / 3); // Lose 1 damage per 3 squares beyond optimal
            damage = (damage - degradation).max(1);
        }

        damage.max(1)
    }
}

/// Ranged attack result with distance information
#[derive(Debug, Clone)]
pub struct RangedCombatResult {
    /// Whether attack hit
    pub hit: bool,
    /// Distance to target
    pub distance: i32,
    /// Damage dealt
    pub damage: i32,
    /// Critical hit type
    pub critical: CriticalHitType,
    /// Message about distance
    pub distance_message: String,
}

impl RangedCombatResult {
    /// Create ranged combat result
    pub fn new(hit: bool, distance: i32, damage: i32, critical: CriticalHitType) -> Self {
        let distance_message = match distance {
            1..=3 => "at close range".to_string(),
            4..=7 => "at medium range".to_string(),
            8..=12 => "at long range".to_string(),
            _ => "from very far away".to_string(),
        };

        Self {
            hit,
            distance,
            damage,
            critical,
            distance_message,
        }
    }
}

/// Execute a ranged attack with distance considerations
pub fn execute_ranged_attack(
    attack: &RangedAttack,
    target_ac: i8,
    rng: &mut crate::rng::GameRng,
) -> RangedCombatResult {
    let ranged_to_hit = attack.calculate_to_hit();

    // Roll for hit
    let roll = rng.rnd(20) as i32;
    let hit = roll + ranged_to_hit > 10 - target_ac as i32;

    // If hit, roll for critical
    let critical = if hit {
        determine_critical_hit(roll, attack.skill_level, rng)
    } else {
        CriticalHitType::None
    };

    RangedCombatResult::new(hit, attack.distance, 0, critical)
}

/// Ammunition tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmmunitionCount {
    /// Number of ammunition pieces
    pub count: i32,
    /// Maximum capacity
    pub capacity: i32,
}

impl AmmunitionCount {
    /// Create new ammunition counter
    pub const fn new(count: i32, capacity: i32) -> Self {
        Self { count, capacity }
    }

    /// Check if has ammunition
    pub fn has_ammo(&self) -> bool {
        self.count > 0
    }

    /// Consume one ammunition
    pub fn consume(&mut self) -> bool {
        if self.count > 0 {
            self.count -= 1;
            true
        } else {
            false
        }
    }

    /// Recover ammunition (pick up arrows after combat)
    pub fn recover(&mut self, amount: i32) {
        self.count = (self.count + amount).min(self.capacity);
    }

    /// Get capacity percentage
    pub fn capacity_percent(&self) -> u8 {
        ((self.count * 100) / self.capacity.max(1)) as u8
    }

    /// Get ammunition status message
    ///
    /// Returns a description of ammunition status for UI display.
    pub fn status_message(&self) -> String {
        match self.capacity_percent() {
            90..=100 => format!("{}/{} - Full", self.count, self.capacity),
            70..=89 => format!("{}/{} - Well-stocked", self.count, self.capacity),
            40..=69 => format!("{}/{} - Adequate", self.count, self.capacity),
            10..=39 => format!("{}/{} - Low", self.count, self.capacity),
            1..=9 => format!("{}/{} - Very low!", self.count, self.capacity),
            _ => format!("{}/{} - Out of ammo!", self.count, self.capacity),
        }
    }

    /// Check if ammunition is running low (less than 1/4 capacity)
    pub fn is_low(&self) -> bool {
        self.count < self.capacity / 4
    }
}

// ============================================================================
// Armor & Defense System (Phase 12)
// ============================================================================

/// Armor types for proficiency tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ArmorType {
    /// Cloth and light leather armor
    Light,
    /// Medium leather and metal armor
    Medium,
    /// Heavy metal armor
    Heavy,
    /// Magical armor
    Magical,
}

impl ArmorType {
    /// Get name for display
    pub const fn name(&self) -> &'static str {
        match self {
            ArmorType::Light => "light armor",
            ArmorType::Medium => "medium armor",
            ArmorType::Heavy => "heavy armor",
            ArmorType::Magical => "magical armor",
        }
    }

    /// Get armor classification from base AC value
    pub fn from_base_ac(base_ac: i32) -> Self {
        match base_ac {
            0..=2 => ArmorType::Light,
            3..=5 => ArmorType::Medium,
            6..=8 => ArmorType::Heavy,
            _ => ArmorType::Magical,
        }
    }
}

/// Armor proficiency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ArmorProficiency {
    /// No armor experience
    Untrained = 0,
    /// Basic armor familiarity
    Novice = 1,
    /// Regular armor use
    Trained = 2,
    /// Expert armor user
    Expert = 3,
    /// Master of armor
    Master = 4,
}

impl ArmorProficiency {
    /// Get AC bonus from proficiency
    pub const fn ac_bonus(&self) -> i32 {
        match self {
            ArmorProficiency::Untrained => 0,
            ArmorProficiency::Novice => 1,
            ArmorProficiency::Trained => 2,
            ArmorProficiency::Expert => 3,
            ArmorProficiency::Master => 4,
        }
    }

    /// Get spell failure penalty (higher = more interference with spells)
    pub const fn spell_failure_penalty(&self) -> i32 {
        match self {
            ArmorProficiency::Untrained => 20,
            ArmorProficiency::Novice => 15,
            ArmorProficiency::Trained => 10,
            ArmorProficiency::Expert => 5,
            ArmorProficiency::Master => 0,
        }
    }

    /// Get dodge penalty from armor encumbrance
    pub const fn dodge_penalty(&self) -> i32 {
        match self {
            ArmorProficiency::Untrained => -3,
            ArmorProficiency::Novice => -2,
            ArmorProficiency::Trained => -1,
            ArmorProficiency::Expert => 0,
            ArmorProficiency::Master => 1,
        }
    }

    /// Advance to next proficiency level
    pub fn advance(&self) -> ArmorProficiency {
        match self {
            ArmorProficiency::Untrained => ArmorProficiency::Novice,
            ArmorProficiency::Novice => ArmorProficiency::Trained,
            ArmorProficiency::Trained => ArmorProficiency::Expert,
            ArmorProficiency::Expert => ArmorProficiency::Master,
            ArmorProficiency::Master => ArmorProficiency::Master,
        }
    }
}

/// Dodging/evasion skill level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DodgeSkill {
    /// No dodge training
    Untrained = 0,
    /// Basic dodge ability
    Basic = 1,
    /// Intermediate dodging
    Practiced = 2,
    /// Expert at evasion
    Expert = 3,
    /// Master dodger
    Master = 4,
}

impl DodgeSkill {
    /// Get dodge chance percentage
    pub const fn dodge_chance(&self) -> u8 {
        match self {
            DodgeSkill::Untrained => 0,
            DodgeSkill::Basic => 5,
            DodgeSkill::Practiced => 15,
            DodgeSkill::Expert => 25,
            DodgeSkill::Master => 35,
        }
    }

    /// Get AC bonus from dodging
    pub const fn ac_bonus(&self) -> i32 {
        match self {
            DodgeSkill::Untrained => 0,
            DodgeSkill::Basic => 1,
            DodgeSkill::Practiced => 2,
            DodgeSkill::Expert => 3,
            DodgeSkill::Master => 4,
        }
    }
}

/// Armor effectiveness against different damage types
#[derive(Debug, Clone, Copy)]
pub struct ArmorEffectiveness {
    /// Physical damage protection (slashing, piercing, blunt)
    pub physical: f32,
    /// Fire damage protection
    pub fire: f32,
    /// Cold damage protection
    pub cold: f32,
    /// Electric damage protection
    pub electric: f32,
    /// Acid damage protection
    pub acid: f32,
    /// Poison damage protection
    pub poison: f32,
}

impl ArmorEffectiveness {
    /// Get effectiveness for a specific damage type
    pub fn for_damage_type(&self, damage_type: DamageType) -> f32 {
        match damage_type {
            DamageType::Physical | DamageType::Cut | DamageType::Stab | DamageType::Slash => {
                self.physical
            }
            DamageType::Fire => self.fire,
            DamageType::Cold => self.cold,
            DamageType::Electric | DamageType::Shock => self.electric,
            DamageType::Acid => self.acid,
            DamageType::Poison | DamageType::PoisonGas => self.poison,
            _ => 1.0, // No protection against unknown types
        }
    }
}

/// Get default armor effectiveness for an armor type
pub fn armor_effectiveness_for_type(armor_type: ArmorType) -> ArmorEffectiveness {
    match armor_type {
        ArmorType::Light => ArmorEffectiveness {
            physical: 0.8,
            fire: 0.5,
            cold: 0.5,
            electric: 0.6,
            acid: 0.3,
            poison: 0.2,
        },
        ArmorType::Medium => ArmorEffectiveness {
            physical: 0.6,
            fire: 0.4,
            cold: 0.4,
            electric: 0.5,
            acid: 0.2,
            poison: 0.1,
        },
        ArmorType::Heavy => ArmorEffectiveness {
            physical: 0.4,
            fire: 0.3,
            cold: 0.3,
            electric: 0.4,
            acid: 0.1,
            poison: 0.0,
        },
        ArmorType::Magical => ArmorEffectiveness {
            physical: 0.5,
            fire: 0.3,
            cold: 0.3,
            electric: 0.2,
            acid: 0.1,
            poison: 0.1,
        },
    }
}

/// Calculate total damage reduction from armor
///
/// Takes into account armor class, proficiency, and damage type
pub fn calculate_armor_damage_reduction(
    base_ac: i32,
    proficiency: ArmorProficiency,
    damage_type: DamageType,
    armor_type: ArmorType,
) -> f32 {
    // Base reduction from AC (each -1 AC reduces damage by 5%)
    let ac_reduction = (10 - base_ac).max(0) as f32 * 0.05;

    // Proficiency bonus reduces damage further
    let proficiency_reduction = proficiency.ac_bonus() as f32 * 0.05;

    // Armor type effectiveness against specific damage type
    let effectiveness = armor_effectiveness_for_type(armor_type);
    let type_effectiveness = effectiveness.for_damage_type(damage_type);

    // Combined reduction (capped at 90%)
    let total = ac_reduction + proficiency_reduction;
    let reduction = total * type_effectiveness;
    reduction.min(0.9)
}

/// Apply damage reduction to incoming damage
pub fn apply_damage_reduction(damage: i32, reduction: f32) -> i32 {
    let reduced = damage as f32 * (1.0 - reduction);
    reduced.max(1.0) as i32 // Minimum 1 damage always gets through
}

/// Attempt to dodge an attack
///
/// Returns true if attack is successfully dodged
pub fn attempt_dodge(
    dodge_skill: DodgeSkill,
    attacker_accuracy: i32,
    rng: &mut crate::rng::GameRng,
) -> bool {
    let base_chance = dodge_skill.dodge_chance() as i32;

    // Attacker accuracy reduces dodge chance
    let adjusted_chance = (base_chance - attacker_accuracy).max(0);

    let roll = rng.rnd(100) as i32;
    roll < adjusted_chance
}

/// Armor degradation/erosion tracking
#[derive(Debug, Clone, Copy)]
pub struct ArmorDegradation {
    /// Current erosion level
    pub erosion: i32,
    /// Maximum erosion before armor breaks
    pub max_erosion: i32,
}

impl ArmorDegradation {
    /// Create new armor degradation tracker
    pub const fn new(max_erosion: i32) -> Self {
        Self {
            erosion: 0,
            max_erosion,
        }
    }

    /// Get effectiveness factor (1.0 = perfect, 0.0 = broken)
    pub fn effectiveness_factor(&self) -> f32 {
        if self.max_erosion == 0 {
            return 1.0;
        }
        ((self.max_erosion - self.erosion) as f32 / self.max_erosion as f32).max(0.0)
    }

    /// Apply erosion damage to armor
    pub fn apply_erosion(&mut self, amount: i32) {
        self.erosion = (self.erosion + amount).min(self.max_erosion);
    }

    /// Check if armor is broken
    pub fn is_broken(&self) -> bool {
        self.erosion >= self.max_erosion
    }

    /// Repair armor (reduce erosion)
    pub fn repair(&mut self, amount: i32) {
        self.erosion = (self.erosion - amount).max(0);
    }
}

/// Comprehensive defense calculation
#[derive(Debug, Clone)]
pub struct DefenseCalculation {
    /// Base armor class
    pub base_ac: i32,
    /// Proficiency level
    pub proficiency: ArmorProficiency,
    /// Dodge skill
    pub dodge_skill: DodgeSkill,
    /// Current armor degradation
    pub degradation: ArmorDegradation,
    /// Effective AC after all modifiers
    pub effective_ac: i8,
    /// Damage reduction percentage
    pub damage_reduction: f32,
}

impl DefenseCalculation {
    /// Calculate total defense from components
    pub fn calculate(
        base_ac: i32,
        proficiency: ArmorProficiency,
        dodge_skill: DodgeSkill,
        degradation: ArmorDegradation,
    ) -> Self {
        // AC bonus from proficiency
        let prof_bonus = proficiency.ac_bonus() as i32;

        // AC bonus from dodge skill
        let dodge_bonus = dodge_skill.ac_bonus() as i32;

        // Apply armor degradation to effectiveness
        let degradation_factor = degradation.effectiveness_factor();

        // Calculate effective AC
        let total_bonus = prof_bonus + dodge_bonus;
        let effective_ac = (base_ac - total_bonus).clamp(-128, 127) as i8;

        // Damage reduction is based on proficiency and degradation
        let damage_reduction = (proficiency.ac_bonus() as f32 * 0.05) * degradation_factor;

        Self {
            base_ac,
            proficiency,
            dodge_skill,
            degradation,
            effective_ac,
            damage_reduction: damage_reduction.min(0.75),
        }
    }
}

/// Trace projectile path using Bresenham's line algorithm
///
/// Checks if a projectile can travel from (from_x, from_y) to (to_x, to_y)
/// without hitting walls or obstacles. Returns true if the path is clear,
/// false if blocked.
///
/// This is used for ranged attacks to prevent shooting through walls.
/// The algorithm walks the line pixel-by-pixel and checks each cell for
/// obstacles.
fn trace_projectile_path(
    from_x: i8,
    from_y: i8,
    to_x: i8,
    to_y: i8,
    level: &crate::dungeon::Level,
) -> bool {
    // Use Bresenham's line algorithm to trace the path
    let mut x = from_x as i32;
    let mut y = from_y as i32;
    let target_x = to_x as i32;
    let target_y = to_y as i32;

    // Calculate deltas
    let dx = (target_x - x).abs();
    let dy = (target_y - y).abs();
    let sx = if target_x > x { 1 } else { -1 };
    let sy = if target_y > y { 1 } else { -1 };

    // Determine which axis is dominant
    let (mut err, is_x_major) = if dx > dy {
        (dx / 2, true)
    } else {
        (dy / 2, false)
    };

    let max_steps = (dx + dy).max(1);

    for _ in 0..=max_steps {
        // Check if current cell blocks line of sight
        if !is_cell_transparent(x as usize, y as usize, level) {
            // Cell is blocked - projectile path is obstructed
            return false;
        }

        // Stop if we've reached the target
        if x == target_x && y == target_y {
            return true;
        }

        // Step along the line using Bresenham's algorithm
        if is_x_major {
            err -= dy;
            if err < 0 {
                y += sy;
                err += dx;
            }
            x += sx;
        } else {
            err -= dx;
            if err < 0 {
                x += sx;
                err += dy;
            }
            y += sy;
        }

        // Prevent going out of bounds
        if x < 0 || x >= 80 || y < 0 || y >= 21 {
            return false;
        }
    }

    true
}

/// Check if a cell allows projectiles to pass through
///
/// Returns true if the cell is passable by projectiles (transparent),
/// false if it's a wall or obstacle that blocks shots.
fn is_cell_transparent(x: usize, y: usize, level: &crate::dungeon::Level) -> bool {
    // Bounds check
    if x >= 80 || y >= 21 {
        return false;
    }

    // Check the cell type
    use crate::dungeon::CellType;

    let cell_type = level.cells[x][y].typ;

    // These cell types block projectiles
    match cell_type {
        // Walls block projectiles
        CellType::Stone
        | CellType::VWall
        | CellType::HWall
        | CellType::TLCorner
        | CellType::TRCorner
        | CellType::BLCorner
        | CellType::BRCorner
        | CellType::CrossWall
        | CellType::TUWall
        | CellType::TDWall
        | CellType::TLWall
        | CellType::TRWall
        | CellType::DBWall
        | CellType::IronBars
        | CellType::Wall => false,

        // Obstacles block projectiles
        CellType::Tree | CellType::Lava | CellType::Water | CellType::SecretDoor => false,

        // These allow projectiles to pass
        CellType::Room
        | CellType::Corridor
        | CellType::Door
        | CellType::SecretCorridor
        | CellType::Stairs
        | CellType::Ladder
        | CellType::Altar
        | CellType::Grave
        | CellType::Air
        | CellType::Cloud
        | CellType::Pool
        | CellType::Moat
        | CellType::DrawbridgeUp
        | CellType::DrawbridgeDown
        | CellType::Fountain
        | CellType::Throne
        | CellType::Sink
        | CellType::Ice
        | CellType::Vault => true,
    }
}

// ============================================================================
// Phase 13: Status Effects & Conditions System
// ============================================================================

/// All possible status effects that can affect entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffect {
    /// Poisoned: Deals damage over time, reduces Constitution
    Poisoned,
    /// Diseased: Attribute penalties, reduced healing effectiveness
    Diseased,
    /// Paralyzed: Severely reduces action capability
    Paralyzed,
    /// Stunned: Cannot act for a turn or two
    Stunned,
    /// Disarmed: Weapon dropped, cannot equip for duration
    Disarmed,
    /// Tripped: Reduced movement, AC penalty
    Tripped,
    /// Cursed: Penalty to all rolls and damage
    Cursed,
    /// Blinded: To-hit penalty, enemy dodge bonus
    Blinded,
    /// Drained: Permanent or long-lasting attribute reduction
    Drained,
    /// Petrified: Complete immobilization, high difficulty to cure
    Petrified,
    /// Bleeding: Gradual HP loss, can be severe if untreated
    Bleeding,
    /// Slimed: Turning into a green slime, fatal if not cured
    Slimed,
    /// Petrifying: Turning to stone, fatal if not cured
    Petrifying,
}

impl StatusEffect {
    /// Display name for the effect
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Poisoned => "Poisoned",
            Self::Diseased => "Diseased",
            Self::Paralyzed => "Paralyzed",
            Self::Stunned => "Stunned",
            Self::Disarmed => "Disarmed",
            Self::Tripped => "Tripped",
            Self::Cursed => "Cursed",
            Self::Blinded => "Blinded",
            Self::Drained => "Drained",
            Self::Petrified => "Petrified",
            Self::Bleeding => "Bleeding",
            Self::Slimed => "Slimed",
            Self::Petrifying => "Petrifying",
        }
    }

    /// All possible status effects
    pub const fn all() -> &'static [StatusEffect] {
        &[
            Self::Poisoned,
            Self::Diseased,
            Self::Paralyzed,
            Self::Stunned,
            Self::Disarmed,
            Self::Tripped,
            Self::Cursed,
            Self::Blinded,
            Self::Drained,
            Self::Petrified,
            Self::Bleeding,
            Self::Slimed,
            Self::Petrifying,
        ]
    }

    /// Default duration in turns for this effect (0 = permanent until cured)
    pub const fn default_duration(&self) -> u32 {
        match self {
            Self::Poisoned => 10,  // 10 turns
            Self::Diseased => 20,  // 20 turns
            Self::Paralyzed => 3,  // 3 turns
            Self::Stunned => 1,    // 1 turn
            Self::Disarmed => 5,   // 5 turns
            Self::Tripped => 2,    // 2 turns
            Self::Cursed => 15,    // 15 turns
            Self::Blinded => 8,    // 8 turns
            Self::Drained => 0,    // Permanent until cured
            Self::Petrified => 0,  // Permanent until cured
            Self::Bleeding => 12,  // 12 turns
            Self::Slimed => 10,    // 10 turns until full transformation
            Self::Petrifying => 5, // 5 turns until fully petrified
        }
    }

    /// Severity scale (1-10) affecting potency
    pub const fn max_severity(&self) -> u8 {
        match self {
            Self::Poisoned => 10,  // 1-10 poison damage per turn
            Self::Diseased => 5,   // Attribute reduction per severity
            Self::Paralyzed => 3,  // Action reduction multiplier
            Self::Stunned => 2,    // Number of missed turns
            Self::Disarmed => 3,   // Duration extension
            Self::Tripped => 3,    // Movement/AC penalty scale
            Self::Cursed => 5,     // Roll penalty scale
            Self::Blinded => 5,    // Accuracy penalty scale
            Self::Drained => 5,    // Attribute reduction amount
            Self::Petrified => 1,  // Binary (you're either petrified or not)
            Self::Bleeding => 10,  // 1-10 HP loss per turn
            Self::Slimed => 1,     // Binary (you're either turning to slime or not)
            Self::Petrifying => 1, // Binary (you're either turning to stone or not)
        }
    }
}

/// An instance of a status effect on an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffectInstance {
    /// The type of effect
    pub effect: StatusEffect,
    /// Remaining duration in turns (0 = expired)
    pub duration: u32,
    /// Severity level (1-max_severity for this effect)
    pub severity: u8,
    /// Source of the effect (e.g., "ogre's bite", "cursed item")
    pub source: String,
}

impl StatusEffectInstance {
    /// Create a new status effect instance
    pub fn new(effect: StatusEffect, severity: u8, source: impl Into<String>) -> Self {
        let max_severity = effect.max_severity();
        let clamped_severity = severity.min(max_severity).max(1);

        Self {
            effect,
            duration: effect.default_duration(),
            severity: clamped_severity,
            source: source.into(),
        }
    }

    /// Create with custom duration
    pub fn with_duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }

    /// Is this effect expired?
    pub const fn is_expired(&self) -> bool {
        self.duration == 0
    }

    /// Apply one turn of effect (decrease duration)
    pub fn tick(&mut self) {
        if self.duration > 0 {
            self.duration -= 1;
        }
    }

    /// Get remaining turns as percentage (0.0-1.0)
    pub fn remaining_percentage(&self) -> f32 {
        if self.effect.default_duration() == 0 {
            1.0 // Permanent effects show 100%
        } else {
            (self.duration as f32) / (self.effect.default_duration() as f32)
        }
    }
}

/// Manages all active status effects on an entity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusEffectTracker {
    /// List of active status effect instances
    effects: Vec<StatusEffectInstance>,
}

impl StatusEffectTracker {
    /// Create a new empty tracker
    pub const fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add a status effect (replaces if already exists, keeping worse effect)
    pub fn add_effect(&mut self, new_effect: StatusEffectInstance) {
        // Check if this effect type already exists
        if let Some(existing) = self
            .effects
            .iter_mut()
            .find(|e| e.effect == new_effect.effect)
        {
            // Keep the worse (higher severity or longer duration) effect
            if new_effect.severity > existing.severity
                || (new_effect.severity == existing.severity
                    && new_effect.duration > existing.duration)
            {
                *existing = new_effect;
            }
        } else {
            self.effects.push(new_effect);
        }
    }

    /// Remove a specific effect type
    pub fn remove_effect(&mut self, effect: StatusEffect) {
        self.effects.retain(|e| e.effect != effect);
    }

    /// Check if entity has a specific effect
    pub fn has_effect(&self, effect: StatusEffect) -> bool {
        self.effects
            .iter()
            .any(|e| e.effect == effect && !e.is_expired())
    }

    /// Get mutable reference to an effect if it exists
    pub fn get_effect_mut(&mut self, effect: StatusEffect) -> Option<&mut StatusEffectInstance> {
        self.effects
            .iter_mut()
            .find(|e| e.effect == effect && !e.is_expired())
    }

    /// Get immutable reference to an effect if it exists
    pub fn get_effect(&self, effect: StatusEffect) -> Option<&StatusEffectInstance> {
        self.effects
            .iter()
            .find(|e| e.effect == effect && !e.is_expired())
    }

    /// Get severity of an effect (0 if not present)
    pub fn get_severity(&self, effect: StatusEffect) -> u8 {
        self.get_effect(effect).map(|e| e.severity).unwrap_or(0)
    }

    /// Get all active effects
    pub fn active_effects(&self) -> impl Iterator<Item = &StatusEffectInstance> {
        self.effects.iter().filter(|e| !e.is_expired())
    }

    /// Tick all effects down by one turn, remove expired ones
    pub fn tick_all(&mut self) {
        for effect in &mut self.effects {
            effect.tick();
        }
        self.effects
            .retain(|e| !e.is_expired() || e.effect.default_duration() == 0);
    }

    /// Check if entity is immobilized (cannot move)
    pub fn is_immobilized(&self) -> bool {
        self.has_effect(StatusEffect::Paralyzed) || self.has_effect(StatusEffect::Petrified)
    }

    /// Check if entity is incapacitated (cannot act at all)
    pub fn is_incapacitated(&self) -> bool {
        self.is_immobilized() || self.has_effect(StatusEffect::Stunned)
    }

    /// Get total penalty to attack rolls from status effects
    pub fn attack_roll_penalty(&self) -> i32 {
        let mut penalty = 0;
        if self.has_effect(StatusEffect::Blinded) {
            penalty += self.get_severity(StatusEffect::Blinded) as i32;
        }
        if self.has_effect(StatusEffect::Paralyzed) {
            penalty += 2;
        }
        if self.has_effect(StatusEffect::Cursed) {
            penalty += self.get_severity(StatusEffect::Cursed) as i32;
        }
        if self.has_effect(StatusEffect::Stunned) {
            penalty += 3;
        }
        penalty
    }

    /// Get total penalty to AC (defense) from status effects
    pub fn ac_penalty(&self) -> i32 {
        let mut penalty = 0;
        if self.has_effect(StatusEffect::Tripped) {
            penalty += self.get_severity(StatusEffect::Tripped) as i32;
        }
        if self.has_effect(StatusEffect::Paralyzed) {
            penalty += 1;
        }
        penalty
    }

    /// Get damage multiplier from status effects (less than 1.0 = reduced)
    pub fn damage_multiplier(&self) -> f32 {
        let mut multiplier: f32 = 1.0;

        if self.has_effect(StatusEffect::Paralyzed) {
            multiplier *= 0.5; // Half damage when paralyzed
        }
        if self.has_effect(StatusEffect::Diseased) {
            multiplier *= 0.75; // 25% damage reduction when diseased
        }

        multiplier.max(0.1) // Minimum 10% damage output
    }

    /// Count total active effects
    pub fn active_effect_count(&self) -> usize {
        self.effects.iter().filter(|e| !e.is_expired()).count()
    }

    /// Clear all effects
    pub fn clear_all(&mut self) {
        self.effects.clear();
    }
}

// ============================================================================
// Combat Resources System (Phase 18)
// ============================================================================

/// Resources consumed during combat (mana, ability charges, cooldowns)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResources {
    /// Current mana pool
    pub mana_current: i32,

    /// Maximum mana capacity
    pub mana_max: i32,

    /// Breath weapon cooldown (turns remaining)
    pub breath_cooldown: u16,

    /// Spell casting cooldown (turns remaining)
    pub spell_cooldown: u16,

    /// Ability-specific cooldowns (turns remaining)
    pub ability_cooldowns: hashbrown::HashMap<String, u16>,

    /// Limited-use ability charges (uses remaining)
    pub ability_charges: hashbrown::HashMap<String, u8>,
}

impl Default for CombatResources {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatResources {
    /// Create a new resource pool for a monster (level scales mana)
    pub fn new() -> Self {
        Self {
            mana_current: 0,
            mana_max: 0,
            breath_cooldown: 0,
            spell_cooldown: 0,
            ability_cooldowns: hashbrown::HashMap::new(),
            ability_charges: hashbrown::HashMap::new(),
        }
    }

    /// Initialize resources based on monster level
    pub fn initialize(&mut self, level: u8) {
        self.mana_max = (level as i32) * 10;
        self.mana_current = self.mana_max;
    }

    /// Check if monster can use an ability (all resources available)
    pub fn can_use_ability(&self, ability_name: &str, mana_cost: i32) -> bool {
        // Check mana
        if self.mana_current < mana_cost {
            return false;
        }

        // Check cooldown
        if let Some(&cooldown) = self.ability_cooldowns.get(ability_name) {
            if cooldown > 0 {
                return false;
            }
        }

        // Check charges
        if let Some(&charges) = self.ability_charges.get(ability_name) {
            if charges == 0 {
                return false;
            }
        }

        true
    }

    /// Consume resources for using an ability
    pub fn use_ability(
        &mut self,
        ability_name: &str,
        mana_cost: i32,
        cooldown_turns: u16,
        uses_charge: bool,
    ) {
        // Consume mana
        self.mana_current = (self.mana_current - mana_cost).max(0);

        // Set cooldown
        self.ability_cooldowns
            .insert(ability_name.to_string(), cooldown_turns);

        // Use charge if applicable
        if uses_charge {
            self.ability_charges
                .entry(ability_name.to_string())
                .and_modify(|c| *c = c.saturating_sub(1))
                .or_insert(0);
        }
    }

    /// Regenerate mana at end of turn (standard: 1/20th of max per turn)
    pub fn regenerate_mana(&mut self) {
        if self.mana_current < self.mana_max {
            let regen = (self.mana_max / 20).max(1);
            self.mana_current = (self.mana_current + regen).min(self.mana_max);
        }
    }

    /// Tick down all cooldowns by one turn
    pub fn tick_cooldowns(&mut self) {
        // Tick breath weapon cooldown
        if self.breath_cooldown > 0 {
            self.breath_cooldown -= 1;
        }

        // Tick spell cooldown
        if self.spell_cooldown > 0 {
            self.spell_cooldown -= 1;
        }

        // Tick ability-specific cooldowns
        for cooldown in self.ability_cooldowns.values_mut() {
            if *cooldown > 0 {
                *cooldown -= 1;
            }
        }

        // Remove expired cooldowns
        self.ability_cooldowns.retain(|_, &mut cd| cd > 0);
    }

    /// Check if breath weapon is ready
    pub fn breath_ready(&self) -> bool {
        self.breath_cooldown == 0
    }

    /// Check if spell casting is ready
    pub fn spells_ready(&self) -> bool {
        self.spell_cooldown == 0
    }

    /// Use breath weapon (sets cooldown)
    pub fn use_breath(&mut self) {
        self.breath_cooldown = 8; // Default 8-turn cooldown
    }

    /// Use spell (sets cooldown)
    pub fn use_spell(&mut self) {
        self.spell_cooldown = 3; // Default 3-turn cooldown
    }
}

// ============================================================================
// Effect Impact Functions
// ============================================================================

/// Calculate damage dealt per turn by poison
pub const fn poison_damage_per_turn(severity: u8) -> i32 {
    let s = severity as i32;
    if s < 10 { s } else { 10 }
}

/// Calculate attribute reduction from disease (per attribute)
pub const fn disease_attribute_reduction(severity: u8) -> i32 {
    match severity {
        1 => 1,
        2 => 1,
        3 => 2,
        4 => 2,
        5 => 3,
        _ => 3,
    }
}

/// Calculate bleeding damage per turn
pub const fn bleeding_damage_per_turn(severity: u8) -> i32 {
    let s = severity as i32;
    if s < 10 { s } else { 10 }
}

/// Calculate paralysis action reduction percentage
pub const fn paralysis_action_reduction(severity: u8) -> f32 {
    match severity {
        1 => 0.33, // 33% reduction
        2 => 0.66, // 66% reduction
        3 => 1.0,  // Complete paralysis
        _ => 1.0,
    }
}

/// Calculate curse penalty to rolls
pub const fn curse_roll_penalty(severity: u8) -> i32 {
    let s = severity as i32;
    if s < 5 { s } else { 5 }
}

/// Calculate blindness to-hit penalty
pub const fn blindness_accuracy_penalty(severity: u8) -> i32 {
    let s = severity as i32;
    if s < 5 { s } else { 5 }
}

// ============================================================================
// Effect Application Functions
// ============================================================================

/// Apply a status effect to an entity's tracker
pub fn apply_status_effect(
    tracker: &mut StatusEffectTracker,
    effect: StatusEffect,
    severity: u8,
    source: &str,
) {
    let instance = StatusEffectInstance::new(effect, severity, source);
    tracker.add_effect(instance);
}

/// Apply a status effect with custom duration
pub fn apply_status_effect_with_duration(
    tracker: &mut StatusEffectTracker,
    effect: StatusEffect,
    severity: u8,
    duration: u32,
    source: &str,
) {
    let instance = StatusEffectInstance::new(effect, severity, source).with_duration(duration);
    tracker.add_effect(instance);
}

/// Remove a status effect from an entity's tracker
pub fn remove_status_effect(tracker: &mut StatusEffectTracker, effect: StatusEffect) {
    tracker.remove_effect(effect);
}

/// Determine if a special combat effect should trigger based on skill level
pub fn should_trigger_special_effect(
    effect_type: &SpecialCombatEffect,
    skill_level: &SkillLevel,
    rng: &mut crate::rng::GameRng,
) -> bool {
    if !effect_type.can_attempt_with_skill(*skill_level) {
        return false;
    }

    let success_chance = effect_type.base_success_chance() as i32;
    rng.rnd(100) as i32 <= success_chance
}

/// Apply effects from a special combat effect to target
pub fn apply_special_effect(
    effect_type: &SpecialCombatEffect,
    target_tracker: &mut StatusEffectTracker,
    source: &str,
    severity: u8,
) {
    match effect_type {
        SpecialCombatEffect::Disarm => {
            apply_status_effect(target_tracker, StatusEffect::Disarmed, 1, source);
        }
        SpecialCombatEffect::Trip => {
            apply_status_effect(
                target_tracker,
                StatusEffect::Tripped,
                severity.min(3),
                source,
            );
        }
        SpecialCombatEffect::Stun => {
            apply_status_effect(target_tracker, StatusEffect::Stunned, 1, source);
        }
        SpecialCombatEffect::Poison => {
            apply_status_effect(target_tracker, StatusEffect::Poisoned, severity, source);
        }
        SpecialCombatEffect::LifeDrain => {
            apply_status_effect(target_tracker, StatusEffect::Drained, severity, source);
        }
        SpecialCombatEffect::Disease => {
            apply_status_effect(target_tracker, StatusEffect::Diseased, severity, source);
        }
        SpecialCombatEffect::Petrify => {
            apply_status_effect(target_tracker, StatusEffect::Petrified, 1, source);
        }
    }
}

/// Get the appropriate severity for an effect based on attacker's skill
pub const fn effect_severity_from_skill(skill_level: &SkillLevel) -> u8 {
    match skill_level {
        SkillLevel::Unskilled => 1,
        SkillLevel::Basic => 2,
        SkillLevel::Skilled => 3,
        SkillLevel::Expert => 4,
        SkillLevel::Master => 5,
    }
}

/// Calculate passive damage from status effects (damage per turn)
pub fn calculate_status_damage(tracker: &StatusEffectTracker) -> i32 {
    let mut damage = 0;

    if let Some(poison) = tracker.get_effect(StatusEffect::Poisoned) {
        damage += poison_damage_per_turn(poison.severity);
    }

    if let Some(bleed) = tracker.get_effect(StatusEffect::Bleeding) {
        damage += bleeding_damage_per_turn(bleed.severity);
    }

    damage.max(0) // Never negative
}

/// Get combat modifier effects from status effects
pub fn get_status_effect_modifiers(tracker: &StatusEffectTracker) -> (i32, i32) {
    let to_hit_mod = -tracker.attack_roll_penalty();
    // damage_multiplier is 0.1 to 1.0, convert to -90 to 0 modifier
    let damage_mod = ((tracker.damage_multiplier() - 1.0) * 100.0) as i32;
    (to_hit_mod, damage_mod)
}

// ============================================================================
// Turn-Based Status Effect Processing
// ============================================================================

/// Process status effects for a player at turn end
pub fn process_player_status_effects(
    player: &mut crate::player::You,
    rng: &mut crate::rng::GameRng,
) {
    // Tick all effects down by one turn
    player.status_effects.tick_all();

    // Apply passive damage from active effects
    let damage = calculate_status_damage(&player.status_effects);
    if damage > 0 {
        player.hp = (player.hp - damage).max(0);
    }

    // Check and sync with old status timeout fields for compatibility
    if player.status_effects.has_effect(StatusEffect::Blinded) {
        player.blinded_timeout = (player.blinded_timeout + 1).min(u16::MAX);
    }

    if player.status_effects.has_effect(StatusEffect::Stunned) {
        player.stunned_timeout = (player.stunned_timeout + 1).min(u16::MAX);
    }

    if player.status_effects.has_effect(StatusEffect::Paralyzed) {
        player.paralyzed_timeout = (player.paralyzed_timeout + 1).min(u16::MAX);
    }
}

/// Process status effects for a monster at turn end
pub fn process_monster_status_effects(monster: &mut Monster, _rng: &mut crate::rng::GameRng) {
    // Tick all effects down by one turn
    monster.status_effects.tick_all();

    // Apply passive damage from active effects
    let damage = calculate_status_damage(&monster.status_effects);
    if damage > 0 {
        monster.hp = (monster.hp - damage).max(0);
    }

    // Check and sync with old status timeout fields for compatibility
    if monster.status_effects.has_effect(StatusEffect::Blinded) {
        monster.blinded_timeout = (monster.blinded_timeout + 1).min(u16::MAX);
    }

    if monster.status_effects.has_effect(StatusEffect::Paralyzed) {
        monster.frozen_timeout = (monster.frozen_timeout + 1).min(u16::MAX);
    }
}

/// Check if entity is under the effects of a specific condition
pub fn has_condition(tracker: &StatusEffectTracker, effect: StatusEffect) -> bool {
    tracker.has_effect(effect)
}

/// Apply effect and get a human-readable message for the application
pub fn apply_condition_with_message(
    tracker: &mut StatusEffectTracker,
    effect: StatusEffect,
    severity: u8,
    source: &str,
) -> String {
    apply_status_effect(tracker, effect, severity, source);
    format!("{} by {}", effect.name(), source)
}

// ============================================================================
// Phase 14: Experience & Leveling System
// ============================================================================

/// Experience reward from defeating an enemy
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExperienceReward {
    /// Base XP for killing this monster
    pub base_xp: u32,
    /// Level bonus: additional XP per level difference
    pub level_bonus: u32,
    /// Difficulty multiplier (1.0 = normal, 1.5 = difficult, 0.5 = easy)
    pub difficulty_multiplier: f32,
}

impl ExperienceReward {
    /// Create a standard XP reward
    pub const fn new(base_xp: u32, level_bonus: u32) -> Self {
        Self {
            base_xp,
            level_bonus,
            difficulty_multiplier: 1.0,
        }
    }

    /// Create with difficulty adjustment
    pub const fn with_difficulty(mut self, multiplier: f32) -> Self {
        self.difficulty_multiplier = multiplier;
        self
    }

    /// Calculate total XP based on player level and monster difficulty
    pub fn calculate_total(
        &self,
        player_level: i32,
        monster_level: i32,
        player_health_percent: f32,
    ) -> u32 {
        let mut xp = self.base_xp as f32;

        // Bonus for defeating higher-level monsters
        let level_diff = (monster_level - player_level).max(-10).min(10);
        if level_diff > 0 {
            xp += self.level_bonus as f32 * level_diff as f32;
        }

        // Penalty for low health (encourage challenging fights, not death-prone victories)
        let health_multiplier = (player_health_percent * 0.5 + 0.5).max(0.1); // 0.1 to 1.0
        xp *= health_multiplier;

        // Apply difficulty multiplier
        xp *= self.difficulty_multiplier;

        xp.max(1.0) as u32 // Minimum 1 XP
    }
}

/// Skill advancement from using a weapon repeatedly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillAdvancement {
    /// Hits in a row with this weapon
    pub hits: u32,
    /// Misses in a row with this weapon
    pub misses: u32,
    /// Experience accumulated toward next skill level
    pub experience: u32,
    /// Experience needed for next skill level
    pub next_level_threshold: u32,
}

impl SkillAdvancement {
    /// Create a new skill advancement tracker
    pub const fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            experience: 0,
            next_level_threshold: 100,
        }
    }

    /// Record a successful hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.misses = 0;
        self.experience += 3; // 3 points per hit
    }

    /// Record a miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.hits = 0;
        self.experience = self.experience.saturating_sub(1); // -1 point per miss
    }

    /// Check if skill should advance and get new threshold
    pub fn should_advance(&self) -> bool {
        self.experience >= self.next_level_threshold
    }

    /// Advance skill level and return new threshold
    pub fn advance(&mut self) -> u32 {
        self.experience = self.experience.saturating_sub(self.next_level_threshold);
        // Each level requires 50% more experience to advance
        self.next_level_threshold = (self.next_level_threshold as f32 * 1.5) as u32;
        self.next_level_threshold.max(50) // Never below 50
    }

    /// Reset hit/miss streak
    pub fn reset_streak(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }
}

impl Default for SkillAdvancement {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute gain from leveling up
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeGain {
    /// Strength gain
    pub strength: i32,
    /// Dexterity gain
    pub dexterity: i32,
    /// Constitution gain
    pub constitution: i32,
    /// Intelligence gain
    pub intelligence: i32,
    /// Wisdom gain
    pub wisdom: i32,
    /// Charisma gain
    pub charisma: i32,
}

impl AttributeGain {
    /// Create new attribute gains
    pub const fn new(
        strength: i32,
        dexterity: i32,
        constitution: i32,
        intelligence: i32,
        wisdom: i32,
        charisma: i32,
    ) -> Self {
        Self {
            strength,
            dexterity,
            constitution,
            intelligence,
            wisdom,
            charisma,
        }
    }

    /// Create uniform gain for all attributes
    pub const fn uniform(gain: i32) -> Self {
        Self {
            strength: gain,
            dexterity: gain,
            constitution: gain,
            intelligence: gain,
            wisdom: gain,
            charisma: gain,
        }
    }

    /// Get total gain
    pub const fn total(&self) -> i32 {
        self.strength
            + self.dexterity
            + self.constitution
            + self.intelligence
            + self.wisdom
            + self.charisma
    }
}

/// Experience threshold for each level
pub const fn xp_for_level(level: i32) -> u64 {
    match level {
        1 => 0,
        2 => 1000,
        3 => 3000,
        4 => 6000,
        5 => 10000,
        6 => 15000,
        7 => 21000,
        8 => 28000,
        9 => 36000,
        10 => 45000,
        11 => 55000,
        12 => 66000,
        13 => 78000,
        14 => 91000,
        15 => 105000,
        16 => 120000,
        17 => 136000,
        18 => 153000,
        19 => 171000,
        20 => 190000,
        21 => 210000,
        22 => 231000,
        23 => 253000,
        24 => 276000,
        25 => 300000,
        26 => 325000,
        27 => 351000,
        28 => 378000,
        29 => 406000,
        30 => 435000,
        n if n > 30 => {
            // After level 30, add 30000 per level
            435000 + ((n - 30) as u64 * 30000)
        }
        _ => u64::MAX,
    }
}

/// Get attribute gain for leveling from level `from` to `to`
pub fn get_attribute_gain_for_level(role: crate::player::Role, _level: i32) -> AttributeGain {
    // Different roles get different attribute gains per level
    match role {
        crate::player::Role::Barbarian => AttributeGain::new(2, 1, 2, 0, 0, -1),
        crate::player::Role::Ranger => AttributeGain::new(1, 2, 1, 1, 1, 0),
        crate::player::Role::Monk => AttributeGain::new(1, 2, 1, 0, 1, 0),
        crate::player::Role::Wizard => AttributeGain::new(0, 1, 0, 2, 1, 0),
        crate::player::Role::Priest => AttributeGain::new(1, 0, 1, 0, 2, 1),
        crate::player::Role::Rogue => AttributeGain::new(0, 2, 1, 1, 0, 1),
        crate::player::Role::Knight => AttributeGain::new(2, 1, 2, 0, 1, 1),
        crate::player::Role::Knight => AttributeGain::new(2, 1, 2, 0, 2, 1),
        crate::player::Role::Valkyrie => AttributeGain::new(2, 1, 2, 0, 1, 0),
        _ => AttributeGain::uniform(1), // Default 1 to each attribute
    }
}

/// HP gain per level (base + CON bonus)
pub fn calculate_hp_gain(constitution: i32, role: crate::player::Role) -> i32 {
    let base_hp_gain = match role {
        crate::player::Role::Barbarian => 12,
        crate::player::Role::Knight
        | crate::player::Role::Knight
        | crate::player::Role::Valkyrie => 10,
        crate::player::Role::Ranger => 9,
        crate::player::Role::Monk => 8,
        crate::player::Role::Rogue => 8,
        crate::player::Role::Priest => 8,
        crate::player::Role::Wizard => 6,
        _ => 8,
    };

    let con_bonus = (constitution - 10) / 2; // +1 per 2 points above 10
    let total = base_hp_gain + con_bonus.max(0);
    total.max(1) // Minimum 1 HP gain
}

/// Mana/Energy gain per level (base + INT bonus)
pub fn calculate_mana_gain(intelligence: i32, wisdom: i32, role: crate::player::Role) -> i32 {
    let base_mana_gain = match role {
        crate::player::Role::Wizard => 10,
        crate::player::Role::Priest => 8,
        crate::player::Role::Ranger => 6,
        _ => 4,
    };

    // Casters benefit more from INT/WIS
    let int_bonus = (intelligence - 10) / 3;
    let wis_bonus = (wisdom - 10) / 4;
    let total = base_mana_gain + int_bonus.max(0) + wis_bonus.max(0);
    total.max(0)
}

// ============================================================================
// XP and Leveling Application Functions
// ============================================================================

/// Calculate XP reward for defeating a monster
pub fn calculate_monster_xp_reward(monster: &Monster) -> ExperienceReward {
    // Base XP scales with monster level (roughly level * level * 50)
    let base_xp = ((monster.level as u32) * (monster.level as u32) * 50).max(10);
    let level_bonus = monster.level as u32 * 25;

    ExperienceReward::new(base_xp, level_bonus)
}

/// Award XP to player and check for level up
pub fn award_player_xp(player: &mut crate::player::You, xp_amount: u32) -> bool {
    player.exp += xp_amount as u64;

    // Check if player levels up
    let mut leveled_up = false;
    loop {
        let next_level = player.exp_level + 1;
        let xp_needed = xp_for_level(next_level);

        if player.exp >= xp_needed {
            level_up_player(player);
            leveled_up = true;
        } else {
            break;
        }
    }

    leveled_up
}

/// Apply level up effects to player
fn level_up_player(player: &mut crate::player::You) {
    player.exp_level += 1;
    player.max_exp_level = player.max_exp_level.max(player.exp_level);

    // Get attribute gains from role
    let attr_gain = get_attribute_gain_for_level(player.role, player.exp_level);

    // Apply attribute increases
    player.attr_current.set(
        crate::player::Attribute::Strength,
        (player.attr_current.get(crate::player::Attribute::Strength) as i32 + attr_gain.strength)
            .max(1) as i8,
    );
    player.attr_current.set(
        crate::player::Attribute::Dexterity,
        (player.attr_current.get(crate::player::Attribute::Dexterity) as i32 + attr_gain.dexterity)
            .max(1) as i8,
    );
    player.attr_current.set(
        crate::player::Attribute::Constitution,
        (player
            .attr_current
            .get(crate::player::Attribute::Constitution) as i32
            + attr_gain.constitution)
            .max(1) as i8,
    );
    player.attr_current.set(
        crate::player::Attribute::Intelligence,
        (player
            .attr_current
            .get(crate::player::Attribute::Intelligence) as i32
            + attr_gain.intelligence)
            .max(1) as i8,
    );
    player.attr_current.set(
        crate::player::Attribute::Wisdom,
        (player.attr_current.get(crate::player::Attribute::Wisdom) as i32 + attr_gain.wisdom).max(1)
            as i8,
    );
    player.attr_current.set(
        crate::player::Attribute::Charisma,
        (player.attr_current.get(crate::player::Attribute::Charisma) as i32 + attr_gain.charisma)
            .max(1) as i8,
    );

    // Get constitution for HP calculation
    let constitution = player
        .attr_current
        .get(crate::player::Attribute::Constitution) as i32;
    let hp_gain = calculate_hp_gain(constitution, player.role);
    player.hp_max += hp_gain;
    player.hp = player.hp.saturating_add(hp_gain); // Heal on level up

    // Calculate mana gain
    let intelligence = player
        .attr_current
        .get(crate::player::Attribute::Intelligence) as i32;
    let wisdom = player.attr_current.get(crate::player::Attribute::Wisdom) as i32;
    let mana_gain = calculate_mana_gain(intelligence, wisdom, player.role);
    if mana_gain > 0 {
        player.energy_max += mana_gain;
        player.energy = player.energy.saturating_add(mana_gain);
    }

    // Record level increase for potential future bonuses
    player.hp_increases.push(hp_gain as i8);
    player.energy_increases.push(mana_gain as i8);
}

/// Award XP to a monster and level it up if needed
pub fn award_monster_xp(monster: &mut Monster, xp_amount: u32) {
    // Simple exponential growth: each level requires level * 100 XP
    let current_xp_needed = (monster.level as u32 + 1) * 100;

    if xp_amount >= current_xp_needed {
        level_up_monster(monster);
    }
}

/// Apply level up effects to monster
fn level_up_monster(monster: &mut Monster) {
    monster.level = monster.level.saturating_add(1).min(30); // Cap at level 30

    // Monsters gain attributes on level up
    // +1 STR, +1 DEX, +1 CON per level
    let constitution_bonus = 1;

    // Increase HP: 5 + CON bonus per level
    let hp_gain = (5 + constitution_bonus).max(1) as i32;
    monster.hp_max += hp_gain;
    monster.hp += hp_gain; // Heal on level up

    // Improve AC by 1 (lower is better)
    monster.ac = monster.ac.saturating_sub(1).max(-20);
}

// ============================================================================
// Phase 15: Spellcasting Combat Integration
// ============================================================================

/// Combat spell types - offensive, defensive, and utility spells for battle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatSpell {
    /// Offensive: Direct damage spell
    ForceBolt,
    /// Offensive: Multi-target damage
    Fireball,
    /// Offensive: Freeze damage
    ConeOfCold,
    /// Offensive: Instant death spell
    FingerOfDeath,
    /// Offensive: Life drain
    Drain,
    /// Offensive: Ranged magic projectile
    MagicMissile,
    /// Debuff: Confuse target
    Confuse,
    /// Debuff: Slow target
    Slow,
    /// Debuff: Put target to sleep
    Sleep,
    /// Debuff: Cancel magic
    Cancellation,
    /// Buff: Haste self
    Haste,
    /// Buff: Invisibility
    Invisibility,
    /// Buff: Stone skin
    StoneSkin,
    /// Buff: Protection
    Protection,
    /// Healing: Restore HP
    Healing,
    /// Healing: Extra healing
    ExtraHealing,
}

impl CombatSpell {
    /// Get the spell's name
    pub const fn name(&self) -> &'static str {
        match self {
            Self::ForceBolt => "force bolt",
            Self::Fireball => "fireball",
            Self::ConeOfCold => "cone of cold",
            Self::FingerOfDeath => "finger of death",
            Self::Drain => "drain life",
            Self::MagicMissile => "magic missile",
            Self::Confuse => "confuse",
            Self::Slow => "slow",
            Self::Sleep => "sleep",
            Self::Cancellation => "cancellation",
            Self::Haste => "haste",
            Self::Invisibility => "invisibility",
            Self::StoneSkin => "stone skin",
            Self::Protection => "protection",
            Self::Healing => "healing",
            Self::ExtraHealing => "extra healing",
        }
    }

    /// Get the associated SpellType from Phase 8
    pub const fn spell_type(&self) -> crate::magic::spell::SpellType {
        match self {
            Self::ForceBolt => crate::magic::spell::SpellType::ForceBolt,
            Self::Fireball => crate::magic::spell::SpellType::Fireball,
            Self::ConeOfCold => crate::magic::spell::SpellType::ConeOfCold,
            Self::FingerOfDeath => crate::magic::spell::SpellType::FingerOfDeath,
            Self::Drain => crate::magic::spell::SpellType::Drain,
            Self::MagicMissile => crate::magic::spell::SpellType::MagicMissile,
            Self::Confuse => crate::magic::spell::SpellType::Confuse,
            Self::Slow => crate::magic::spell::SpellType::Slow,
            Self::Sleep => crate::magic::spell::SpellType::Sleep,
            Self::Cancellation => crate::magic::spell::SpellType::Cancellation,
            Self::Haste => crate::magic::spell::SpellType::Haste,
            Self::Invisibility => crate::magic::spell::SpellType::Invisibility,
            Self::StoneSkin => crate::magic::spell::SpellType::StoneSkin,
            Self::Protection => crate::magic::spell::SpellType::Protection,
            Self::Healing => crate::magic::spell::SpellType::Healing,
            Self::ExtraHealing => crate::magic::spell::SpellType::ExtraHealing,
        }
    }

    /// Get the base mana cost for this spell
    pub const fn mana_cost(&self) -> i32 {
        match self {
            Self::ForceBolt => 15,
            Self::Fireball => 50,
            Self::ConeOfCold => 50,
            Self::FingerOfDeath => 100,
            Self::Drain => 60,
            Self::MagicMissile => 20,
            Self::Confuse => 25,
            Self::Slow => 20,
            Self::Sleep => 25,
            Self::Cancellation => 40,
            Self::Haste => 30,
            Self::Invisibility => 35,
            Self::StoneSkin => 40,
            Self::Protection => 35,
            Self::Healing => 20,
            Self::ExtraHealing => 40,
        }
    }

    /// Get casting time in turns (higher = slower to cast)
    pub const fn casting_time(&self) -> u32 {
        match self {
            Self::ForceBolt | Self::MagicMissile | Self::Healing => 1,
            Self::Confuse | Self::Slow | Self::Sleep | Self::Haste => 2,
            Self::Fireball | Self::ConeOfCold | Self::Drain | Self::ExtraHealing => 3,
            Self::StoneSkin | Self::Protection | Self::Invisibility | Self::Cancellation => 2,
            Self::FingerOfDeath => 5, // Instant kill is slow!
        }
    }

    /// Get damage for offensive spells
    pub const fn base_damage(&self) -> i32 {
        match self {
            Self::ForceBolt => 20,
            Self::Fireball => 50,
            Self::ConeOfCold => 50,
            Self::FingerOfDeath => 999, // Instant kill
            Self::Drain => 40,
            Self::MagicMissile => 15,
            _ => 0,
        }
    }

    /// Can this spell affect a specific target type (undead, etc)?
    pub const fn affects_undead(&self) -> bool {
        matches!(self, Self::FingerOfDeath | Self::Drain | Self::Slow)
    }

    /// Get failure chance based on spell difficulty and caster level
    pub fn failure_chance(&self, caster_level: i32, intelligence: i8) -> i32 {
        let base_failure = match self {
            Self::ForceBolt | Self::Healing => 10,
            Self::MagicMissile | Self::Confuse | Self::Slow | Self::Sleep => 20,
            Self::Fireball | Self::ConeOfCold => 30,
            Self::Drain => 40,
            Self::FingerOfDeath => 60,
            _ => 15,
        };

        // Intelligence bonus reduces failure
        let int_bonus = (intelligence as i32 - 10) / 2;
        let level_bonus = caster_level / 2;

        (base_failure - int_bonus - level_bonus).max(5)
    }

    /// All combat spells
    pub const fn all() -> &'static [CombatSpell] {
        &[
            Self::ForceBolt,
            Self::Fireball,
            Self::ConeOfCold,
            Self::FingerOfDeath,
            Self::Drain,
            Self::MagicMissile,
            Self::Confuse,
            Self::Slow,
            Self::Sleep,
            Self::Cancellation,
            Self::Haste,
            Self::Invisibility,
            Self::StoneSkin,
            Self::Protection,
            Self::Healing,
            Self::ExtraHealing,
        ]
    }
}

/// Result of spell casting in combat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellCastResult {
    /// Whether the spell succeeded
    pub success: bool,
    /// Whether the spell was interrupted mid-cast
    pub interrupted: bool,
    /// Damage dealt (if applicable)
    pub damage: i32,
    /// Status effect applied (if applicable)
    pub effect_applied: Option<StatusEffect>,
}

impl SpellCastResult {
    /// Create a successful spell cast
    pub const fn success() -> Self {
        Self {
            success: true,
            interrupted: false,
            damage: 0,
            effect_applied: None,
        }
    }

    /// Create a failed spell cast
    pub const fn failed() -> Self {
        Self {
            success: false,
            interrupted: false,
            damage: 0,
            effect_applied: None,
        }
    }

    /// Create an interrupted spell
    pub const fn interrupted() -> Self {
        Self {
            success: false,
            interrupted: true,
            damage: 0,
            effect_applied: None,
        }
    }

    /// Add damage to result
    pub const fn with_damage(mut self, damage: i32) -> Self {
        self.damage = damage;
        self
    }

    /// Add status effect to result
    pub const fn with_effect(mut self, effect: StatusEffect) -> Self {
        self.effect_applied = Some(effect);
        self
    }
}

/// Role-specific combat spell lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatSpellList;

impl CombatSpellList {
    /// Get combat spells available to a specific role
    pub fn get_spells_for_role(role: crate::player::Role) -> &'static [CombatSpell] {
        match role {
            crate::player::Role::Wizard => &[
                CombatSpell::ForceBolt,
                CombatSpell::MagicMissile,
                CombatSpell::Fireball,
                CombatSpell::ConeOfCold,
                CombatSpell::Confuse,
                CombatSpell::Slow,
                CombatSpell::Sleep,
                CombatSpell::Cancellation,
                CombatSpell::FingerOfDeath,
            ],
            crate::player::Role::Priest => &[
                CombatSpell::Healing,
                CombatSpell::ExtraHealing,
                CombatSpell::Protection,
                CombatSpell::StoneSkin,
                CombatSpell::Confuse,
                CombatSpell::Slow,
                CombatSpell::Drain,
            ],
            crate::player::Role::Ranger => &[
                CombatSpell::ForceBolt,
                CombatSpell::Healing,
                CombatSpell::Slow,
                CombatSpell::Protection,
            ],
            crate::player::Role::Monk => &[
                CombatSpell::ForceBolt,
                CombatSpell::Haste,
                CombatSpell::Protection,
                CombatSpell::Healing,
            ],
            crate::player::Role::Rogue => &[
                CombatSpell::MagicMissile,
                CombatSpell::Invisibility,
                CombatSpell::Confuse,
                CombatSpell::Healing,
            ],
            crate::player::Role::Knight => &[
                CombatSpell::Healing,
                CombatSpell::Protection,
                CombatSpell::StoneSkin,
                CombatSpell::ForceBolt,
            ],
            _ => &[
                CombatSpell::ForceBolt,
                CombatSpell::Healing,
                CombatSpell::Protection,
            ], // Default spells
        }
    }
}

/// Check if player can cast a spell (has mana and spell known)
pub fn can_player_cast_spell(player: &crate::player::You, spell: CombatSpell) -> bool {
    // Check if player has enough mana
    let mana_cost = spell.mana_cost();
    if player.energy < mana_cost {
        return false;
    }

    // Check if spell is known
    let spell_type = spell.spell_type();
    player
        .known_spells
        .iter()
        .any(|s| s.spell_type == spell_type && !s.is_forgotten())
}

/// Get mana cost adjusted for caster's intelligence and spell mastery
pub fn get_adjusted_mana_cost(player: &crate::player::You, spell: CombatSpell) -> i32 {
    let base_cost = spell.mana_cost();
    let intelligence = player
        .attr_current
        .get(crate::player::Attribute::Intelligence) as f32;

    // Intelligence reduces mana cost: every 2 points above 10 = 5% reduction
    let int_modifier = ((intelligence - 10.0) / 2.0) * 0.05;
    let reduced_cost = (base_cost as f32 * (1.0 - int_modifier)).max(1.0) as i32;

    reduced_cost
}

/// Calculate spell damage with caster modifiers
pub fn calculate_spell_damage(
    caster_level: i32,
    spell: CombatSpell,
    intelligence: i8,
    status_effects: &StatusEffectTracker,
) -> i32 {
    let base_damage = spell.base_damage();
    if base_damage <= 0 {
        return 0;
    }

    // Intelligence adds damage (every 2 points above 10 = +1 damage)
    let int_bonus = (intelligence as i32 - 10) / 2;

    // Level adds damage (1 per 3 levels)
    let level_bonus = caster_level / 3;

    // Status effects reduce damage output
    let damage_multiplier = status_effects.damage_multiplier();

    let total = (base_damage as f32 + int_bonus as f32 + level_bonus as f32) * damage_multiplier;
    total.max(1.0) as i32
}

/// Check if spell cast is interrupted by damage
pub fn check_spell_interruption(
    damage_taken: i32,
    casting_time: u32,
    concentration_dc: i32,
    rng: &mut crate::rng::GameRng,
) -> bool {
    // Need to make a concentration check
    // DC = 10 + damage taken / 10 + casting time bonus
    let difficulty = 10 + (damage_taken / 10) + casting_time as i32;

    // Roll d20 + caster modifier
    let roll = rng.rnd(20) as i32 + concentration_dc;

    roll < difficulty
}

/// Cast a combat spell with full effects
pub fn cast_combat_spell(
    caster: &mut crate::player::You,
    target: &mut Monster,
    spell: CombatSpell,
    rng: &mut crate::rng::GameRng,
) -> SpellCastResult {
    // Check if can cast
    if !can_player_cast_spell(caster, spell) {
        return SpellCastResult::failed();
    }

    // Get adjusted mana cost
    let mana_cost = get_adjusted_mana_cost(caster, spell);
    caster.energy -= mana_cost;

    // Roll for casting success (failure chance based on spell difficulty)
    let intelligence = caster
        .attr_current
        .get(crate::player::Attribute::Intelligence);
    let failure_chance = spell.failure_chance(caster.exp_level, intelligence);

    if rng.rnd(100) as i32 <= failure_chance {
        return SpellCastResult::failed();
    }

    // Calculate damage and apply effects
    let damage = calculate_spell_damage(
        caster.exp_level,
        spell,
        intelligence,
        &caster.status_effects,
    );

    let mut result = SpellCastResult::success().with_damage(damage);

    match spell {
        CombatSpell::ForceBolt | CombatSpell::MagicMissile => {
            target.hp -= damage;
        }
        CombatSpell::Fireball | CombatSpell::ConeOfCold => {
            // Multi-target: damage is full damage
            target.hp -= damage;
            result = result.with_effect(StatusEffect::Stunned);
        }
        CombatSpell::FingerOfDeath => {
            // Instant kill
            target.hp = 0;
        }
        CombatSpell::Drain => {
            // Damage + drain effect
            target.hp -= damage;
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Drained,
                2,
                "spell drain",
            );
            result = result.with_effect(StatusEffect::Drained);
        }
        CombatSpell::Confuse => {
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Paralyzed,
                2,
                "confusion",
            );
            result = result.with_effect(StatusEffect::Paralyzed);
        }
        CombatSpell::Slow => {
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Paralyzed,
                1,
                "slow",
            );
            result = result.with_effect(StatusEffect::Paralyzed);
        }
        CombatSpell::Sleep => {
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Stunned,
                3,
                "sleep",
            );
            result = result.with_effect(StatusEffect::Stunned);
        }
        CombatSpell::Cancellation => {
            // Remove all effects from target
            target.status_effects.clear_all();
        }
        CombatSpell::Haste => {
            // Buff player (separate function)
            apply_status_effect(
                &mut caster.status_effects,
                StatusEffect::Stunned,
                0,
                "haste",
            ); // Dummy
        }
        CombatSpell::Invisibility => {
            // Buff player
        }
        CombatSpell::StoneSkin => {
            // Buff player
        }
        CombatSpell::Protection => {
            // Buff player
        }
        CombatSpell::Healing => {
            caster.hp = (caster.hp + damage).min(caster.hp_max);
        }
        CombatSpell::ExtraHealing => {
            caster.hp = (caster.hp + damage).min(caster.hp_max);
        }
    }

    result
}

// ============================================================================
// Phase 16: Loot & Treasure System
// ============================================================================

/// Item rarity tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ItemRarity {
    /// Common loot (basic weapons, armor)
    Common,
    /// Uncommon loot (good quality items)
    Uncommon,
    /// Rare loot (enchanted items, special properties)
    Rare,
    /// Very rare loot (powerful enchantments, artifacts)
    VeryRare,
    /// Legendary loot (unique artifacts, extremely powerful)
    Legendary,
}

impl ItemRarity {
    /// Get drop chance percentage for this rarity (0-100)
    pub const fn drop_chance(&self) -> i32 {
        match self {
            Self::Common => 100,  // Always dropped
            Self::Uncommon => 50, // 50% chance
            Self::Rare => 20,     // 20% chance
            Self::VeryRare => 5,  // 5% chance
            Self::Legendary => 1, // 1% chance
        }
    }

    /// Get quality multiplier for enchantment
    pub const fn enchantment_bonus(&self) -> i8 {
        match self {
            Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::VeryRare => 3,
            Self::Legendary => 5,
        }
    }

    /// Get name suffix for display
    pub const fn name_suffix(&self) -> &'static str {
        match self {
            Self::Common => "",
            Self::Uncommon => " +1",
            Self::Rare => " +2",
            Self::VeryRare => " +3",
            Self::Legendary => " +5",
        }
    }

    /// All rarity levels
    pub const fn all() -> &'static [ItemRarity] {
        &[
            Self::Common,
            Self::Uncommon,
            Self::Rare,
            Self::VeryRare,
            Self::Legendary,
        ]
    }
}

/// Item categories for loot generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LootCategory {
    /// Weapons and melee weapons
    Weapon,
    /// Armor pieces
    Armor,
    /// Rings and amulets
    Jewelry,
    /// Potions and elixirs
    Potion,
    /// Scrolls and spellbooks
    Scroll,
    /// Gems and precious stones
    Gem,
    /// Gold and treasure
    Gold,
    /// Magical wands and staves
    Wand,
}

impl LootCategory {
    /// Get drop weight for this category (higher = more likely)
    pub const fn drop_weight(&self) -> i32 {
        match self {
            Self::Weapon => 20,
            Self::Armor => 15,
            Self::Jewelry => 10,
            Self::Potion => 15,
            Self::Scroll => 10,
            Self::Gem => 8,
            Self::Gold => 25,
            Self::Wand => 5,
        }
    }

    /// All loot categories
    pub const fn all() -> &'static [LootCategory] {
        &[
            Self::Weapon,
            Self::Armor,
            Self::Jewelry,
            Self::Potion,
            Self::Scroll,
            Self::Gem,
            Self::Gold,
            Self::Wand,
        ]
    }
}

/// Monster loot drop table
#[derive(Debug, Clone)]
pub struct LootTable {
    /// Categories this monster drops
    pub categories: Vec<LootCategory>,
    /// Base drop chance (0-100%)
    pub drop_chance: i32,
    /// Gold drop multiplier based on level
    pub gold_multiplier: f32,
    /// Rarity bias (higher = better items)
    pub rarity_bias: i32,
}

impl LootTable {
    /// Create default loot table
    pub const fn default_table() -> Self {
        Self {
            categories: vec![], // Will be initialized in function
            drop_chance: 30,
            gold_multiplier: 1.0,
            rarity_bias: 0,
        }
    }

    /// Get loot table for a monster type
    pub fn for_monster_type(monster_level: u8, is_magical: bool, is_boss: bool) -> Self {
        let base_chance = 30 + (monster_level as i32 * 2);
        let drop_chance = base_chance.min(100);

        let gold_multiplier = (monster_level as f32 / 5.0).max(1.0);

        let rarity_bias = if is_boss {
            3
        } else if is_magical {
            2
        } else {
            0
        };

        Self {
            categories: vec![], // Set via function logic
            drop_chance,
            gold_multiplier,
            rarity_bias,
        }
    }

    /// Should this monster drop loot?
    pub fn should_drop_loot(&self, rng: &mut crate::rng::GameRng) -> bool {
        (rng.rnd(100) as i32) < self.drop_chance
    }
}

/// Generate a random loot item
pub struct LootGenerator;

impl LootGenerator {
    /// Generate a loot drop for a defeated monster
    pub fn generate_loot(
        monster_level: u8,
        is_magical: bool,
        rng: &mut crate::rng::GameRng,
    ) -> Option<LootDrop> {
        // Use level-based loot table
        let table = LootTable::for_monster_type(monster_level, is_magical, false);

        // Check if drops loot
        if !table.should_drop_loot(rng) {
            return None;
        }

        // Determine category
        let category = Self::select_loot_category(rng);

        // Determine rarity
        let rarity = Self::determine_rarity(monster_level, table.rarity_bias, rng);

        // Generate item based on category and rarity
        let item_type = Self::generate_item_type(category, monster_level, rarity, rng);

        let value = Self::calculate_item_value(category, rarity, monster_level);

        Some(LootDrop {
            category,
            rarity,
            item_type,
            value,
            gold_bonus: 0,
        })
    }

    /// Generate gold loot specifically
    pub fn generate_gold(monster_level: u8, rng: &mut crate::rng::GameRng) -> i32 {
        let base_gold = (monster_level as i32 + 1) * 10;
        let variance = rng.rnd(base_gold as u32) as i32;
        base_gold + variance
    }

    /// Select a random loot category weighted by drop weights
    fn select_loot_category(rng: &mut crate::rng::GameRng) -> LootCategory {
        let categories = LootCategory::all();
        let total_weight: i32 = categories.iter().map(|c| c.drop_weight()).sum();
        let roll = (rng.rnd(total_weight as u32)) as i32;

        let mut accumulated = 0;
        for &category in categories {
            accumulated += category.drop_weight();
            if roll < accumulated {
                return category;
            }
        }

        LootCategory::Gold // Fallback
    }

    /// Determine item rarity based on monster level
    fn determine_rarity(monster_level: u8, bias: i32, rng: &mut crate::rng::GameRng) -> ItemRarity {
        // Higher level monsters drop better items
        let rarity_roll = (rng.rnd(100) as i32) - (monster_level as i32 / 2) - bias * 5;

        if rarity_roll > 90 {
            ItemRarity::Legendary
        } else if rarity_roll > 70 {
            ItemRarity::VeryRare
        } else if rarity_roll > 40 {
            ItemRarity::Rare
        } else if rarity_roll > 15 {
            ItemRarity::Uncommon
        } else {
            ItemRarity::Common
        }
    }

    /// Generate item type string based on category
    fn generate_item_type(
        category: LootCategory,
        monster_level: u8,
        _rarity: ItemRarity,
        rng: &mut crate::rng::GameRng,
    ) -> String {
        match category {
            LootCategory::Weapon => {
                let weapons = [
                    "short sword",
                    "long sword",
                    "dagger",
                    "axe",
                    "mace",
                    "spear",
                    "bow",
                ];
                weapons[rng.rn2(weapons.len() as u32) as usize].to_string()
            }
            LootCategory::Armor => {
                let armors = [
                    "leather armor",
                    "chain mail",
                    "plate mail",
                    "shield",
                    "helmet",
                    "gloves",
                ];
                armors[rng.rn2(armors.len() as u32) as usize].to_string()
            }
            LootCategory::Jewelry => {
                let jewelry = ["ring of protection", "amulet of health", "bracers"];
                jewelry[rng.rn2(jewelry.len() as u32) as usize].to_string()
            }
            LootCategory::Potion => {
                let potions = ["potion of healing", "potion of strength", "potion of speed"];
                potions[rng.rn2(potions.len() as u32) as usize].to_string()
            }
            LootCategory::Scroll => {
                let scrolls = [
                    "scroll of identify",
                    "scroll of teleport",
                    "scroll of enchant",
                ];
                scrolls[rng.rn2(scrolls.len() as u32) as usize].to_string()
            }
            LootCategory::Gem => {
                let gems = ["ruby", "sapphire", "emerald", "diamond"];
                format!(
                    "{} ({} carats)",
                    gems[rng.rn2(gems.len() as u32) as usize],
                    monster_level * 10
                )
            }
            LootCategory::Gold => "gold coins".to_string(),
            LootCategory::Wand => {
                let wands = ["wand of fireball", "wand of lightning", "wand of healing"];
                wands[rng.rn2(wands.len() as u32) as usize].to_string()
            }
        }
    }

    /// Calculate item value in gold
    fn calculate_item_value(category: LootCategory, rarity: ItemRarity, monster_level: u8) -> i32 {
        let base_value = match category {
            LootCategory::Weapon => 50,
            LootCategory::Armor => 40,
            LootCategory::Jewelry => 100,
            LootCategory::Potion => 30,
            LootCategory::Scroll => 20,
            LootCategory::Gem => 200,
            LootCategory::Gold => 1,
            LootCategory::Wand => 150,
        };

        let rarity_multiplier = match rarity {
            ItemRarity::Common => 1.0,
            ItemRarity::Uncommon => 2.0,
            ItemRarity::Rare => 4.0,
            ItemRarity::VeryRare => 8.0,
            ItemRarity::Legendary => 16.0,
        };

        let level_multiplier = (monster_level as f32 / 5.0).max(1.0);

        (base_value as f32 * rarity_multiplier * level_multiplier) as i32
    }
}

/// A loot drop from a defeated monster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootDrop {
    /// Category of item
    pub category: LootCategory,
    /// Rarity tier
    pub rarity: ItemRarity,
    /// Item type description
    pub item_type: String,
    /// Item value in gold
    pub value: i32,
    /// Bonus gold from defeating monster
    pub gold_bonus: i32,
}

impl LootDrop {
    /// Get full description of loot
    pub fn description(&self) -> String {
        format!(
            "{} {} (value: {} gp)",
            self.rarity.name_suffix(),
            self.item_type,
            self.value
        )
    }

    /// Get total value including gold bonus
    pub fn total_value(&self) -> i32 {
        self.value + self.gold_bonus
    }
}

/// Generate monster hoard (treasure from treasure chest or dragon hoard)
pub struct TreasureHoard;

impl TreasureHoard {
    /// Generate a treasure hoard for a monster type
    pub fn generate_hoard(
        monster_level: u8,
        hoard_size: i32,
        rng: &mut crate::rng::GameRng,
    ) -> Vec<LootDrop> {
        let mut hoard = Vec::new();

        // Generate multiple items based on hoard size
        for _ in 0..hoard_size {
            if let Some(loot) = LootGenerator::generate_loot(monster_level, true, rng) {
                hoard.push(loot);
            }
        }

        // Add significant gold
        let gold_total = LootGenerator::generate_gold(monster_level, rng) * hoard_size;
        if gold_total > 0 {
            hoard.push(LootDrop {
                category: LootCategory::Gold,
                rarity: ItemRarity::Common,
                item_type: format!("{} gold coins", gold_total),
                value: gold_total,
                gold_bonus: 0,
            });
        }

        hoard
    }

    /// Generate hoard for a boss/unique monster
    pub fn generate_boss_hoard(monster_level: u8, rng: &mut crate::rng::GameRng) -> Vec<LootDrop> {
        let hoard_size = (monster_level as i32 / 5 + 2).min(5);
        Self::generate_hoard(monster_level, hoard_size, rng)
    }
}

/// Award loot to player and calculate total value
pub fn award_loot_to_player(player: &mut crate::player::You, loot: &[LootDrop]) -> i32 {
    let mut total_value = 0;

    for drop in loot {
        // Add gold to player
        player.gold += drop.total_value();
        total_value += drop.total_value();

        // Item-to-inventory transfer deferred: requires full inventory add_object integration
    }

    total_value
}

// ============================================================================
// Phase 17: Combat Encounters System
// ============================================================================

/// Monster formation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Formation {
    /// Single monster (no formation)
    Solo,
    /// Two monsters side by side
    Pair,
    /// Three monsters in line
    Line,
    /// Circle formation around player
    Circle,
    /// Wedge formation (strong point forward)
    Wedge,
    /// Loose group (spread out)
    Loose,
    /// Defensive formation (clustered)
    Defensive,
}

impl Formation {
    /// Get flanking bonus multiplier from formation
    pub const fn flanking_bonus(&self) -> f32 {
        match self {
            Self::Solo => 1.0,      // No bonus
            Self::Pair => 1.1,      // 10% bonus
            Self::Line => 1.15,     // 15% bonus
            Self::Circle => 1.2,    // 20% bonus (surrounded)
            Self::Wedge => 1.25,    // 25% bonus (coordinated)
            Self::Loose => 1.0,     // No bonus
            Self::Defensive => 0.9, // 10% penalty (defensive, not offensive)
        }
    }

    /// Get coordination level (how well monsters work together)
    pub const fn coordination(&self) -> u8 {
        match self {
            Self::Solo => 0,
            Self::Pair => 2,
            Self::Line => 3,
            Self::Circle => 4,
            Self::Wedge => 5,
            Self::Loose => 1,
            Self::Defensive => 2,
        }
    }

    /// Get dodge penalty from formation (easier to hit many monsters)
    pub const fn dodge_penalty(&self) -> i32 {
        match self {
            Self::Solo => 0,
            Self::Pair => 1,
            Self::Line => 2,
            Self::Circle => 2,
            Self::Wedge => 1,
            Self::Loose => 0,
            Self::Defensive => -1, // Harder to hit defensive
        }
    }

    /// All formation types
    pub const fn all() -> &'static [Formation] {
        &[
            Self::Solo,
            Self::Pair,
            Self::Line,
            Self::Circle,
            Self::Wedge,
            Self::Loose,
            Self::Defensive,
        ]
    }
}

/// A combat encounter with multiple monsters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEncounter {
    /// All monsters in encounter
    pub monsters: Vec<crate::monster::MonsterId>,
    /// Current formation
    pub formation: Formation,
    /// Encounter difficulty rating
    pub difficulty: i32,
    /// Total XP if player wins
    pub total_xp: u32,
    /// Whether encounter is complete
    pub complete: bool,
}

impl CombatEncounter {
    /// Create new encounter with monsters
    pub fn new(monster_ids: Vec<crate::monster::MonsterId>) -> Self {
        let difficulty = 10 + (monster_ids.len() as i32 * 5); // Base difficulty

        Self {
            monsters: monster_ids,
            formation: Formation::Loose,
            difficulty,
            total_xp: 0,
            complete: false,
        }
    }

    /// Get number of monsters in encounter
    pub fn monster_count(&self) -> usize {
        self.monsters.len()
    }

    /// Get number of remaining alive monsters
    pub fn alive_count(&self, monsters: &[crate::monster::Monster]) -> usize {
        self.monsters
            .iter()
            .filter(|id| {
                monsters
                    .iter()
                    .find(|m| m.id == **id)
                    .map(|m| m.hp > 0)
                    .unwrap_or(false)
            })
            .count()
    }

    /// Check if encounter is won (all monsters dead)
    pub fn is_won(&self, monsters: &[crate::monster::Monster]) -> bool {
        self.alive_count(monsters) == 0
    }

    /// Check if encounter is lost (player dead)
    pub fn is_lost(&self, player: &crate::player::You) -> bool {
        player.hp <= 0
    }

    /// Update formation based on monster count
    pub fn update_formation_for_count(&mut self) {
        self.formation = match self.monster_count() {
            1 => Formation::Solo,
            2 => Formation::Pair,
            3 => Formation::Line,
            4 => Formation::Circle,
            5..=6 => Formation::Wedge,
            _ => Formation::Loose,
        };
    }
}

/// Encounter difficulty rating system
pub struct DifficultyRating;

impl DifficultyRating {
    /// Calculate base difficulty for a single monster
    pub const fn monster_difficulty(monster_level: u8) -> i32 {
        let d = monster_level as i32 * 2;
        if d > 1 { d } else { 1 }
    }

    /// Calculate encounter difficulty multiplier for party size
    pub fn party_difficulty_multiplier(monster_count: usize) -> f32 {
        match monster_count {
            1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            4 => 2.8,
            5 => 3.5,
            6 => 4.5,
            n => ((n as f32 * 1.5).min(10.0)), // Cap multiplier
        }
    }

    /// Calculate formation difficulty bonus
    pub fn formation_difficulty(formation: Formation) -> f32 {
        match formation {
            Formation::Solo => 1.0,
            Formation::Pair => 1.2,
            Formation::Line => 1.3,
            Formation::Circle => 1.5,
            Formation::Wedge => 1.4,
            Formation::Loose => 1.1,
            Formation::Defensive => 0.9,
        }
    }

    /// Calculate total encounter difficulty
    pub fn calculate_total_difficulty(
        monsters: &[crate::monster::Monster],
        formation: Formation,
    ) -> i32 {
        let base_difficulty: i32 = monsters
            .iter()
            .map(|m| Self::monster_difficulty(m.level))
            .sum();

        let party_multiplier = Self::party_difficulty_multiplier(monsters.len());
        let formation_bonus = Self::formation_difficulty(formation);

        (base_difficulty as f32 * party_multiplier * formation_bonus) as i32
    }

    /// Get difficulty label
    pub fn difficulty_label(difficulty: i32) -> &'static str {
        match difficulty {
            0..=5 => "Trivial",
            6..=15 => "Easy",
            16..=25 => "Moderate",
            26..=40 => "Challenging",
            41..=60 => "Hard",
            61..=80 => "Very Hard",
            81..=100 => "Deadly",
            _ => "Impossible",
        }
    }
}

/// Monster group tactics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroupTactic {
    /// Overwhelming force - attack together
    Swarm,
    /// Focus fire on one target
    FocusFire,
    /// Support weak members
    Defend,
    /// Hit and run - strike then retreat
    Harass,
    /// Surround target for flanking
    Flank,
    /// Scatter to avoid AOE attacks
    Scatter,
}

impl GroupTactic {
    /// Determine appropriate tactic based on formation and situation
    pub fn best_tactic_for_formation(formation: Formation) -> Self {
        match formation {
            Formation::Solo => Self::Swarm,
            Formation::Pair => Self::FocusFire,
            Formation::Line => Self::Flank,
            Formation::Circle => Self::Swarm,
            Formation::Wedge => Self::FocusFire,
            Formation::Loose => Self::Harass,
            Formation::Defensive => Self::Defend,
        }
    }

    /// Get to-hit bonus from tactic
    pub const fn to_hit_bonus(&self) -> i32 {
        match self {
            Self::Swarm => 2,
            Self::FocusFire => 3,
            Self::Defend => 0,
            Self::Harass => 1,
            Self::Flank => 2,
            Self::Scatter => -2,
        }
    }

    /// Get damage modifier from tactic
    pub const fn damage_modifier(&self) -> f32 {
        match self {
            Self::Swarm => 1.1,
            Self::FocusFire => 1.3,
            Self::Defend => 0.8,
            Self::Harass => 1.0,
            Self::Flank => 1.15,
            Self::Scatter => 0.7,
        }
    }
}

/// Encounter state tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterState {
    /// Current encounter
    pub encounter: CombatEncounter,
    /// Current group tactic
    pub tactic: GroupTactic,
    /// Rounds of combat elapsed
    pub rounds: u32,
    /// Total damage dealt to player
    pub total_player_damage: i32,
    /// Total damage dealt by player to all monsters
    pub total_monster_damage: i32,
}

impl EncounterState {
    /// Create new encounter state
    pub fn new(monster_ids: Vec<crate::monster::MonsterId>) -> Self {
        let mut encounter = CombatEncounter::new(monster_ids);
        encounter.update_formation_for_count();

        let tactic = GroupTactic::best_tactic_for_formation(encounter.formation);

        Self {
            encounter,
            tactic,
            rounds: 0,
            total_player_damage: 0,
            total_monster_damage: 0,
        }
    }

    /// Update encounter state after a round
    pub fn end_round(&mut self) {
        self.rounds += 1;
    }

    /// Get round-based fatigue penalty for monsters (fights last longer)
    pub fn monster_fatigue_penalty(&self) -> i32 {
        (self.rounds / 5) as i32 // -1 to-hit per 5 rounds
    }

    /// Get morale bonus for player based on monsters defeated
    pub fn player_morale_bonus(&self, total_monsters: usize) -> f32 {
        let defeated = (total_monsters - self.encounter.monster_count()) as f32;
        1.0 + (defeated * 0.1) // +10% per monster defeated
    }
}

/// Calculate if monsters can flank the player
pub fn check_flanking(
    monsters: &[crate::monster::Monster],
    formation: Formation,
    player_pos: &crate::player::Position,
) -> bool {
    if formation != Formation::Circle && formation != Formation::Wedge {
        return false;
    }

    // Monsters can flank if surrounding player
    monsters.len() >= 3 && formation == Formation::Circle
}

/// Calculate flanking damage bonus
pub fn flanking_damage_bonus(monsters: &[crate::monster::Monster], formation: Formation) -> f32 {
    let flank_bonus = formation.flanking_bonus();
    let coordination_bonus = 1.0 + (formation.coordination() as f32 * 0.05);

    flank_bonus * coordination_bonus
}

/// Determine best monster target based on encounter state
pub fn select_monster_target(
    current_target: Option<crate::monster::MonsterId>,
    tactic: GroupTactic,
    monsters: &[crate::monster::Monster],
    player_hp: i32,
    player_hp_max: i32,
) -> crate::monster::MonsterId {
    match tactic {
        GroupTactic::FocusFire => {
            // Focus on current target or the first alive one
            if let Some(target) = current_target {
                if monsters.iter().any(|m| m.id == target && m.hp > 0) {
                    return target;
                }
            }
            monsters
                .iter()
                .find(|m| m.hp > 0)
                .map(|m| m.id)
                .unwrap_or(crate::monster::MonsterId::NONE)
        }
        GroupTactic::Defend => {
            // Protect the weakest monster
            monsters
                .iter()
                .filter(|m| m.hp > 0)
                .min_by_key(|m| m.hp)
                .map(|m| m.id)
                .unwrap_or(crate::monster::MonsterId::NONE)
        }
        _ => {
            // Default: attack any alive monster
            monsters
                .iter()
                .find(|m| m.hp > 0)
                .map(|m| m.id)
                .unwrap_or(crate::monster::MonsterId::NONE)
        }
    }
}

/// Apply encounter modifiers to monster combat
pub fn apply_encounter_modifiers(
    monster: &mut crate::monster::Monster,
    encounter_state: &EncounterState,
    formation: Formation,
) {
    // Apply fatigue penalty
    let fatigue = encounter_state.monster_fatigue_penalty();
    if fatigue > 0 {
        monster.ac = monster.ac.saturating_add(fatigue as i8); // Worse AC when fatigued
    }

    // Apply formation bonus
    let formation_bonus = formation.coordination() as i32;
    if formation_bonus > 0 {
        // Improved hit chance from formation
        // This gets applied during actual combat rolls
    }
}

/// Calculate total XP for defeating encounter
pub fn calculate_encounter_xp(
    encounter: &CombatEncounter,
    difficulty: i32,
    player_hp_at_end: i32,
    player_hp_max: i32,
) -> u32 {
    // Base XP from monsters
    let mut xp = encounter.total_xp;

    // Difficulty bonus: harder encounters give more XP
    let difficulty_multiplier = 1.0 + (difficulty as f32 / 100.0).min(1.0);
    xp = (xp as f32 * difficulty_multiplier) as u32;

    // Survival bonus: more HP remaining = more XP
    let survival_percent = (player_hp_at_end as f32 / player_hp_max as f32)
        .max(0.0)
        .min(1.0);
    let survival_bonus = (survival_percent * 0.5 + 0.5).max(0.1); // 10% to 60%
    xp = (xp as f32 * survival_bonus) as u32;

    xp.max(10) // Minimum 10 XP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attk_protection() {
        use crate::action::wear::worn_mask::*;

        // Magic attacks don't need armor (special case)
        assert_eq!(attk_protection(AttackType::Magic), !0);
        assert_eq!(attk_protection(AttackType::Breath), !0);

        // Physical attacks
        assert_eq!(attk_protection(AttackType::Kick), W_ARMF);
        assert_eq!(attk_protection(AttackType::Butt), W_ARMH);
        assert_eq!(attk_protection(AttackType::Hug), W_ARMC | W_ARMG);
        assert_eq!(attk_protection(AttackType::Claw), W_ARMG);

        // Unarmored attacks
        assert_eq!(attk_protection(AttackType::Bite), 0);
        assert_eq!(attk_protection(AttackType::Sting), 0);
    }

    #[test]
    fn test_attack_average_damage() {
        let attack = Attack::new(AttackType::Claw, DamageType::Physical, 2, 6);
        // Average of 2d6 is 7
        assert!(attack.average_damage() > 6.5 && attack.average_damage() < 7.5);
    }

    // ========================================================================
    // Tests for find_mac() - Monster AC Calculation
    // ========================================================================

    #[test]
    fn test_find_mac_no_armor() {
        use crate::monster::Monster;
        let monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        // AC = monster's base ac (no worn items reduce it)
        let ac = find_mac(&monster);
        assert_eq!(ac, monster.ac);
    }

    #[test]
    fn test_find_mac_single_armor_piece() {
        use crate::monster::Monster;
        use crate::object::{Object, ObjectClass};

        let mut monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        monster.ac = 10; // Base AC

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 5;
        armor.enchantment = 0;
        armor.worn_mask = 1; // Being worn

        monster.inventory.push(armor);

        let ac = find_mac(&monster);
        // AC = 10 - (5 + 0 - 0) = 10 - 5 = 5
        assert_eq!(ac, 5);
    }

    #[test]
    fn test_find_mac_multiple_armor_pieces() {
        use crate::monster::Monster;
        use crate::object::{Object, ObjectClass};

        let mut monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        monster.ac = 12;

        let mut armor1 = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor1.base_ac = 3;
        armor1.enchantment = 1;
        armor1.worn_mask = 1;

        let mut armor2 = Object::new(crate::object::ObjectId(2), 2, ObjectClass::Armor);
        armor2.base_ac = 2;
        armor2.enchantment = 0;
        armor2.worn_mask = 1;

        monster.inventory.push(armor1);
        monster.inventory.push(armor2);

        let ac = find_mac(&monster);
        // armor1 bonus = (3 + 1 - 0).max(0) = 4
        // armor2 bonus = (2 + 0 - 0).max(0) = 2
        // AC = 12 - 4 - 2 = 6
        assert_eq!(ac, 6);
    }

    #[test]
    fn test_find_mac_with_erosion() {
        use crate::monster::Monster;
        use crate::object::{Object, ObjectClass};

        let mut monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        monster.ac = 10;

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 4;
        armor.enchantment = 2;
        armor.erosion1 = 2; // Some erosion damage
        armor.worn_mask = 1;

        monster.inventory.push(armor);

        let ac = find_mac(&monster);
        // bonus = (4 + 2 - 2).max(0) = 4
        // AC = 10 - 4 = 6
        assert_eq!(ac, 6);
    }

    #[test]
    fn test_find_mac_armor_bonus_clamped_to_zero() {
        use crate::monster::Monster;
        use crate::object::{Object, ObjectClass};

        let mut monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        monster.ac = 10;

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 2;
        armor.enchantment = 0;
        armor.erosion1 = 2;
        armor.erosion2 = 3; // Total erosion = 5
        armor.worn_mask = 1;

        monster.inventory.push(armor);

        let ac = find_mac(&monster);
        // bonus = (2 + 0 - 5).max(0) = (-3).max(0) = 0
        // AC = 10 - 0 = 10
        assert_eq!(ac, 10);
    }

    #[test]
    fn test_find_mac_ac_clamping_lower() {
        use crate::monster::Monster;
        use crate::object::{Object, ObjectClass};

        let mut monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        monster.ac = -100; // Already very good

        let ac = find_mac(&monster);
        // AC is clamped to -128
        assert_eq!(ac, -100);
    }

    #[test]
    fn test_find_mac_ac_clamping_upper() {
        use crate::monster::Monster;
        use crate::object::{Object, ObjectClass};

        let mut monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        monster.ac = 127; // Maximum for i8

        let ac = find_mac(&monster);
        // AC is already at max
        assert_eq!(ac, 127);
    }

    #[test]
    fn test_find_mac_unworn_items_ignored() {
        use crate::monster::Monster;
        use crate::object::{Object, ObjectClass};

        let mut monster = Monster::new(crate::monster::MonsterId(1), 1, 5, 5);
        monster.ac = 10;

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 5;
        armor.enchantment = 0;
        armor.worn_mask = 0; // NOT being worn

        monster.inventory.push(armor);

        let ac = find_mac(&monster);
        // Unworn items don't affect AC
        assert_eq!(ac, 10);
    }

    // ========================================================================
    // Tests for armor_bonus() helper function
    // ========================================================================

    #[test]
    fn test_armor_bonus_basic() {
        use crate::object::{Object, ObjectClass};

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 3;
        armor.enchantment = 2;

        let bonus = armor_bonus(&armor);
        // bonus = (3 + 2 - 0).max(0) = 5
        assert_eq!(bonus, 5);
    }

    #[test]
    fn test_armor_bonus_with_erosion() {
        use crate::object::{Object, ObjectClass};

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 6;
        armor.enchantment = 1;
        armor.erosion1 = 1;

        let bonus = armor_bonus(&armor);
        // bonus = (6 + 1 - 1).max(0) = 6
        assert_eq!(bonus, 6);
    }

    #[test]
    fn test_armor_bonus_erosion_exceeds_base() {
        use crate::object::{Object, ObjectClass};

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 2;
        armor.enchantment = 1;
        armor.erosion1 = 2;

        let bonus = armor_bonus(&armor);
        // bonus = (2 + 1 - 2).max(0) = 1.max(0) = 1
        assert_eq!(bonus, 1);
    }

    #[test]
    fn test_armor_bonus_clamped_to_zero() {
        use crate::object::{Object, ObjectClass};

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 1;
        armor.enchantment = 0;
        armor.erosion1 = 1;
        armor.erosion2 = 2; // Total erosion = 3

        let bonus = armor_bonus(&armor);
        // bonus = (1 + 0 - 3).max(0) = (-2).max(0) = 0
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_armor_bonus_negative_base_ac() {
        use crate::object::{Object, ObjectClass};

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = -2; // Stored as negative
        armor.enchantment = 1;

        let bonus = armor_bonus(&armor);
        // bonus = (-2 + 1 - 0).max(0) = (-1).max(0) = 0
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_grease_protect_no_grease() {
        use crate::object::{Object, ObjectClass};
        use crate::rng::GameRng;

        let mut obj = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        obj.greased = false;
        let mut rng = GameRng::from_entropy();

        let result = grease_protect(&mut obj, &mut rng);
        assert!(!result);
        assert!(!obj.greased);
    }

    #[test]
    fn test_grease_protect_with_grease() {
        use crate::object::{Object, ObjectClass};
        use crate::rng::GameRng;

        let mut obj = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        obj.greased = true;
        let mut rng = GameRng::from_entropy();

        let result = grease_protect(&mut obj, &mut rng);
        assert!(result);
        // Grease may or may not be consumed (50/50 chance)
        // but we know it was greased before
    }
}
