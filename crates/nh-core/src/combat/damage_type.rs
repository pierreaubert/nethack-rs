//! Damage types from monattk.h
//!
//! These define WHAT kind of damage is dealt.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Damage type - what kind of damage is dealt (AD_* from monattk.h)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum DamageType {
    /// Physical damage (AD_PHYS)
    #[default]
    Physical = 0,

    /// Magic missile (AD_MAGM)
    MagicMissile = 1,

    /// Fire damage (AD_FIRE)
    Fire = 2,

    /// Cold damage (AD_COLD)
    Cold = 3,

    /// Sleep (AD_SLEE)
    Sleep = 4,

    /// Disintegration (AD_DISN)
    Disintegrate = 5,

    /// Electric shock (AD_ELEC)
    Electric = 6,

    /// Drain strength (AD_DRST)
    DrainStrength = 7,

    /// Acid damage (AD_ACID)
    Acid = 8,

    // 9-10 unused (special attack letter)
    /// Blindness (AD_BLND)
    Blind = 11,

    /// Stun (AD_STUN)
    Stun = 12,

    /// Slow (AD_SLOW)
    Slow = 13,

    /// Paralysis (AD_PLYS)
    Paralyze = 14,

    /// Drain life/experience levels (AD_DRLI)
    DrainLife = 15,

    /// Drain magic energy (AD_DREN)
    DrainEnergy = 16,

    /// Leg wound (AD_LEGS)
    Legs = 17,

    /// Petrification (AD_STON)
    Stone = 18,

    /// Sticking (AD_STCK)
    Stick = 19,

    /// Steal gold (AD_SGLD)
    StealGold = 20,

    /// Steal item (AD_SITM)
    StealItem = 21,

    /// Seduce and steal (AD_SEDU)
    Seduce = 22,

    /// Teleport (AD_TLPT)
    Teleport = 23,

    /// Rust (AD_RUST)
    Rust = 24,

    /// Confusion (AD_CONF)
    Confuse = 25,

    /// Digestion (AD_DGST)
    Digest = 26,

    /// Healing (AD_HEAL)
    Heal = 27,

    /// Wrap/constrict (AD_WRAP)
    Wrap = 28,

    /// Lycanthropy (AD_WERE)
    Lycanthropy = 29,

    /// Drain dexterity (AD_DRDX)
    DrainDexterity = 30,

    /// Drain constitution (AD_DRCO)
    DrainConstitution = 31,

    /// Drain intelligence (AD_DRIN)
    DrainIntelligence = 32,

    /// Disease (AD_DISE)
    Disease = 33,

    /// Decay (AD_DCAY)
    Decay = 34,

    /// Seduction (special) (AD_SSEX)
    SeduceSpecial = 35,

    /// Hallucination (AD_HALU)
    Hallucinate = 36,

    /// Death touch (AD_DETH)
    Death = 37,

    /// Pestilence (AD_PEST)
    Pestilence = 38,

    /// Famine (AD_FAMN)
    Famine = 39,

    /// Slime (AD_SLIM)
    Slime = 40,

    /// Disenchant (AD_ENCH)
    Disenchant = 41,

    /// Corrosion (AD_CORR)
    Corrode = 42,

    /// Life drain (for resistance mapping)
    Drain = 43,

    /// Poison damage (for resistance mapping)
    Poison = 44,

    /// Electric/Shock (alias for Electric for resistance mapping)
    Shock = 45,

    /// Cutting damage (physical subtype)
    Cut = 46,

    /// Stabbing damage (physical subtype)
    Stab = 47,

    /// Slashing damage (physical subtype)
    Slash = 48,

    /// Poison gas (breath weapon)
    PoisonGas = 49,

    // Spell types (240+)
    /// Clerical spells (AD_CLRC)
    ClericSpell = 240,

    /// Mage spells (AD_SPEL)
    MageSpell = 241,

    /// Random breath weapon (AD_RBRE)
    RandomBreath = 242,

    // 243-251 unused
    /// Steal amulet of yendor (AD_SAMU)
    StealAmulet = 252,

    /// Curse items (AD_CURS)
    Curse = 253,
}

