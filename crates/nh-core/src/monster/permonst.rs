//! Monster templates (permonst.h)

#[cfg(not(feature = "std"))]
use crate::compat::*;

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
        // C: M1_NOLIMBS = 0x00006000L = NOHANDS | 0x4000
        // NOLIMBS is a SUPERSET of NOHANDS: monsters with no limbs also have no hands.
        const NOLIMBS = 0x00006000;
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
    pub struct MonsterResistances: u16 {
        const FIRE = 0x0001;
        const COLD = 0x0002;
        const SLEEP = 0x0004;
        const DISINT = 0x0008;
        const ELEC = 0x0010;
        const POISON = 0x0020;
        const ACID = 0x0040;
        const STONE = 0x0080;
        const MAGIC = 0x0100;
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
        let bits = u16::deserialize(deserializer)?;
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

    // ========================================================================
    // Additional monster type checks (from mondata.h macros)
    // ========================================================================

    /// Check if monster is amorphous (can squeeze through tight spaces)
    pub const fn is_amorphous(&self) -> bool {
        self.flags.contains(MonsterFlags::AMORPHOUS)
    }

    /// Check if monster can cling to ceilings
    pub const fn can_cling(&self) -> bool {
        self.flags.contains(MonsterFlags::CLING)
    }

    /// Check if monster can tunnel through rock
    pub const fn can_tunnel(&self) -> bool {
        self.flags.contains(MonsterFlags::TUNNEL)
    }

    /// Check if monster needs a pick to tunnel
    pub const fn needs_pick(&self) -> bool {
        self.flags.contains(MonsterFlags::NEEDPICK)
    }

    /// Check if monster can conceal itself (like a mimic)
    pub const fn can_conceal(&self) -> bool {
        self.flags.contains(MonsterFlags::CONCEAL)
    }

    /// Check if monster can hide under objects
    pub const fn can_hide(&self) -> bool {
        self.flags.contains(MonsterFlags::HIDE)
    }

    /// Check if monster is amphibious (can breathe in water)
    pub const fn is_amphibious(&self) -> bool {
        self.flags.contains(MonsterFlags::AMPHIBIOUS)
    }

    /// Check if monster doesn't need to breathe
    pub const fn is_breathless(&self) -> bool {
        self.flags.contains(MonsterFlags::BREATHLESS)
    }

    /// Check if monster doesn't pick up items
    pub const fn cannot_take(&self) -> bool {
        self.flags.contains(MonsterFlags::NOTAKE)
    }

    /// Check if monster has no eyes (can't be blinded)
    pub const fn has_no_eyes(&self) -> bool {
        self.flags.contains(MonsterFlags::NOEYES)
    }

    /// Check if monster has no hands (can't use tools)
    pub const fn has_no_hands(&self) -> bool {
        self.flags.contains(MonsterFlags::NOHANDS)
    }

    /// Check if monster has no limbs
    pub const fn has_no_limbs(&self) -> bool {
        self.flags.contains(MonsterFlags::NOLIMBS)
    }

    /// Check if monster has no head
    pub const fn has_no_head(&self) -> bool {
        self.flags.contains(MonsterFlags::NOHEAD)
    }

    /// Check if monster is mindless
    pub const fn is_mindless(&self) -> bool {
        self.flags.contains(MonsterFlags::MINDLESS)
    }

    /// Check if monster is humanoid shaped
    pub const fn is_humanoid(&self) -> bool {
        self.flags.contains(MonsterFlags::HUMANOID)
    }

    /// Check if monster is an animal
    pub const fn is_animal(&self) -> bool {
        self.flags.contains(MonsterFlags::ANIMAL)
    }

    /// Check if monster is snake/worm-like (slithy)
    pub const fn is_slithy(&self) -> bool {
        self.flags.contains(MonsterFlags::SLITHY)
    }

    /// Check if monster is insubstantial (unsolid)
    pub const fn is_unsolid(&self) -> bool {
        self.flags.contains(MonsterFlags::UNSOLID)
    }

    /// Check if monster has thick hide (harder to hit)
    pub const fn has_thick_hide(&self) -> bool {
        self.flags.contains(MonsterFlags::THICK_HIDE)
    }

    /// Check if monster lays eggs
    pub const fn is_oviparous(&self) -> bool {
        self.flags.contains(MonsterFlags::OVIPAROUS)
    }

    /// Check if monster is acidic
    pub const fn is_acidic(&self) -> bool {
        self.flags.contains(MonsterFlags::ACID)
    }

    /// Check if monster is poisonous
    pub const fn is_poisonous(&self) -> bool {
        self.flags.contains(MonsterFlags::POIS)
    }

    /// Check if monster is carnivorous
    pub const fn is_carnivore(&self) -> bool {
        self.flags.contains(MonsterFlags::CARNIVORE)
    }

    /// Check if monster is herbivorous
    pub const fn is_herbivore(&self) -> bool {
        self.flags.contains(MonsterFlags::HERBIVORE)
    }

    /// Check if monster eats metal
    pub const fn is_metallivore(&self) -> bool {
        self.flags.contains(MonsterFlags::METALLIVORE)
    }

    /// Check if monster can teleport
    pub const fn can_teleport(&self) -> bool {
        self.flags.contains(MonsterFlags::TPORT)
    }

    /// Check if monster has teleport control
    pub const fn has_teleport_control(&self) -> bool {
        self.flags.contains(MonsterFlags::TPORT_CNTRL)
    }

    // M2 flags

    /// Check if monster can't be polymorphed
    pub const fn cannot_poly(&self) -> bool {
        self.flags.contains(MonsterFlags::NOPOLY)
    }

    /// Check if monster is a were-creature
    pub const fn is_were(&self) -> bool {
        self.flags.contains(MonsterFlags::WERE)
    }

    /// Check if monster is human
    pub const fn is_human(&self) -> bool {
        self.flags.contains(MonsterFlags::HUMAN)
    }

    /// Check if monster is an elf
    pub const fn is_elf(&self) -> bool {
        self.flags.contains(MonsterFlags::ELF)
    }

    /// Check if monster is a dwarf
    pub const fn is_dwarf(&self) -> bool {
        self.flags.contains(MonsterFlags::DWARF)
    }

    /// Check if monster is a gnome
    pub const fn is_gnome(&self) -> bool {
        self.flags.contains(MonsterFlags::GNOME)
    }

    /// Check if monster is an orc
    pub const fn is_orc(&self) -> bool {
        self.flags.contains(MonsterFlags::ORC)
    }

    /// Check if monster is a mercenary
    pub const fn is_mercenary(&self) -> bool {
        self.flags.contains(MonsterFlags::MERC)
    }

    /// Check if monster is a lord (powerful)
    pub const fn is_lord(&self) -> bool {
        self.flags.contains(MonsterFlags::LORD)
    }

    /// Check if monster is a prince (very powerful)
    pub const fn is_prince(&self) -> bool {
        self.flags.contains(MonsterFlags::PRINCE)
    }

    /// Check if monster is a minion (servant of a deity)
    pub const fn is_minion(&self) -> bool {
        self.flags.contains(MonsterFlags::MINION)
    }

    /// Check if monster is a giant
    pub const fn is_giant(&self) -> bool {
        self.flags.contains(MonsterFlags::GIANT)
    }

    /// Check if monster is a shapeshifter
    pub const fn is_shapeshifter(&self) -> bool {
        self.flags.contains(MonsterFlags::SHAPESHIFTER)
    }

    /// Check if monster is male
    pub const fn is_male(&self) -> bool {
        self.flags.contains(MonsterFlags::MALE)
    }

    /// Check if monster is female
    pub const fn is_female(&self) -> bool {
        self.flags.contains(MonsterFlags::FEMALE)
    }

    /// Check if monster is neuter
    pub const fn is_neuter(&self) -> bool {
        self.flags.contains(MonsterFlags::NEUTER)
    }

    /// Check if monster has a proper name (unique)
    pub const fn has_pname(&self) -> bool {
        self.flags.contains(MonsterFlags::PNAME)
    }

    /// Check if monster is domestic (can be tamed easily)
    pub const fn is_domestic(&self) -> bool {
        self.flags.contains(MonsterFlags::DOMESTIC)
    }

    /// Check if monster wanders randomly
    pub const fn wanders(&self) -> bool {
        self.flags.contains(MonsterFlags::WANDER)
    }

    /// Check if monster stalks prey
    pub const fn stalks(&self) -> bool {
        self.flags.contains(MonsterFlags::STALK)
    }

    /// Check if monster is nasty (extra dangerous)
    pub const fn is_nasty(&self) -> bool {
        self.flags.contains(MonsterFlags::NASTY)
    }

    /// Check if monster is strong
    pub const fn is_strong(&self) -> bool {
        self.flags.contains(MonsterFlags::STRONG)
    }

    /// Check if monster throws rocks
    pub const fn throws_rocks(&self) -> bool {
        self.flags.contains(MonsterFlags::ROCKTHROW)
    }

    /// Check if monster is greedy (collects gold)
    pub const fn is_greedy(&self) -> bool {
        self.flags.contains(MonsterFlags::GREEDY)
    }

    /// Check if monster collects jewels
    pub const fn collects_jewels(&self) -> bool {
        self.flags.contains(MonsterFlags::JEWELS)
    }

    /// Check if monster collects items
    pub const fn collects(&self) -> bool {
        self.flags.contains(MonsterFlags::COLLECT)
    }

    /// Check if monster picks up magic items
    pub const fn wants_magic(&self) -> bool {
        self.flags.contains(MonsterFlags::MAGIC)
    }

    // Resistance checks from resistances field

    /// Check if resists fire
    pub const fn resists_fire(&self) -> bool {
        self.resistances.contains(MonsterResistances::FIRE)
    }

    /// Check if resists cold
    pub const fn resists_cold(&self) -> bool {
        self.resistances.contains(MonsterResistances::COLD)
    }

    /// Check if resists sleep
    pub const fn resists_sleep(&self) -> bool {
        self.resistances.contains(MonsterResistances::SLEEP)
    }

    /// Check if resists disintegration
    pub const fn resists_disint(&self) -> bool {
        self.resistances.contains(MonsterResistances::DISINT)
    }

    /// Check if resists electricity
    pub const fn resists_elec(&self) -> bool {
        self.resistances.contains(MonsterResistances::ELEC)
    }

    /// Check if resists poison
    pub const fn resists_poison(&self) -> bool {
        self.resistances.contains(MonsterResistances::POISON)
    }

    /// Check if resists acid
    pub const fn resists_acid(&self) -> bool {
        self.resistances.contains(MonsterResistances::ACID)
    }

    /// Check if resists petrification
    pub const fn resists_stone(&self) -> bool {
        self.resistances.contains(MonsterResistances::STONE)
    }

    /// Check if resists blindness (resists_blnd equivalent)
    ///
    /// Eyeless monsters and certain monster types are immune to blinding effects.
    pub const fn resists_blnd(&self) -> bool {
        // Monsters without eyes can't be blinded
        // This includes eyeless monsters and those with special vision
        !self.flags.contains(MonsterFlags::ANIMAL)
            || self.flags.contains(MonsterFlags::TUNNEL)
            || self.is_breathless()
    }

    /// Check if resists level drain (resists_drli equivalent)
    ///
    /// Undead and certain other creatures are immune to level drain.
    pub const fn resists_drli(&self) -> bool {
        // Undead and demons resist level drain
        // Non-living creatures (golems) would also resist but are handled separately
        self.flags.contains(MonsterFlags::UNDEAD) || self.is_demon()
    }

    /// Check if resists magic missiles (resists_magm equivalent)
    ///
    /// Based on magic resistance value.
    pub const fn resists_magm(&self) -> bool {
        // High magic resistance provides immunity to magic missiles
        self.magic_resistance >= 50
    }

    // Compound checks

    /// Check if monster can survive in water without drowning
    pub const fn can_survive_water(&self) -> bool {
        self.swims() || self.is_amphibious() || self.is_breathless() || self.flies()
    }

    /// Check if monster can survive in lava
    pub const fn can_survive_lava(&self) -> bool {
        self.resists_fire() || self.flies() || self.is_breathless()
    }

    /// Check if monster is flyer (flies intrinsically)
    pub const fn is_flyer(&self) -> bool {
        self.flies()
    }

    /// Check if monster is swimmer
    pub const fn is_swimmer(&self) -> bool {
        self.swims()
    }

    /// Check if monster can pass through walls (phasing)
    pub const fn is_phaser(&self) -> bool {
        self.passes_walls()
    }

    /// Check if monster is a floater (levitates)
    pub const fn is_floater(&self) -> bool {
        // Would check specific monster types like eye of the deep
        // For now, same as flying
        self.flies()
    }

    /// Check if monster is a clinger (can cling to ceiling)
    pub const fn is_clinger(&self) -> bool {
        self.can_cling()
    }

    /// Check if monster breathes (needs air)
    pub const fn needs_air(&self) -> bool {
        !self.is_breathless()
    }

    /// Check if monster can pass through iron bars (passes_bars equivalent)
    ///
    /// Monsters can pass through bars if they:
    /// - Can phase through walls
    /// - Are amorphous (can squeeze through)
    /// - Are tiny size (small enough to fit between bars)
    /// - Are whirly/non-corporeal
    pub const fn passes_bars(&self) -> bool {
        self.passes_walls()
            || self.is_amorphous()
            || matches!(self.size, MonsterSize::Tiny)
            || self.flags.contains(MonsterFlags::NOEYES) // whirly creatures
    }

    /// Check if monster is big (large or bigger)
    pub const fn is_big(&self) -> bool {
        matches!(
            self.size,
            MonsterSize::Large | MonsterSize::Huge | MonsterSize::Gigantic
        )
    }

    /// Check if monster is very small (tiny)
    pub const fn is_very_small(&self) -> bool {
        matches!(self.size, MonsterSize::Tiny)
    }

    /// Check if any of the monster's attacks deal the specified damage type (dmgtype from mondata.c)
    pub fn dmgtype(&self, dt: crate::combat::DamageType) -> bool {
        self.attacks.iter().any(|atk| {
            atk.damage_type == dt
                && atk.attack_type != crate::combat::AttackType::None
        })
    }

    /// Check if touching/eating this monster's flesh causes petrification (flesh_petrifies from mondata.h)
    pub fn flesh_petrifies(&self) -> bool {
        self.name == "cockatrice" || self.name == "chickatrice"
    }

    /// Check if this monster's corpse doesn't rot (nonrotting_corpse from eat.c)
    pub fn nonrotting_corpse(&self) -> bool {
        // Lizards, lichens, and Riders don't rot
        self.symbol == ':' // lizard class
            || self.name == "lichen"
            || self.name == "Death"
            || self.name == "Pestilence"
            || self.name == "Famine"
    }

    /// Check if the monster is telepathic
    pub fn is_telepathic(&self) -> bool {
        self.name == "floating eye" || self.name == "mind flayer" || self.name == "master mind flayer"
    }
}

