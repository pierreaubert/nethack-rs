//! Role and race definitions (role.c, u_init.c)
//!
//! Player roles (classes) and races with their starting equipment

use crate::artifacts::{Alignment, NON_PM, PM_HUMAN, PM_ELF, PM_DWARF, PM_GNOME, PM_ORC};
use crate::objects::ObjectType;

/// Stat indices
pub const A_STR: usize = 0;
pub const A_INT: usize = 1;
pub const A_WIS: usize = 2;
pub const A_DEX: usize = 3;
pub const A_CON: usize = 4;
pub const A_CHA: usize = 5;
pub const A_MAX: usize = 6;

/// Race mask bits
pub const MH_HUMAN: u16 = 0x0001;
pub const MH_ELF: u16 = 0x0002;
pub const MH_DWARF: u16 = 0x0004;
pub const MH_GNOME: u16 = 0x0008;
pub const MH_ORC: u16 = 0x0010;

/// Gender mask bits
pub const ROLE_MALE: u16 = 0x1000;
pub const ROLE_FEMALE: u16 = 0x2000;

/// Alignment mask bits
pub const ROLE_LAWFUL: u16 = 0x0001;
pub const ROLE_NEUTRAL: u16 = 0x0002;
pub const ROLE_CHAOTIC: u16 = 0x0004;

/// Role name with optional male/female variants
#[derive(Debug, Clone)]
pub struct RoleName {
    /// Male name (or common name if female is None)
    pub male: &'static str,
    /// Female name (None if same as male)
    pub female: Option<&'static str>,
}

impl RoleName {
    pub const fn new(male: &'static str, female: Option<&'static str>) -> Self {
        Self { male, female }
    }

    pub fn get(&self, is_female: bool) -> &'static str {
        if is_female {
            self.female.unwrap_or(self.male)
        } else {
            self.male
        }
    }
}

/// Advancement stats for HP and energy per level
#[derive(Debug, Clone, Copy)]
pub struct Advancement {
    /// Fixed amount at initialization
    pub init_fix: i8,
    /// Random amount at initialization
    pub init_rnd: i8,
    /// Fixed amount per level below xlev
    pub low_fix: i8,
    /// Random amount per level below xlev
    pub low_rnd: i8,
    /// Fixed amount per level at/above xlev
    pub high_fix: i8,
    /// Random amount per level at/above xlev
    pub high_rnd: i8,
}

impl Advancement {
    pub const fn new(
        init_fix: i8,
        init_rnd: i8,
        low_fix: i8,
        low_rnd: i8,
        high_fix: i8,
        high_rnd: i8,
    ) -> Self {
        Self {
            init_fix,
            init_rnd,
            low_fix,
            low_rnd,
            high_fix,
            high_rnd,
        }
    }
}

/// Starting inventory item
#[derive(Debug, Clone, Copy)]
pub struct StartingItem {
    /// Object type (None for random of class)
    pub otyp: Option<ObjectType>,
    /// Enchantment/charge (-128 for undefined/random)
    pub spe: i8,
    /// Quantity
    pub quantity: u8,
    /// Blessed status (0=uncursed, 1=blessed, 2=random)
    pub bless: u8,
}

impl StartingItem {
    pub const fn new(otyp: Option<ObjectType>, spe: i8, quantity: u8, bless: u8) -> Self {
        Self {
            otyp,
            spe,
            quantity,
            bless,
        }
    }

    pub const fn item(otyp: ObjectType, spe: i8, quantity: u8, bless: u8) -> Self {
        Self::new(Some(otyp), spe, quantity, bless)
    }
}

/// Special enchantment value meaning "undefined/random"
pub const UNDEF_SPE: i8 = -128;
/// Bless status: random
pub const UNDEF_BLESS: u8 = 2;

