//! Player initialization (u_init.c)
//!
//! Sets up the player's starting inventory, skills, and attributes
//! based on their role and race. Called at character creation.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::{BucStatus, Object, ObjectClass};
use crate::player::{Attribute, Attributes, Race, Role, SkillLevel, SkillSet, SkillType, You};
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
        Self {
            otyp,
            spe,
            class,
            quantity,
            bless,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Role starting inventories
// ─────────────────────────────────────────────────────────────────────────────

/// Archeologist starting inventory
static ARCHEOLOGIST_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Bullwhip as i16,
        2,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // BULLWHIP
    StartingItem::new(
        crate::data::objects::ObjectType::LeatherJacket as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // LEATHER_JACKET
    StartingItem::new(
        crate::data::objects::ObjectType::Fedora as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // FEDORA
    StartingItem::new(
        crate::data::objects::ObjectType::FoodRation as i16,
        0,
        ObjectClass::Food,
        3,
        0,
    ), // FOOD_RATION
    StartingItem::new(
        crate::data::objects::ObjectType::PickAxe as i16,
        UNDEF_SPE,
        ObjectClass::Tool,
        1,
        UNDEF_BLESS,
    ), // PICK_AXE
    StartingItem::new(
        crate::data::objects::ObjectType::TinningKit as i16,
        UNDEF_SPE,
        ObjectClass::Tool,
        1,
        UNDEF_BLESS,
    ), // TINNING_KIT
    StartingItem::new(
        crate::data::objects::ObjectType::Touchstone as i16,
        0,
        ObjectClass::Gem,
        1,
        0,
    ), // TOUCHSTONE
    StartingItem::new(
        crate::data::objects::ObjectType::Sack as i16,
        0,
        ObjectClass::Tool,
        1,
        0,
    ), // SACK
];

/// Barbarian starting inventory
static BARBARIAN_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::TwoHandedSword as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // TWO_HANDED_SWORD
    StartingItem::new(
        crate::data::objects::ObjectType::Axe as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // AXE
    StartingItem::new(
        crate::data::objects::ObjectType::RingMail as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // RING_MAIL
    StartingItem::new(
        crate::data::objects::ObjectType::FoodRation as i16,
        0,
        ObjectClass::Food,
        1,
        0,
    ), // FOOD_RATION
];

/// Caveman starting inventory
static CAVEMAN_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Club as i16,
        1,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // CLUB
    StartingItem::new(
        crate::data::objects::ObjectType::Sling as i16,
        2,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // SLING
    StartingItem::new(
        crate::data::objects::ObjectType::Flint as i16,
        0,
        ObjectClass::Gem,
        15,
        UNDEF_BLESS,
    ), // FLINT (qty variable)
    StartingItem::new(
        crate::data::objects::ObjectType::Rock as i16,
        0,
        ObjectClass::Gem,
        3,
        0,
    ), // ROCK
    StartingItem::new(
        crate::data::objects::ObjectType::LeatherArmor as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // LEATHER_ARMOR
];

/// Healer starting inventory
static HEALER_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Scalpel as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // SCALPEL
    StartingItem::new(
        crate::data::objects::ObjectType::LeatherGloves as i16,
        1,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // LEATHER_GLOVES
    StartingItem::new(
        crate::data::objects::ObjectType::Stethoscope as i16,
        0,
        ObjectClass::Tool,
        1,
        0,
    ), // STETHOSCOPE
    StartingItem::new(
        crate::data::objects::ObjectType::PotionHealing as i16,
        0,
        ObjectClass::Potion,
        4,
        UNDEF_BLESS,
    ), // POT_HEALING
    StartingItem::new(
        crate::data::objects::ObjectType::PotionExtraHealing as i16,
        0,
        ObjectClass::Potion,
        4,
        UNDEF_BLESS,
    ), // POT_EXTRA_HEALING
    StartingItem::new(
        crate::data::objects::ObjectType::Sleep as i16,
        UNDEF_SPE,
        ObjectClass::Wand,
        1,
        UNDEF_BLESS,
    ), // WAN_SLEEP
    StartingItem::new(
        crate::data::objects::ObjectType::Healing as i16,
        0,
        ObjectClass::Spellbook,
        1,
        1,
    ), // SPE_HEALING
    StartingItem::new(
        crate::data::objects::ObjectType::ExtraHealing as i16,
        0,
        ObjectClass::Spellbook,
        1,
        1,
    ), // SPE_EXTRA_HEALING
    StartingItem::new(
        crate::data::objects::ObjectType::StoneToFlesh as i16,
        0,
        ObjectClass::Spellbook,
        1,
        1,
    ), // SPE_STONE_TO_FLESH
    StartingItem::new(
        crate::data::objects::ObjectType::Apple as i16,
        0,
        ObjectClass::Food,
        5,
        0,
    ), // APPLE
];

/// Knight starting inventory
static KNIGHT_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::LongSword as i16,
        1,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // LONG_SWORD
    StartingItem::new(
        crate::data::objects::ObjectType::Lance as i16,
        1,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // LANCE
    StartingItem::new(
        crate::data::objects::ObjectType::RingMail as i16,
        1,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // RING_MAIL
    StartingItem::new(
        crate::data::objects::ObjectType::Helmet as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // HELMET
    StartingItem::new(
        crate::data::objects::ObjectType::SmallShield as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // SMALL_SHIELD
    StartingItem::new(
        crate::data::objects::ObjectType::LeatherGloves as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // LEATHER_GLOVES
    StartingItem::new(
        crate::data::objects::ObjectType::Apple as i16,
        0,
        ObjectClass::Food,
        10,
        0,
    ), // APPLE
    StartingItem::new(
        crate::data::objects::ObjectType::Carrot as i16,
        0,
        ObjectClass::Food,
        10,
        0,
    ), // CARROT
];

/// Monk starting inventory
static MONK_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::LeatherGloves as i16,
        2,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // LEATHER_GLOVES
    StartingItem::new(
        crate::data::objects::ObjectType::Robe as i16,
        1,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // ROBE
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Spellbook,
        1,
        1,
    ), // Random spellbook
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Scroll,
        1,
        UNDEF_BLESS,
    ), // Random scroll
    StartingItem::new(
        crate::data::objects::ObjectType::PotionHealing as i16,
        0,
        ObjectClass::Potion,
        3,
        UNDEF_BLESS,
    ), // POT_HEALING
    StartingItem::new(
        crate::data::objects::ObjectType::FoodRation as i16,
        0,
        ObjectClass::Food,
        3,
        0,
    ), // FOOD_RATION
    StartingItem::new(
        crate::data::objects::ObjectType::Apple as i16,
        0,
        ObjectClass::Food,
        5,
        UNDEF_BLESS,
    ), // APPLE
    StartingItem::new(
        crate::data::objects::ObjectType::Orange as i16,
        0,
        ObjectClass::Food,
        5,
        UNDEF_BLESS,
    ), // ORANGE
    StartingItem::new(
        crate::data::objects::ObjectType::FortuneCookie as i16,
        0,
        ObjectClass::Food,
        3,
        UNDEF_BLESS,
    ), // FORTUNE_COOKIE
];

/// Priest starting inventory
static PRIEST_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Mace as i16,
        1,
        ObjectClass::Weapon,
        1,
        1,
    ), // MACE (blessed)
    StartingItem::new(
        crate::data::objects::ObjectType::Robe as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // ROBE
    StartingItem::new(
        crate::data::objects::ObjectType::SmallShield as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // SMALL_SHIELD
    StartingItem::new(
        crate::data::objects::ObjectType::Water as i16,
        0,
        ObjectClass::Potion,
        4,
        1,
    ), // POT_WATER (holy)
    StartingItem::new(
        crate::data::objects::ObjectType::CloveOfGarlic as i16,
        0,
        ObjectClass::Food,
        1,
        0,
    ), // CLOVE_OF_GARLIC
    StartingItem::new(
        crate::data::objects::ObjectType::SprigOfWolfsbane as i16,
        0,
        ObjectClass::Food,
        1,
        0,
    ), // SPRIG_OF_WOLFSBANE
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Spellbook,
        2,
        UNDEF_BLESS,
    ), // Random spellbooks
];

/// Ranger starting inventory
static RANGER_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Dagger as i16,
        1,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // DAGGER
    StartingItem::new(
        crate::data::objects::ObjectType::Bow as i16,
        1,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // BOW
    StartingItem::new(
        crate::data::objects::ObjectType::Arrow as i16,
        2,
        ObjectClass::Weapon,
        50,
        UNDEF_BLESS,
    ), // ARROW (qty variable)
    StartingItem::new(
        crate::data::objects::ObjectType::Arrow as i16,
        0,
        ObjectClass::Weapon,
        30,
        UNDEF_BLESS,
    ), // ARROW
    StartingItem::new(
        crate::data::objects::ObjectType::CloakOfDisplacement as i16,
        2,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // CLOAK_OF_DISPLACEMENT
    StartingItem::new(
        crate::data::objects::ObjectType::CramRation as i16,
        0,
        ObjectClass::Food,
        4,
        0,
    ), // CRAM_RATION
];

/// Rogue starting inventory
static ROGUE_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::ShortSword as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // SHORT_SWORD
    StartingItem::new(
        crate::data::objects::ObjectType::Dagger as i16,
        0,
        ObjectClass::Weapon,
        10,
        0,
    ), // DAGGER (qty variable)
    StartingItem::new(
        crate::data::objects::ObjectType::LeatherArmor as i16,
        1,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // LEATHER_ARMOR
    StartingItem::new(
        crate::data::objects::ObjectType::Sickness as i16,
        0,
        ObjectClass::Potion,
        1,
        0,
    ), // POT_SICKNESS
    StartingItem::new(
        crate::data::objects::ObjectType::LockPick as i16,
        0,
        ObjectClass::Tool,
        1,
        0,
    ), // LOCK_PICK
    StartingItem::new(
        crate::data::objects::ObjectType::Sack as i16,
        0,
        ObjectClass::Tool,
        1,
        0,
    ), // SACK
];

/// Samurai starting inventory
static SAMURAI_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Katana as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // KATANA
    StartingItem::new(
        crate::data::objects::ObjectType::ShortSword as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // SHORT_SWORD (wakizashi)
    StartingItem::new(
        crate::data::objects::ObjectType::Yumi as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // YUMI
    StartingItem::new(
        crate::data::objects::ObjectType::Ya as i16,
        0,
        ObjectClass::Weapon,
        25,
        UNDEF_BLESS,
    ), // YA (qty variable)
    StartingItem::new(
        crate::data::objects::ObjectType::SplintMail as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // SPLINT_MAIL
];

/// Tourist starting inventory
static TOURIST_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Dart as i16,
        2,
        ObjectClass::Weapon,
        25,
        UNDEF_BLESS,
    ), // DART (qty variable)
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Food,
        10,
        0,
    ), // Random food
    StartingItem::new(
        crate::data::objects::ObjectType::PotionExtraHealing as i16,
        0,
        ObjectClass::Potion,
        2,
        UNDEF_BLESS,
    ), // POT_EXTRA_HEALING
    StartingItem::new(
        crate::data::objects::ObjectType::MagicMapping as i16,
        0,
        ObjectClass::Scroll,
        4,
        UNDEF_BLESS,
    ), // SCR_MAGIC_MAPPING
    StartingItem::new(
        crate::data::objects::ObjectType::HawaiianShirt as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // HAWAIIAN_SHIRT
    StartingItem::new(
        crate::data::objects::ObjectType::ExpensiveCamera as i16,
        UNDEF_SPE,
        ObjectClass::Tool,
        1,
        0,
    ), // EXPENSIVE_CAMERA
    StartingItem::new(
        crate::data::objects::ObjectType::CreditCard as i16,
        0,
        ObjectClass::Tool,
        1,
        0,
    ), // CREDIT_CARD
];

/// Valkyrie starting inventory
static VALKYRIE_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::LongSword as i16,
        1,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // LONG_SWORD
    StartingItem::new(
        crate::data::objects::ObjectType::Dagger as i16,
        0,
        ObjectClass::Weapon,
        1,
        UNDEF_BLESS,
    ), // DAGGER
    StartingItem::new(
        crate::data::objects::ObjectType::SmallShield as i16,
        3,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // SMALL_SHIELD
    StartingItem::new(
        crate::data::objects::ObjectType::FoodRation as i16,
        0,
        ObjectClass::Food,
        1,
        0,
    ), // FOOD_RATION
];

/// Wizard starting inventory
static WIZARD_INV: &[StartingItem] = &[
    StartingItem::new(
        crate::data::objects::ObjectType::Quarterstaff as i16,
        1,
        ObjectClass::Weapon,
        1,
        1,
    ), // QUARTERSTAFF (blessed)
    StartingItem::new(
        crate::data::objects::ObjectType::CloakOfMagicResistance as i16,
        0,
        ObjectClass::Armor,
        1,
        UNDEF_BLESS,
    ), // CLOAK_OF_MAGIC_RESISTANCE
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Wand,
        1,
        UNDEF_BLESS,
    ), // Random wand
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Ring,
        2,
        UNDEF_BLESS,
    ), // Random rings
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Potion,
        3,
        UNDEF_BLESS,
    ), // Random potions
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Scroll,
        3,
        UNDEF_BLESS,
    ), // Random scrolls
    StartingItem::new(
        crate::data::objects::ObjectType::ForceBolt as i16,
        0,
        ObjectClass::Spellbook,
        1,
        1,
    ), // SPE_FORCE_BOLT
    StartingItem::new(
        crate::data::objects::ObjectType::StrangeObject as i16,
        UNDEF_SPE,
        ObjectClass::Spellbook,
        1,
        UNDEF_BLESS,
    ), // Random spellbook
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

/// Burn the same RNG calls that C's mksobj(otyp, init=TRUE, artif=FALSE) makes.
///
/// In C, ini_inv calls mksobj which generates random enchantment/BUC/quantity
/// for every item, then ini_inv overwrites most values from the trobj table.
/// Those "phantom" RNG calls still consume RNG state. We must replicate them
/// so the Rust RNG stays in sync with C after initialization.
///
/// Returns the enchantment and blessed values that mksobj would have produced,
/// which ini_inv may or may not keep (depending on trspe/trbless).
fn mksobj_phantom_rng(class: ObjectClass, otyp: i16, rng: &mut GameRng) -> (i8, i8) {
    let mut spe: i8 = 0;
    let mut blessed: i8 = 0;

    match class {
        ObjectClass::Weapon => {
            let is_multigen = if let Some(def) = crate::data::objects::OBJECTS.get(otyp as usize) {
                use crate::data::objects::{P_BOW, P_SHURIKEN};
                def.skill >= -P_SHURIKEN && def.skill <= -P_BOW
            } else {
                false
            };
            if is_multigen {
                let _quan = rng.rnd(6) + 5;
            }
            if rng.rn2(11) == 0 {
                spe = rne(rng, 3) as i8;
                blessed = rng.rn2(2) as i8;
            } else if rng.rn2(10) == 0 {
                spe = -(rne(rng, 3) as i8);
            } else if rng.rn2(10) == 0 {
                blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
            }
            if is_multigen {
                let _poison = rng.rn2(100);
            }
        }
        ObjectClass::Food => {
            // Food items: FOOD_RATION is not CORPSE/EGG/TIN/SLIME_MOLD/KELP_FROND
            // so the switch body is skipped. Then:
            // if (otyp != CORPSE && otyp != MEAT_RING && otyp != KELP_FROND && !rn2(6))
            //     quan = 2;
            let _double = rng.rn2(6);
        }
        ObjectClass::Armor => {
            // C: if (rn2(10) && (special || !rn2(11))) { curse; spe=-rne(3) }
            //    else if (!rn2(10)) { blessed=rn2(2); spe=rne(3) }
            //    else blessorcurse(10)
            use crate::data::objects::ObjectType;
            let is_special = matches!(
                otyp,
                x if x == ObjectType::FumbleBoots as i16
                    || x == ObjectType::LevitationBoots as i16
                    || x == ObjectType::HelmOfOppositeAlignment as i16
                    || x == ObjectType::GauntletsOfFumbling as i16
            );

            let r1 = rng.rn2(10);
            // C: short-circuit &&: if r1==0, second part not evaluated
            let first_branch = if r1 != 0 {
                if is_special {
                    true // special armor → always curse
                } else {
                    rng.rn2(11) == 0 // !rn2(11)
                }
            } else {
                false
            };

            if first_branch {
                // curse + negative enchant; C: curse(otmp) → blessed=0,cursed=1
                spe = -(rne(rng, 3) as i8);
                blessed = -1;
            } else if rng.rn2(10) == 0 {
                // C: blessed = rn2(2); spe = rne(3) (no curse)
                blessed = rng.rn2(2) as i8;
                spe = rne(rng, 3) as i8;
            } else {
                // C: blessorcurse(10) — if rn2(10)==0: rn2(2) ? bless : curse
                if rng.rn2(10) == 0 {
                    blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
                }
            }
            // artif=FALSE, no rn2(40)
        }
        ObjectClass::Gem => {
            // For starting items: TOUCHSTONE, FLINT, ROCK
            // ROCK: rn1(6,6) = rnd(6)+5
            // FLINT (not LUCKSTONE): rn2(6) for double quantity
            // TOUCHSTONE: oc_name check... touchstone IS a LUCKSTONE? No.
            // C: if (otyp == LOADSTONE) curse
            //    else if (otyp == ROCK) quan = rn1(6,6)
            //    else if (otyp != LUCKSTONE && !rn2(6)) quan = 2
            //    else quan = 1
            use crate::data::objects::ObjectType;
            if otyp == ObjectType::Rock as i16 {
                let _quan = rng.rnd(6) + 5; // rn1(6,6)
            } else if otyp != ObjectType::Luckstone as i16 {
                let _double = rng.rn2(6);
            }
        }
        ObjectClass::Tool => {
            // Tools have specific init per otyp
            use crate::data::objects::ObjectType;
            if otyp == ObjectType::TinningKit as i16
                || otyp == ObjectType::ExpensiveCamera as i16
                || otyp == ObjectType::MagicMarker as i16
            {
                spe = (rng.rnd(70) + 29) as i8; // rn1(70,30)
            } else if otyp == ObjectType::OilLamp as i16 {
                // C: spe=1, age=rn1(500,1000), blessorcurse(5)
                let _age = rng.rnd(500) + 999; // rn1(500,1000)
                if rng.rn2(5) == 0 {
                    blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
                }
            }
            // SACK: mkbox_cnts — may make RNG calls for box contents
            // For starting SACK, mkbox_cnts with init box usually empty
            // Skip for now — SACK contents generation is complex
        }
        ObjectClass::Wand => {
            // C mksobj.c: spe = rn1(5, nodir ? 11 : 4); WAN_WISHING uses rnd(3).
            // Direction comes from objects[otyp].oc_dir == NODIR.
            const WAN_WISHING: i16 = 387;
            if otyp == WAN_WISHING {
                spe = rng.rnd(3) as i8;
            } else {
                let nodir = crate::data::objects::OBJECTS
                    .get(otyp as usize)
                    .map(|d| d.direction == crate::object::DirectionType::None)
                    .unwrap_or(false);
                let bias = if nodir { 11 } else { 4 };
                spe = (rng.rnd(5) + (bias - 1)) as i8;
            }
            // blessorcurse(17)
            if rng.rn2(17) == 0 {
                blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
            }
        }
        ObjectClass::Potion | ObjectClass::Scroll
            // blessorcurse(4)
            if rng.rn2(4) == 0 => {
                blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
            }
        ObjectClass::Spellbook
            // blessorcurse(17)
            if rng.rn2(17) == 0 => {
                blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
            }
        ObjectClass::Ring => {
            // C mkobj.c:1028-1051. Two paths based on oc_charged:
            //   charged (6 rings: adornment, gain strength, gain con,
            //            increase accuracy, increase damage, protection):
            //     blessorcurse(3); rn2(10); if non-zero: rn2(10) and either
            //     bcsign-rne or rn2(2)+rne; if spe==0: rn2(4)+rn2(3);
            //     if spe<0: rn2(5).
            //   non-charged (22 rings):
            //     if rn2(10) && otyp in {TELEPORT, POLYMORPH, AGGRAVATE,
            //                            HUNGER}: curse;
            //     else blessorcurse(10).
            const FIRST_RING: i16 = 150;
            const FIRST_NON_CHARGED_RING: i16 = 156;
            let oc_charged = otyp >= FIRST_RING && otyp < FIRST_NON_CHARGED_RING;
            if oc_charged {
                // blessorcurse(3): rn2(3); if 0: rn2(2)
                let bc_outer = rng.rn2(3);
                let mut blessed_local = 0i8;
                let mut cursed_local = false;
                if bc_outer == 0 {
                    if rng.rn2(2) == 0 {
                        cursed_local = true;
                    } else {
                        blessed_local = 1;
                    }
                }
                let mut spe_local = 0i32;
                if rng.rn2(10) != 0 {
                    if rng.rn2(10) != 0 && (blessed_local != 0 || cursed_local) {
                        let bcsign = if blessed_local != 0 { 1 } else { -1 };
                        spe_local = bcsign * rne(rng, 3) as i32;
                    } else {
                        spe_local = if rng.rn2(2) != 0 {
                            rne(rng, 3) as i32
                        } else {
                            -(rne(rng, 3) as i32)
                        };
                    }
                }
                if spe_local == 0 {
                    spe_local = rng.rn2(4) as i32 - rng.rn2(3) as i32;
                }
                if spe_local < 0 && rng.rn2(5) != 0 {
                    cursed_local = true;
                }
                spe = spe_local as i8;
                blessed = if cursed_local { -1 } else { blessed_local };
            } else {
                const RIN_HUNGER: i16 = 161;
                const RIN_AGGRAVATE_MONSTER: i16 = 162;
                const RIN_TELEPORTATION: i16 = 171;
                const RIN_POLYMORPH: i16 = 173;
                let outer = rng.rn2(10);
                let curse_otype = matches!(
                    otyp,
                    RIN_TELEPORTATION | RIN_POLYMORPH | RIN_AGGRAVATE_MONSTER | RIN_HUNGER
                );
                if outer != 0 && curse_otype {
                    blessed = -1;
                } else if rng.rn2(10) == 0 {
                    blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
                }
            }
        }
        ObjectClass::Amulet => {
            // C mkobj.c:967-976: rn2(10) + matches strangulation/change/sleep
            // → curse, else blessorcurse(10).
            const AMULET_OF_STRANGULATION: i16 = 180;
            const AMULET_OF_CHANGE: i16 = 183;
            const AMULET_OF_RESTFUL_SLEEP: i16 = 181;
            let outer = rng.rn2(10);
            let curse_otype = matches!(
                otyp,
                AMULET_OF_STRANGULATION | AMULET_OF_CHANGE | AMULET_OF_RESTFUL_SLEEP
            );
            if outer != 0 && curse_otype {
                blessed = -1;
            } else if rng.rn2(10) == 0 {
                blessed = if rng.rn2(2) == 0 { -1 } else { 1 };
            }
        }
        _ => {}
    }

    (spe, blessed)
}

/// C's rne(x): exponential distribution, min 1 RNG call
fn rne(rng: &mut GameRng, x: u32) -> u32 {
    // C: utmp = (u.ulevel < 15) ? 5 : u.ulevel / 3
    // At game start, ulevel=1, so utmp=5
    let utmp = 5u32;
    let mut tmp = 1u32;
    while tmp < utmp && rng.rn2(x) == 0 {
        tmp += 1;
    }
    tmp
}

/// Cross-call reroll state for `make_starting_object`, mirroring C's static
/// `nocreate{,2,3,4}` in u_init.c:1007-1010. A picked polymorph item bans
/// its dual on the next pick (so the player never starts with both
/// polymorph and polymorph-control); rings/spellbooks ban repeats of
/// themselves (no two of the same ring/spellbook).
#[derive(Default, Clone, Copy)]
pub struct StartingInvRerollState {
    pub nocreate: i16,
    pub nocreate2: i16,
    pub nocreate3: i16,
    pub nocreate4: i16,
}

/// Convert a starting item descriptor into an Object (C: ini_inv per-item logic).
///
/// `role` and `race` drive the role/race-specific forbidden-item exclusions in
/// the random-otype reroll loop (C u_init.c:1023-1047). `reroll_state` carries
/// the cross-call nocreate1-4 state.
pub fn make_starting_object_full(
    item: &StartingItem,
    role: Role,
    race: Race,
    rng: &mut GameRng,
    next_id: &mut u32,
    reroll_state: &mut StartingInvRerollState,
) -> Object {
    let id = *next_id;
    *next_id += 1;

    use crate::data::objects::ObjectType;
    // C: mkobj(class) calls rnd(1000) + the full per-class init in a single
    // function call. If the result is forbidden, dealloc and call mkobj
    // again — meaning each reroll consumes BOTH rnd(1000) and the per-class
    // RNG block. We mirror that exactly: call select_object_type AND
    // mksobj_phantom_rng on every iteration. Only the LAST iteration's
    // (spe, blessed) values are kept for the final object.
    let trace_obj = std::env::var("NH_TRACE_INIT").is_ok();
    let (resolved_otyp, mksobj_spe, mksobj_blessed) =
        if item.otyp == ObjectType::StrangeObject as i16 {
            let bases = crate::object::ClassBases::compute(crate::data::objects::OBJECTS);
            let mut last_otyp = item.otyp;
            let mut last_spe = 0i8;
            let mut last_blessed = 0i8;
            for iter in 0..64 {
                let candidate = crate::object::select_object_type(
                    crate::data::objects::OBJECTS,
                    &bases,
                    rng,
                    item.class,
                )
                .map(|i| i as i16)
                .unwrap_or(item.otyp);
                if trace_obj {
                    eprintln!("RS reroll iter={} class={:?} candidate={} rng_after_select={}",
                        iter, item.class, candidate, rng.call_count());
                }
                last_otyp = candidate;
                let (s, b) = mksobj_phantom_rng(item.class, candidate, rng);
                last_spe = s;
                last_blessed = b;
                if trace_obj {
                    eprintln!("RS reroll iter={} after_mksobj rng={} forbidden={}",
                        iter, rng.call_count(),
                        is_forbidden_starting_otyp(candidate, item.class, role, race, reroll_state));
                }
                if !is_forbidden_starting_otyp(candidate, item.class, role, race, reroll_state) {
                    update_nocreate_state(candidate, item.class, reroll_state);
                    break;
                }
            }
            // C u_init.c:1033 — "Don't start with +0 or negative rings":
            // if (objects[otyp].oc_charged && obj->spe <= 0) obj->spe = rne(3);
            // This consumes RNG and must fire whenever the starting random
            // ring is charged and rolled non-positive enchant.
            if item.class == ObjectClass::Ring {
                const FIRST_RING: i16 = 150;
                const FIRST_NON_CHARGED_RING: i16 = 156;
                let oc_charged =
                    last_otyp >= FIRST_RING && last_otyp < FIRST_NON_CHARGED_RING;
                if oc_charged && last_spe <= 0 {
                    last_spe = rne(rng, 3) as i8;
                }
            }
            (last_otyp, last_spe, last_blessed)
        } else {
            let (s, b) = mksobj_phantom_rng(item.class, item.otyp, rng);
            (item.otyp, s, b)
        };

    let mut obj = Object::new(crate::object::ObjectId(id), resolved_otyp, item.class);
    obj.quantity = item.quantity as i32;

    // Set enchantment
    if item.spe != UNDEF_SPE {
        obj.enchantment = item.spe;
    } else {
        // UNDEF_SPE: keep the value mksobj produced
        obj.enchantment = mksobj_spe;
    }

    // Set BUC status
    // C u_init.c:1093,1105 — `obj->cursed = 0` (always), then
    // `if (trbless != UNDEF_BLESS) obj->blessed = trbless`. So for non-UNDEF
    // trobj entries we honor trbless; for UNDEF_BLESS we keep mksobj's
    // blessed value (which can be 0 or 1 depending on the in-class roll).
    obj.buc = match item.bless {
        0 => BucStatus::Uncursed,
        1 => BucStatus::Blessed,
        _ => {
            // UNDEF_BLESS: cursed forced to 0 by C; blessed kept from mksobj.
            if mksobj_blessed > 0 {
                BucStatus::Blessed
            } else {
                BucStatus::Uncursed
            }
        }
    };

    // Copy static properties from object definition
    if let Some(def) = crate::data::objects::OBJECTS.get(resolved_otyp as usize) {
        obj.weight = def.weight as u32;
        if obj.name.is_none() {
            obj.name = Some(def.name.to_string());
        }
        // Armor AC: C stores a_ac = 10 - arm_ac (objects.c ARMOR macro)
        if item.class == ObjectClass::Armor {
            obj.base_ac = (10 - def.bonus as i32) as i8;
        }
    }

    obj
}

/// Backward-compatible entry that uses neutral role/race defaults — preserved
/// for callers that don't yet thread role/race through. Consumers that DO have
/// the role/race (init_inventory, ini_inv) should call `make_starting_object_full`.
pub fn make_starting_object(item: &StartingItem, rng: &mut GameRng, next_id: &mut u32) -> Object {
    let mut state = StartingInvRerollState::default();
    make_starting_object_full(item, Role::Valkyrie, Race::Human, rng, next_id, &mut state)
}

/// True when `otyp` is forbidden as a starting random pick. Mirrors the C
/// u_init.c:1023-1047 `while` condition.
///
/// Currently unused — see the gap note in `make_starting_object_full`. Kept
/// in source so the reroll loop can be re-enabled once the OBJECTS array is
/// realigned to C's onames.h indices.
#[allow(dead_code)]
fn is_forbidden_starting_otyp(
    otyp: i16,
    class: ObjectClass,
    role: Role,
    race: Race,
    state: &StartingInvRerollState,
) -> bool {
    // C ID constants (from include/onames.h)
    const WAN_WISHING: i16 = 387;
    const RIN_LEVITATION: i16 = 160;
    const RIN_HUNGER: i16 = 161;
    const RIN_AGGRAVATE_MONSTER: i16 = 162;
    const RIN_POISON_RESISTANCE: i16 = 165;
    const POT_HALLUCINATION: i16 = 279;
    const POT_ACID: i16 = 295;
    const SCR_ENCHANT_WEAPON: i16 = 303;
    const SCR_AMNESIA: i16 = 313;
    const SCR_FIRE: i16 = 314;
    const SCR_BLANK_PAPER: i16 = 339;
    const SPE_FORCE_BOLT: i16 = 350;
    const SPE_BLANK_PAPER: i16 = 380;
    const WAN_NOTHING: i16 = 388;

    if otyp == WAN_WISHING
        || otyp == state.nocreate
        || otyp == state.nocreate2
        || otyp == state.nocreate3
        || otyp == state.nocreate4
        || otyp == RIN_LEVITATION
        || otyp == POT_HALLUCINATION
        || otyp == POT_ACID
        || otyp == SCR_AMNESIA
        || otyp == SCR_FIRE
        || otyp == SCR_BLANK_PAPER
        || otyp == SPE_BLANK_PAPER
        || otyp == RIN_AGGRAVATE_MONSTER
        || otyp == RIN_HUNGER
        || otyp == WAN_NOTHING
    {
        return true;
    }
    // Race-specific: orcs already have poison resistance
    if otyp == RIN_POISON_RESISTANCE && race == Race::Orc {
        return true;
    }
    // Role-specific: Monks don't use weapons; Wizards already have force bolt
    if otyp == SCR_ENCHANT_WEAPON && role == Role::Monk {
        return true;
    }
    if otyp == SPE_FORCE_BOLT && role == Role::Wizard {
        return true;
    }
    // Spellbook level filter (C: oc_level > 3). ObjClassDef doesn't expose
    // oc_level, so we hard-code the high-level spellbook IDs from
    // NetHack-3.6.7/src/objects.c via onames.h:
    if class == ObjectClass::Spellbook {
        const HIGH_LEVEL_SPELLBOOKS: &[i16] = &[
            340, // SPE_DIG (5)
            342, // SPE_FIREBALL (4)
            343, // SPE_CONE_OF_COLD (4)
            345, // SPE_FINGER_OF_DEATH (7)
            364, // SPE_LEVITATION (4)
            366, // SPE_RESTORE_ABILITY (4)
            367, // SPE_INVISIBILITY (4)
            368, // SPE_DETECT_TREASURE (4)
            370, // SPE_MAGIC_MAPPING (5)
            372, // SPE_TURN_UNDEAD (6)
            373, // SPE_POLYMORPH (6)
            374, // SPE_TELEPORT_AWAY (6)
            375, // SPE_CREATE_FAMILIAR (6)
            376, // SPE_CANCELLATION (7)
        ];
        if HIGH_LEVEL_SPELLBOOKS.contains(&otyp) {
            return true;
        }
    }
    false
}

/// Update the cross-call nocreate state after accepting a random pick.
/// Mirrors u_init.c:1063-1076. Currently unused — see gap note above.
#[allow(dead_code)]
fn update_nocreate_state(
    otyp: i16,
    class: ObjectClass,
    state: &mut StartingInvRerollState,
) {
    const WAN_POLYMORPH: i16 = 394;
    const RIN_POLYMORPH: i16 = 173;
    const POT_POLYMORPH: i16 = 291;
    const RIN_POLYMORPH_CONTROL: i16 = 174;
    const SPE_POLYMORPH: i16 = 373;

    match otyp {
        WAN_POLYMORPH | RIN_POLYMORPH | POT_POLYMORPH => {
            state.nocreate = RIN_POLYMORPH_CONTROL;
        }
        RIN_POLYMORPH_CONTROL => {
            state.nocreate = RIN_POLYMORPH;
            state.nocreate2 = SPE_POLYMORPH;
            state.nocreate3 = POT_POLYMORPH;
        }
        _ => {}
    }
    if class == ObjectClass::Ring || class == ObjectClass::Spellbook {
        state.nocreate4 = otyp;
    }
}

/// Initialize a player's starting inventory (C: u_init inventory section)
pub fn init_inventory(rng: &mut GameRng, role: Role) -> Vec<Object> {
    let items = starting_inventory(role);
    let mut inventory = Vec::with_capacity(items.len());
    let mut next_id: u32 = 1;
    let mut letter = b'a';
    let mut reroll = StartingInvRerollState::default();

    for item in items {
        let mut obj = make_starting_object_full(
            item,
            role,
            Race::Human,
            rng,
            &mut next_id,
            &mut reroll,
        );
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
    for (v, &base) in values.iter_mut().zip(role_data.attrbase.iter()) {
        *v = base;
        np -= *v as i32;
    }

    // Distribute remaining points based on role distribution
    // C: ATTRMAX(STR) = STR18(100) = 118 for non-polymorphed; other attrs use racial max.
    // STR can exceed 18 during init (values 19+ represent exceptional strength 18/01+).
    // We must allow this or the distribution diverges from C.
    let mut attrmax = race_data.attrmax;
    attrmax[0] = 125; // STR: allow up to 125 during init (C: STR18(100)=118)

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

        // Check racial max (uses exceptional STR max for index 0)
        if values[i] >= attrmax[i] {
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
        player
            .properties
            .grant_intrinsic(crate::player::Property::Jumping);
    }

    // Set initial nutrition
    player.nutrition = 900;
    player.hunger_state = crate::player::HungerState::NotHungry;

    // Initialize inventory (C: role-specific setup + ini_inv calls)
    let mut inventory = Vec::new();
    let mut next_id: u32 = 1;
    let mut letter = b'a';
    let mut reroll = StartingInvRerollState::default();
    let race = player.race;

    // Helper closure to add an item
    let trace = std::env::var("NH_TRACE_INIT").is_ok();
    let add_item = |inv: &mut Vec<Object>,
                    item: &StartingItem,
                    rng: &mut GameRng,
                    next_id: &mut u32,
                    letter: &mut u8,
                    reroll: &mut StartingInvRerollState| {
        // C ini_inv (u_init.c:1155-1163) decrements `trop->trquan` and loops
        // `continue` to make a similar object — meaning each unit in the stack
        // is a separate `mksobj` call. Only Weapon/Tool/Coin classes
        // short-circuit (they set `obj->quan = trquan; trquan = 1`).
        //
        // For RNG correctness we make `trquan` mksobj_phantom calls. For
        // **explicit-otyp** stacks (e.g. POT_HEALING ×4), C creates 4 objects
        // that share the same otyp/spe/buc and merge in addinv → we present
        // one Object with quantity=trquan. For **random-otyp** stacks (e.g.
        // Wizard's "2 random rings"), each call produces a different otyp
        // → no merge → emit each as its own Object.
        let single_stack = matches!(
            item.class,
            ObjectClass::Weapon | ObjectClass::Tool | ObjectClass::Coin
        ) || item.otyp != ObjectType::StrangeObject as i16;
        let calls = if matches!(
            item.class,
            ObjectClass::Weapon | ObjectClass::Tool | ObjectClass::Coin
        ) {
            1
        } else {
            item.quantity.max(1)
        };
        // For explicit-otyp stacks (single_stack=true), units share the same
        // otyp but mksobj's blessorcurse roll can produce different BUC for
        // each unit. C's `addinv` merges only same-(otyp,spe,buc,...) objects,
        // so a 4-stack with one blessed unit appears as TWO inventory
        // entries (3-uncursed + 1-blessed). Mirror that by tracking per-unit
        // mksobj outputs and grouping consecutive identicals.
        let mut pending: Vec<Object> = Vec::new();
        for k in 0..calls {
            if trace {
                eprintln!(
                    "RS add_item: class={:?} otyp={} qty={} k={} rng={}",
                    item.class, item.otyp, item.quantity, k, rng.call_count()
                );
            }
            let mut per_item = *item;
            if calls > 1 {
                per_item.quantity = 1;
            }
            let obj = make_starting_object_full(&per_item, role, race, rng, next_id, reroll);
            pending.push(obj);
        }
        if single_stack {
            // Group consecutive units with identical (otyp, spe, buc) into
            // merged stacks (mirrors C addinv merge rules).
            let mut groups: Vec<Object> = Vec::new();
            for obj in pending {
                if let Some(last) = groups.last_mut() {
                    if last.object_type == obj.object_type
                        && last.enchantment == obj.enchantment
                        && last.buc == obj.buc
                    {
                        last.quantity += 1;
                        continue;
                    }
                }
                let mut o = obj;
                o.quantity = 1;
                groups.push(o);
            }
            for mut obj in groups {
                obj.inv_letter = *letter as char;
                if *letter < b'z' {
                    *letter += 1;
                }
                inv.push(obj);
            }
        } else {
            // Random-otyp stack: each unit is its own object (different
            // otyps usually preclude merging).
            for mut obj in pending {
                obj.inv_letter = *letter as char;
                if *letter < b'z' {
                    *letter += 1;
                }
                inv.push(obj);
            }
        }
    };

    use crate::data::objects::ObjectType;

    // Role-specific pre-init, variable quantities, and ini_inv (C: u_init.c:662-800)
    match role {
        Role::Archeologist => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional extras (C: u_init.c:669-674)
            if rng.rn2(10) == 0 {
                let item =
                    StartingItem::new(ObjectType::TinOpener as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            } else if rng.rn2(4) == 0 {
                let item =
                    StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            } else if rng.rn2(10) == 0 {
                let item = StartingItem::new(
                    ObjectType::MagicMarker as i16,
                    UNDEF_SPE,
                    ObjectClass::Tool,
                    1,
                    0,
                );
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Barbarian => {
            // C: 50% chance of battle-axe/short-sword swap (u_init.c:680-683)
            if rng.rn2(100) >= 50 {
                // Use battle-axe + short-sword instead of two-handed sword + axe
                let items: &[StartingItem] = &[
                    StartingItem::new(
                        ObjectType::BattleAxe as i16,
                        0,
                        ObjectClass::Weapon,
                        1,
                        UNDEF_BLESS,
                    ),
                    StartingItem::new(
                        ObjectType::ShortSword as i16,
                        0,
                        ObjectClass::Weapon,
                        1,
                        UNDEF_BLESS,
                    ),
                    StartingItem::new(
                        ObjectType::RingMail as i16,
                        0,
                        ObjectClass::Armor,
                        1,
                        UNDEF_BLESS,
                    ),
                    StartingItem::new(ObjectType::FoodRation as i16, 0, ObjectClass::Food, 1, 0),
                ];
                for item in items {
                    add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
                }
            } else {
                let base_items = starting_inventory(role);
                for item in base_items {
                    add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
                }
            }
            // Optional lamp (C: u_init.c:685-686)
            if rng.rn2(6) == 0 {
                let item =
                    StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Caveman => {
            // C: variable flint quantity rn1(11,10) = 10..20 (u_init.c:692)
            let flint_qty = (rng.rnd(11) + 9) as u8; // rn1(11,10) = rnd(11)+10-1 = 10..20
            let items: &[StartingItem] = &[
                StartingItem::new(
                    ObjectType::Club as i16,
                    1,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Sling as i16,
                    2,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Flint as i16,
                    0,
                    ObjectClass::Gem,
                    flint_qty,
                    UNDEF_BLESS,
                ),
                StartingItem::new(ObjectType::Rock as i16, 0, ObjectClass::Gem, 3, 0),
                StartingItem::new(
                    ObjectType::LeatherArmor as i16,
                    0,
                    ObjectClass::Armor,
                    1,
                    UNDEF_BLESS,
                ),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Healer => {
            // C: gold set before ini_inv (u_init.c:697)
            player.gold = (rng.rnd(1000) + 1000) as i32; // rn1(1000, 1001) = 1001..2000
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional lamp (C: u_init.c:699-700)
            if rng.rn2(25) == 0 {
                let item =
                    StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Knight => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Monk => {
            // C: select spellbook type rn2(90)/30 → [0..2] (u_init.c:715)
            // C M_spell array (u_init.c:713): SPE_HEALING, SPE_PROTECTION,
            // SPE_SLEEP. NOTE: SPE_SLEEP=344 (the spellbook), distinct from
            // WAN_SLEEP=404. Earlier code used `ObjectType::Sleep` which is
            // the wand otyp — silently wrong for Monk's starting spellbook.
            let spell_choices = [
                ObjectType::Healing,
                ObjectType::Protection,
                ObjectType::SpellbookSleep,
            ];
            let spell_idx = (rng.rn2(90) / 30) as usize;
            let spell_type = spell_choices[spell_idx.min(2)];
            let items: &[StartingItem] = &[
                StartingItem::new(
                    ObjectType::LeatherGloves as i16,
                    2,
                    ObjectClass::Armor,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Robe as i16,
                    1,
                    ObjectClass::Armor,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(spell_type as i16, UNDEF_SPE, ObjectClass::Spellbook, 1, 1),
                StartingItem::new(
                    ObjectType::StrangeObject as i16,
                    UNDEF_SPE,
                    ObjectClass::Scroll,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::PotionHealing as i16,
                    0,
                    ObjectClass::Potion,
                    3,
                    UNDEF_BLESS,
                ),
                StartingItem::new(ObjectType::FoodRation as i16, 0, ObjectClass::Food, 3, 0),
                StartingItem::new(
                    ObjectType::Apple as i16,
                    0,
                    ObjectClass::Food,
                    5,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Orange as i16,
                    0,
                    ObjectClass::Food,
                    5,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::FortuneCookie as i16,
                    0,
                    ObjectClass::Food,
                    3,
                    UNDEF_BLESS,
                ),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional extras (C: u_init.c:717-720)
            if rng.rn2(5) == 0 {
                let item = StartingItem::new(
                    ObjectType::MagicMarker as i16,
                    UNDEF_SPE,
                    ObjectClass::Tool,
                    1,
                    0,
                );
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            } else if rng.rn2(10) == 0 {
                let item =
                    StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Priest => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional extras (C: u_init.c:729-732)
            if rng.rn2(10) == 0 {
                let item = StartingItem::new(
                    ObjectType::MagicMarker as i16,
                    UNDEF_SPE,
                    ObjectClass::Tool,
                    1,
                    0,
                );
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            } else if rng.rn2(10) == 0 {
                let item =
                    StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Ranger => {
            // C: variable arrow quantities (u_init.c:744-745)
            let arrow2_qty = (rng.rnd(10) + 49) as u8; // rn1(10, 50) = 50..59
            let arrow0_qty = (rng.rnd(10) + 29) as u8; // rn1(10, 30) = 30..39
            let items: &[StartingItem] = &[
                StartingItem::new(
                    ObjectType::Dagger as i16,
                    1,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Bow as i16,
                    1,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Arrow as i16,
                    2,
                    ObjectClass::Weapon,
                    arrow2_qty,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Arrow as i16,
                    0,
                    ObjectClass::Weapon,
                    arrow0_qty,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::CloakOfDisplacement as i16,
                    2,
                    ObjectClass::Armor,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(ObjectType::CramRation as i16, 0, ObjectClass::Food, 4, 0),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Rogue => {
            // C: variable dagger quantity rn1(10,6) = 6..15 (u_init.c:750)
            let dagger_qty = (rng.rnd(10) + 5) as u8; // rn1(10,6) = rnd(10)+6-1 = 6..15
            let items: &[StartingItem] = &[
                StartingItem::new(
                    ObjectType::ShortSword as i16,
                    0,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Dagger as i16,
                    0,
                    ObjectClass::Weapon,
                    dagger_qty,
                    0,
                ),
                StartingItem::new(
                    ObjectType::LeatherArmor as i16,
                    1,
                    ObjectClass::Armor,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(ObjectType::Sickness as i16, 0, ObjectClass::Potion, 1, 0),
                StartingItem::new(ObjectType::LockPick as i16, 0, ObjectClass::Tool, 1, 0),
                StartingItem::new(ObjectType::Sack as i16, 0, ObjectClass::Tool, 1, 0),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional blindfold (C: u_init.c:753-754)
            if rng.rn2(5) == 0 {
                let item =
                    StartingItem::new(ObjectType::Blindfold as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Samurai => {
            // C: variable ya quantity rn1(20,26) = 26..45 (u_init.c:759)
            let ya_qty = (rng.rnd(20) + 25) as u8; // rn1(20,26) = rnd(20)+26-1 = 26..45
            let items: &[StartingItem] = &[
                StartingItem::new(
                    ObjectType::Katana as i16,
                    0,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::ShortSword as i16,
                    0,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Yumi as i16,
                    0,
                    ObjectClass::Weapon,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::Ya as i16,
                    0,
                    ObjectClass::Weapon,
                    ya_qty,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::SplintMail as i16,
                    0,
                    ObjectClass::Armor,
                    1,
                    UNDEF_BLESS,
                ),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional blindfold (C: u_init.c:761-762)
            if rng.rn2(5) == 0 {
                let item =
                    StartingItem::new(ObjectType::Blindfold as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Tourist => {
            // C: variable dart quantity rn1(20,21) = 21..40 (u_init.c:768)
            let dart_qty = (rng.rnd(20) + 20) as u8; // rn1(20,21) = rnd(20)+21-1 = 21..40
            // C: gold rnd(1000) (u_init.c:769)
            player.gold = rng.rnd(1000) as i32;
            let items: &[StartingItem] = &[
                StartingItem::new(
                    ObjectType::Dart as i16,
                    2,
                    ObjectClass::Weapon,
                    dart_qty,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::StrangeObject as i16,
                    UNDEF_SPE,
                    ObjectClass::Food,
                    10,
                    0,
                ),
                StartingItem::new(
                    ObjectType::PotionExtraHealing as i16,
                    0,
                    ObjectClass::Potion,
                    2,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::MagicMapping as i16,
                    0,
                    ObjectClass::Scroll,
                    4,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::HawaiianShirt as i16,
                    0,
                    ObjectClass::Armor,
                    1,
                    UNDEF_BLESS,
                ),
                StartingItem::new(
                    ObjectType::ExpensiveCamera as i16,
                    UNDEF_SPE,
                    ObjectClass::Tool,
                    1,
                    0,
                ),
                StartingItem::new(ObjectType::CreditCard as i16, 0, ObjectClass::Tool, 1, 0),
            ];
            for item in items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional extras (C: u_init.c:771-778)
            if rng.rn2(25) == 0 {
                let item =
                    StartingItem::new(ObjectType::TinOpener as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            } else if rng.rn2(25) == 0 {
                let item = StartingItem::new(ObjectType::Leash as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            } else if rng.rn2(25) == 0 {
                let item = StartingItem::new(ObjectType::Towel as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            } else if rng.rn2(25) == 0 {
                let item = StartingItem::new(
                    ObjectType::MagicMarker as i16,
                    UNDEF_SPE,
                    ObjectClass::Tool,
                    1,
                    0,
                );
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Valkyrie => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            // Optional lamp (C: u_init.c:783-784)
            if rng.rn2(6) == 0 {
                let item =
                    StartingItem::new(ObjectType::OilLamp as i16, 1, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
        Role::Wizard => {
            let base_items = starting_inventory(role);
            for item in base_items {
                add_item(&mut inventory, item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            if rng.rn2(5) == 0 {
                let item = StartingItem::new(
                    ObjectType::MagicMarker as i16,
                    UNDEF_SPE,
                    ObjectClass::Tool,
                    1,
                    0,
                );
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
            if rng.rn2(5) == 0 {
                let item =
                    StartingItem::new(ObjectType::Blindfold as i16, 0, ObjectClass::Tool, 1, 0);
                add_item(&mut inventory, &item, rng, &mut next_id, &mut letter, &mut reroll);
            }
        }
    }

    // Roll attributes (C: init_attr(75)) - happens after inventory in C
    roll_attributes(player, rng);

    // Auto-equip starting items (C: u_init.c:1114-1146)
    auto_equip_starting_inventory(&mut inventory);

    // Boost STR/CON if overloaded by starting inventory (C: u_init.c:912-919)
    // C: while (inv_weight() > 0) { adjattrib(A_STR, 1) || adjattrib(A_CON, 1) }
    // inv_weight() = total_weight - weight_cap()
    {
        use crate::player::Attribute;
        loop {
            let total_wt: i32 = inventory.iter().map(|o| o.weight as i32 * o.quantity).sum();
            let cap = player.weight_cap();
            if total_wt <= cap {
                break;
            }
            if player.adjattrib(Attribute::Strength, 1) {
                continue;
            }
            if player.adjattrib(Attribute::Constitution, 1) {
                continue;
            }
            break;
        }
    }

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
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
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
        let mut wizard = crate::player::You::new(
            "Test".into(),
            Role::Wizard,
            Race::Human,
            crate::player::Gender::Male,
        );
        let mut barb = crate::player::You::new(
            "Test".into(),
            Role::Barbarian,
            Race::Human,
            crate::player::Gender::Male,
        );
        u_init(&mut wizard, &mut rng);
        u_init(&mut barb, &mut rng);
        assert!(barb.hp_max > wizard.hp_max);
    }

    #[test]
    fn test_u_init_energy_varies_by_role() {
        let mut rng = GameRng::new(42);
        let mut wizard = crate::player::You::new(
            "Test".into(),
            Role::Wizard,
            Race::Human,
            crate::player::Gender::Male,
        );
        let mut barb = crate::player::You::new(
            "Test".into(),
            Role::Barbarian,
            Race::Human,
            crate::player::Gender::Male,
        );
        u_init(&mut wizard, &mut rng);
        u_init(&mut barb, &mut rng);
        assert!(wizard.energy_max > barb.energy_max);
    }

    #[test]
    fn test_u_init_knight_gets_jumping() {
        let mut rng = GameRng::new(42);
        let mut knight = crate::player::You::new(
            "Test".into(),
            Role::Knight,
            Race::Human,
            crate::player::Gender::Male,
        );
        u_init(&mut knight, &mut rng);
        assert!(knight.properties.has(crate::player::Property::Jumping));
    }

    #[test]
    fn test_u_init_bless_count() {
        let mut rng = GameRng::new(42);
        let mut player = crate::player::You::new(
            "Test".into(),
            Role::Valkyrie,
            Race::Human,
            crate::player::Gender::Male,
        );
        u_init(&mut player, &mut rng);
        assert_eq!(player.bless_count, 300);
    }

    #[test]
    fn test_u_init_healer_gold() {
        let mut rng = GameRng::new(42);
        let mut healer = crate::player::You::new(
            "Test".into(),
            Role::Healer,
            Race::Human,
            crate::player::Gender::Male,
        );
        u_init(&mut healer, &mut rng);
        assert!(
            healer.gold >= 1001,
            "Healer gold should be 1001..2000, got {}",
            healer.gold
        );
    }

    #[test]
    fn test_restricted_spell_discipline_wizard() {
        // Wizard has Attack, Healing, Divination, Enchantment, Clerical, Escape, Matter
        assert!(!restricted_spell_discipline(
            Role::Wizard,
            SkillType::AttackSpells
        ));
        assert!(!restricted_spell_discipline(
            Role::Wizard,
            SkillType::MatterSpells
        ));
        // Non-spell skills always return false
        assert!(!restricted_spell_discipline(
            Role::Wizard,
            SkillType::Dagger
        ));
    }

    #[test]
    fn test_restricted_spell_discipline_barbarian() {
        // Barbarian has no spell skills in table
        assert!(restricted_spell_discipline(
            Role::Barbarian,
            SkillType::AttackSpells
        ));
        assert!(restricted_spell_discipline(
            Role::Barbarian,
            SkillType::HealingSpells
        ));
    }

    #[test]
    fn test_make_starting_object_blessed() {
        let mut rng = GameRng::new(42);
        let mut next_id = 1;
        let item = StartingItem::new(
            crate::data::objects::ObjectType::Mace as i16,
            1,
            ObjectClass::Weapon,
            1,
            1,
        ); // blessed mace
        let obj = make_starting_object(&item, &mut rng, &mut next_id);
        assert_eq!(obj.buc, BucStatus::Blessed);
        assert_eq!(obj.enchantment, 1);
    }

    #[test]
    fn test_make_starting_object_uncursed() {
        let mut rng = GameRng::new(42);
        let mut next_id = 1;
        let item = StartingItem::new(
            crate::data::objects::ObjectType::Light as i16,
            0,
            ObjectClass::Food,
            3,
            0,
        ); // uncursed food
        let obj = make_starting_object(&item, &mut rng, &mut next_id);
        assert_eq!(obj.buc, BucStatus::Uncursed);
        assert_eq!(obj.quantity, 3);
    }

    #[test]
    fn test_skill_tables_complete() {
        // Every role should have at least 4 skill entries
        for role in [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ] {
            let table = skill_table_for_role(role);
            assert!(
                table.len() >= 4,
                "Too few skills for {:?}: {}",
                role,
                table.len()
            );
        }
    }
}