// ============================================================================
// Monster class symbols (S_* constants from monsym.h)
// ============================================================================

/// Monster class symbols mapping characters to monster classes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MonsterClass {
    Ant = 0,          // 'a'
    Blob = 1,         // 'b'
    Cockatrice = 2,   // 'c'
    Dog = 3,          // 'd'
    Eye = 4,          // 'e'
    Feline = 5,       // 'f'
    Gremlin = 6,      // 'g'
    Humanoid = 7,     // 'h'
    Imp = 8,          // 'i'
    Jelly = 9,        // 'j'
    Kobold = 10,      // 'k'
    Leprechaun = 11,  // 'l'
    Mimic = 12,       // 'm'
    Nymph = 13,       // 'n'
    Orc = 14,         // 'o'
    Piercer = 15,     // 'p'
    Quadruped = 16,   // 'q'
    Rodent = 17,      // 'r'
    Spider = 18,      // 's'
    Trapper = 19,     // 't'
    Unicorn = 20,     // 'u'
    Vortex = 21,      // 'v'
    Worm = 22,        // 'w'
    Xan = 23,         // 'x'
    Light = 24,       // 'y'
    Zruty = 25,       // 'z'
    Angel = 26,       // 'A'
    Bat = 27,         // 'B'
    Centaur = 28,     // 'C'
    Dragon = 29,      // 'D'
    Elemental = 30,   // 'E'
    Fungus = 31,      // 'F'
    Gnome = 32,       // 'G'
    Giant = 33,       // 'H'
    Jabberwock = 35,  // 'J'
    Kop = 36,         // 'K'
    Lich = 37,        // 'L'
    Mummy = 38,       // 'M'
    Naga = 39,        // 'N'
    Ogre = 40,        // 'O'
    Pudding = 41,     // 'P'
    Quantum = 42,     // 'Q'
    RustMonster = 43, // 'R'
    Snake = 44,       // 'S'
    Troll = 45,       // 'T'
    UmberHulk = 46,   // 'U'
    Vampire = 47,     // 'V'
    Wraith = 48,      // 'W'
    Xorn = 49,        // 'X'
    Ape = 50,         // 'Y'
    Zombie = 51,      // 'Z'
    Human = 52,       // '@'
    Ghost = 53,       // ' '
    Golem = 54,       // '\''
    Demon = 55,       // '&'
    Eel = 56,         // ';'
    Lizard = 57,      // ':'
    WormTail = 58,    // '~'
    MimicDef = 59,    // ']'
    MaxMClasses = 60,
}