impl DamageType {
    /// Check if this damage type can be resisted by fire resistance
    pub const fn is_fire(&self) -> bool {
        matches!(self, DamageType::Fire)
    }

    /// Check if this damage type can be resisted by cold resistance
    pub const fn is_cold(&self) -> bool {
        matches!(self, DamageType::Cold)
    }

    /// Check if this damage type can be resisted by shock resistance
    pub const fn is_electric(&self) -> bool {
        matches!(self, DamageType::Electric)
    }

    /// Check if this damage type can be resisted by poison resistance
    pub const fn is_poison(&self) -> bool {
        matches!(
            self,
            DamageType::DrainStrength
                | DamageType::DrainDexterity
                | DamageType::DrainConstitution
                | DamageType::Disease
        )
    }

    /// Check if this damage type can be resisted by sleep resistance
    pub const fn is_sleep(&self) -> bool {
        matches!(self, DamageType::Sleep)
    }

    /// Check if this damage type can be resisted by disintegration resistance
    pub const fn is_disintegrate(&self) -> bool {
        matches!(self, DamageType::Disintegrate)
    }

    /// Check if this damage type can be resisted by magic resistance
    pub const fn is_magic(&self) -> bool {
        matches!(
            self,
            DamageType::MagicMissile
                | DamageType::Death
                | DamageType::Teleport
                | DamageType::ClericSpell
                | DamageType::MageSpell
        )
    }

    /// Check if this is a draining attack (level/stat drain)
    pub const fn is_drain(&self) -> bool {
        matches!(
            self,
            DamageType::DrainLife
                | DamageType::DrainEnergy
                | DamageType::DrainStrength
                | DamageType::DrainDexterity
                | DamageType::DrainConstitution
                | DamageType::DrainIntelligence
        )
    }

    /// Check if this damage type involves stealing
    pub const fn is_theft(&self) -> bool {
        matches!(
            self,
            DamageType::StealGold
                | DamageType::StealItem
                | DamageType::Seduce
                | DamageType::SeduceSpecial
                | DamageType::StealAmulet
        )
    }

    /// Check if this damage type involves petrification
    pub const fn is_petrification(&self) -> bool {
        matches!(self, DamageType::Stone)
    }

    /// Check if this is a spell-based damage type
    pub const fn is_spell(&self) -> bool {
        matches!(
            self,
            DamageType::ClericSpell | DamageType::MageSpell | DamageType::RandomBreath
        )
    }

    /// Get the resistance that protects against this damage type
    pub const fn resistance(&self) -> Option<Resistance> {
        match self {
            DamageType::Fire => Some(Resistance::Fire),
            DamageType::Cold => Some(Resistance::Cold),
            DamageType::Electric => Some(Resistance::Electric),
            DamageType::Sleep => Some(Resistance::Sleep),
            DamageType::Disintegrate => Some(Resistance::Disintegrate),
            DamageType::Acid => Some(Resistance::Acid),
            DamageType::Stone => Some(Resistance::Stone),
            DamageType::DrainStrength
            | DamageType::DrainDexterity
            | DamageType::DrainConstitution
            | DamageType::Disease => Some(Resistance::Poison),
            _ => None,
        }
    }
}

/// Resistance types that can protect against damage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resistance {
    Fire,
    Cold,
    Electric,
    Sleep,
    Disintegrate,
    Poison,
    Acid,
    Stone,
}

// ============================================================================
// Damage calculation functions
// ============================================================================

use crate::combat::Attack;
use crate::rng::GameRng;

/// Get the damage type from an attack (dmgtype_fromattack equivalent)
///
/// Returns the DamageType associated with an attack.
pub const fn dmgtype_fromattack(attack: &Attack) -> DamageType {
    attack.damage_type
}

/// Check if an attack has a specific damage type
pub fn attack_has_dmgtype(attack: &Attack, dmg_type: DamageType) -> bool {
    attack.damage_type == dmg_type
}

