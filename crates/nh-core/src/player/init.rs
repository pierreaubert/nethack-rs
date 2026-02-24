//! Player initialization (u_init.c)
//!
//! Sets up the player's starting inventory, skills, and attributes
//! based on their role and race. Called at character creation.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::{BucStatus, Object, ObjectClass};
use crate::player::{Attribute, Attributes, Role, SkillLevel, SkillSet, SkillType, You};
use crate::rng::GameRng;

// ─────────────────────────────────────────────────────────────────────────────
// Starting inventory item descriptor
// ─────────────────────────────────────────────────────────────────────────────

/// One item in a role's starting inventory (C: struct trobj)
#[derive(Debug, Clone, Copy)]
pub struct StartingItem {
    /// Object type index (0 = random within class)
    pub otyp: i16,
    /// Enchantment/charges (i8::MAX = random)
    pub spe: i8,
    /// Object class
    pub class: ObjectClass,
    /// Quantity
    pub quantity: u8,
    /// BUC status: 0=uncursed, 1=blessed, 2=random
    pub bless: u8,
}

const UNDEF_SPE: i8 = i8::MAX;
const UNDEF_BLESS: u8 = 2;

impl StartingItem {
    const fn new(otyp: i16, spe: i8, class: ObjectClass, quantity: u8, bless: u8) -> Self {
        Self { otyp, spe, class, quantity, bless }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Role starting inventories
// ─────────────────────────────────────────────────────────────────────────────

/// Archeologist starting inventory
static ARCHEOLOGIST_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Bullwhip as i16, 2, ObjectClass::Weapon, 1, UNDEF_BLESS),   // BULLWHIP
    StartingItem::new(crate::data::objects::ObjectType::LeatherJacket as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // LEATHER_JACKET
    StartingItem::new(crate::data::objects::ObjectType::Fedora as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // FEDORA
    StartingItem::new(crate::data::objects::ObjectType::FoodRation as i16, 0, ObjectClass::Food, 3, 0),               // FOOD_RATION
    StartingItem::new(crate::data::objects::ObjectType::PickAxe as i16, UNDEF_SPE, ObjectClass::Tool, 1, UNDEF_BLESS), // PICK_AXE
    StartingItem::new(crate::data::objects::ObjectType::TinningKit as i16, UNDEF_SPE, ObjectClass::Tool, 1, UNDEF_BLESS), // TINNING_KIT
    StartingItem::new(crate::data::objects::ObjectType::Touchstone as i16, 0, ObjectClass::Gem, 1, 0),                // TOUCHSTONE
    StartingItem::new(crate::data::objects::ObjectType::Sack as i16, 0, ObjectClass::Tool, 1, 0),               // SACK
];

/// Barbarian starting inventory
static BARBARIAN_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::TwoHandedSword as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // TWO_HANDED_SWORD
    StartingItem::new(crate::data::objects::ObjectType::Axe as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // AXE
    StartingItem::new(crate::data::objects::ObjectType::RingMail as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // RING_MAIL
    StartingItem::new(crate::data::objects::ObjectType::FoodRation as i16, 0, ObjectClass::Food, 1, 0),               // FOOD_RATION
];

/// Caveman starting inventory
static CAVEMAN_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Club as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),   // CLUB
    StartingItem::new(crate::data::objects::ObjectType::Sling as i16, 2, ObjectClass::Weapon, 1, UNDEF_BLESS),   // SLING
    StartingItem::new(crate::data::objects::ObjectType::Flint as i16, 0, ObjectClass::Gem, 15, UNDEF_BLESS),     // FLINT (qty variable)
    StartingItem::new(crate::data::objects::ObjectType::Rock as i16, 0, ObjectClass::Gem, 3, 0),                // ROCK
    StartingItem::new(crate::data::objects::ObjectType::LeatherArmor as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // LEATHER_ARMOR
];

/// Healer starting inventory
static HEALER_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Scalpel as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // SCALPEL
    StartingItem::new(crate::data::objects::ObjectType::LeatherGloves as i16, 1, ObjectClass::Armor, 1, UNDEF_BLESS),    // LEATHER_GLOVES
    StartingItem::new(crate::data::objects::ObjectType::Stethoscope as i16, 0, ObjectClass::Tool, 1, 0),               // STETHOSCOPE
    StartingItem::new(crate::data::objects::ObjectType::Healing as i16, 0, ObjectClass::Potion, 4, UNDEF_BLESS),   // POT_HEALING
    StartingItem::new(crate::data::objects::ObjectType::ExtraHealing as i16, 0, ObjectClass::Potion, 4, UNDEF_BLESS),   // POT_EXTRA_HEALING
    StartingItem::new(crate::data::objects::ObjectType::Sleep as i16, UNDEF_SPE, ObjectClass::Wand, 1, UNDEF_BLESS), // WAN_SLEEP
    StartingItem::new(crate::data::objects::ObjectType::Healing as i16, 0, ObjectClass::Spellbook, 1, 1),          // SPE_HEALING
    StartingItem::new(crate::data::objects::ObjectType::ExtraHealing as i16, 0, ObjectClass::Spellbook, 1, 1),          // SPE_EXTRA_HEALING
    StartingItem::new(crate::data::objects::ObjectType::StoneToFlesh as i16, 0, ObjectClass::Spellbook, 1, 1),          // SPE_STONE_TO_FLESH
    StartingItem::new(crate::data::objects::ObjectType::Apple as i16, 0, ObjectClass::Food, 5, 0),               // APPLE
];

/// Knight starting inventory
static KNIGHT_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::LongSword as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),   // LONG_SWORD
    StartingItem::new(crate::data::objects::ObjectType::Lance as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),   // LANCE
    StartingItem::new(crate::data::objects::ObjectType::RingMail as i16, 1, ObjectClass::Armor, 1, UNDEF_BLESS),    // RING_MAIL
    StartingItem::new(crate::data::objects::ObjectType::Helmet as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // HELMET
    StartingItem::new(crate::data::objects::ObjectType::SmallShield as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // SMALL_SHIELD
    StartingItem::new(crate::data::objects::ObjectType::LeatherGloves as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // LEATHER_GLOVES
    StartingItem::new(crate::data::objects::ObjectType::Apple as i16, 0, ObjectClass::Food, 10, 0),              // APPLE
    StartingItem::new(crate::data::objects::ObjectType::Carrot as i16, 0, ObjectClass::Food, 10, 0),              // CARROT
];

/// Monk starting inventory
static MONK_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::LeatherGloves as i16, 2, ObjectClass::Armor, 1, UNDEF_BLESS),    // LEATHER_GLOVES
    StartingItem::new(crate::data::objects::ObjectType::Robe as i16, 1, ObjectClass::Armor, 1, UNDEF_BLESS),    // ROBE
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Spellbook, 1, 1),    // Random spellbook
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Scroll, 1, UNDEF_BLESS), // Random scroll
    StartingItem::new(crate::data::objects::ObjectType::Healing as i16, 0, ObjectClass::Potion, 3, UNDEF_BLESS),   // POT_HEALING
    StartingItem::new(crate::data::objects::ObjectType::FoodRation as i16, 0, ObjectClass::Food, 3, 0),               // FOOD_RATION
    StartingItem::new(crate::data::objects::ObjectType::Apple as i16, 0, ObjectClass::Food, 5, UNDEF_BLESS),     // APPLE
    StartingItem::new(crate::data::objects::ObjectType::Orange as i16, 0, ObjectClass::Food, 5, UNDEF_BLESS),     // ORANGE
    StartingItem::new(crate::data::objects::ObjectType::FortuneCookie as i16, 0, ObjectClass::Food, 3, UNDEF_BLESS),     // FORTUNE_COOKIE
];