impl MonsterClass {
    /// Get the display symbol for this monster class
    pub const fn symbol(&self) -> char {
        match self {
            Self::Ant => 'a',
            Self::Blob => 'b',
            Self::Cockatrice => 'c',
            Self::Dog => 'd',
            Self::Eye => 'e',
            Self::Feline => 'f',
            Self::Gremlin => 'g',
            Self::Humanoid => 'h',
            Self::Imp => 'i',
            Self::Jelly => 'j',
            Self::Kobold => 'k',
            Self::Leprechaun => 'l',
            Self::Mimic => 'm',
            Self::Nymph => 'n',
            Self::Orc => 'o',
            Self::Piercer => 'p',
            Self::Quadruped => 'q',
            Self::Rodent => 'r',
            Self::Spider => 's',
            Self::Trapper => 't',
            Self::Unicorn => 'u',
            Self::Vortex => 'v',
            Self::Worm => 'w',
            Self::Xan => 'x',
            Self::Light => 'y',
            Self::Zruty => 'z',
            Self::Angel => 'A',
            Self::Bat => 'B',
            Self::Centaur => 'C',
            Self::Dragon => 'D',
            Self::Elemental => 'E',
            Self::Fungus => 'F',
            Self::Gnome => 'G',
            Self::Giant => 'H',
            Self::Jabberwock => 'J',
            Self::Kop => 'K',
            Self::Lich => 'L',
            Self::Mummy => 'M',
            Self::Naga => 'N',
            Self::Ogre => 'O',
            Self::Pudding => 'P',
            Self::Quantum => 'Q',
            Self::RustMonster => 'R',
            Self::Snake => 'S',
            Self::Troll => 'T',
            Self::UmberHulk => 'U',
            Self::Vampire => 'V',
            Self::Wraith => 'W',
            Self::Xorn => 'X',
            Self::Ape => 'Y',
            Self::Zombie => 'Z',
            Self::Human => '@',
            Self::Ghost => ' ',
            Self::Golem => '\'',
            Self::Demon => '&',
            Self::Eel => ';',
            Self::Lizard => ':',
            Self::WormTail => '~',
            Self::MimicDef => ']',
            Self::MaxMClasses => '?',
        }
    }
}

