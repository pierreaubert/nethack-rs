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