/// A player role definition
#[derive(Debug, Clone)]
pub struct Role {
    /// Role name (male/female variants)
    pub name: RoleName,
    /// Experience level titles (9 ranks)
    pub ranks: [RoleName; 9],
    /// God names: lawful, neutral, chaotic
    pub gods: (&'static str, &'static str, &'static str),
    /// File code (3-letter abbreviation)
    pub filecode: &'static str,
    /// Quest home location
    pub homebase: &'static str,
    /// Quest intermediate goal
    pub intermed: &'static str,
    /// Monster index as male
    pub malenum: i16,
    /// Monster index as female (NON_PM if same)
    pub femalenum: i16,
    /// Preferred pet monster (NON_PM for random)
    pub petnum: i16,
    /// Quest leader monster
    pub ldrnum: i16,
    /// Quest guardian monster
    pub guardnum: i16,
    /// Quest nemesis monster
    pub neminum: i16,
    /// Quest artifact index
    pub questarti: &'static str,
    /// Allowed races/genders/alignments bitmask
    pub allow: u16,
    /// Base attributes [Str, Int, Wis, Dex, Con, Cha]
    pub attrbase: [i8; A_MAX],
    /// Attribute distribution for random bonus
    pub attrdist: [i8; A_MAX],
    /// HP advancement
    pub hpadv: Advancement,
    /// Energy advancement
    pub enadv: Advancement,
    /// Level cutoff for advancement formula change
    pub xlev: u8,
    /// Initial alignment record
    pub initrecord: i8,
    /// Base spellcasting penalty
    pub spelbase: i8,
    /// Healing spell penalty
    pub spelheal: i8,
    /// Shield spell penalty
    pub spelshld: i8,
    /// Metal armor spell penalty
    pub spelarmr: i8,
    /// Stat used for spellcasting
    pub spelstat: usize,
    /// Starting equipment
    pub starting_items: &'static [StartingItem],
}

/// A player race definition
#[derive(Debug, Clone)]
pub struct Race {
    /// Race noun ("human", "elf")
    pub noun: &'static str,
    /// Race adjective ("human", "elven")
    pub adj: &'static str,
    /// Collective noun ("humanity", "elvenkind")
    pub collective: &'static str,
    /// File code
    pub filecode: &'static str,
    /// Individual names (male/female)
    pub individual: RoleName,
    /// Monster index as male
    pub malenum: i16,
    /// Monster index as female (NON_PM if same)
    pub femalenum: i16,
    /// Monster index as mummy
    pub mummynum: i16,
    /// Monster index as zombie
    pub zombienum: i16,
    /// Allowed genders/alignments bitmask
    pub allow: u16,
    /// Self race mask
    pub selfmask: u16,
    /// Always peaceful races mask
    pub lovemask: u16,
    /// Always hostile races mask
    pub hatemask: u16,
    /// Minimum attributes
    pub attrmin: [i8; A_MAX],
    /// Maximum attributes
    pub attrmax: [i8; A_MAX],
    /// HP advancement bonus
    pub hpadv: Advancement,
    /// Energy advancement bonus
    pub enadv: Advancement,
}

// ==================== STARTING EQUIPMENT ====================

static ARCHEOLOGIST_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Boomerang, 2, 1, UNDEF_BLESS), // Bullwhip placeholder
    StartingItem::item(ObjectType::LeatherJacket, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::FedoraHat, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::FoodRation, 0, 3, 0),
    StartingItem::item(ObjectType::PickAxe, UNDEF_SPE, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Tinning, UNDEF_SPE, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Touchstone, 0, 1, 0),
    StartingItem::item(ObjectType::Sack, 0, 1, 0),
];

static BARBARIAN_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::TwoHandedSword, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Axe, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::RingMail, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::FoodRation, 0, 1, 0),
];

static CAVEMAN_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Club, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Boomerang, 2, 1, UNDEF_BLESS), // Sling placeholder
    StartingItem::item(ObjectType::Flint, 0, 15, UNDEF_BLESS),
    StartingItem::item(ObjectType::Rock, 0, 3, 0),
    StartingItem::item(ObjectType::LeatherArmor, 0, 1, UNDEF_BLESS),
];

static HEALER_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Scalpel, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::LeatherGloves, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Stethoscope, 0, 1, 0),
    StartingItem::item(ObjectType::PotionOfHealing, 0, 4, UNDEF_BLESS),
    StartingItem::item(ObjectType::PotionOfExtraHealing, 0, 4, UNDEF_BLESS),
    StartingItem::item(ObjectType::WandOfSleep, UNDEF_SPE, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::SpellbookOfHealing, 0, 1, 1),
    StartingItem::item(ObjectType::SpellbookOfExtraHealing, 0, 1, 1),
    StartingItem::item(ObjectType::SpellbookOfStoneSkin, 0, 1, 1), // Stone to flesh placeholder
    StartingItem::item(ObjectType::Apple, 0, 5, 0),
];

static KNIGHT_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::LongSword, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Lance, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::RingMail, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Helmet, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::SmallShield, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::LeatherGloves, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Apple, 0, 10, 0),
    StartingItem::item(ObjectType::Carrot, 0, 10, 0),
];

static MONK_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::LeatherGloves, 2, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Robe, 1, 1, UNDEF_BLESS),
    StartingItem::new(None, UNDEF_SPE, 1, 1), // Random spellbook
    StartingItem::new(None, UNDEF_SPE, 1, UNDEF_BLESS), // Random scroll
    StartingItem::item(ObjectType::PotionOfHealing, 0, 3, UNDEF_BLESS),
    StartingItem::item(ObjectType::FoodRation, 0, 3, 0),
    StartingItem::item(ObjectType::Apple, 0, 5, UNDEF_BLESS),
    StartingItem::item(ObjectType::Orange, 0, 5, UNDEF_BLESS),
];