/// Calculate base damage value for an attack (dmgval equivalent)
///
/// This calculates the base damage dice roll for an attack.
///
/// # Arguments
/// * `rng` - Random number generator
/// * `attack` - The attack being used
///
/// # Returns
/// The damage rolled from the attack's dice
pub fn dmgval(rng: &mut GameRng, attack: &Attack) -> i32 {
    if attack.dice_num == 0 || attack.dice_sides == 0 {
        return 0;
    }
    rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32
}

/// Calculate weapon damage bonus based on weapon type vs target
///
/// In NetHack, some weapons do extra damage against certain monster types.
/// This is a simplified version.
pub fn weapon_dmgval_bonus(weapon_type: i16, is_large_target: bool) -> i32 {
    // Simplified - in full NetHack this looks up the weapon's damage dice
    // vs small vs large creatures. Here we just provide a bonus/penalty.
    if is_large_target {
        // Large monsters - some weapons do more or less damage
        match weapon_type {
            // Long sword does more vs large
            _ => 0,
        }
    } else {
        0
    }
}

// ============================================================================
// Elemental damage and resistance
// ============================================================================

/// Result of applying elemental damage
#[derive(Debug, Clone)]
pub struct ElementalDamageResult {
    /// Final damage after resistances
    pub damage: i32,
    /// Whether fully resisted
    pub resisted: bool,
    /// Message to display
    pub message: Option<String>,
}

/// Apply fire damage with resistance check
pub fn fire_damage(
    base_damage: i32,
    has_resistance: bool,
    is_player: bool,
) -> ElementalDamageResult {
    if has_resistance {
        ElementalDamageResult {
            damage: 0,
            resisted: true,
            message: Some(if is_player {
                "You feel mildly warm.".to_string()
            } else {
                "seems unaffected by the fire.".to_string()
            }),
        }
    } else {
        ElementalDamageResult {
            damage: base_damage,
            resisted: false,
            message: None,
        }
    }
}

/// Apply cold damage with resistance check
pub fn cold_damage(
    base_damage: i32,
    has_resistance: bool,
    is_player: bool,
) -> ElementalDamageResult {
    if has_resistance {
        ElementalDamageResult {
            damage: 0,
            resisted: true,
            message: Some(if is_player {
                "You feel a mild chill.".to_string()
            } else {
                "seems unaffected by the cold.".to_string()
            }),
        }
    } else {
        ElementalDamageResult {
            damage: base_damage,
            resisted: false,
            message: None,
        }
    }
}

/// Apply shock/electric damage with resistance check
pub fn shock_damage(
    base_damage: i32,
    has_resistance: bool,
    is_player: bool,
) -> ElementalDamageResult {
    if has_resistance {
        ElementalDamageResult {
            damage: 0,
            resisted: true,
            message: Some(if is_player {
                "You feel a mild tingle.".to_string()
            } else {
                "seems unaffected by the electricity.".to_string()
            }),
        }
    } else {
        ElementalDamageResult {
            damage: base_damage,
            resisted: false,
            message: None,
        }
    }
}

/// Apply acid damage with resistance check (acid_damage equivalent)
///
/// Acid damage can also corrode equipment.
///
/// # Arguments
/// * `base_damage` - The base acid damage
/// * `has_resistance` - Whether target has acid resistance
/// * `is_player` - Whether the target is the player
///
/// # Returns
/// The damage result
pub fn acid_damage(
    base_damage: i32,
    has_resistance: bool,
    is_player: bool,
) -> ElementalDamageResult {
    if has_resistance {
        ElementalDamageResult {
            damage: 0,
            resisted: true,
            message: Some(if is_player {
                "The acid doesn't affect you.".to_string()
            } else {
                "seems unaffected by the acid.".to_string()
            }),
        }
    } else {
        ElementalDamageResult {
            damage: base_damage,
            resisted: false,
            message: if is_player {
                Some("The acid burns!".to_string())
            } else {
                None
            },
        }
    }
}

