//! Monster templates (permonst.h)

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::combat::AttackSet;

/// Monster size
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum MonsterSize {
    Tiny = 0,
    Small = 1,
    #[default]
    Medium = 2,
    Large = 3,
    Huge = 4,
    Gigantic = 7,
}

impl MonsterSize {
    /// Get weight multiplier for this size
    pub const fn weight_factor(&self) -> i32 {
        match self {
            MonsterSize::Tiny => 1,
            MonsterSize::Small => 3,
            MonsterSize::Medium => 5,
            MonsterSize::Large => 10,
            MonsterSize::Huge => 20,
            MonsterSize::Gigantic => 100,
        }
    }
}

/// Monster sound types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum MonsterSound {
    #[default]
    Silent = 0,
    Bark = 1,
    Mew = 2,
    Roar = 3,
    Growl = 4,
    Sqeek = 5,
    Sqawk = 6,
    Hiss = 7,
    Buzz = 8,
    Grunt = 9,
    Neigh = 10,
    Wail = 11,
    Gurgle = 12,
    Burble = 13,
    Animal = 14,
    Shriek = 15,
    Bones = 16,
    Laugh = 17,
    Mumble = 18,
    Imitate = 19,
    Orc = 20,
    Humanoid = 21,
    Arrest = 22,
    Soldier = 23,
    Guard = 24,
    Djinni = 25,
    Nurse = 26,
    Seduce = 27,
    Vampire = 28,
    Bribe = 29,
    Cuss = 30,
    Rider = 31,
    Leader = 32,
    Nemesis = 33,
    Guardian = 34,
    Sell = 35,
    Oracle = 36,
    Priest = 37,
    Spell = 38,
    Were = 39,
    Boast = 40,
}

bitflags! {
    /// Monster flags (M1_*, M2_*, M3_* from monst.h)
    #[derive(Debug, Clone, Copy, Default)]
    pub struct MonsterFlags: u64 {
        // M1 flags (movement and basic properties)
        const FLY = 0x00000001;
        const SWIM = 0x00000002;
        const AMORPHOUS = 0x00000004;
        const WALLWALK = 0x00000008;
        const CLING = 0x00000010;
        const TUNNEL = 0x00000020;
        const NEEDPICK = 0x00000040;
        const CONCEAL = 0x00000080;
        const HIDE = 0x00000100;
        const AMPHIBIOUS = 0x00000200;
        const BREATHLESS = 0x00000400;
        const NOTAKE = 0x00000800;
        const NOEYES = 0x00001000;
        const NOHANDS = 0x00002000;
        const NOLIMBS = 0x00004000;
        const NOHEAD = 0x00008000;
        const MINDLESS = 0x00010000;
        const HUMANOID = 0x00020000;
        const ANIMAL = 0x00040000;
        const SLITHY = 0x00080000;
        const UNSOLID = 0x00100000;
        const THICK_HIDE = 0x00200000;
        const OVIPAROUS = 0x00400000;
        const REGEN = 0x00800000;
        const SEE_INVIS = 0x01000000;
        const TPORT = 0x02000000;
        const TPORT_CNTRL = 0x04000000;
        const ACID = 0x08000000;
        const POIS = 0x10000000;
        const CARNIVORE = 0x20000000;
        const HERBIVORE = 0x40000000;
        const METALLIVORE = 0x80000000;

        // M2 flags (behavior and special properties) - shifted
        const NOPOLY = 0x0000000100000000;
        const UNDEAD = 0x0000000200000000;
        const WERE = 0x0000000400000000;
        const HUMAN = 0x0000000800000000;
        const ELF = 0x0000001000000000;
        const DWARF = 0x0000002000000000;
        const GNOME = 0x0000004000000000;
        const ORC = 0x0000008000000000;
        const DEMON = 0x0000010000000000;
        const MERC = 0x0000020000000000;
        const LORD = 0x0000040000000000;
        const PRINCE = 0x0000080000000000;
        const MINION = 0x0000100000000000;
        const GIANT = 0x0000200000000000;
        const SHAPESHIFTER = 0x0000400000000000;
        const MALE = 0x0000800000000000;
        const FEMALE = 0x0001000000000000;
        const NEUTER = 0x0002000000000000;
        const PNAME = 0x0004000000000000;
        const HOSTILE = 0x0008000000000000;
        const PEACEFUL = 0x0010000000000000;
        const DOMESTIC = 0x0020000000000000;
        const WANDER = 0x0040000000000000;
        const STALK = 0x0080000000000000;
        const NASTY = 0x0100000000000000;
        const STRONG = 0x0200000000000000;
        const ROCKTHROW = 0x0400000000000000;
        const GREEDY = 0x0800000000000000;
        const JEWELS = 0x1000000000000000;
        const COLLECT = 0x2000000000000000;
        const MAGIC = 0x4000000000000000;
    }
}