static PRIEST_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Mace, 1, 1, 1),
    StartingItem::item(ObjectType::Robe, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::SmallShield, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::PotionOfWater, 0, 4, 1), // Holy water
    StartingItem::item(ObjectType::Clove, 0, 1, 0), // Garlic
    StartingItem::item(ObjectType::Sprig, 0, 1, 0), // Wolfsbane
    StartingItem::new(None, UNDEF_SPE, 2, UNDEF_BLESS), // Random spellbooks
];

static RANGER_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Dagger, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Arrow, 1, 1, UNDEF_BLESS), // Bow placeholder
    StartingItem::item(ObjectType::Arrow, 2, 50, UNDEF_BLESS),
    StartingItem::item(ObjectType::Arrow, 0, 30, UNDEF_BLESS),
    StartingItem::item(ObjectType::CloakOfDisplacement, 2, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Cram, 0, 4, 0),
];

static ROGUE_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::ShortSword, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Dagger, 0, 10, 0),
    StartingItem::item(ObjectType::LeatherArmor, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::PotionOfSickness, 0, 1, 0),
    StartingItem::item(ObjectType::LockPick, 0, 1, 0),
    StartingItem::item(ObjectType::Sack, 0, 1, 0),
];

static SAMURAI_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Katana, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::ShortSword, 0, 1, UNDEF_BLESS), // Wakizashi
    StartingItem::item(ObjectType::Arrow, 0, 1, UNDEF_BLESS), // Yumi placeholder
    StartingItem::item(ObjectType::Ya, 0, 25, UNDEF_BLESS),
    StartingItem::item(ObjectType::SplintMail, 0, 1, UNDEF_BLESS),
];

static TOURIST_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Dart, 2, 25, UNDEF_BLESS),
    StartingItem::new(None, UNDEF_SPE, 10, 0), // Random food
    StartingItem::item(ObjectType::PotionOfExtraHealing, 0, 2, UNDEF_BLESS),
    StartingItem::item(ObjectType::ScrollOfMagicMapping, 0, 4, UNDEF_BLESS),
    StartingItem::item(ObjectType::HawaiianShirt, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::ExpensiveCamera, UNDEF_SPE, 1, 0),
    StartingItem::item(ObjectType::CreditCard, 0, 1, 0),
];

static VALKYRIE_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::LongSword, 1, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::Dagger, 0, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::SmallShield, 3, 1, UNDEF_BLESS),
    StartingItem::item(ObjectType::FoodRation, 0, 1, 0),
];

static WIZARD_ITEMS: &[StartingItem] = &[
    StartingItem::item(ObjectType::Quarterstaff, 1, 1, 1),
    StartingItem::item(ObjectType::CloakOfMagicResistance, 0, 1, UNDEF_BLESS),
    StartingItem::new(None, UNDEF_SPE, 1, UNDEF_BLESS), // Random wand
    StartingItem::new(None, UNDEF_SPE, 2, UNDEF_BLESS), // Random rings
    StartingItem::new(None, UNDEF_SPE, 3, UNDEF_BLESS), // Random potions
    StartingItem::new(None, UNDEF_SPE, 3, UNDEF_BLESS), // Random scrolls
    StartingItem::item(ObjectType::SpellbookOfForceBolt, 0, 1, 1),
    StartingItem::new(None, UNDEF_SPE, 1, UNDEF_BLESS), // Random spellbook
];

// ==================== ROLE DEFINITIONS ====================