/// Priest starting inventory
static PRIEST_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Mace as i16, 1, ObjectClass::Weapon, 1, 1),             // MACE (blessed)
    StartingItem::new(crate::data::objects::ObjectType::Robe as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // ROBE
    StartingItem::new(crate::data::objects::ObjectType::SmallShield as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // SMALL_SHIELD
    StartingItem::new(crate::data::objects::ObjectType::Water as i16, 0, ObjectClass::Potion, 4, 1),             // POT_WATER (holy)
    StartingItem::new(crate::data::objects::ObjectType::CloveOfGarlic as i16, 0, ObjectClass::Food, 1, 0),               // CLOVE_OF_GARLIC
    StartingItem::new(crate::data::objects::ObjectType::SprigOfWolfsbane as i16, 0, ObjectClass::Food, 1, 0),               // SPRIG_OF_WOLFSBANE
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Spellbook, 2, UNDEF_BLESS), // Random spellbooks
];

/// Ranger starting inventory
static RANGER_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Dagger as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),   // DAGGER
    StartingItem::new(crate::data::objects::ObjectType::Bow as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),   // BOW
    StartingItem::new(crate::data::objects::ObjectType::Arrow as i16, 2, ObjectClass::Weapon, 50, UNDEF_BLESS),  // ARROW (qty variable)
    StartingItem::new(crate::data::objects::ObjectType::Arrow as i16, 0, ObjectClass::Weapon, 30, UNDEF_BLESS),  // ARROW
    StartingItem::new(crate::data::objects::ObjectType::CloakOfDisplacement as i16, 2, ObjectClass::Armor, 1, UNDEF_BLESS),    // CLOAK_OF_DISPLACEMENT
    StartingItem::new(crate::data::objects::ObjectType::CramRation as i16, 0, ObjectClass::Food, 4, 0),               // CRAM_RATION
];

/// Rogue starting inventory
static ROGUE_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::ShortSword as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // SHORT_SWORD
    StartingItem::new(crate::data::objects::ObjectType::Dagger as i16, 0, ObjectClass::Weapon, 10, 0),            // DAGGER (qty variable)
    StartingItem::new(crate::data::objects::ObjectType::LeatherArmor as i16, 1, ObjectClass::Armor, 1, UNDEF_BLESS),    // LEATHER_ARMOR
    StartingItem::new(crate::data::objects::ObjectType::Sickness as i16, 0, ObjectClass::Potion, 1, 0),             // POT_SICKNESS
    StartingItem::new(crate::data::objects::ObjectType::LockPick as i16, 0, ObjectClass::Tool, 1, 0),               // LOCK_PICK
    StartingItem::new(crate::data::objects::ObjectType::Sack as i16, 0, ObjectClass::Tool, 1, 0),               // SACK
];

/// Samurai starting inventory
static SAMURAI_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Katana as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // KATANA
    StartingItem::new(crate::data::objects::ObjectType::ShortSword as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // SHORT_SWORD (wakizashi)
    StartingItem::new(crate::data::objects::ObjectType::Yumi as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // YUMI
    StartingItem::new(crate::data::objects::ObjectType::Ya as i16, 0, ObjectClass::Weapon, 25, UNDEF_BLESS),  // YA (qty variable)
    StartingItem::new(crate::data::objects::ObjectType::SplintMail as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // SPLINT_MAIL
];

/// Tourist starting inventory
static TOURIST_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Dart as i16, 2, ObjectClass::Weapon, 25, UNDEF_BLESS),  // DART (qty variable)
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Food, 10, 0),        // Random food
    StartingItem::new(crate::data::objects::ObjectType::ExtraHealing as i16, 0, ObjectClass::Potion, 2, UNDEF_BLESS),   // POT_EXTRA_HEALING
    StartingItem::new(crate::data::objects::ObjectType::MagicMapping as i16, 0, ObjectClass::Scroll, 4, UNDEF_BLESS),   // SCR_MAGIC_MAPPING
    StartingItem::new(crate::data::objects::ObjectType::HawaiianShirt as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // HAWAIIAN_SHIRT
    StartingItem::new(crate::data::objects::ObjectType::ExpensiveCamera as i16, UNDEF_SPE, ObjectClass::Tool, 1, 0),       // EXPENSIVE_CAMERA
    StartingItem::new(crate::data::objects::ObjectType::CreditCard as i16, 0, ObjectClass::Tool, 1, 0),               // CREDIT_CARD
];

/// Valkyrie starting inventory
static VALKYRIE_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::LongSword as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),   // LONG_SWORD
    StartingItem::new(crate::data::objects::ObjectType::Dagger as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),   // DAGGER
    StartingItem::new(crate::data::objects::ObjectType::SmallShield as i16, 3, ObjectClass::Armor, 1, UNDEF_BLESS),    // SMALL_SHIELD
    StartingItem::new(crate::data::objects::ObjectType::FoodRation as i16, 0, ObjectClass::Food, 1, 0),               // FOOD_RATION
];