/// Convert a character to monster class (def_char_to_monclass from mondata.c)
///
/// Maps display symbols to their corresponding monster class.
/// Returns None if the character doesn't correspond to a monster class.
pub fn def_char_to_monclass(c: char) -> Option<MonsterClass> {
    match c {
        'a' => Some(MonsterClass::Ant),
        'b' => Some(MonsterClass::Blob),
        'c' => Some(MonsterClass::Cockatrice),
        'd' => Some(MonsterClass::Dog),
        'e' => Some(MonsterClass::Eye),
        'f' => Some(MonsterClass::Feline),
        'g' => Some(MonsterClass::Gremlin),
        'h' => Some(MonsterClass::Humanoid),
        'i' => Some(MonsterClass::Imp),
        'j' => Some(MonsterClass::Jelly),
        'k' => Some(MonsterClass::Kobold),
        'l' => Some(MonsterClass::Leprechaun),
        'm' => Some(MonsterClass::Mimic),
        'n' => Some(MonsterClass::Nymph),
        'o' => Some(MonsterClass::Orc),
        'p' => Some(MonsterClass::Piercer),
        'q' => Some(MonsterClass::Quadruped),
        'r' => Some(MonsterClass::Rodent),
        's' => Some(MonsterClass::Spider),
        't' => Some(MonsterClass::Trapper),
        'u' => Some(MonsterClass::Unicorn),
        'v' => Some(MonsterClass::Vortex),
        'w' => Some(MonsterClass::Worm),
        'x' => Some(MonsterClass::Xan),
        'y' => Some(MonsterClass::Light),
        'z' => Some(MonsterClass::Zruty),
        'A' => Some(MonsterClass::Angel),
        'B' => Some(MonsterClass::Bat),
        'C' => Some(MonsterClass::Centaur),
        'D' => Some(MonsterClass::Dragon),
        'E' => Some(MonsterClass::Elemental),
        'F' => Some(MonsterClass::Fungus),
        'G' => Some(MonsterClass::Gnome),
        'H' => Some(MonsterClass::Giant),
        'J' => Some(MonsterClass::Jabberwock),
        'K' => Some(MonsterClass::Kop),
        'L' => Some(MonsterClass::Lich),
        'M' => Some(MonsterClass::Mummy),
        'N' => Some(MonsterClass::Naga),
        'O' => Some(MonsterClass::Ogre),
        'P' => Some(MonsterClass::Pudding),
        'Q' => Some(MonsterClass::Quantum),
        'R' => Some(MonsterClass::RustMonster),
        'S' => Some(MonsterClass::Snake),
        'T' => Some(MonsterClass::Troll),
        'U' => Some(MonsterClass::UmberHulk),
        'V' => Some(MonsterClass::Vampire),
        'W' => Some(MonsterClass::Wraith),
        'X' => Some(MonsterClass::Xorn),
        'Y' => Some(MonsterClass::Ape),
        'Z' => Some(MonsterClass::Zombie),
        '@' => Some(MonsterClass::Human),
        '\'' => Some(MonsterClass::Golem),
        '&' => Some(MonsterClass::Demon),
        ';' => Some(MonsterClass::Eel),
        ':' => Some(MonsterClass::Lizard),
        '~' => Some(MonsterClass::WormTail),
        ']' => Some(MonsterClass::MimicDef),
        _ => None,
    }
}