/// Apply poison damage with resistance check
pub fn poison_damage(
    base_damage: i32,
    has_resistance: bool,
    is_player: bool,
) -> ElementalDamageResult {
    if has_resistance {
        ElementalDamageResult {
            damage: 0,
            resisted: true,
            message: Some(if is_player {
                "You are unaffected by the poison.".to_string()
            } else {
                "seems unaffected by the poison.".to_string()
            }),
        }
    } else {
        ElementalDamageResult {
            damage: base_damage,
            resisted: false,
            message: if is_player {
                Some("You feel very sick!".to_string())
            } else {
                None
            },
        }
    }
}

// ============================================================================
// Erosion functions
// ============================================================================

/// Erosion type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErosionType {
    /// Rust (from water, rust monster) - affects iron/steel
    Rust,
    /// Burn (from fire) - affects organic materials
    Burn,
    /// Corrode (from acid) - affects most materials
    Corrode,
    /// Rot (from decay) - affects organic materials
    Rot,
}

impl ErosionType {
    /// Get the erosion slot (0 for rust/burn, 1 for corrode/rot)
    pub const fn slot(&self) -> u8 {
        match self {
            ErosionType::Rust | ErosionType::Burn => 0,
            ErosionType::Corrode | ErosionType::Rot => 1,
        }
    }

    /// Get erosion description for level 1
    pub const fn level1_name(&self) -> &'static str {
        match self {
            ErosionType::Rust => "rusty",
            ErosionType::Burn => "burnt",
            ErosionType::Corrode => "corroded",
            ErosionType::Rot => "rotted",
        }
    }

    /// Get erosion description for level 2
    pub const fn level2_name(&self) -> &'static str {
        match self {
            ErosionType::Rust => "very rusty",
            ErosionType::Burn => "very burnt",
            ErosionType::Corrode => "very corroded",
            ErosionType::Rot => "very rotted",
        }
    }

    /// Get erosion description for level 3
    pub const fn level3_name(&self) -> &'static str {
        match self {
            ErosionType::Rust => "thoroughly rusty",
            ErosionType::Burn => "thoroughly burnt",
            ErosionType::Corrode => "thoroughly corroded",
            ErosionType::Rot => "thoroughly rotted",
        }
    }
}

/// Result of trying to erode an object
#[derive(Debug, Clone)]
pub struct ErodeResult {
    /// Whether erosion occurred
    pub eroded: bool,
    /// Whether object was destroyed
    pub destroyed: bool,
    /// Whether object was protected (greased, erosion-proof)
    pub protected: bool,
    /// Message to display
    pub message: String,
}

/// Get erosion text description for an object (erode_obj_text equivalent)
///
/// Returns descriptive text like "rusty", "very corroded", etc.
pub fn erode_obj_text(erosion1: u8, erosion2: u8, erosion_type: ErosionType) -> &'static str {
    let level = match erosion_type.slot() {
        0 => erosion1,
        _ => erosion2,
    };

    match level {
        0 => "",
        1 => erosion_type.level1_name(),
        2 => erosion_type.level2_name(),
        _ => erosion_type.level3_name(),
    }
}

/// Check if a material can be affected by an erosion type
pub fn can_erode_material(material: u8, erosion_type: ErosionType) -> bool {
    // Material constants (simplified - would come from objclass)
    const METAL: u8 = 0;
    const IRON: u8 = 1;
    const COPPER: u8 = 2;
    const SILVER: u8 = 3;
    const GOLD: u8 = 4;
    const WOOD: u8 = 5;
    const LEATHER: u8 = 6;
    const CLOTH: u8 = 7;
    const BONE: u8 = 8;
    const GLASS: u8 = 9;
    const GEMSTONE: u8 = 10;
    const MINERAL: u8 = 11;
    const PAPER: u8 = 12;
    const WAX: u8 = 13;
    const PLASTIC: u8 = 14;

    match erosion_type {
        ErosionType::Rust => {
            // Only iron/steel rusts
            material == IRON || material == METAL
        }
        ErosionType::Burn => {
            // Organic materials burn
            matches!(material, WOOD | LEATHER | CLOTH | PAPER | WAX)
        }
        ErosionType::Corrode => {
            // Acid corrodes most metals and organics
            !matches!(
                material,
                GLASS | GEMSTONE | MINERAL | GOLD | SILVER | PLASTIC
            )
        }
        ErosionType::Rot => {
            // Organic materials rot
            matches!(material, WOOD | LEATHER | CLOTH | BONE | PAPER)
        }
    }
}