bitflags! {
    /// Monster resistances
    #[derive(Debug, Clone, Copy, Default)]
    pub struct MonsterResistances: u8 {
        const FIRE = 0x01;
        const COLD = 0x02;
        const SLEEP = 0x04;
        const DISINT = 0x08;
        const ELEC = 0x10;
        const POISON = 0x20;
        const ACID = 0x40;
        const STONE = 0x80;
    }
}

// Manual serde for MonsterFlags
impl Serialize for MonsterFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MonsterFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u64::deserialize(deserializer)?;
        Ok(MonsterFlags::from_bits_truncate(bits))
    }
}

// Manual serde for MonsterResistances
impl Serialize for MonsterResistances {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MonsterResistances {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        Ok(MonsterResistances::from_bits_truncate(bits))
    }
}

/// Monster template (static data for each monster type)
#[derive(Debug, Clone)]
pub struct PerMonst {
    /// Monster name
    pub name: &'static str,

    /// Display symbol
    pub symbol: char,

    /// Base level (difficulty)
    pub level: i8,

    /// Movement speed
    pub move_speed: i8,

    /// Base armor class
    pub armor_class: i8,

    /// Magic resistance (0-100)
    pub magic_resistance: i8,

    /// Alignment (-128 to 127)
    pub alignment: i8,

    /// Generation flags
    pub gen_flags: u16,

    /// Attacks (up to 6)
    pub attacks: AttackSet,

    /// Corpse weight
    pub corpse_weight: u16,

    /// Nutrition from corpse
    pub corpse_nutrition: u16,

    /// Sound type
    pub sound: MonsterSound,

    /// Physical size
    pub size: MonsterSize,

    /// Resistances
    pub resistances: MonsterResistances,

    /// Resistances conveyed by eating corpse
    pub conveys: MonsterResistances,

    /// Monster flags
    pub flags: MonsterFlags,

    /// Difficulty rating
    pub difficulty: u8,

    /// Display color
    pub color: u8,
}

impl PerMonst {
    /// Check if monster flies
    pub const fn flies(&self) -> bool {
        self.flags.contains(MonsterFlags::FLY)
    }

    /// Check if monster swims
    pub const fn swims(&self) -> bool {
        self.flags.contains(MonsterFlags::SWIM)
    }

    /// Check if monster passes through walls
    pub const fn passes_walls(&self) -> bool {
        self.flags.contains(MonsterFlags::WALLWALK)
    }

    /// Check if monster is undead
    pub const fn is_undead(&self) -> bool {
        self.flags.contains(MonsterFlags::UNDEAD)
    }

    /// Check if monster is a demon
    pub const fn is_demon(&self) -> bool {
        self.flags.contains(MonsterFlags::DEMON)
    }

    /// Check if monster regenerates
    pub const fn regenerates(&self) -> bool {
        self.flags.contains(MonsterFlags::REGEN)
    }

    /// Check if monster can see invisible
    pub const fn sees_invisible(&self) -> bool {
        self.flags.contains(MonsterFlags::SEE_INVIS)
    }

    /// Check if monster is hostile by default
    pub const fn is_hostile(&self) -> bool {
        self.flags.contains(MonsterFlags::HOSTILE)
    }

    /// Check if monster is peaceful by default
    pub const fn is_peaceful(&self) -> bool {
        self.flags.contains(MonsterFlags::PEACEFUL)
    }
}