/// Growth table for monster progression (grownups from mondata.c)
/// Maps (smaller form index, larger form index)
const GROWNUPS: &[(i16, i16)] = &[
    (10, 11),   // chickatrice -> cockatrice
    (35, 37),   // little dog -> dog
    (37, 38),   // dog -> large dog
    (44, 45),   // hell hound pup -> hell hound
    (41, 43),   // winter wolf cub -> winter wolf
    (54, 55),   // kitten -> housecat
    (55, 60),   // housecat -> large cat
    (140, 141), // pony -> horse
    (141, 142), // horse -> warhorse
    (85, 86),   // kobold -> large kobold
    (86, 87),   // large kobold -> kobold lord
    (217, 218), // gnome -> gnome lord
    (218, 220), // gnome lord -> gnome king
    (67, 69),   // dwarf -> dwarf lord
    (69, 70),   // dwarf lord -> dwarf king
    (71, 72),   // mind flayer -> master mind flayer
    (103, 108), // orc -> orc captain
    (104, 108), // hill orc -> orc captain
    (105, 108), // mordor orc -> orc captain
    (106, 108), // uruk-hai -> orc captain
    (122, 123), // sewer rat -> giant rat
    (129, 131), // cave spider -> giant spider
    (259, 260), // ogre -> ogre lord
    (260, 261), // ogre lord -> ogre king
    (250, 252), // elf -> elf lord
    (252, 253), // elf lord -> elvenking
    (241, 242), // lich -> demilich
    (242, 243), // demilich -> master lich
    (243, 244), // master lich -> arch-lich
    (173, 174), // vampire -> vampire lord
    (170, 171), // bat -> giant bat
    // Baby dragons -> adult dragons
    (181, 191), // baby gray dragon -> gray dragon
    (182, 192), // baby silver dragon -> silver dragon
    (183, 193), // baby red dragon -> red dragon
    (184, 194), // baby white dragon -> white dragon
    (185, 195), // baby orange dragon -> orange dragon
    (186, 196), // baby black dragon -> black dragon
    (187, 197), // baby blue dragon -> blue dragon
    (188, 198), // baby green dragon -> green dragon
    (189, 199), // baby yellow dragon -> yellow dragon
    // Naga hatchlings -> adult nagas
    (254, 255), // red naga hatchling -> red naga
    (256, 257), // black naga hatchling -> black naga
    (258, 259), // golden naga hatchling -> golden naga
    // Mimics
    (92, 93), // small mimic -> large mimic
    (93, 94), // large mimic -> giant mimic
    // Worms
    (151, 153), // baby long worm -> long worm
    (152, 154), // baby purple worm -> purple worm
    // Soldiers
    (326, 327), // soldier -> sergeant
    (327, 328), // sergeant -> lieutenant
    (328, 329), // lieutenant -> captain
    (330, 331), // watchman -> watch captain
];

