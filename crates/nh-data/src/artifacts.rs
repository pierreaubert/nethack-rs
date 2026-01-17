//! Artifact definitions (artilist.h)
//!
//! All artifacts from NetHack 3.6.7

use nh_core::combat::{Attack, AttackType, DamageType};

use crate::colors::*;
use crate::objects::ObjectType;

/// Special property flags for artifacts (from artifact.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ArtifactFlags(u32);

impl ArtifactFlags {
    pub const NONE: Self = Self(0x00000000);
    /// Item is special, bequeathed by gods
    pub const NOGEN: Self = Self(0x00000001);
    /// Item is restricted - can't be named
    pub const RESTR: Self = Self(0x00000002);
    /// Item is self-willed - intelligent
    pub const INTEL: Self = Self(0x00000004);
    /// Item can speak (not implemented)
    pub const SPEAK: Self = Self(0x00000008);
    /// Item helps you search for things
    pub const SEEK: Self = Self(0x00000010);
    /// Item warns you of danger
    pub const WARN: Self = Self(0x00000020);
    /// Item has a special attack (attk)
    pub const ATTK: Self = Self(0x00000040);
    /// Item has a special defence (defn)
    pub const DEFN: Self = Self(0x00000080);
    /// Drains a level from monsters
    pub const DRLI: Self = Self(0x00000100);
    /// Helps searching
    pub const SEARCH: Self = Self(0x00000200);
    /// Beheads monsters
    pub const BEHEAD: Self = Self(0x00000400);
    /// Blocks hallucinations
    pub const HALRES: Self = Self(0x00000800);
    /// ESP (like amulet of ESP)
    pub const ESP: Self = Self(0x00001000);
    /// Stealth
    pub const STLTH: Self = Self(0x00002000);
    /// Regeneration
    pub const REGEN: Self = Self(0x00004000);
    /// Energy Regeneration
    pub const EREGEN: Self = Self(0x00008000);
    /// 1/2 spell damage (on player) in combat
    pub const HSPDAM: Self = Self(0x00010000);
    /// 1/2 physical damage (on player) in combat
    pub const HPHDAM: Self = Self(0x00020000);
    /// Teleportation Control
    pub const TCTRL: Self = Self(0x00040000);
    /// Increase Luck (like Luckstone)
    pub const LUCK: Self = Self(0x00080000);
    /// Attack bonus on one monster type
    pub const DMONS: Self = Self(0x00100000);
    /// Attack bonus on monsters w/ symbol mtype
    pub const DCLAS: Self = Self(0x00200000);
    /// Attack bonus on monsters w/ mflags1 flag
    pub const DFLAG1: Self = Self(0x00400000);
    /// Attack bonus on monsters w/ mflags2 flag
    pub const DFLAG2: Self = Self(0x00800000);
    /// Attack bonus on non-aligned monsters
    pub const DALIGN: Self = Self(0x01000000);
    /// Attack bonus mask
    pub const DBONUS: Self = Self(0x01F00000);
    /// Gives X-RAY vision to player
    pub const XRAY: Self = Self(0x02000000);
    /// Reflection
    pub const REFLECT: Self = Self(0x04000000);
    /// Protection
    pub const PROTECT: Self = Self(0x08000000);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Alignment type for artifacts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    None,
    Lawful,
    Neutral,
    Chaotic,
}

/// Invocation property types (from artifact.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvokeProperty {
    None,
    Taming,
    Healing,
    EnergyBoost,
    Untrap,
    ChargeObj,
    LevTele,
    CreatePortal,
    Enlightening,
    CreateAmmo,
    Invis,
    Levitation,
    Conflict,
}

/// Role/class constants (PM_* from pm.h)
/// -1 means "no specific role" (NON_PM)
pub const NON_PM: i16 = -1;
pub const PM_ARCHEOLOGIST: i16 = 0;
pub const PM_BARBARIAN: i16 = 1;
pub const PM_CAVEMAN: i16 = 2;
pub const PM_HEALER: i16 = 3;
pub const PM_KNIGHT: i16 = 4;
pub const PM_MONK: i16 = 5;
pub const PM_PRIEST: i16 = 6;
pub const PM_RANGER: i16 = 7;
pub const PM_ROGUE: i16 = 8;
pub const PM_SAMURAI: i16 = 9;
pub const PM_TOURIST: i16 = 10;
pub const PM_VALKYRIE: i16 = 11;
pub const PM_WIZARD: i16 = 12;