// ============================================================================
// Object erosion functions (objnam.c, do_wear.c)
// ============================================================================

use crate::object::{Object, ObjectClass};

/// Check if erosion matters for an object (erosion_matters equivalent)
///
/// Returns true if the object type can be affected by erosion
/// and erosion would make a meaningful difference to the object.
pub fn erosion_matters(obj: &Object) -> bool {
    // Erosion matters for weapons and armor
    // but not for gold, gems, or objects that are already erosion-proof
    if obj.erosion_proof {
        return false;
    }

    match obj.class {
        ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Tool => true,
        // Erosion doesn't matter for these classes
        ObjectClass::Coin | ObjectClass::Gem | ObjectClass::Rock => false,
        // Wands and rings can corrode in some versions
        ObjectClass::Wand | ObjectClass::Ring => true,
        // Food can rot
        ObjectClass::Food => true,
        // Paper items can burn/rot
        ObjectClass::Scroll | ObjectClass::Spellbook => true,
        // Potions and other items generally don't erode
        _ => false,
    }
}

/// Add erosion words to a name buffer (add_erosion_words equivalent)
///
/// Appends appropriate erosion descriptors (rusty, corroded, etc.) to the prefix.
pub fn add_erosion_words(prefix: &mut String, obj: &Object) {
    // Don't add erosion words if erosion-proof and known
    if obj.rust_known && obj.erosion_proof {
        // Add erosion-proof descriptor
        match obj.class {
            ObjectClass::Weapon | ObjectClass::Armor => {
                if !prefix.is_empty() && !prefix.ends_with(' ') {
                    prefix.push(' ');
                }
                prefix.push_str("rustproof");
            }
            _ => {
                if !prefix.is_empty() && !prefix.ends_with(' ') {
                    prefix.push(' ');
                }
                prefix.push_str("fireproof");
            }
        }
        return;
    }

    // Add erosion1 description (rust/burn)
    if obj.erosion1 > 0 {
        if !prefix.is_empty() && !prefix.ends_with(' ') {
            prefix.push(' ');
        }

        // Determine erosion type based on object class
        let erosion_type = match obj.class {
            ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Ring | ObjectClass::Wand => {
                ErosionType::Rust
            }
            _ => ErosionType::Burn,
        };

        prefix.push_str(match obj.erosion1 {
            1 => erosion_type.level1_name(),
            2 => erosion_type.level2_name(),
            _ => erosion_type.level3_name(),
        });
    }

    // Add erosion2 description (corrode/rot)
    if obj.erosion2 > 0 {
        if !prefix.is_empty() && !prefix.ends_with(' ') {
            prefix.push(' ');
        }

        let erosion_type = match obj.class {
            ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Ring | ObjectClass::Wand => {
                ErosionType::Corrode
            }
            _ => ErosionType::Rot,
        };

        prefix.push_str(match obj.erosion2 {
            1 => erosion_type.level1_name(),
            2 => erosion_type.level2_name(),
            _ => erosion_type.level3_name(),
        });
    }
}