/// Get the grown-up form of a monster (little_to_big from mondata.c)
///
/// Returns the monster type index of the larger/more mature form,
/// or the same index if there is no larger form.
pub fn little_to_big(montype: i16) -> i16 {
    for &(little, big) in GROWNUPS {
        if montype == little {
            return big;
        }
    }
    montype
}

/// Get the smaller form of a monster (big_to_little from mondata.c)
///
/// Returns the monster type index of the smaller/younger form,
/// or the same index if there is no smaller form.
pub fn big_to_little(montype: i16) -> i16 {
    for &(little, big) in GROWNUPS {
        if montype == big {
            return little;
        }
    }
    montype
}

/// Check if two monster types are part of the same growth progression
/// (big_little_match from mondata.c)
pub fn big_little_match(montyp1: i16, montyp2: i16) -> bool {
    if montyp1 == montyp2 {
        return true;
    }

    // Check if montyp1 can grow into montyp2
    let mut current = montyp1;
    loop {
        let next = little_to_big(current);
        if next == current {
            break;
        }
        if next == montyp2 {
            return true;
        }
        current = next;
    }

    // Check if montyp2 can grow into montyp1
    current = montyp2;
    loop {
        let next = little_to_big(current);
        if next == current {
            break;
        }
        if next == montyp1 {
            return true;
        }
        current = next;
    }

    false
}

/// Find monster type by name (name_to_mon from mondata.c)
///
/// Searches for a monster by name, handling:
/// - Article stripping ("a ", "an ", "the ")
/// - Plural forms
/// - Alternate spellings
///
/// Returns the monster type index or None if not found.
pub fn name_to_mon(name: &str) -> Option<i16> {
    use crate::data::MONSTERS;

    let mut search_name = name.to_lowercase();

    // Strip leading articles
    if search_name.starts_with("a ") {
        search_name = search_name[2..].to_string();
    } else if search_name.starts_with("an ") {
        search_name = search_name[3..].to_string();
    } else if search_name.starts_with("the ") {
        search_name = search_name[4..].to_string();
    }

    // Handle common alternate spellings
    let alt_spellings: &[(&str, &str)] = &[
        ("grey dragon", "gray dragon"),
        ("grey unicorn", "gray unicorn"),
        ("grey ooze", "gray ooze"),
        ("gray-elf", "grey elf"),
        ("gray elf", "grey elf"),
        ("mindflayer", "mind flayer"),
        ("master mindflayer", "master mind flayer"),
        ("ki rin", "ki-rin"),
        ("uruk hai", "uruk-hai"),
        ("olog hai", "olog-hai"),
        ("arch lich", "arch-lich"),
        ("halfling", "hobbit"),
        ("genie", "djinni"),
    ];

    for &(alt, canonical) in alt_spellings {
        if search_name == alt {
            search_name = canonical.to_string();
            break;
        }
    }

    // Handle plural forms
    if search_name.ends_with("ies") && search_name.len() > 3 {
        // flies -> fly, etc.
        let singular = format!("{}y", &search_name[..search_name.len() - 3]);
        if let Some(idx) = find_monster_by_name(&singular, MONSTERS) {
            return Some(idx);
        }
    }
    if search_name.ends_with("ves") && search_name.len() > 3 {
        // wolves -> wolf, etc.
        let singular = format!("{}f", &search_name[..search_name.len() - 3]);
        if let Some(idx) = find_monster_by_name(&singular, MONSTERS) {
            return Some(idx);
        }
    }
    if search_name.ends_with("es") && search_name.len() > 2 {
        let singular = &search_name[..search_name.len() - 2];
        if let Some(idx) = find_monster_by_name(singular, MONSTERS) {
            return Some(idx);
        }
    }
    if search_name.ends_with('s') && search_name.len() > 1 {
        let singular = &search_name[..search_name.len() - 1];
        if let Some(idx) = find_monster_by_name(singular, MONSTERS) {
            return Some(idx);
        }
    }

    // Direct match
    find_monster_by_name(&search_name, MONSTERS)
}

