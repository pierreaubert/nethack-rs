//! Player intrinsic and extrinsic properties
//!
//! Properties are abilities/resistances that can be intrinsic (permanent)
//! or extrinsic (from worn items).

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Property types (from prop.h)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum Property {
    // Movement properties
    Speed = 0,
    VeryFast = 1,
    Levitation = 2,
    Flying = 3,
    Swimming = 4,
    MagicBreathing = 5,
    PassesWalls = 6,
    Jumping = 7,

    // Resistances
    FireResistance = 10,
    ColdResistance = 11,
    SleepResistance = 12,
    DisintResistance = 13,
    ShockResistance = 14,
    PoisonResistance = 15,
    AcidResistance = 16,
    StoneResistance = 17,
    DrainResistance = 18,
    SickResistance = 19,

    // Vision
    SeeInvisible = 20,
    Telepathy = 21,
    Infravision = 22,
    Xray = 23,
    Searching = 24,
    Clairvoyant = 25,
    Warning = 26,
    WarnOfMon = 27,

    // Stealth
    Stealth = 30,
    Invisibility = 31,
    Displaced = 32,
    Aggravate = 33,
    Conflict = 34,

    // Protection
    Protection = 40,
    ProtFromShapechangers = 41,
    FreeAction = 42,
    Reflection = 43,
    MagicResistance = 44,
    HalfSpellDamage = 45,
    HalfPhysDamage = 46,
    Regeneration = 47,
    EnergyRegeneration = 48,

    // Misc
    Teleportation = 50,
    TeleportControl = 51,
    Polymorph = 52,
    PolyControl = 53,
    Unchanging = 54,
    Fumbling = 55,
    WoundedLegs = 56,
    Sleepy = 57,
    Hunger = 58,
    SlowDigestion = 59,
    SustainAbility = 60,
    LifeSaving = 61,
}

impl Property {
    pub const LAST: Property = Property::LifeSaving;

    /// Check if this is a resistance property
    pub const fn is_resistance(&self) -> bool {
        (*self as u8) >= 10 && (*self as u8) <= 19
    }

    /// Check if this is a vision property
    pub const fn is_vision(&self) -> bool {
        (*self as u8) >= 20 && (*self as u8) <= 27
    }
}

bitflags! {
    /// Flags for property sources
    #[derive(Debug, Clone, Copy, Default)]
    pub struct PropertyFlags: u32 {
        /// From intrinsic (permanent)
        const INTRINSIC = 0x0001;
        /// Blocked by worn item
        const BLOCKED = 0x0002;
        /// From timeout (temporary)
        const TIMEOUT = 0x0004;

        // Extrinsic sources (from equipment)
        const FROM_HELM = 0x0010;
        const FROM_ARMOR = 0x0020;
        const FROM_CLOAK = 0x0040;
        const FROM_GLOVES = 0x0080;
        const FROM_BOOTS = 0x0100;
        const FROM_SHIELD = 0x0200;
        const FROM_WEAPON = 0x0400;
        const FROM_RING_L = 0x0800;
        const FROM_RING_R = 0x1000;
        const FROM_AMULET = 0x2000;
        const FROM_ARTIFACT = 0x4000;

        /// Any extrinsic source
        const EXTRINSIC = Self::FROM_HELM.bits()
            | Self::FROM_ARMOR.bits()
            | Self::FROM_CLOAK.bits()
            | Self::FROM_GLOVES.bits()
            | Self::FROM_BOOTS.bits()
            | Self::FROM_SHIELD.bits()
            | Self::FROM_WEAPON.bits()
            | Self::FROM_RING_L.bits()
            | Self::FROM_RING_R.bits()
            | Self::FROM_AMULET.bits()
            | Self::FROM_ARTIFACT.bits();
    }
}

// Manual serde impl for PropertyFlags
impl Serialize for PropertyFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PropertyFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Ok(PropertyFlags::from_bits_truncate(bits))
    }
}

/// Property state array for tracking all properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySet {
    props: Vec<PropertyFlags>,
    timeouts: Vec<u32>,
}

impl Default for PropertySet {
    fn default() -> Self {
        let size = Property::LAST as usize + 1;
        Self {
            props: vec![PropertyFlags::empty(); size],
            timeouts: vec![0; size],
        }
    }
}

impl PropertySet {
    /// Check if player has a property (from any source)
    pub fn has(&self, prop: Property) -> bool {
        let flags = self.props[prop as usize];
        if flags.contains(PropertyFlags::BLOCKED) {
            return false;
        }
        flags.intersects(PropertyFlags::INTRINSIC | PropertyFlags::EXTRINSIC | PropertyFlags::TIMEOUT)
    }

    /// Check if player has intrinsic property
    pub fn has_intrinsic(&self, prop: Property) -> bool {
        self.props[prop as usize].contains(PropertyFlags::INTRINSIC)
    }

    /// Check if player has extrinsic property
    pub fn has_extrinsic(&self, prop: Property) -> bool {
        self.props[prop as usize].intersects(PropertyFlags::EXTRINSIC)
    }

    /// Grant intrinsic property
    pub fn grant_intrinsic(&mut self, prop: Property) {
        self.props[prop as usize].insert(PropertyFlags::INTRINSIC);
    }

    /// Remove intrinsic property
    pub fn remove_intrinsic(&mut self, prop: Property) {
        self.props[prop as usize].remove(PropertyFlags::INTRINSIC);
    }

    /// Grant extrinsic property from a source
    pub fn grant_extrinsic(&mut self, prop: Property, source: PropertyFlags) {
        self.props[prop as usize].insert(source);
    }

    /// Remove extrinsic property from a source
    pub fn remove_extrinsic(&mut self, prop: Property, source: PropertyFlags) {
        self.props[prop as usize].remove(source);
    }

    /// Set property timeout
    pub fn set_timeout(&mut self, prop: Property, turns: u32) {
        self.timeouts[prop as usize] = turns;
        if turns > 0 {
            self.props[prop as usize].insert(PropertyFlags::TIMEOUT);
        } else {
            self.props[prop as usize].remove(PropertyFlags::TIMEOUT);
        }
    }

    /// Decrement all timeouts by 1
    pub fn tick_timeouts(&mut self) {
        for (i, timeout) in self.timeouts.iter_mut().enumerate() {
            if *timeout > 0 {
                *timeout -= 1;
                if *timeout == 0 {
                    self.props[i].remove(PropertyFlags::TIMEOUT);
                }
            }
        }
    }

    /// Get timeout remaining for a property
    pub fn timeout(&self, prop: Property) -> u32 {
        self.timeouts[prop as usize]
    }

    /// Block a property
    pub fn block(&mut self, prop: Property) {
        self.props[prop as usize].insert(PropertyFlags::BLOCKED);
    }

    /// Unblock a property
    pub fn unblock(&mut self, prop: Property) {
        self.props[prop as usize].remove(PropertyFlags::BLOCKED);
    }
}