/// Try to erode an object (erode_obj equivalent)
///
/// # Arguments
/// * `obj` - The object to erode
/// * `erosion_type` - Type of erosion (rust, burn, corrode, rot)
/// * `force` - If true, bypass erosion-proof check
/// * `victim_is_player` - True if the victim is the player (for messages)
///
/// # Returns
/// ErodeResult with details of what happened
pub fn erode_obj(
    obj: &mut Object,
    erosion_type: ErosionType,
    force: bool,
    victim_is_player: bool,
) -> ErodeResult {
    // Check if object can erode
    if !erosion_matters(obj) {
        return ErodeResult {
            eroded: false,
            destroyed: false,
            protected: false,
            message: String::new(),
        };
    }

    // Check if erosion-proof (unless forced)
    if obj.erosion_proof && !force {
        return ErodeResult {
            eroded: false,
            destroyed: false,
            protected: true,
            message: if victim_is_player {
                format!("Your {} is not affected.", obj.class_name())
            } else {
                String::new()
            },
        };
    }

    // Check if greased
    if obj.greased {
        // Grease protects but gets used up
        obj.greased = false;
        return ErodeResult {
            eroded: false,
            destroyed: false,
            protected: true,
            message: if victim_is_player {
                format!("The grease protects your {}.", obj.class_name())
            } else {
                String::new()
            },
        };
    }

    // Get the relevant erosion field
    let erosion_slot = erosion_type.slot();
    let current_erosion = if erosion_slot == 0 {
        obj.erosion1
    } else {
        obj.erosion2
    };

    // Check if already maximally eroded
    if current_erosion >= 3 {
        return ErodeResult {
            eroded: false,
            destroyed: false,
            protected: false,
            message: if victim_is_player {
                format!("Your {} can't get any worse.", obj.class_name())
            } else {
                String::new()
            },
        };
    }

    // Apply erosion
    if erosion_slot == 0 {
        obj.erosion1 += 1;
    } else {
        obj.erosion2 += 1;
    }

    let new_erosion = if erosion_slot == 0 {
        obj.erosion1
    } else {
        obj.erosion2
    };

    // Check if destroyed
    let destroyed = new_erosion >= 3 && obj.is_destroyed();

    let message = if victim_is_player {
        let verb = match erosion_type {
            ErosionType::Rust => "rusts",
            ErosionType::Burn => "burns",
            ErosionType::Corrode => "corrodes",
            ErosionType::Rot => "rots",
        };
        if destroyed {
            format!("Your {} {} away completely!", obj.class_name(), verb)
        } else {
            format!("Your {} {}!", obj.class_name(), verb)
        }
    } else {
        String::new()
    };

    ErodeResult {
        eroded: true,
        destroyed,
        protected: false,
        message,
    }
}

/// Erode armor being worn (erode_armor equivalent)
///
/// Tries to erode armor in a specific slot.
///
/// # Arguments
/// * `armor` - The armor to erode
/// * `erosion_type` - Type of erosion
///
/// # Returns
/// ErodeResult with details of what happened
pub fn erode_armor(armor: &mut Object, erosion_type: ErosionType) -> ErodeResult {
    if !armor.is_armor() {
        return ErodeResult {
            eroded: false,
            destroyed: false,
            protected: false,
            message: String::new(),
        };
    }

    erode_obj(armor, erosion_type, false, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::AttackType;

    #[test]
    fn test_damage_type_resistance() {
        assert_eq!(DamageType::Fire.resistance(), Some(Resistance::Fire));
        assert_eq!(DamageType::Cold.resistance(), Some(Resistance::Cold));
        assert_eq!(DamageType::Acid.resistance(), Some(Resistance::Acid));
        assert_eq!(DamageType::Physical.resistance(), None);
    }

    #[test]
    fn test_acid_damage() {
        // With resistance
        let result = acid_damage(10, true, true);
        assert_eq!(result.damage, 0);
        assert!(result.resisted);

        // Without resistance
        let result = acid_damage(10, false, true);
        assert_eq!(result.damage, 10);
        assert!(!result.resisted);
    }

    #[test]
    fn test_dmgval() {
        let mut rng = GameRng::from_entropy();
        let attack = Attack::new(AttackType::Claw, DamageType::Physical, 2, 6);

        // Should return value between 2 and 12
        let damage = dmgval(&mut rng, &attack);
        assert!(damage >= 2 && damage <= 12);
    }

    #[test]
    fn test_erosion_type() {
        assert_eq!(ErosionType::Rust.slot(), 0);
        assert_eq!(ErosionType::Corrode.slot(), 1);
        assert_eq!(ErosionType::Rust.level1_name(), "rusty");
        assert_eq!(ErosionType::Corrode.level2_name(), "very corroded");
    }
}