/// Helper to find monster by name in the MONSTERS array
fn find_monster_by_name(name: &str, monsters: &[PerMonst]) -> Option<i16> {
    let name_lower = name.to_lowercase();
    let mut best_match: Option<(i16, usize)> = None;

    for (i, mon) in monsters.iter().enumerate() {
        let mon_name = mon.name.to_lowercase();
        if mon_name == name_lower {
            // Exact match
            return Some(i as i16);
        }
        // Check if name is a prefix (for handling "corpse" suffix etc.)
        if name_lower.starts_with(&mon_name) {
            let len = mon_name.len();
            if best_match.is_none() || len > best_match.unwrap().1 {
                best_match = Some((i as i16, len));
            }
        }
    }

    best_match.map(|(idx, _)| idx)
}

/// Get monster class from name (name_to_monclass from mondata.c)
///
/// Returns the monster class for a given name string.
/// First checks single-character input against class symbols,
/// then searches class descriptions and individual monster names.
pub fn name_to_monclass(name: &str) -> Option<MonsterClass> {
    if name.is_empty() {
        return None;
    }

    // Single character - check against class symbols
    if name.len() == 1 {
        let c = name.chars().next()?;
        return def_char_to_monclass(c);
    }

    // Multi-character - check class descriptions
    let name_lower = name.to_lowercase();

    let class_names: &[(&str, MonsterClass)] = &[
        ("ant", MonsterClass::Ant),
        ("blob", MonsterClass::Blob),
        ("cockatrice", MonsterClass::Cockatrice),
        ("dog", MonsterClass::Dog),
        ("canine", MonsterClass::Dog),
        ("eye", MonsterClass::Eye),
        ("sphere", MonsterClass::Eye),
        ("feline", MonsterClass::Feline),
        ("cat", MonsterClass::Feline),
        ("gremlin", MonsterClass::Gremlin),
        ("gargoyle", MonsterClass::Gremlin),
        ("humanoid", MonsterClass::Humanoid),
        ("imp", MonsterClass::Imp),
        ("minor demon", MonsterClass::Imp),
        ("jelly", MonsterClass::Jelly),
        ("kobold", MonsterClass::Kobold),
        ("leprechaun", MonsterClass::Leprechaun),
        ("mimic", MonsterClass::Mimic),
        ("nymph", MonsterClass::Nymph),
        ("orc", MonsterClass::Orc),
        ("piercer", MonsterClass::Piercer),
        ("quadruped", MonsterClass::Quadruped),
        ("rodent", MonsterClass::Rodent),
        ("spider", MonsterClass::Spider),
        ("arachnid", MonsterClass::Spider),
        ("trapper", MonsterClass::Trapper),
        ("lurker", MonsterClass::Trapper),
        ("unicorn", MonsterClass::Unicorn),
        ("horse", MonsterClass::Unicorn),
        ("vortex", MonsterClass::Vortex),
        ("worm", MonsterClass::Worm),
        ("xan", MonsterClass::Xan),
        ("light", MonsterClass::Light),
        ("zruty", MonsterClass::Zruty),
        ("angel", MonsterClass::Angel),
        ("bat", MonsterClass::Bat),
        ("centaur", MonsterClass::Centaur),
        ("dragon", MonsterClass::Dragon),
        ("elemental", MonsterClass::Elemental),
        ("fungus", MonsterClass::Fungus),
        ("mold", MonsterClass::Fungus),
        ("gnome", MonsterClass::Gnome),
        ("giant", MonsterClass::Giant),
        ("jabberwock", MonsterClass::Jabberwock),
        ("kop", MonsterClass::Kop),
        ("keystone", MonsterClass::Kop),
        ("lich", MonsterClass::Lich),
        ("mummy", MonsterClass::Mummy),
        ("naga", MonsterClass::Naga),
        ("ogre", MonsterClass::Ogre),
        ("pudding", MonsterClass::Pudding),
        ("ooze", MonsterClass::Pudding),
        ("quantum", MonsterClass::Quantum),
        ("rust monster", MonsterClass::RustMonster),
        ("disenchanter", MonsterClass::RustMonster),
        ("snake", MonsterClass::Snake),
        ("troll", MonsterClass::Troll),
        ("umber hulk", MonsterClass::UmberHulk),
        ("vampire", MonsterClass::Vampire),
        ("wraith", MonsterClass::Wraith),
        ("xorn", MonsterClass::Xorn),
        ("ape", MonsterClass::Ape),
        ("yeti", MonsterClass::Ape),
        ("zombie", MonsterClass::Zombie),
        ("human", MonsterClass::Human),
        ("ghost", MonsterClass::Ghost),
        ("golem", MonsterClass::Golem),
        ("demon", MonsterClass::Demon),
        ("eel", MonsterClass::Eel),
        ("lizard", MonsterClass::Lizard),
    ];

    for &(class_name, class) in class_names {
        if name_lower.contains(class_name) {
            return Some(class);
        }
    }

    // Try to find by individual monster name
    if let Some(idx) = name_to_mon(name) {
        if let Some(mon) = crate::data::get_monster(idx) {
            return def_char_to_monclass(mon.symbol);
        }
    }

    None
}