/// Wizard starting inventory
static WIZARD_INV: &[StartingItem] = &[
    StartingItem::new(crate::data::objects::ObjectType::Quarterstaff as i16, 1, ObjectClass::Weapon, 1, 1),             // QUARTERSTAFF (blessed)
    StartingItem::new(crate::data::objects::ObjectType::CloakOfMagicResistance as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),    // CLOAK_OF_MAGIC_RESISTANCE
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Wand, 1, UNDEF_BLESS), // Random wand
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Ring, 2, UNDEF_BLESS), // Random rings
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Potion, 3, UNDEF_BLESS), // Random potions
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Scroll, 3, UNDEF_BLESS), // Random scrolls
    StartingItem::new(crate::data::objects::ObjectType::ForceBolt as i16, 0, ObjectClass::Spellbook, 1, 1),          // SPE_FORCE_BOLT
    StartingItem::new(crate::data::objects::ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Spellbook, 1, UNDEF_BLESS), // Random spellbook
];

// ─────────────────────────────────────────────────────────────────────────────
// Role skill initialization tables
// ─────────────────────────────────────────────────────────────────────────────

/// (SkillType, max SkillLevel) pairs for a role
type SkillTable = &'static [(SkillType, SkillLevel)];

fn skill_table_for_role(role: Role) -> SkillTable {
    match role {
        Role::Archeologist => &[
            (SkillType::Whip, SkillLevel::Expert),
            (SkillType::PickAxe, SkillLevel::Expert),
            (SkillType::Club, SkillLevel::Skilled),
            (SkillType::Sling, SkillLevel::Skilled),
            (SkillType::Dart, SkillLevel::Basic),
            (SkillType::BareHanded, SkillLevel::Basic),
            (SkillType::AttackSpells, SkillLevel::Basic),
            (SkillType::DivinationSpells, SkillLevel::Expert),
        ],
        Role::Barbarian => &[
            (SkillType::Dagger, SkillLevel::Expert),
            (SkillType::Axe, SkillLevel::Expert),
            (SkillType::ShortSword, SkillLevel::Expert),
            (SkillType::BroadSword, SkillLevel::Expert),
            (SkillType::TwoHandedSword, SkillLevel::Expert),
            (SkillType::Club, SkillLevel::Skilled),
            (SkillType::Mace, SkillLevel::Skilled),
            (SkillType::Hammer, SkillLevel::Expert),
            (SkillType::Spear, SkillLevel::Skilled),
            (SkillType::TwoWeapon, SkillLevel::Skilled),
            (SkillType::BareHanded, SkillLevel::Master),
            (SkillType::Riding, SkillLevel::Basic),
        ],
        Role::Caveman => &[
            (SkillType::Club, SkillLevel::Expert),
            (SkillType::Sling, SkillLevel::Expert),
            (SkillType::Mace, SkillLevel::Skilled),
            (SkillType::Flail, SkillLevel::Skilled),
            (SkillType::Hammer, SkillLevel::Expert),
            (SkillType::Quarterstaff, SkillLevel::Skilled),
            (SkillType::Spear, SkillLevel::Skilled),
            (SkillType::Javelin, SkillLevel::Skilled),
            (SkillType::BareHanded, SkillLevel::Expert),
            (SkillType::AttackSpells, SkillLevel::Basic),
        ],
        Role::Healer => &[
            (SkillType::Dagger, SkillLevel::Skilled),
            (SkillType::Knife, SkillLevel::Expert),
            (SkillType::Quarterstaff, SkillLevel::Skilled),
            (SkillType::Crossbow, SkillLevel::Skilled),
            (SkillType::Dart, SkillLevel::Expert),
            (SkillType::BareHanded, SkillLevel::Basic),
            (SkillType::HealingSpells, SkillLevel::Expert),
            (SkillType::EnchantmentSpells, SkillLevel::Skilled),
        ],
        Role::Knight => &[
            (SkillType::Dagger, SkillLevel::Basic),
            (SkillType::BroadSword, SkillLevel::Skilled),
            (SkillType::LongSword, SkillLevel::Expert),
            (SkillType::TwoHandedSword, SkillLevel::Skilled),
            (SkillType::Lance, SkillLevel::Expert),
            (SkillType::Mace, SkillLevel::Skilled),
            (SkillType::MorningStar, SkillLevel::Skilled),
            (SkillType::Spear, SkillLevel::Skilled),
            (SkillType::Javelin, SkillLevel::Skilled),
            (SkillType::Crossbow, SkillLevel::Skilled),
            (SkillType::TwoWeapon, SkillLevel::Skilled),
            (SkillType::BareHanded, SkillLevel::Expert),
            (SkillType::Riding, SkillLevel::Expert),
            (SkillType::HealingSpells, SkillLevel::Skilled),
            (SkillType::ClericalSpells, SkillLevel::Skilled),
        ],
        Role::Monk => &[
            (SkillType::Quarterstaff, SkillLevel::Skilled),
            (SkillType::Shuriken, SkillLevel::Basic),
            (SkillType::Spear, SkillLevel::Basic),
            (SkillType::Javelin, SkillLevel::Basic),
            (SkillType::Crossbow, SkillLevel::Basic),
            (SkillType::BareHanded, SkillLevel::GrandMaster),
            (SkillType::HealingSpells, SkillLevel::Expert),
            (SkillType::ClericalSpells, SkillLevel::Skilled),
            (SkillType::EscapeSpells, SkillLevel::Skilled),
            (SkillType::AttackSpells, SkillLevel::Basic),
        ],
        Role::Priest => &[
            (SkillType::Club, SkillLevel::Expert),
            (SkillType::Mace, SkillLevel::Expert),
            (SkillType::MorningStar, SkillLevel::Expert),
            (SkillType::Flail, SkillLevel::Expert),
            (SkillType::Hammer, SkillLevel::Expert),
            (SkillType::Quarterstaff, SkillLevel::Expert),
            (SkillType::Sling, SkillLevel::Skilled),
            (SkillType::BareHanded, SkillLevel::Skilled),
            (SkillType::HealingSpells, SkillLevel::Expert),
            (SkillType::ClericalSpells, SkillLevel::Expert),
            (SkillType::DivinationSpells, SkillLevel::Expert),
        ],
        Role::Ranger => &[
            (SkillType::Dagger, SkillLevel::Expert),
            (SkillType::Knife, SkillLevel::Skilled),
            (SkillType::ShortSword, SkillLevel::Skilled),
            (SkillType::Bow, SkillLevel::Expert),
            (SkillType::Crossbow, SkillLevel::Expert),
            (SkillType::Dart, SkillLevel::Expert),
            (SkillType::Spear, SkillLevel::Skilled),
            (SkillType::Javelin, SkillLevel::Skilled),
            (SkillType::TwoWeapon, SkillLevel::Skilled),
            (SkillType::BareHanded, SkillLevel::Basic),
            (SkillType::DivinationSpells, SkillLevel::Skilled),
            (SkillType::EscapeSpells, SkillLevel::Skilled),
        ],
        Role::Rogue => &[
            (SkillType::Dagger, SkillLevel::Expert),
            (SkillType::Knife, SkillLevel::Expert),
            (SkillType::ShortSword, SkillLevel::Expert),
            (SkillType::BroadSword, SkillLevel::Skilled),
            (SkillType::LongSword, SkillLevel::Skilled),
            (SkillType::Club, SkillLevel::Skilled),
            (SkillType::Saber, SkillLevel::Skilled),
            (SkillType::Crossbow, SkillLevel::Expert),
            (SkillType::Dart, SkillLevel::Expert),
            (SkillType::Sling, SkillLevel::Skilled),
            (SkillType::TwoWeapon, SkillLevel::Expert),
            (SkillType::BareHanded, SkillLevel::Skilled),
            (SkillType::DivinationSpells, SkillLevel::Skilled),
            (SkillType::EscapeSpells, SkillLevel::Skilled),
            (SkillType::MatterSpells, SkillLevel::Skilled),
        ],
        Role::Samurai => &[
            (SkillType::Dagger, SkillLevel::Basic),
            (SkillType::Knife, SkillLevel::Skilled),
            (SkillType::ShortSword, SkillLevel::Expert),
            (SkillType::BroadSword, SkillLevel::Expert),
            (SkillType::LongSword, SkillLevel::Expert),
            (SkillType::Bow, SkillLevel::Expert),
            (SkillType::Spear, SkillLevel::Skilled),
            (SkillType::Polearms, SkillLevel::Skilled),
            (SkillType::Lance, SkillLevel::Skilled),
            (SkillType::Flail, SkillLevel::Skilled),
            (SkillType::TwoWeapon, SkillLevel::Expert),
            (SkillType::BareHanded, SkillLevel::Master),
            (SkillType::Riding, SkillLevel::Skilled),
            (SkillType::ClericalSpells, SkillLevel::Skilled),
        ],
        Role::Tourist => &[
            (SkillType::Dagger, SkillLevel::Expert),
            (SkillType::Dart, SkillLevel::Expert),
            (SkillType::Sling, SkillLevel::Skilled),
            (SkillType::Whip, SkillLevel::Skilled),
            (SkillType::UnicornHorn, SkillLevel::Skilled),
            (SkillType::BareHanded, SkillLevel::Skilled),
            (SkillType::Riding, SkillLevel::Basic),
            (SkillType::EnchantmentSpells, SkillLevel::Skilled),
            (SkillType::DivinationSpells, SkillLevel::Basic),
        ],
        Role::Valkyrie => &[
            (SkillType::Dagger, SkillLevel::Expert),
            (SkillType::Axe, SkillLevel::Expert),
            (SkillType::ShortSword, SkillLevel::Skilled),
            (SkillType::BroadSword, SkillLevel::Skilled),
            (SkillType::LongSword, SkillLevel::Expert),
            (SkillType::TwoHandedSword, SkillLevel::Expert),
            (SkillType::Scimitar, SkillLevel::Skilled),
            (SkillType::Spear, SkillLevel::Skilled),
            (SkillType::Hammer, SkillLevel::Expert),
            (SkillType::Lance, SkillLevel::Skilled),
            (SkillType::TwoWeapon, SkillLevel::Skilled),
            (SkillType::BareHanded, SkillLevel::Expert),
            (SkillType::Riding, SkillLevel::Skilled),
        ],
        Role::Wizard => &[
            (SkillType::Dagger, SkillLevel::Expert),
            (SkillType::Quarterstaff, SkillLevel::Expert),
            (SkillType::BareHanded, SkillLevel::Basic),
            (SkillType::AttackSpells, SkillLevel::Expert),
            (SkillType::HealingSpells, SkillLevel::Skilled),
            (SkillType::DivinationSpells, SkillLevel::Expert),
            (SkillType::EnchantmentSpells, SkillLevel::Expert),
            (SkillType::ClericalSpells, SkillLevel::Skilled),
            (SkillType::EscapeSpells, SkillLevel::Expert),
            (SkillType::MatterSpells, SkillLevel::Expert),
        ],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main initialization
// ─────────────────────────────────────────────────────────────────────────────

/// Get the starting inventory table for a role
pub fn starting_inventory(role: Role) -> &'static [StartingItem] {
    match role {
        Role::Archeologist => ARCHEOLOGIST_INV,
        Role::Barbarian => BARBARIAN_INV,
        Role::Caveman => CAVEMAN_INV,
        Role::Healer => HEALER_INV,
        Role::Knight => KNIGHT_INV,
        Role::Monk => MONK_INV,
        Role::Priest => PRIEST_INV,
        Role::Ranger => RANGER_INV,
        Role::Rogue => ROGUE_INV,
        Role::Samurai => SAMURAI_INV,
        Role::Tourist => TOURIST_INV,
        Role::Valkyrie => VALKYRIE_INV,
        Role::Wizard => WIZARD_INV,
    }
}

/// Initialize player skills based on role (C: skill_init)
pub fn init_skills(skills: &mut SkillSet, role: Role) {
    let table = skill_table_for_role(role);
    for &(skill_type, max_level) in table {
        skills.set_max(skill_type, max_level);
        // Set starting level to Unskilled for skills above Restricted
        let skill = skills.get_mut(skill_type);
        if skill.level == SkillLevel::Restricted {
            skill.level = SkillLevel::Unskilled;
        }
    }
}

/// Convert a starting item descriptor into an Object (C: ini_inv per-item logic)
pub fn make_starting_object(item: &StartingItem, rng: &mut GameRng, next_id: &mut u32) -> Object {
    let id = *next_id;
    *next_id += 1;

    let mut obj = Object::new(crate::object::ObjectId(id), item.otyp, item.class);
    obj.quantity = item.quantity as i32;

    // Set enchantment
    if item.spe != UNDEF_SPE {
        obj.enchantment = item.spe;
    } else {
        // Random enchantment based on class (C: mksobj init=TRUE, then ini_inv
        // keeps mksobj value when trspe == UNDEF_SPE)
        obj.enchantment = match item.class {
            ObjectClass::Wand => {
                // C wand init: spe = rn1(5, 4) = rnd(5)+3 = 4..8 for most wands
                // But specific wands like WAN_SLEEP use this range
                (rng.rnd(5) + 3) as i8
            }
            ObjectClass::Tool => {
                // C tool init: MAGIC_MARKER/TINNING_KIT/EXPENSIVE_CAMERA: rn1(70,30) = 30..99
                // Other tools like PICK_AXE: no special init (0)
                use crate::data::objects::ObjectType;
                let otyp = item.otyp;
                if otyp == ObjectType::TinningKit as i16
                    || otyp == ObjectType::ExpensiveCamera as i16
                    || otyp == ObjectType::MagicMarker as i16
                {
                    (rng.rnd(70) + 29) as i8
                } else {
                    0
                }
            }
            _ => 0,
        };
    }

    // Set BUC status
    // In C, ini_inv always sets obj->cursed = 0, and only overrides blessed
    // if trbless != UNDEF_BLESS. Since mksobj starts objects uncursed by default,
    // UNDEF_BLESS effectively means "uncursed" in C.
    obj.buc = match item.bless {
        0 => BucStatus::Uncursed,
        1 => BucStatus::Blessed,
        _ => BucStatus::Uncursed, // UNDEF_BLESS = uncursed (C: obj->cursed = 0)
    };

    obj
}

/// Initialize a player's starting inventory (C: u_init inventory section)
pub fn init_inventory(rng: &mut GameRng, role: Role) -> Vec<Object> {
    let items = starting_inventory(role);
    let mut inventory = Vec::with_capacity(items.len());
    let mut next_id: u32 = 1;
    let mut letter = b'a';

    for item in items {
        let mut obj = make_starting_object(item, rng, &mut next_id);
        obj.inv_letter = letter as char;
        if letter < b'z' {
            letter += 1;
        }
        inventory.push(obj);
    }

    inventory
}

/// Roll initial attributes (C: init_attr(75))
fn roll_attributes(player: &mut You, rng: &mut GameRng) {
    let role_data = crate::data::roles::find_role(&format!("{:?}", player.role)).unwrap();
    let race_data = crate::data::roles::find_race(&format!("{:?}", player.race)).unwrap();

    let mut np = 75i32;
    let mut values = [0i8; 6];

    // Initial base from role
    for i in 0..6 {
        values[i] = role_data.attrbase[i] as i8;
        np -= values[i] as i32;
    }

    // Distribute remaining points based on role distribution
    let mut try_count = 0;
    while np > 0 && try_count < 100 {
        let mut x = rng.rn2(100) as i32;
        let mut i = 0;
        while i < 6 {
            x -= role_data.attrdist[i] as i32;
            if x <= 0 {
                break;
            }
            i += 1;
        }
        if i >= 6 {
            continue;
        }

        // Check racial max
        if values[i] >= race_data.attrmax[i] {
            try_count += 1;
            continue;
        }

        try_count = 0;
        values[i] += 1;
        np -= 1;
    }

    player.attr_current = Attributes::new(values);
    player.attr_max = Attributes::new(values);

    // Biased variation (C: u_init.c:887-894)
    for i in 0..6 {
        if rng.rn2(20) == 0 {
            let xd = rng.rn2(7) as i8 - 2;
            let attr = Attribute::from_index(i).unwrap();
            player.adjattrib(attr, xd);
            // C: if (ABASE(i) < AMAX(i)) AMAX(i) = ABASE(i);
            // Cap AMAX to ABASE if the adjustment reduced the base
            let base = player.attr_current.get(attr);
            let max = player.attr_max.get(attr);
            if base < max {
                player.attr_max.set(attr, base);
            }
        }
    }
}

/// Full player initialization (C: u_init)
///
/// Sets initial HP, energy, attributes, skills, inventory, and prayer timeout.
pub fn u_init(player: &mut crate::player::You, rng: &mut GameRng) -> Vec<Object> {
    let role = player.role;

    // Set initial HP and Energy (C: u_init calls newhp/newpw)
    // We set ulevel=0 temporarily to trigger initial HP logic in newhp
    let old_level = player.exp_level;
    player.exp_level = 0;
    player.hp_max = crate::player::you::newhp(player, rng);
    player.hp = player.hp_max;
    player.energy_max = crate::player::you::newpw(player, rng);
    player.energy = player.energy_max;
    player.exp_level = old_level;

    // Initialize skills
    init_skills(&mut player.skills, role);

    // Set prayer timeout (C: u.ublesscnt = 300)
    player.bless_count = 300;

    // Give Knight intrinsic jumping
    if role == Role::Knight {
        player.properties.grant_intrinsic(crate::player::Property::Jumping);
    }

    // Set initial nutrition
    player.nutrition = 900;
    player.hunger_state = crate::player::HungerState::NotHungry;

    // Initialize inventory (C: role-specific setup + ini_inv calls)
    let mut inventory = Vec::new();
    let mut next_id: u32 = 1;
    let mut letter = b'a';

    // Helper closure to add an item
    let mut add_item = |inv: &mut Vec<Object>, item: &StartingItem, rng: &mut GameRng, next_id: &mut u32, letter: &mut u8| {
        let mut obj = make_starting_object(item, rng, next_id);
        obj.inv_letter = *letter as char;
        if *letter < b'z' {
            *letter += 1;
        }
        inv.push(obj);
    };

    use crate::data::objects::ObjectType;

    // Role-specific pre-init, variable quantities, and ini_inv (C: u_init.c:662-800)
    match role {
        Role::Archeologist => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional extras (C: u_init.c:669-674)
            if rng.rn2(10) == 0 {
                let item = StartingItem::new(ObjectType::TinOpener as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            } else if rng.rn2(4) == 0 {
                let item = StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            } else if rng.rn2(10) == 0 {
                let item = StartingItem::new(ObjectType::MagicMarker as i16, UNDEF_SPE, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Barbarian => {
            // C: 50% chance of battle-axe/short-sword swap (u_init.c:680-683)
            if rng.rn2(100) >= 50 {
                // Use battle-axe + short-sword instead of two-handed sword + axe
                let items: &[StartingItem] = &[
                    StartingItem::new(ObjectType::BattleAxe as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),
                    StartingItem::new(ObjectType::ShortSword as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),
                    StartingItem::new(ObjectType::RingMail as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),
                    StartingItem::new(ObjectType::FoodRation as i16, 0, ObjectClass::Food, 1, 0),
                ];
                for item in items {
                    add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
                }
            } else {
                let base_items = starting_inventory(role);
                for item in base_items {
                    add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
                }
            }
            // Optional lamp (C: u_init.c:685-686)
            if rng.rn2(6) == 0 {
                let item = StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Caveman => {
            // C: variable flint quantity rn1(11,10) = 10..20 (u_init.c:692)
            let flint_qty = (rng.rnd(11) + 9) as u8; // rn1(11,10) = rnd(11)+10-1 = 10..20
            let items: &[StartingItem] = &[
                StartingItem::new(ObjectType::Club as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Sling as i16, 2, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Flint as i16, 0, ObjectClass::Gem, flint_qty, UNDEF_BLESS),
                StartingItem::new(ObjectType::Rock as i16, 0, ObjectClass::Gem, 3, 0),
                StartingItem::new(ObjectType::LeatherArmor as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Healer => {
            // C: gold set before ini_inv (u_init.c:697)
            player.gold = (rng.rnd(1000) + 1000) as i32; // rn1(1000, 1001) = 1001..2000
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional lamp (C: u_init.c:699-700)
            if rng.rn2(25) == 0 {
                let item = StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Knight => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Monk => {
            // C: select spellbook type rn2(90)/30 → [0..2] (u_init.c:715)
            let spell_choices = [ObjectType::Healing, ObjectType::Protection, ObjectType::Sleep];
            let spell_idx = (rng.rn2(90) / 30) as usize;
            let spell_type = spell_choices[spell_idx.min(2)];
            let items: &[StartingItem] = &[
                StartingItem::new(ObjectType::LeatherGloves as i16, 2, ObjectClass::Armor, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Robe as i16, 1, ObjectClass::Armor, 1, UNDEF_BLESS),
                StartingItem::new(spell_type as i16, UNDEF_SPE, ObjectClass::Spellbook, 1, 1),
                StartingItem::new(ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Scroll, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Healing as i16, 0, ObjectClass::Potion, 3, UNDEF_BLESS),
                StartingItem::new(ObjectType::FoodRation as i16, 0, ObjectClass::Food, 3, 0),
                StartingItem::new(ObjectType::Apple as i16, 0, ObjectClass::Food, 5, UNDEF_BLESS),
                StartingItem::new(ObjectType::Orange as i16, 0, ObjectClass::Food, 5, UNDEF_BLESS),
                StartingItem::new(ObjectType::FortuneCookie as i16, 0, ObjectClass::Food, 3, UNDEF_BLESS),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional extras (C: u_init.c:717-720)
            if rng.rn2(5) == 0 {
                let item = StartingItem::new(ObjectType::MagicMarker as i16, UNDEF_SPE, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            } else if rng.rn2(10) == 0 {
                let item = StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Priest => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional extras (C: u_init.c:729-732)
            if rng.rn2(10) == 0 {
                let item = StartingItem::new(ObjectType::MagicMarker as i16, UNDEF_SPE, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            } else if rng.rn2(10) == 0 {
                let item = StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Ranger => {
            // C: variable arrow quantities (u_init.c:744-745)
            let arrow2_qty = (rng.rnd(10) + 49) as u8; // rn1(10, 50) = 50..59
            let arrow0_qty = (rng.rnd(10) + 29) as u8; // rn1(10, 30) = 30..39
            let items: &[StartingItem] = &[
                StartingItem::new(ObjectType::Dagger as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Bow as i16, 1, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Arrow as i16, 2, ObjectClass::Weapon, arrow2_qty, UNDEF_BLESS),
                StartingItem::new(ObjectType::Arrow as i16, 0, ObjectClass::Weapon, arrow0_qty, UNDEF_BLESS),
                StartingItem::new(ObjectType::CloakOfDisplacement as i16, 2, ObjectClass::Armor, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::CramRation as i16, 0, ObjectClass::Food, 4, 0),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Rogue => {
            // C: variable dagger quantity rn1(10,6) = 6..15 (u_init.c:750)
            let dagger_qty = (rng.rnd(10) + 5) as u8; // rn1(10,6) = rnd(10)+6-1 = 6..15
            let items: &[StartingItem] = &[
                StartingItem::new(ObjectType::ShortSword as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Dagger as i16, 0, ObjectClass::Weapon, dagger_qty, 0),
                StartingItem::new(ObjectType::LeatherArmor as i16, 1, ObjectClass::Armor, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Sickness as i16, 0, ObjectClass::Potion, 1, 0),
                StartingItem::new(ObjectType::LockPick as i16, 0, ObjectClass::Tool, 1, 0),
                StartingItem::new(ObjectType::Sack as i16, 0, ObjectClass::Tool, 1, 0),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional blindfold (C: u_init.c:753-754)
            if rng.rn2(5) == 0 {
                let item = StartingItem::new(ObjectType::Blindfold as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Samurai => {
            // C: variable ya quantity rn1(20,26) = 26..45 (u_init.c:759)
            let ya_qty = (rng.rnd(20) + 25) as u8; // rn1(20,26) = rnd(20)+26-1 = 26..45
            let items: &[StartingItem] = &[
                StartingItem::new(ObjectType::Katana as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::ShortSword as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Yumi as i16, 0, ObjectClass::Weapon, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::Ya as i16, 0, ObjectClass::Weapon, ya_qty, UNDEF_BLESS),
                StartingItem::new(ObjectType::SplintMail as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional blindfold (C: u_init.c:761-762)
            if rng.rn2(5) == 0 {
                let item = StartingItem::new(ObjectType::Blindfold as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Tourist => {
            // C: variable dart quantity rn1(20,21) = 21..40 (u_init.c:768)
            let dart_qty = (rng.rnd(20) + 20) as u8; // rn1(20,21) = rnd(20)+21-1 = 21..40
            // C: gold rnd(1000) (u_init.c:769)
            player.gold = rng.rnd(1000) as i32;
            let items: &[StartingItem] = &[
                StartingItem::new(ObjectType::Dart as i16, 2, ObjectClass::Weapon, dart_qty, UNDEF_BLESS),
                StartingItem::new(ObjectType::StrangeObject as i16, UNDEF_SPE, ObjectClass::Food, 10, 0),
                StartingItem::new(ObjectType::ExtraHealing as i16, 0, ObjectClass::Potion, 2, UNDEF_BLESS),
                StartingItem::new(ObjectType::MagicMapping as i16, 0, ObjectClass::Scroll, 4, UNDEF_BLESS),
                StartingItem::new(ObjectType::HawaiianShirt as i16, 0, ObjectClass::Armor, 1, UNDEF_BLESS),
                StartingItem::new(ObjectType::ExpensiveCamera as i16, UNDEF_SPE, ObjectClass::Tool, 1, 0),
                StartingItem::new(ObjectType::CreditCard as i16, 0, ObjectClass::Tool, 1, 0),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional extras (C: u_init.c:771-778)
            if rng.rn2(25) == 0 {
                let item = StartingItem::new(ObjectType::TinOpener as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            } else if rng.rn2(25) == 0 {
                let item = StartingItem::new(ObjectType::Leash as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            } else if rng.rn2(25) == 0 {
                let item = StartingItem::new(ObjectType::Towel as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            } else if rng.rn2(25) == 0 {
                let item = StartingItem::new(ObjectType::MagicMarker as i16, UNDEF_SPE, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Valkyrie => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional lamp (C: u_init.c:783-784)
            if rng.rn2(6) == 0 {
                let item = StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
        Role::Wizard => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter);
            }
            // Optional extras (C: u_init.c:791-794)
            if rng.rn2(5) == 0 {
                let item = StartingItem::new(ObjectType::MagicMarker as i16, UNDEF_SPE, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
            if rng.rn2(5) == 0 {
                let item = StartingItem::new(ObjectType::Blindfold as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter);
            }
        }
    }

    // Roll attributes (C: init_attr(75)) - happens after inventory in C
    roll_attributes(player, rng);

    // Auto-equip starting items (C: u_init.c:1114-1146)
    auto_equip_starting_inventory(&mut inventory);

    inventory
}

/// Auto-equip starting inventory items (C: u_init.c:1114-1146).
///
/// Sets worn_mask on inventory items based on their type:
/// - Armor: equipped to appropriate slot (shield, helmet, gloves, etc.)
/// - Weapons: first weapon wielded, second becomes swap weapon, ammo quivered
fn auto_equip_starting_inventory(inventory: &mut [Object]) {
    use crate::action::wear::worn_mask::*;
    use crate::data::objects::OBJECTS;
    use crate::object::ArmorCategory;

    let mut has_wep = false;
    let mut has_swapwep = false;
    let mut has_quiver = false;
    let mut has_shield = false;

    for obj in inventory.iter_mut() {
        let otyp = obj.object_type as usize;
        if otyp >= OBJECTS.len() {
            continue;
        }
        let def = &OBJECTS[otyp];

        // Armor auto-equip (C: u_init.c:1114-1133)
        if obj.class == ObjectClass::Armor {
            if let Some(cat) = def.armor_category {
                let mask = match cat {
                    ArmorCategory::Shield => {
                        if !has_shield {
                            has_shield = true;
                            W_ARMS
                        } else {
                            0
                        }
                    }
                    ArmorCategory::Helm => W_ARMH,
                    ArmorCategory::Gloves => W_ARMG,
                    ArmorCategory::Shirt => W_ARMU,
                    ArmorCategory::Cloak => W_ARMC,
                    ArmorCategory::Boots => W_ARMF,
                    ArmorCategory::Suit => W_ARM,
                };
                if mask != 0 {
                    obj.worn_mask = mask;
                }
            }
            continue;
        }

        // Weapon auto-equip (C: u_init.c:1136-1146)
        if obj.class == ObjectClass::Weapon {
            // Check if this is ammo (negative skill value in C means ammo for that launcher)
            // Simplified: arrows, bolts, darts, sling bullets, shuriken, boomerangs = quiver
            let is_ammo_like = def.skill < 0; // Negative skill = ammo type
            if is_ammo_like {
                if !has_quiver {
                    obj.worn_mask = W_QUIVER;
                    has_quiver = true;
                }
            } else if !has_wep {
                obj.worn_mask = W_WEP;
                has_wep = true;
            } else if !has_swapwep {
                obj.worn_mask = W_SWAPWEP;
                has_swapwep = true;
            }
        }
    }
}

/// Check if a spell discipline is restricted for the player's role (C: restricted_spell_discipline)
pub fn restricted_spell_discipline(role: Role, skill: SkillType) -> bool {
    let table = skill_table_for_role(role);
    if !skill.is_spell() {
        return false;
    }
    !table.iter().any(|&(st, _)| st == skill)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Race;

    #[test]
    fn test_starting_inventory_all_roles() {
        for role in [
            Role::Archeologist, Role::Barbarian, Role::Caveman, Role::Healer,
            Role::Knight, Role::Monk, Role::Priest, Role::Ranger,
            Role::Rogue, Role::Samurai, Role::Tourist, Role::Valkyrie, Role::Wizard,
        ] {
            let items = starting_inventory(role);
            assert!(!items.is_empty(), "No inventory for {:?}", role);
            // Verify all items have valid classes
            for item in items {
                assert!(item.quantity > 0, "Zero quantity for {:?} item", role);
            }
        }
    }

    #[test]
    fn test_init_inventory_creates_objects() {
        let mut rng = GameRng::new(42);
        let inventory = init_inventory(&mut rng, Role::Valkyrie);
        assert_eq!(inventory.len(), VALKYRIE_INV.len());
        // Check letters are sequential
        for (i, obj) in inventory.iter().enumerate() {
            assert_eq!(obj.inv_letter, (b'a' + i as u8) as char);
        }
    }

    #[test]
    fn test_init_skills_barbarian() {
        let mut skills = SkillSet::default();
        init_skills(&mut skills, Role::Barbarian);
        // Barbarian should have Expert in two-handed sword
        assert_eq!(
            skills.get(SkillType::TwoHandedSword).max_level,
            SkillLevel::Expert
        );
        // And Master in bare-handed
        assert_eq!(
            skills.get(SkillType::BareHanded).max_level,
            SkillLevel::Master
        );
        // Unskilled initially (not restricted)
        assert_eq!(
            skills.get(SkillType::BareHanded).level,
            SkillLevel::Unskilled
        );
    }

    #[test]
    fn test_init_skills_wizard() {
        let mut skills = SkillSet::default();
        init_skills(&mut skills, Role::Wizard);
        assert_eq!(
            skills.get(SkillType::AttackSpells).max_level,
            SkillLevel::Expert
        );
        assert_eq!(
            skills.get(SkillType::MatterSpells).max_level,
            SkillLevel::Expert
        );
        // Riding should still be restricted
        assert_eq!(
            skills.get(SkillType::Riding).max_level,
            SkillLevel::Restricted
        );
    }

    #[test]
    fn test_init_skills_monk_grandmaster() {
        let mut skills = SkillSet::default();
        init_skills(&mut skills, Role::Monk);
        assert_eq!(
            skills.get(SkillType::BareHanded).max_level,
            SkillLevel::GrandMaster
        );
    }

    #[test]
    fn test_u_init_hp_varies_by_role() {
        let mut rng = GameRng::new(42);
        let mut wizard = crate::player::You::new("Test".into(), Role::Wizard, Race::Human, crate::player::Gender::Male);
        let mut barb = crate::player::You::new("Test".into(), Role::Barbarian, Race::Human, crate::player::Gender::Male);
        u_init(&mut wizard, &mut rng);
        u_init(&mut barb, &mut rng);
        assert!(barb.hp_max > wizard.hp_max);
    }

    #[test]
    fn test_u_init_energy_varies_by_role() {
        let mut rng = GameRng::new(42);
        let mut wizard = crate::player::You::new("Test".into(), Role::Wizard, Race::Human, crate::player::Gender::Male);
        let mut barb = crate::player::You::new("Test".into(), Role::Barbarian, Race::Human, crate::player::Gender::Male);
        u_init(&mut wizard, &mut rng);
        u_init(&mut barb, &mut rng);
        assert!(wizard.energy_max > barb.energy_max);
    }

    #[test]
    fn test_u_init_knight_gets_jumping() {
        let mut rng = GameRng::new(42);
        let mut knight = crate::player::You::new("Test".into(), Role::Knight, Race::Human, crate::player::Gender::Male);
        u_init(&mut knight, &mut rng);
        assert!(knight.properties.has(crate::player::Property::Jumping));
    }

    #[test]
    fn test_u_init_bless_count() {
        let mut rng = GameRng::new(42);
        let mut player = crate::player::You::new("Test".into(), Role::Valkyrie, Race::Human, crate::player::Gender::Male);
        u_init(&mut player, &mut rng);
        assert_eq!(player.bless_count, 300);
    }

    #[test]
    fn test_u_init_healer_gold() {
        let mut rng = GameRng::new(42);
        let mut healer = crate::player::You::new("Test".into(), Role::Healer, Race::Human, crate::player::Gender::Male);
        u_init(&mut healer, &mut rng);
        assert!(healer.gold >= 1001, "Healer gold should be 1001..2000, got {}", healer.gold);
    }

    #[test]
    fn test_restricted_spell_discipline_wizard() {
        // Wizard has Attack, Healing, Divination, Enchantment, Clerical, Escape, Matter
        assert!(!restricted_spell_discipline(Role::Wizard, SkillType::AttackSpells));
        assert!(!restricted_spell_discipline(Role::Wizard, SkillType::MatterSpells));
        // Non-spell skills always return false
        assert!(!restricted_spell_discipline(Role::Wizard, SkillType::Dagger));
    }

    #[test]
    fn test_restricted_spell_discipline_barbarian() {
        // Barbarian has no spell skills in table
        assert!(restricted_spell_discipline(Role::Barbarian, SkillType::AttackSpells));
        assert!(restricted_spell_discipline(Role::Barbarian, SkillType::HealingSpells));
    }

    #[test]
    fn test_make_starting_object_blessed() {
        let mut rng = GameRng::new(42);
        let mut next_id = 1;
        let item = StartingItem::new(crate::data::objects::ObjectType::Mace as i16, 1, ObjectClass::Weapon, 1, 1); // blessed mace
        let obj = make_starting_object(&item, &mut rng, &mut next_id);
        assert_eq!(obj.buc, BucStatus::Blessed);
        assert_eq!(obj.enchantment, 1);
    }

    #[test]
    fn test_make_starting_object_uncursed() {
        let mut rng = GameRng::new(42);
        let mut next_id = 1;
        let item = StartingItem::new(crate::data::objects::ObjectType::Light as i16, 0, ObjectClass::Food, 3, 0); // uncursed food
        let obj = make_starting_object(&item, &mut rng, &mut next_id);
        assert_eq!(obj.buc, BucStatus::Uncursed);
        assert_eq!(obj.quantity, 3);
    }

    #[test]
    fn test_skill_tables_complete() {
        // Every role should have at least 4 skill entries
        for role in [
            Role::Archeologist, Role::Barbarian, Role::Caveman, Role::Healer,
            Role::Knight, Role::Monk, Role::Priest, Role::Ranger,
            Role::Rogue, Role::Samurai, Role::Tourist, Role::Valkyrie, Role::Wizard,
        ] {
            let table = skill_table_for_role(role);
            assert!(table.len() >= 4, "Too few skills for {:?}: {}", role, table.len());
        }
    }
}