/// Race constants
pub const PM_HUMAN: i16 = 100;
pub const PM_ELF: i16 = 101;
pub const PM_DWARF: i16 = 102;
pub const PM_GNOME: i16 = 103;
pub const PM_ORC: i16 = 104;

/// Monster class symbols for DCLAS targeting
pub const S_DRAGON: char = 'D';
pub const S_OGRE: char = 'O';
pub const S_TROLL: char = 'T';

/// M2 monster flags for DFLAG2 targeting
pub const M2_ELF: u32 = 0x0010;
pub const M2_ORC: u32 = 0x0020;
pub const M2_DEMON: u32 = 0x0040;
pub const M2_WERE: u32 = 0x0004;
pub const M2_UNDEAD: u32 = 0x0002;
pub const M2_GIANT: u32 = 0x0080;

/// An artifact definition
#[derive(Debug, Clone)]
pub struct Artifact {
    /// Name of the artifact
    pub name: &'static str,
    /// Base object type
    pub otyp: ObjectType,
    /// Special effects when wielded/worn
    pub spfx: ArtifactFlags,
    /// Special effects just from carrying
    pub cspfx: ArtifactFlags,
    /// Monster type, symbol, or flag for targeting
    pub mtype: u32,
    /// Special attack when hitting
    pub attk: Attack,
    /// Passive defense effect
    pub defn: Attack,
    /// Effect from carrying
    pub cary: Attack,
    /// Property obtained by invoking artifact
    pub inv_prop: InvokeProperty,
    /// Alignment of bequeathing gods
    pub alignment: Alignment,
    /// Character role associated with
    pub role: i16,
    /// Character race associated with
    pub race: i16,
    /// Price when sold to hero
    pub cost: u32,
    /// Color to use if artifact 'glows'
    pub color: u8,
}

/// No attack
const NO_ATTK: Attack = Attack::new(AttackType::None, DamageType::Physical, 0, 0);

/// Physical damage attack
const fn phys(dice: u8, sides: u8) -> Attack {
    Attack::new(AttackType::None, DamageType::Physical, dice, sides)
}

/// Life drain attack
const fn drli(dice: u8, sides: u8) -> Attack {
    Attack::new(AttackType::None, DamageType::DrainLife, dice, sides)
}

/// Cold damage attack
const fn cold(dice: u8, sides: u8) -> Attack {
    Attack::new(AttackType::None, DamageType::Cold, dice, sides)
}

/// Fire damage attack
const fn fire(dice: u8, sides: u8) -> Attack {
    Attack::new(AttackType::None, DamageType::Fire, dice, sides)
}

/// Electrical damage attack
const fn elec(dice: u8, sides: u8) -> Attack {
    Attack::new(AttackType::None, DamageType::Electric, dice, sides)
}

/// Stun attack
const fn stun(dice: u8, sides: u8) -> Attack {
    Attack::new(AttackType::None, DamageType::Stun, dice, sides)
}

/// Defense against damage type
const fn dfns(dt: DamageType) -> Attack {
    Attack::new(AttackType::None, dt, 0, 0)
}

/// Carry effect
const fn cary(dt: DamageType) -> Attack {
    Attack::new(AttackType::None, dt, 0, 0)
}

/// Helper to combine multiple flags
const fn spfx(flags: &[ArtifactFlags]) -> ArtifactFlags {
    let mut result = 0u32;
    let mut i = 0;
    while i < flags.len() {
        result |= flags[i].bits();
        i += 1;
    }
    ArtifactFlags::from_bits(result)
}