/// Check if two monster types are the same race (same_race from mondata.c)
///
/// Determines whether two monster types belong to the same species/race.
/// This is used for cannibalism checks, polymorph restrictions, etc.
pub fn same_race(pm1: &PerMonst, pm2: &PerMonst) -> bool {
    // Same monster type
    if core::ptr::eq(pm1, pm2) {
        return true;
    }

    // Check player races
    if pm1.is_human() {
        return pm2.is_human();
    }
    if pm1.is_elf() {
        return pm2.is_elf();
    }
    if pm1.is_dwarf() {
        return pm2.is_dwarf();
    }
    if pm1.is_gnome() {
        return pm2.is_gnome();
    }
    if pm1.is_orc() {
        return pm2.is_orc();
    }

    // Other creature types
    if pm1.is_giant() {
        return pm2.is_giant();
    }
    if pm1.symbol == '\'' {
        // Golems
        return pm2.symbol == '\'';
    }
    if pm1.is_demon() {
        return pm2.is_demon();
    }

    // Same symbol class often means same race
    if pm1.symbol == pm2.symbol {
        return true;
    }

    false
}

/// Get the genus (base type) for a monster (genus from mondata.c)
///
/// Returns the monster type index of the "base" form for special handling.
/// For most monsters, returns the input. For special cases like
/// player monsters, returns the base race monster.
pub fn genus(mndx: i16, _mode: i32) -> i16 {
    // For now, just return the input - full implementation would
    // handle special cases like player monster forms
    mndx
}

/// Check if monster is a valid vampire form (validvamp from mondata.c)
pub fn validvamp(mndx: i16) -> bool {
    if let Some(mon) = crate::data::get_monster(mndx) {
        // Vampires can turn into bats, wolves, and fog clouds
        mon.symbol == 'B' || mon.symbol == 'd' || mon.symbol == 'v'
    } else {
        false
    }
}

/// Check if monster type is valid for special monster creation
pub fn validspecmon(mndx: i16) -> bool {
    mndx >= 0 && (mndx as usize) < crate::data::num_monsters()
}

/// Check if monster is "green" (affected by green slime)
pub fn green_mon(pm: &PerMonst) -> bool {
    // Green slimes and green dragons are "green"
    pm.name.to_lowercase().contains("green")
}

/// Check if monster is a home elemental for a particular dungeon branch
pub fn is_home_elemental(pm: &PerMonst, _branch: i32) -> bool {
    // Simplified - would check against specific dungeon branches
    pm.symbol == 'E' // Elementals
}

/// Check if monster can propagate (lay eggs, split, etc.)
pub fn propagate(mndx: i16, _birth: bool) -> bool {
    if let Some(mon) = crate::data::get_monster(mndx) {
        mon.is_oviparous() || mon.name.contains("pudding")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_def_char_to_monclass() {
        assert_eq!(def_char_to_monclass('a'), Some(MonsterClass::Ant));
        assert_eq!(def_char_to_monclass('D'), Some(MonsterClass::Dragon));
        assert_eq!(def_char_to_monclass('@'), Some(MonsterClass::Human));
        assert_eq!(def_char_to_monclass('&'), Some(MonsterClass::Demon));
        assert_eq!(def_char_to_monclass('?'), None);
    }

    #[test]
    fn test_monster_class_symbol() {
        assert_eq!(MonsterClass::Ant.symbol(), 'a');
        assert_eq!(MonsterClass::Dragon.symbol(), 'D');
        assert_eq!(MonsterClass::Human.symbol(), '@');
    }

    #[test]
    fn test_little_to_big() {
        // Test that little_to_big returns same value for non-growth monsters
        assert_eq!(little_to_big(0), 0); // giant ant has no growth
    }

    #[test]
    fn test_big_to_little() {
        // Test that big_to_little returns same value for non-shrink monsters
        assert_eq!(big_to_little(0), 0); // giant ant has no smaller form
    }

    #[test]
    fn test_big_little_match_same() {
        assert!(big_little_match(5, 5));
    }

    #[test]
    fn test_name_to_monclass_single_char() {
        assert_eq!(name_to_monclass("a"), Some(MonsterClass::Ant));
        assert_eq!(name_to_monclass("D"), Some(MonsterClass::Dragon));
    }

    #[test]
    fn test_name_to_monclass_word() {
        assert_eq!(name_to_monclass("dragon"), Some(MonsterClass::Dragon));
        assert_eq!(name_to_monclass("ant"), Some(MonsterClass::Ant));
        assert_eq!(name_to_monclass("demon"), Some(MonsterClass::Demon));
    }

    #[test]
    fn test_validspecmon() {
        assert!(validspecmon(0));
        assert!(validspecmon(10));
        assert!(!validspecmon(-1));
    }
}