/// All player roles
pub static ROLES: &[Role] = &[
    // Archeologist
    Role {
        name: RoleName::new("Archeologist", None),
        ranks: [
            RoleName::new("Digger", None),
            RoleName::new("Field Worker", None),
            RoleName::new("Investigator", None),
            RoleName::new("Exhumer", None),
            RoleName::new("Excavator", None),
            RoleName::new("Spelunker", None),
            RoleName::new("Speleologist", None),
            RoleName::new("Collector", None),
            RoleName::new("Curator", None),
        ],
        gods: ("Quetzalcoatl", "Camaxtli", "Huhetotl"),
        filecode: "Arc",
        homebase: "the College of Archeology",
        intermed: "the Tomb of the Toltec Kings",
        malenum: 0, // PM_ARCHEOLOGIST
        femalenum: NON_PM,
        petnum: NON_PM,
        ldrnum: 0, // PM_LORD_CARNARVON
        guardnum: 0, // PM_STUDENT
        neminum: 0, // PM_MINION_OF_HUHETOTL
        questarti: "The Orb of Detection",
        allow: MH_HUMAN | MH_DWARF | MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL,
        attrbase: [7, 10, 10, 7, 7, 7],
        attrdist: [20, 20, 20, 10, 20, 10],
        hpadv: Advancement::new(11, 0, 0, 8, 1, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 14,
        initrecord: 10,
        spelbase: 5,
        spelheal: 0,
        spelshld: 2,
        spelarmr: 10,
        spelstat: A_INT,
        starting_items: ARCHEOLOGIST_ITEMS,
    },
    // Barbarian
    Role {
        name: RoleName::new("Barbarian", None),
        ranks: [
            RoleName::new("Plunderer", Some("Plunderess")),
            RoleName::new("Pillager", None),
            RoleName::new("Bandit", None),
            RoleName::new("Brigand", None),
            RoleName::new("Raider", None),
            RoleName::new("Reaver", None),
            RoleName::new("Slayer", None),
            RoleName::new("Chieftain", Some("Chieftainess")),
            RoleName::new("Conqueror", Some("Conqueress")),
        ],
        gods: ("Mitra", "Crom", "Set"),
        filecode: "Bar",
        homebase: "the Camp of the Duali Tribe",
        intermed: "the Duali Oasis",
        malenum: 1, // PM_BARBARIAN
        femalenum: NON_PM,
        petnum: NON_PM,
        ldrnum: 0, // PM_PELIAS
        guardnum: 0, // PM_CHIEFTAIN
        neminum: 0, // PM_THOTH_AMON
        questarti: "The Heart of Ahriman",
        allow: MH_HUMAN | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL | ROLE_CHAOTIC,
        attrbase: [16, 7, 7, 15, 16, 6],
        attrdist: [30, 6, 7, 20, 30, 7],
        hpadv: Advancement::new(14, 0, 0, 10, 2, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 10,
        initrecord: 10,
        spelbase: 14,
        spelheal: 0,
        spelshld: 0,
        spelarmr: 8,
        spelstat: A_INT,
        starting_items: BARBARIAN_ITEMS,
    },
    // Caveman
    Role {
        name: RoleName::new("Caveman", Some("Cavewoman")),
        ranks: [
            RoleName::new("Troglodyte", None),
            RoleName::new("Aborigine", None),
            RoleName::new("Wanderer", None),
            RoleName::new("Vagrant", None),
            RoleName::new("Wayfarer", None),
            RoleName::new("Roamer", None),
            RoleName::new("Nomad", None),
            RoleName::new("Rover", None),
            RoleName::new("Pioneer", None),
        ],
        gods: ("Anu", "_Ishtar", "Anshar"),
        filecode: "Cav",
        homebase: "the Caves of the Ancestors",
        intermed: "the Dragon's Lair",
        malenum: 2, // PM_CAVEMAN
        femalenum: NON_PM, // PM_CAVEWOMAN
        petnum: NON_PM, // PM_LITTLE_DOG
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Sceptre of Might",
        allow: MH_HUMAN | MH_DWARF | MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL,
        attrbase: [10, 7, 7, 7, 8, 6],
        attrdist: [30, 6, 7, 20, 30, 7],
        hpadv: Advancement::new(14, 0, 0, 8, 2, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 10,
        initrecord: 0,
        spelbase: 12,
        spelheal: 0,
        spelshld: 1,
        spelarmr: 8,
        spelstat: A_INT,
        starting_items: CAVEMAN_ITEMS,
    },
    // Healer
    Role {
        name: RoleName::new("Healer", None),
        ranks: [
            RoleName::new("Rhizotomist", None),
            RoleName::new("Empiric", None),
            RoleName::new("Embalmer", None),
            RoleName::new("Dresser", None),
            RoleName::new("Medicus ossium", Some("Medica ossium")),
            RoleName::new("Herbalist", None),
            RoleName::new("Magister", Some("Magistra")),
            RoleName::new("Physician", None),
            RoleName::new("Chirurgeon", None),
        ],
        gods: ("_Athena", "Hermes", "Poseidon"),
        filecode: "Hea",
        homebase: "the Temple of Epidaurus",
        intermed: "the Temple of Coeus",
        malenum: 3, // PM_HEALER
        femalenum: NON_PM,
        petnum: NON_PM,
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Staff of Aesculapius",
        allow: MH_HUMAN | MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL,
        attrbase: [7, 7, 13, 7, 11, 16],
        attrdist: [15, 20, 20, 15, 25, 5],
        hpadv: Advancement::new(11, 0, 0, 8, 1, 0),
        enadv: Advancement::new(1, 4, 0, 1, 0, 2),
        xlev: 20,
        initrecord: 10,
        spelbase: 3,
        spelheal: -3,
        spelshld: 2,
        spelarmr: 10,
        spelstat: A_WIS,
        starting_items: HEALER_ITEMS,
    },
    // Knight
    Role {
        name: RoleName::new("Knight", None),
        ranks: [
            RoleName::new("Gallant", None),
            RoleName::new("Esquire", None),
            RoleName::new("Bachelor", None),
            RoleName::new("Sergeant", None),
            RoleName::new("Knight", None),
            RoleName::new("Banneret", None),
            RoleName::new("Chevalier", Some("Chevaliere")),
            RoleName::new("Seignieur", Some("Dame")),
            RoleName::new("Paladin", None),
        ],
        gods: ("Lugh", "_Brigit", "Manannan Mac Lir"),
        filecode: "Kni",
        homebase: "Camelot Castle",
        intermed: "the Isle of Glass",
        malenum: 4, // PM_KNIGHT
        femalenum: NON_PM,
        petnum: NON_PM, // PM_PONY
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Magic Mirror of Merlin",
        allow: MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL,
        attrbase: [13, 7, 14, 8, 10, 17],
        attrdist: [30, 15, 15, 10, 20, 10],
        hpadv: Advancement::new(14, 0, 0, 8, 2, 0),
        enadv: Advancement::new(1, 4, 0, 1, 0, 2),
        xlev: 10,
        initrecord: 10,
        spelbase: 8,
        spelheal: -2,
        spelshld: 0,
        spelarmr: 9,
        spelstat: A_WIS,
        starting_items: KNIGHT_ITEMS,
    },
    // Monk
    Role {
        name: RoleName::new("Monk", None),
        ranks: [
            RoleName::new("Candidate", None),
            RoleName::new("Novice", None),
            RoleName::new("Initiate", None),
            RoleName::new("Student of Stones", None),
            RoleName::new("Student of Waters", None),
            RoleName::new("Student of Metals", None),
            RoleName::new("Student of Winds", None),
            RoleName::new("Student of Fire", None),
            RoleName::new("Master", None),
        ],
        gods: ("Shan Lai Ching", "Chih Sung-tzu", "Huan Ti"),
        filecode: "Mon",
        homebase: "the Monastery of Chan-Sune",
        intermed: "the Monastery of the Earth-Lord",
        malenum: 5, // PM_MONK
        femalenum: NON_PM,
        petnum: NON_PM,
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Eyes of the Overworld",
        allow: MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC,
        attrbase: [10, 7, 8, 8, 7, 7],
        attrdist: [25, 10, 20, 20, 15, 10],
        hpadv: Advancement::new(12, 0, 0, 8, 1, 0),
        enadv: Advancement::new(2, 2, 0, 2, 0, 2),
        xlev: 10,
        initrecord: 10,
        spelbase: 8,
        spelheal: -2,
        spelshld: 2,
        spelarmr: 20,
        spelstat: A_WIS,
        starting_items: MONK_ITEMS,
    },
    // Priest
    Role {
        name: RoleName::new("Priest", Some("Priestess")),
        ranks: [
            RoleName::new("Aspirant", None),
            RoleName::new("Acolyte", None),
            RoleName::new("Adept", None),
            RoleName::new("Priest", Some("Priestess")),
            RoleName::new("Curate", None),
            RoleName::new("Canon", Some("Canoness")),
            RoleName::new("Lama", None),
            RoleName::new("Patriarch", Some("Matriarch")),
            RoleName::new("High Priest", Some("High Priestess")),
        ],
        gods: ("", "", ""), // Deities chosen from another role
        filecode: "Pri",
        homebase: "the Great Temple",
        intermed: "the Temple of Nalzok",
        malenum: 6, // PM_PRIEST
        femalenum: NON_PM, // PM_PRIESTESS
        petnum: NON_PM,
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Mitre of Holiness",
        allow: MH_HUMAN | MH_ELF | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC,
        attrbase: [7, 7, 10, 7, 7, 7],
        attrdist: [15, 10, 30, 15, 20, 10],
        hpadv: Advancement::new(12, 0, 0, 8, 1, 0),
        enadv: Advancement::new(4, 3, 0, 2, 0, 2),
        xlev: 10,
        initrecord: 0,
        spelbase: 3,
        spelheal: -2,
        spelshld: 2,
        spelarmr: 10,
        spelstat: A_WIS,
        starting_items: PRIEST_ITEMS,
    },
    // Rogue (before Ranger for -R command line compatibility)
    Role {
        name: RoleName::new("Rogue", None),
        ranks: [
            RoleName::new("Footpad", None),
            RoleName::new("Cutpurse", None),
            RoleName::new("Rogue", None),
            RoleName::new("Pilferer", None),
            RoleName::new("Robber", None),
            RoleName::new("Burglar", None),
            RoleName::new("Filcher", None),
            RoleName::new("Magsman", Some("Magswoman")),
            RoleName::new("Thief", None),
        ],
        gods: ("Issek", "Mog", "Kos"),
        filecode: "Rog",
        homebase: "the Thieves' Guild Hall",
        intermed: "the Assassins' Guild Hall",
        malenum: 8, // PM_ROGUE
        femalenum: NON_PM,
        petnum: NON_PM,
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Master Key of Thievery",
        allow: MH_HUMAN | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_CHAOTIC,
        attrbase: [7, 7, 7, 10, 7, 6],
        attrdist: [20, 10, 10, 30, 20, 10],
        hpadv: Advancement::new(10, 0, 0, 8, 1, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 11,
        initrecord: 10,
        spelbase: 8,
        spelheal: 0,
        spelshld: 1,
        spelarmr: 9,
        spelstat: A_INT,
        starting_items: ROGUE_ITEMS,
    },
    // Ranger
    Role {
        name: RoleName::new("Ranger", None),
        ranks: [
            RoleName::new("Tenderfoot", None),
            RoleName::new("Lookout", None),
            RoleName::new("Trailblazer", None),
            RoleName::new("Reconnoiterer", Some("Reconnoiteress")),
            RoleName::new("Scout", None),
            RoleName::new("Arbalester", None),
            RoleName::new("Archer", None),
            RoleName::new("Sharpshooter", None),
            RoleName::new("Marksman", Some("Markswoman")),
        ],
        gods: ("Mercury", "_Venus", "Mars"),
        filecode: "Ran",
        homebase: "Orion's camp",
        intermed: "the cave of the wumpus",
        malenum: 7, // PM_RANGER
        femalenum: NON_PM,
        petnum: NON_PM, // PM_LITTLE_DOG
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Longbow of Diana",
        allow: MH_HUMAN | MH_ELF | MH_GNOME | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL | ROLE_CHAOTIC,
        attrbase: [13, 13, 13, 9, 13, 7],
        attrdist: [30, 10, 10, 20, 20, 10],
        hpadv: Advancement::new(13, 0, 0, 6, 1, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 12,
        initrecord: 10,
        spelbase: 9,
        spelheal: 2,
        spelshld: 1,
        spelarmr: 10,
        spelstat: A_INT,
        starting_items: RANGER_ITEMS,
    },
    // Samurai
    Role {
        name: RoleName::new("Samurai", None),
        ranks: [
            RoleName::new("Hatamoto", None),
            RoleName::new("Ronin", None),
            RoleName::new("Ninja", Some("Kunoichi")),
            RoleName::new("Joshu", None),
            RoleName::new("Ryoshu", None),
            RoleName::new("Kokushu", None),
            RoleName::new("Daimyo", None),
            RoleName::new("Kuge", None),
            RoleName::new("Shogun", None),
        ],
        gods: ("_Amaterasu Omikami", "Raijin", "Susanowo"),
        filecode: "Sam",
        homebase: "the Castle of the Taro Clan",
        intermed: "the Shogun's Castle",
        malenum: 9, // PM_SAMURAI
        femalenum: NON_PM,
        petnum: NON_PM, // PM_LITTLE_DOG
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Tsurugi of Muramasa",
        allow: MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL,
        attrbase: [10, 8, 7, 10, 17, 6],
        attrdist: [30, 10, 8, 30, 14, 8],
        hpadv: Advancement::new(13, 0, 0, 8, 1, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 11,
        initrecord: 10,
        spelbase: 10,
        spelheal: 0,
        spelshld: 0,
        spelarmr: 8,
        spelstat: A_INT,
        starting_items: SAMURAI_ITEMS,
    },
    // Tourist
    Role {
        name: RoleName::new("Tourist", None),
        ranks: [
            RoleName::new("Rambler", None),
            RoleName::new("Sightseer", None),
            RoleName::new("Excursionist", None),
            RoleName::new("Peregrinator", Some("Peregrinatrix")),
            RoleName::new("Traveler", None),
            RoleName::new("Journeyer", None),
            RoleName::new("Voyager", None),
            RoleName::new("Explorer", None),
            RoleName::new("Adventurer", None),
        ],
        gods: ("Blind Io", "_The Lady", "Offler"),
        filecode: "Tou",
        homebase: "Ankh-Morpork",
        intermed: "the Thieves' Guild Hall",
        malenum: 10, // PM_TOURIST
        femalenum: NON_PM,
        petnum: NON_PM,
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Platinum Yendorian Express Card",
        allow: MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL,
        attrbase: [7, 10, 6, 7, 7, 10],
        attrdist: [15, 10, 10, 15, 30, 20],
        hpadv: Advancement::new(8, 0, 0, 8, 0, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 14,
        initrecord: 0,
        spelbase: 5,
        spelheal: 1,
        spelshld: 2,
        spelarmr: 10,
        spelstat: A_INT,
        starting_items: TOURIST_ITEMS,
    },
    // Valkyrie
    Role {
        name: RoleName::new("Valkyrie", None),
        ranks: [
            RoleName::new("Stripling", None),
            RoleName::new("Skirmisher", None),
            RoleName::new("Fighter", None),
            RoleName::new("Man-at-arms", Some("Woman-at-arms")),
            RoleName::new("Warrior", None),
            RoleName::new("Swashbuckler", None),
            RoleName::new("Hero", Some("Heroine")),
            RoleName::new("Champion", None),
            RoleName::new("Lord", Some("Lady")),
        ],
        gods: ("Tyr", "Odin", "Loki"),
        filecode: "Val",
        homebase: "the Shrine of Destiny",
        intermed: "the cave of Surtur",
        malenum: 11, // PM_VALKYRIE
        femalenum: NON_PM,
        petnum: NON_PM,
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Orb of Fate",
        allow: MH_HUMAN | MH_DWARF | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL,
        attrbase: [10, 7, 7, 7, 10, 7],
        attrdist: [30, 6, 7, 20, 30, 7],
        hpadv: Advancement::new(14, 0, 0, 8, 2, 0),
        enadv: Advancement::new(1, 0, 0, 1, 0, 1),
        xlev: 10,
        initrecord: 0,
        spelbase: 10,
        spelheal: -2,
        spelshld: 0,
        spelarmr: 9,
        spelstat: A_WIS,
        starting_items: VALKYRIE_ITEMS,
    },
    // Wizard
    Role {
        name: RoleName::new("Wizard", None),
        ranks: [
            RoleName::new("Evoker", None),
            RoleName::new("Conjurer", None),
            RoleName::new("Thaumaturge", None),
            RoleName::new("Magician", None),
            RoleName::new("Enchanter", Some("Enchantress")),
            RoleName::new("Sorcerer", Some("Sorceress")),
            RoleName::new("Necromancer", None),
            RoleName::new("Wizard", None),
            RoleName::new("Mage", None),
        ],
        gods: ("Ptah", "Thoth", "Anhur"),
        filecode: "Wiz",
        homebase: "the Lonely Tower",
        intermed: "the Tower of Darkness",
        malenum: 12, // PM_WIZARD
        femalenum: NON_PM,
        petnum: NON_PM, // PM_KITTEN
        ldrnum: 0,
        guardnum: 0,
        neminum: 0,
        questarti: "The Eye of the Aethiopica",
        allow: MH_HUMAN | MH_ELF | MH_GNOME | MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL | ROLE_CHAOTIC,
        attrbase: [7, 10, 7, 7, 7, 7],
        attrdist: [10, 30, 10, 20, 20, 10],
        hpadv: Advancement::new(10, 0, 0, 8, 1, 0),
        enadv: Advancement::new(4, 3, 0, 2, 0, 3),
        xlev: 12,
        initrecord: 0,
        spelbase: 1,
        spelheal: 0,
        spelshld: 3,
        spelarmr: 10,
        spelstat: A_INT,
        starting_items: WIZARD_ITEMS,
    },
];

// ==================== RACE DEFINITIONS ====================

/// All player races
pub static RACES: &[Race] = &[
    // Human
    Race {
        noun: "human",
        adj: "human",
        collective: "humanity",
        filecode: "Hum",
        individual: RoleName::new("man", Some("woman")),
        malenum: PM_HUMAN,
        femalenum: NON_PM,
        mummynum: 0, // PM_HUMAN_MUMMY
        zombienum: 0, // PM_HUMAN_ZOMBIE
        allow: MH_HUMAN | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL | ROLE_NEUTRAL | ROLE_CHAOTIC,
        selfmask: MH_HUMAN,
        lovemask: 0,
        hatemask: MH_GNOME | MH_ORC,
        attrmin: [3, 3, 3, 3, 3, 3],
        attrmax: [18, 18, 18, 18, 18, 18], // STR18(100) simplified
        hpadv: Advancement::new(2, 0, 0, 2, 1, 0),
        enadv: Advancement::new(1, 0, 2, 0, 2, 0),
    },
    // Elf
    Race {
        noun: "elf",
        adj: "elven",
        collective: "elvenkind",
        filecode: "Elf",
        individual: RoleName::new("elf", None),
        malenum: PM_ELF,
        femalenum: NON_PM,
        mummynum: 0, // PM_ELF_MUMMY
        zombienum: 0, // PM_ELF_ZOMBIE
        allow: MH_ELF | ROLE_MALE | ROLE_FEMALE | ROLE_CHAOTIC,
        selfmask: MH_ELF,
        lovemask: MH_ELF,
        hatemask: MH_ORC,
        attrmin: [3, 3, 3, 3, 3, 3],
        attrmax: [18, 20, 20, 18, 16, 18],
        hpadv: Advancement::new(1, 0, 0, 1, 1, 0),
        enadv: Advancement::new(2, 0, 3, 0, 3, 0),
    },
    // Dwarf
    Race {
        noun: "dwarf",
        adj: "dwarven",
        collective: "dwarvenkind",
        filecode: "Dwa",
        individual: RoleName::new("dwarf", None),
        malenum: PM_DWARF,
        femalenum: NON_PM,
        mummynum: 0, // PM_DWARF_MUMMY
        zombienum: 0, // PM_DWARF_ZOMBIE
        allow: MH_DWARF | ROLE_MALE | ROLE_FEMALE | ROLE_LAWFUL,
        selfmask: MH_DWARF,
        lovemask: MH_DWARF | MH_GNOME,
        hatemask: MH_ORC,
        attrmin: [3, 3, 3, 3, 3, 3],
        attrmax: [18, 16, 16, 20, 20, 16], // STR18(100) simplified
        hpadv: Advancement::new(4, 0, 0, 3, 2, 0),
        enadv: Advancement::new(0, 0, 0, 0, 0, 0),
    },
    // Gnome
    Race {
        noun: "gnome",
        adj: "gnomish",
        collective: "gnomehood",
        filecode: "Gno",
        individual: RoleName::new("gnome", None),
        malenum: PM_GNOME,
        femalenum: NON_PM,
        mummynum: 0, // PM_GNOME_MUMMY
        zombienum: 0, // PM_GNOME_ZOMBIE
        allow: MH_GNOME | ROLE_MALE | ROLE_FEMALE | ROLE_NEUTRAL,
        selfmask: MH_GNOME,
        lovemask: MH_DWARF | MH_GNOME,
        hatemask: MH_HUMAN,
        attrmin: [3, 3, 3, 3, 3, 3],
        attrmax: [18, 19, 18, 18, 18, 18], // STR18(50) simplified
        hpadv: Advancement::new(1, 0, 0, 1, 0, 0),
        enadv: Advancement::new(2, 0, 2, 0, 2, 0),
    },
    // Orc
    Race {
        noun: "orc",
        adj: "orcish",
        collective: "orcdom",
        filecode: "Orc",
        individual: RoleName::new("orc", None),
        malenum: PM_ORC,
        femalenum: NON_PM,
        mummynum: 0, // PM_ORC_MUMMY
        zombienum: 0, // PM_ORC_ZOMBIE
        allow: MH_ORC | ROLE_MALE | ROLE_FEMALE | ROLE_CHAOTIC,
        selfmask: MH_ORC,
        lovemask: 0,
        hatemask: MH_HUMAN | MH_ELF | MH_DWARF,
        attrmin: [3, 3, 3, 3, 3, 3],
        attrmax: [18, 16, 16, 18, 18, 16], // STR18(50) simplified
        hpadv: Advancement::new(1, 0, 0, 1, 0, 0),
        enadv: Advancement::new(1, 0, 1, 0, 1, 0),
    },
];

/// Number of roles
pub fn num_roles() -> usize {
    ROLES.len()
}

/// Number of races
pub fn num_races() -> usize {
    RACES.len()
}

/// Get a role by index
pub fn get_role(index: usize) -> Option<&'static Role> {
    ROLES.get(index)
}

/// Get a race by index
pub fn get_race(index: usize) -> Option<&'static Race> {
    RACES.get(index)
}

/// Find a role by name
pub fn find_role(name: &str) -> Option<&'static Role> {
    ROLES.iter().find(|r| {
        r.name.male.eq_ignore_ascii_case(name)
            || r.name.female.is_some_and(|f| f.eq_ignore_ascii_case(name))
            || r.filecode.eq_ignore_ascii_case(name)
    })
}

/// Find a race by name
pub fn find_race(name: &str) -> Option<&'static Race> {
    RACES.iter().find(|r| {
        r.noun.eq_ignore_ascii_case(name)
            || r.adj.eq_ignore_ascii_case(name)
            || r.filecode.eq_ignore_ascii_case(name)
    })
}

/// Check if a role can be played by a specific race
pub fn role_allows_race(role: &Role, race: &Race) -> bool {
    (role.allow & race.selfmask) != 0
}

/// Check if a role can have a specific alignment
pub fn role_allows_alignment(role: &Role, alignment: Alignment) -> bool {
    let mask = match alignment {
        Alignment::Lawful => ROLE_LAWFUL,
        Alignment::Neutral => ROLE_NEUTRAL,
        Alignment::Chaotic => ROLE_CHAOTIC,
        Alignment::None => return false,
    };
    (role.allow & mask) != 0
}

/// Check if a race can have a specific alignment
pub fn race_allows_alignment(race: &Race, alignment: Alignment) -> bool {
    let mask = match alignment {
        Alignment::Lawful => ROLE_LAWFUL,
        Alignment::Neutral => ROLE_NEUTRAL,
        Alignment::Chaotic => ROLE_CHAOTIC,
        Alignment::None => return false,
    };
    (race.allow & mask) != 0
}