/// All artifacts in the game
pub static ARTIFACTS: &[Artifact] = &[
    // Excalibur - The sword of King Arthur
    Artifact {
        name: "Excalibur",
        otyp: ObjectType::LongSword,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::SEEK,
            ArtifactFlags::DEFN,
            ArtifactFlags::INTEL,
            ArtifactFlags::SEARCH,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: phys(5, 10),
        defn: drli(0, 0),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Lawful,
        role: PM_KNIGHT,
        race: NON_PM,
        cost: 4000,
        color: NO_COLOR,
    },
    // Stormbringer - The black blade that drinks souls
    Artifact {
        name: "Stormbringer",
        otyp: ObjectType::Runesword,
        spfx: spfx(&[
            ArtifactFlags::RESTR,
            ArtifactFlags::ATTK,
            ArtifactFlags::DEFN,
            ArtifactFlags::INTEL,
            ArtifactFlags::DRLI,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: drli(5, 2),
        defn: drli(0, 0),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Chaotic,
        role: NON_PM,
        race: NON_PM,
        cost: 8000,
        color: NO_COLOR,
    },
    // Mjollnir - Thor's hammer
    Artifact {
        name: "Mjollnir",
        otyp: ObjectType::WarHammer,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::ATTK]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: elec(5, 24),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Neutral,
        role: PM_VALKYRIE,
        race: NON_PM,
        cost: 4000,
        color: NO_COLOR,
    },
    // Cleaver - Barbarian's axe
    Artifact {
        name: "Cleaver",
        otyp: ObjectType::BattleAxe,
        spfx: ArtifactFlags::RESTR,
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: phys(3, 6),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Neutral,
        role: PM_BARBARIAN,
        race: NON_PM,
        cost: 1500,
        color: NO_COLOR,
    },
    // Grimtooth - Orcish dagger that warns of elves
    Artifact {
        name: "Grimtooth",
        otyp: ObjectType::OrcishDagger,
        spfx: spfx(&[
            ArtifactFlags::RESTR,
            ArtifactFlags::WARN,
            ArtifactFlags::DFLAG2,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_ELF,
        attk: phys(2, 6),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Chaotic,
        role: NON_PM,
        race: PM_ORC,
        cost: 300,
        color: CLR_RED,
    },
    // Orcrist - Elven sword that warns of orcs
    Artifact {
        name: "Orcrist",
        otyp: ObjectType::ElvenBroadSword,
        spfx: spfx(&[ArtifactFlags::WARN, ArtifactFlags::DFLAG2]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_ORC,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Chaotic,
        role: NON_PM,
        race: PM_ELF,
        cost: 2000,
        color: CLR_BRIGHT_BLUE,
    },
    // Sting - Bilbo's dagger
    Artifact {
        name: "Sting",
        otyp: ObjectType::ElvenDagger,
        spfx: spfx(&[ArtifactFlags::WARN, ArtifactFlags::DFLAG2]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_ORC,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Chaotic,
        role: NON_PM,
        race: PM_ELF,
        cost: 800,
        color: CLR_BRIGHT_BLUE,
    },
    // Magicbane - Wizard's athame
    Artifact {
        name: "Magicbane",
        otyp: ObjectType::Athame,
        spfx: spfx(&[
            ArtifactFlags::RESTR,
            ArtifactFlags::ATTK,
            ArtifactFlags::DEFN,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: stun(3, 4),
        defn: dfns(DamageType::MagicMissile),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Neutral,
        role: PM_WIZARD,
        race: NON_PM,
        cost: 3500,
        color: NO_COLOR,
    },
    // Frost Brand
    Artifact {
        name: "Frost Brand",
        otyp: ObjectType::LongSword,
        spfx: spfx(&[
            ArtifactFlags::RESTR,
            ArtifactFlags::ATTK,
            ArtifactFlags::DEFN,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: cold(5, 0),
        defn: cold(0, 0),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::None,
        role: NON_PM,
        race: NON_PM,
        cost: 3000,
        color: NO_COLOR,
    },
    // Fire Brand
    Artifact {
        name: "Fire Brand",
        otyp: ObjectType::LongSword,
        spfx: spfx(&[
            ArtifactFlags::RESTR,
            ArtifactFlags::ATTK,
            ArtifactFlags::DEFN,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: fire(5, 0),
        defn: fire(0, 0),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::None,
        role: NON_PM,
        race: NON_PM,
        cost: 3000,
        color: NO_COLOR,
    },
    // Dragonbane
    Artifact {
        name: "Dragonbane",
        otyp: ObjectType::BroadSword,
        spfx: spfx(&[
            ArtifactFlags::RESTR,
            ArtifactFlags::DCLAS,
            ArtifactFlags::REFLECT,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: S_DRAGON as u32,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::None,
        role: NON_PM,
        race: NON_PM,
        cost: 500,
        color: NO_COLOR,
    },
    // Demonbane
    Artifact {
        name: "Demonbane",
        otyp: ObjectType::LongSword,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::DFLAG2]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_DEMON,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Lawful,
        role: NON_PM,
        race: NON_PM,
        cost: 2500,
        color: NO_COLOR,
    },
    // Werebane
    Artifact {
        name: "Werebane",
        otyp: ObjectType::SilverSaber,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::DFLAG2]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_WERE,
        attk: phys(5, 0),
        defn: dfns(DamageType::Lycanthropy),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::None,
        role: NON_PM,
        race: NON_PM,
        cost: 1500,
        color: NO_COLOR,
    },
    // Grayswandir
    Artifact {
        name: "Grayswandir",
        otyp: ObjectType::SilverSaber,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::HALRES]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Lawful,
        role: NON_PM,
        race: NON_PM,
        cost: 8000,
        color: NO_COLOR,
    },
    // Giantslayer
    Artifact {
        name: "Giantslayer",
        otyp: ObjectType::LongSword,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::DFLAG2]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_GIANT,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Neutral,
        role: NON_PM,
        race: NON_PM,
        cost: 200,
        color: NO_COLOR,
    },
    // Ogresmasher
    Artifact {
        name: "Ogresmasher",
        otyp: ObjectType::WarHammer,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::DCLAS]),
        cspfx: ArtifactFlags::NONE,
        mtype: S_OGRE as u32,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::None,
        role: NON_PM,
        race: NON_PM,
        cost: 200,
        color: NO_COLOR,
    },
    // Trollsbane
    Artifact {
        name: "Trollsbane",
        otyp: ObjectType::MorningStar,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::DCLAS]),
        cspfx: ArtifactFlags::NONE,
        mtype: S_TROLL as u32,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::None,
        role: NON_PM,
        race: NON_PM,
        cost: 200,
        color: NO_COLOR,
    },
    // Vorpal Blade
    Artifact {
        name: "Vorpal Blade",
        otyp: ObjectType::LongSword,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::BEHEAD]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: phys(5, 1),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Neutral,
        role: NON_PM,
        race: NON_PM,
        cost: 4000,
        color: NO_COLOR,
    },
    // Snickersnee
    Artifact {
        name: "Snickersnee",
        otyp: ObjectType::Katana,
        spfx: ArtifactFlags::RESTR,
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: phys(0, 8),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Lawful,
        role: PM_SAMURAI,
        race: NON_PM,
        cost: 1200,
        color: NO_COLOR,
    },
    // Sunsword
    Artifact {
        name: "Sunsword",
        otyp: ObjectType::LongSword,
        spfx: spfx(&[ArtifactFlags::RESTR, ArtifactFlags::DFLAG2]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_UNDEAD,
        attk: phys(5, 0),
        defn: dfns(DamageType::Blind),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Lawful,
        role: NON_PM,
        race: NON_PM,
        cost: 1500,
        color: NO_COLOR,
    },
    // ==================== QUEST ARTIFACTS ====================
    // The Orb of Detection - Archeologist's quest artifact
    Artifact {
        name: "The Orb of Detection",
        otyp: ObjectType::CrystalBall,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
        ]),
        cspfx: spfx(&[ArtifactFlags::ESP, ArtifactFlags::HSPDAM]),
        mtype: 0,
        attk: NO_ATTK,
        defn: NO_ATTK,
        cary: cary(DamageType::MagicMissile),
        inv_prop: InvokeProperty::Invis,
        alignment: Alignment::Lawful,
        role: PM_ARCHEOLOGIST,
        race: NON_PM,
        cost: 2500,
        color: NO_COLOR,
    },
    // The Heart of Ahriman - Barbarian's quest artifact
    Artifact {
        name: "The Heart of Ahriman",
        otyp: ObjectType::Luckstone,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
        ]),
        cspfx: ArtifactFlags::STLTH,
        mtype: 0,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::Levitation,
        alignment: Alignment::Neutral,
        role: PM_BARBARIAN,
        race: NON_PM,
        cost: 2500,
        color: NO_COLOR,
    },
    // The Sceptre of Might - Caveman's quest artifact
    Artifact {
        name: "The Sceptre of Might",
        otyp: ObjectType::Mace,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::DALIGN,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: phys(5, 0),
        defn: dfns(DamageType::MagicMissile),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::Conflict,
        alignment: Alignment::Lawful,
        role: PM_CAVEMAN,
        race: NON_PM,
        cost: 2500,
        color: NO_COLOR,
    },
    // The Staff of Aesculapius - Healer's quest artifact
    Artifact {
        name: "The Staff of Aesculapius",
        otyp: ObjectType::Quarterstaff,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::ATTK,
            ArtifactFlags::INTEL,
            ArtifactFlags::DRLI,
            ArtifactFlags::REGEN,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: drli(0, 0),
        defn: drli(0, 0),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::Healing,
        alignment: Alignment::Neutral,
        role: PM_HEALER,
        race: NON_PM,
        cost: 5000,
        color: NO_COLOR,
    },
    // The Magic Mirror of Merlin - Knight's quest artifact
    Artifact {
        name: "The Magic Mirror of Merlin",
        otyp: ObjectType::Mirror,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::SPEAK,
        ]),
        cspfx: ArtifactFlags::ESP,
        mtype: 0,
        attk: NO_ATTK,
        defn: NO_ATTK,
        cary: cary(DamageType::MagicMissile),
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Lawful,
        role: PM_KNIGHT,
        race: NON_PM,
        cost: 1500,
        color: NO_COLOR,
    },
    // The Eyes of the Overworld - Monk's quest artifact
    Artifact {
        name: "The Eyes of the Overworld",
        otyp: ObjectType::Lenses,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::XRAY,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: NO_ATTK,
        defn: dfns(DamageType::MagicMissile),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::Enlightening,
        alignment: Alignment::Neutral,
        role: PM_MONK,
        race: NON_PM,
        cost: 2500,
        color: NO_COLOR,
    },
    // The Mitre of Holiness - Priest's quest artifact
    Artifact {
        name: "The Mitre of Holiness",
        otyp: ObjectType::HelmOfBrilliancE,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::DFLAG2,
            ArtifactFlags::INTEL,
            ArtifactFlags::PROTECT,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: M2_UNDEAD,
        attk: NO_ATTK,
        defn: NO_ATTK,
        cary: cary(DamageType::Fire),
        inv_prop: InvokeProperty::EnergyBoost,
        alignment: Alignment::Lawful,
        role: PM_PRIEST,
        race: NON_PM,
        cost: 2000,
        color: NO_COLOR,
    },
    // The Longbow of Diana - Ranger's quest artifact
    Artifact {
        name: "The Longbow of Diana",
        otyp: ObjectType::Arrow, // Should be Bow - we'll use Arrow as placeholder
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::REFLECT,
        ]),
        cspfx: ArtifactFlags::ESP,
        mtype: 0,
        attk: phys(5, 0),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::CreateAmmo,
        alignment: Alignment::Chaotic,
        role: PM_RANGER,
        race: NON_PM,
        cost: 4000,
        color: NO_COLOR,
    },
    // The Palantir of Westernesse - Ranger's alternate quest artifact (for elves)
    Artifact {
        name: "The Palantir of Westernesse",
        otyp: ObjectType::CrystalBall,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
        ]),
        cspfx: spfx(&[
            ArtifactFlags::ESP,
            ArtifactFlags::REGEN,
            ArtifactFlags::HSPDAM,
        ]),
        mtype: 0,
        attk: NO_ATTK,
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::Taming,
        alignment: Alignment::Chaotic,
        role: NON_PM,
        race: PM_ELF,
        cost: 8000,
        color: NO_COLOR,
    },
    // The Master Key of Thievery - Rogue's quest artifact
    Artifact {
        name: "The Master Key of Thievery",
        otyp: ObjectType::SkeletonKey,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::SPEAK,
        ]),
        cspfx: spfx(&[
            ArtifactFlags::WARN,
            ArtifactFlags::TCTRL,
            ArtifactFlags::HPHDAM,
        ]),
        mtype: 0,
        attk: NO_ATTK,
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::Untrap,
        alignment: Alignment::Chaotic,
        role: PM_ROGUE,
        race: NON_PM,
        cost: 3500,
        color: NO_COLOR,
    },
    // The Tsurugi of Muramasa - Samurai's quest artifact
    Artifact {
        name: "The Tsurugi of Muramasa",
        otyp: ObjectType::Tsurugi,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::BEHEAD,
            ArtifactFlags::LUCK,
            ArtifactFlags::PROTECT,
        ]),
        cspfx: ArtifactFlags::NONE,
        mtype: 0,
        attk: phys(0, 8),
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::None,
        alignment: Alignment::Lawful,
        role: PM_SAMURAI,
        race: NON_PM,
        cost: 4500,
        color: NO_COLOR,
    },
    // The Platinum Yendorian Express Card - Tourist's quest artifact
    Artifact {
        name: "The Platinum Yendorian Express Card",
        otyp: ObjectType::CreditCard,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::DEFN,
        ]),
        cspfx: spfx(&[ArtifactFlags::ESP, ArtifactFlags::HSPDAM]),
        mtype: 0,
        attk: NO_ATTK,
        defn: NO_ATTK,
        cary: cary(DamageType::MagicMissile),
        inv_prop: InvokeProperty::ChargeObj,
        alignment: Alignment::Neutral,
        role: PM_TOURIST,
        race: NON_PM,
        cost: 7000,
        color: NO_COLOR,
    },
    // The Orb of Fate - Valkyrie's quest artifact
    Artifact {
        name: "The Orb of Fate",
        otyp: ObjectType::CrystalBall,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
            ArtifactFlags::LUCK,
        ]),
        cspfx: spfx(&[
            ArtifactFlags::WARN,
            ArtifactFlags::HSPDAM,
            ArtifactFlags::HPHDAM,
        ]),
        mtype: 0,
        attk: NO_ATTK,
        defn: NO_ATTK,
        cary: NO_ATTK,
        inv_prop: InvokeProperty::LevTele,
        alignment: Alignment::Neutral,
        role: PM_VALKYRIE,
        race: NON_PM,
        cost: 3500,
        color: NO_COLOR,
    },
    // The Eye of the Aethiopica - Wizard's quest artifact
    Artifact {
        name: "The Eye of the Aethiopica",
        otyp: ObjectType::AmuletOfEsp,
        spfx: spfx(&[
            ArtifactFlags::NOGEN,
            ArtifactFlags::RESTR,
            ArtifactFlags::INTEL,
        ]),
        cspfx: spfx(&[ArtifactFlags::EREGEN, ArtifactFlags::HSPDAM]),
        mtype: 0,
        attk: NO_ATTK,
        defn: dfns(DamageType::MagicMissile),
        cary: NO_ATTK,
        inv_prop: InvokeProperty::CreatePortal,
        alignment: Alignment::Neutral,
        role: PM_WIZARD,
        race: NON_PM,
        cost: 4000,
        color: NO_COLOR,
    },
];

/// Get an artifact by name
pub fn get_artifact(name: &str) -> Option<&'static Artifact> {
    ARTIFACTS.iter().find(|a| a.name == name)
}

/// Get an artifact by index
pub fn get_artifact_by_index(index: usize) -> Option<&'static Artifact> {
    ARTIFACTS.get(index)
}

/// Get the number of artifacts
pub fn num_artifacts() -> usize {
    ARTIFACTS.len()
}

/// Check if an object type has an associated artifact
pub fn is_artifact_base(otyp: ObjectType) -> bool {
    ARTIFACTS.iter().any(|a| a.otyp == otyp)
}

/// Get the artifact associated with a role (quest artifact)
pub fn get_quest_artifact(role: i16) -> Option<&'static Artifact> {
    ARTIFACTS.iter().find(|a| {
        a.role == role && a.spfx.contains(ArtifactFlags::NOGEN) && a.spfx.contains(ArtifactFlags::INTEL)
    })
}
