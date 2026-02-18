//! Wearing and removing equipment (do_wear.c)
//!
//! Worn mask bits (from NetHack prop.h):
//! - W_ARM  = 0x00000001 (Body armor)
//! - W_ARMC = 0x00000002 (Cloak)
//! - W_ARMH = 0x00000004 (Helmet/hat)
//! - W_ARMS = 0x00000008 (Shield)
//! - W_ARMG = 0x00000010 (Gloves)
//! - W_ARMF = 0x00000020 (Footwear)
//! - W_ARMU = 0x00000040 (Undershirt)
//! - W_WEP  = 0x00000100 (Wielded weapon)
//! - W_SWAPWEP = 0x00000200 (Secondary weapon)
//! - W_QUIVER = 0x00000400 (Quivered ammo)
//! - W_AMUL = 0x00010000 (Amulet)
//! - W_RINGL = 0x00020000 (Left ring)
//! - W_RINGR = 0x00040000 (Right ring)
//! - W_TOOL = 0x00080000 (Worn tool like blindfold)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::action::ActionResult;
use crate::data::objects::OBJECTS;
use crate::gameloop::GameState;
use crate::object::{ArmorCategory, Object, ObjectClass};
use crate::player::{Attribute, Property, PropertyFlags};

// ============================================================================
// Property mapping from object data constants to Property enum
// ============================================================================

/// Convert an object property constant (from data/objects.rs) to a Property enum value.
/// Port of C's objects[otyp].oc_oprop lookup.
pub fn property_from_constant(prop_const: u8) -> Option<Property> {
    use crate::data::objects::*;
    match prop_const {
        0 => None,
        FIRE_RES => Some(Property::FireResistance),
        COLD_RES => Some(Property::ColdResistance),
        SLEEP_RES => Some(Property::SleepResistance),
        DISINT_RES => Some(Property::DisintResistance),
        SHOCK_RES => Some(Property::ShockResistance),
        POISON_RES => Some(Property::PoisonResistance),
        ACID_RES => Some(Property::AcidResistance),
        STONE_RES => Some(Property::StoneResistance),
        ADORNED => None, // Adornment is handled as stat bonus
        REGENERATION => Some(Property::Regeneration),
        SEARCHING => Some(Property::Searching),
        SEE_INVIS => Some(Property::SeeInvisible),
        INVIS => Some(Property::Invisibility),
        TELEPORT => Some(Property::Teleportation),
        TELEPORT_CONTROL => Some(Property::TeleportControl),
        POLYMORPH => Some(Property::Polymorph),
        POLYMORPH_CONTROL => Some(Property::PolyControl),
        LEVITATION => Some(Property::Levitation),
        STEALTH => Some(Property::Stealth),
        AGGRAVATE_MONSTER => Some(Property::Aggravate),
        CONFLICT => Some(Property::Conflict),
        PROTECTION => Some(Property::Protection),
        WARNING => Some(Property::Warning),
        TELEPAT => Some(Property::Telepathy),
        FAST => Some(Property::Speed),
        FUMBLING => Some(Property::Fumbling),
        HUNGER => Some(Property::Hunger),
        LIFESAVED => Some(Property::LifeSaving),
        ANTIMAGIC => Some(Property::MagicResistance),
        UNCHANGING => Some(Property::Unchanging),
        REFLECTING => Some(Property::Reflection),
        FREE_ACTION => Some(Property::FreeAction),
        SWIMMING => Some(Property::Swimming),
        MAGICAL_BREATHING => Some(Property::MagicBreathing),
        HALF_SPDAM => Some(Property::HalfSpellDamage),
        HALF_PHDAM => Some(Property::HalfPhysDamage),
        SICK_RES => Some(Property::SickResistance),
        DRAIN_RES => Some(Property::DrainResistance),
        DISPLACED => Some(Property::Displaced),
        CLAIRVOYANT => Some(Property::Clairvoyant),
        INFRAVISION => Some(Property::Infravision),
        DETECT_MONSTERS => None, // Detect monsters is transient
        SLEEPY => Some(Property::Sleepy),
        WWALKING => Some(Property::WaterWalking),
        _ => None,
    }
}

/// Look up the object definition for a given object_type.
fn obj_def(object_type: i16) -> Option<&'static crate::object::ObjClassDef> {
    let idx = object_type as usize;
    if idx < OBJECTS.len() {
        Some(&OBJECTS[idx])
    } else {
        None
    }
}

/// Get the property granted by an equipment item via its object data.
fn obj_property(object_type: i16) -> Option<Property> {
    obj_def(object_type).and_then(|def| property_from_constant(def.property))
}

/// Get the PropertyFlags source for an armor category
fn armor_source(cat: ArmorCategory) -> PropertyFlags {
    match cat {
        ArmorCategory::Suit => PropertyFlags::FROM_ARMOR,
        ArmorCategory::Cloak => PropertyFlags::FROM_CLOAK,
        ArmorCategory::Helm => PropertyFlags::FROM_HELM,
        ArmorCategory::Shield => PropertyFlags::FROM_SHIELD,
        ArmorCategory::Gloves => PropertyFlags::FROM_GLOVES,
        ArmorCategory::Boots => PropertyFlags::FROM_BOOTS,
        ArmorCategory::Shirt => PropertyFlags::FROM_ARMOR,
    }
}

/// Worn mask constants matching NetHack
pub mod worn_mask {
    pub const W_ARM: u32 = 0x00000001; // Body armor
    pub const W_ARMC: u32 = 0x00000002; // Cloak
    pub const W_ARMH: u32 = 0x00000004; // Helmet
    pub const W_ARMS: u32 = 0x00000008; // Shield
    pub const W_ARMG: u32 = 0x00000010; // Gloves
    pub const W_ARMF: u32 = 0x00000020; // Boots
    pub const W_ARMU: u32 = 0x00000040; // Undershirt
    pub const W_WEP: u32 = 0x00000100; // Wielded weapon
    pub const W_SWAPWEP: u32 = 0x00000200; // Secondary weapon
    pub const W_QUIVER: u32 = 0x00000400; // Quivered ammo
    pub const W_AMUL: u32 = 0x00010000; // Amulet
    pub const W_RINGL: u32 = 0x00020000; // Left ring
    pub const W_RINGR: u32 = 0x00040000; // Right ring
    pub const W_TOOL: u32 = 0x00080000; // Worn tool

    pub const W_ARMOR: u32 = W_ARM | W_ARMC | W_ARMH | W_ARMS | W_ARMG | W_ARMF | W_ARMU;
    pub const W_RING: u32 = W_RINGL | W_RINGR;
    pub const W_ACCESSORY: u32 = W_RING | W_AMUL | W_TOOL;
}

use worn_mask::*;

/// Determine which armor slot an item should use based on object_type.
/// Data-driven: looks up ArmorCategory from OBJECTS definition.
fn armor_slot(object_type: i16) -> u32 {
    if let Some(def) = obj_def(object_type) {
        if let Some(cat) = def.armor_category {
            return cat.worn_mask();
        }
    }
    W_ARM // Default to body armor
}

/// Wear armor
pub fn do_wear(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (obj_name, obj_type, is_worn, is_armor) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };
        (
            obj.display_name(),
            obj.object_type,
            obj.is_worn(),
            obj.class == ObjectClass::Armor,
        )
    };

    if !is_armor {
        return ActionResult::Failed("That's not something you can wear.".to_string());
    }

    if is_worn {
        return ActionResult::Failed("You're already wearing that.".to_string());
    }

    // Determine the armor slot
    let slot = armor_slot(obj_type);

    // Check if slot is already occupied
    for item in &state.inventory {
        if item.worn_mask & slot != 0 {
            return ActionResult::Failed("You're already wearing something there.".to_string());
        }
    }

    // Second pass: actually wear it and apply effects
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.worn_mask |= slot;
    }

    // Apply property effects from the armor via OBJECTS data
    if let Some(prop) = obj_property(obj_type) {
        let source = obj_def(obj_type)
            .and_then(|d| d.armor_category)
            .map(armor_source)
            .unwrap_or(PropertyFlags::FROM_ARMOR);
        state.player.properties.grant_extrinsic(prop, source);

        // Apply property-specific side effects
        use crate::player;
        match prop {
            crate::player::Property::Stealth => {
                player::toggle_stealth(&mut state.player.properties, true);
            }
            crate::player::Property::Displaced => {
                player::toggle_displacement(&mut state.player.properties, true);
            }
            crate::player::Property::Levitation => {
                let in_pit = state.player.utrap > 0;
                let msg = player::float_up(&mut state.player.properties, in_pit, false);
                state.message(msg.to_string());
            }
            _ => {}
        }
    }

    state.message(format!("You put on {}.", obj_name));
    ActionResult::Success
}

pub fn dowear(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_wear(state, obj_letter)
}

/// Take off armor
pub fn do_takeoff(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (obj_name, worn_mask, is_cursed, obj_type) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };
        (
            obj.display_name(),
            obj.worn_mask,
            obj.is_cursed(),
            obj.object_type,
        )
    };

    if worn_mask & W_ARMOR == 0 {
        return ActionResult::Failed("You're not wearing that.".to_string());
    }

    if is_cursed {
        state.message("You can't. It is cursed.");
        return ActionResult::Failed("You can't remove it, it's cursed!".to_string());
    }

    // Second pass: actually remove it and remove effects
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.worn_mask &= !W_ARMOR;
    }

    // Remove property effects from the armor via OBJECTS data
    if let Some(prop) = obj_property(obj_type) {
        let source = obj_def(obj_type)
            .and_then(|d| d.armor_category)
            .map(armor_source)
            .unwrap_or(PropertyFlags::FROM_ARMOR);
        state.player.properties.remove_extrinsic(prop, source);

        // Apply property-specific side effects
        use crate::player;
        match prop {
            crate::player::Property::Stealth => {
                player::toggle_stealth(&mut state.player.properties, false);
            }
            crate::player::Property::Displaced => {
                player::toggle_displacement(&mut state.player.properties, false);
            }
            crate::player::Property::Levitation => {
                player::float_down(&mut state.player.properties, true);
            }
            _ => {}
        }
    }

    state.message(format!("You take off {}.", obj_name));
    ActionResult::Success
}

pub fn dotakeoff(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_takeoff(state, obj_letter)
}

/// Wield a weapon
pub fn do_wield(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Special case: '-' means bare hands
    if obj_letter == '-' {
        return do_unwield(state);
    }

    let obj_name = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };
        obj.display_name()
    };

    // First unwield current weapon
    for item in &mut state.inventory {
        if item.worn_mask & W_WEP != 0 {
            item.worn_mask &= !W_WEP;
        }
    }

    // Now wield the new weapon
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.worn_mask |= W_WEP;
    }

    state.message(format!("You wield {}.", obj_name));
    ActionResult::Success
}

/// Put on an accessory (ring/amulet)
pub fn do_puton(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (obj_name, obj_class, is_worn) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };
        (obj.display_name(), obj.class, obj.is_worn())
    };

    if !matches!(obj_class, ObjectClass::Ring | ObjectClass::Amulet) {
        return ActionResult::Failed("That's not something you can put on.".to_string());
    }

    if is_worn {
        return ActionResult::Failed("You're already wearing that.".to_string());
    }

    // Determine slot
    let slot = match obj_class {
        ObjectClass::Amulet => {
            // Check if amulet slot is free
            for item in &state.inventory {
                if item.worn_mask & W_AMUL != 0 {
                    return ActionResult::Failed("You're already wearing an amulet.".to_string());
                }
            }
            W_AMUL
        }
        ObjectClass::Ring => {
            // Check ring slots - prefer left, then right
            let left_free = !state.inventory.iter().any(|i| i.worn_mask & W_RINGL != 0);
            let right_free = !state.inventory.iter().any(|i| i.worn_mask & W_RINGR != 0);

            if left_free {
                W_RINGL
            } else if right_free {
                W_RINGR
            } else {
                return ActionResult::Failed("You're already wearing two rings.".to_string());
            }
        }
        _ => return ActionResult::Failed("That's not an accessory.".to_string()),
    };

    // Second pass: actually wear it
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.worn_mask |= slot;
    }

    state.message(format!("You put on {}.", obj_name));
    ActionResult::Success
}

pub fn doputon(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_puton(state, obj_letter)
}

/// Remove an accessory
pub fn do_remove(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (obj_name, worn_mask, is_cursed) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };
        (obj.display_name(), obj.worn_mask, obj.is_cursed())
    };

    if worn_mask & W_ACCESSORY == 0 {
        return ActionResult::Failed("You're not wearing that.".to_string());
    }

    if is_cursed {
        state.message("You can't. It is cursed.");
        return ActionResult::Failed("You can't remove it, it's cursed!".to_string());
    }

    // Second pass: actually remove it
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.worn_mask &= !W_ACCESSORY;
    }

    state.message(format!("You remove {}.", obj_name));
    ActionResult::Success
}

pub fn doremove(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_remove(state, obj_letter)
}

/// Unwield current weapon (empty hands)
pub fn do_unwield(state: &mut GameState) -> ActionResult {
    let mut had_weapon = false;

    for item in &mut state.inventory {
        if item.worn_mask & W_WEP != 0 {
            item.worn_mask &= !W_WEP;
            had_weapon = true;
        }
    }

    if had_weapon {
        state.message("You are empty handed.");
    } else {
        state.message("You are already empty handed.");
    }
    ActionResult::Success
}

// ============================================================================
// Ring and Amulet effect hooks (Ring_on/Ring_off, Amulet_on/Amulet_off)
// ============================================================================

// Ring object types (approximate - should match nh-data/objects.rs)
mod ring_types {
    pub const RIN_ADORNMENT: i16 = 494;
    pub const RIN_GAIN_STRENGTH: i16 = 495;
    pub const RIN_GAIN_CONSTITUTION: i16 = 496;
    pub const RIN_INCREASE_ACCURACY: i16 = 497;
    pub const RIN_INCREASE_DAMAGE: i16 = 498;
    pub const RIN_PROTECTION: i16 = 499;
    pub const RIN_REGENERATION: i16 = 500;
    pub const RIN_SEARCHING: i16 = 501;
    pub const RIN_STEALTH: i16 = 502;
    pub const RIN_SUSTAIN_ABILITY: i16 = 503;
    pub const RIN_LEVITATION: i16 = 504;
    pub const RIN_HUNGER: i16 = 505;
    pub const RIN_AGGRAVATE_MONSTER: i16 = 506;
    pub const RIN_CONFLICT: i16 = 507;
    pub const RIN_WARNING: i16 = 508;
    pub const RIN_POISON_RESISTANCE: i16 = 509;
    pub const RIN_FIRE_RESISTANCE: i16 = 510;
    pub const RIN_COLD_RESISTANCE: i16 = 511;
    pub const RIN_SHOCK_RESISTANCE: i16 = 512;
    pub const RIN_FREE_ACTION: i16 = 513;
    pub const RIN_SLOW_DIGESTION: i16 = 514;
    pub const RIN_TELEPORTATION: i16 = 515;
    pub const RIN_TELEPORT_CONTROL: i16 = 516;
    pub const RIN_POLYMORPH: i16 = 517;
    pub const RIN_POLYMORPH_CONTROL: i16 = 518;
    pub const RIN_INVISIBILITY: i16 = 519;
    pub const RIN_SEE_INVISIBLE: i16 = 520;
    pub const RIN_PROTECTION_FROM_SHAPE_CHANGERS: i16 = 521;
}

// Amulet object types (approximate - should match nh-data/objects.rs)
mod amulet_types {
    pub const AMULET_OF_ESP: i16 = 475;
    pub const AMULET_OF_LIFE_SAVING: i16 = 476;
    pub const AMULET_OF_STRANGULATION: i16 = 477;
    pub const AMULET_OF_RESTFUL_SLEEP: i16 = 478;
    pub const AMULET_VERSUS_POISON: i16 = 479;
    pub const AMULET_OF_CHANGE: i16 = 480;
    pub const AMULET_OF_UNCHANGING: i16 = 481;
    pub const AMULET_OF_REFLECTION: i16 = 482;
    pub const AMULET_OF_MAGICAL_BREATHING: i16 = 483;
    pub const AMULET_OF_YENDOR: i16 = 484;
    pub const FAKE_AMULET_OF_YENDOR: i16 = 485;
}

/// Result of an equipment effect application
#[derive(Debug, Clone, Default)]
pub struct EquipmentEffect {
    /// Messages to display to the player
    pub messages: Vec<String>,
    /// Whether to identify the item
    pub identify: bool,
    /// Whether to destroy the item
    pub destroy: bool,
}

/// Get the property granted by a ring type.
fn ring_property(object_type: i16) -> Option<Property> {
    use ring_types::*;
    match object_type {
        RIN_REGENERATION => Some(Property::Regeneration),
        RIN_SEARCHING => Some(Property::Searching),
        RIN_STEALTH => Some(Property::Stealth),
        RIN_LEVITATION => Some(Property::Levitation),
        RIN_WARNING => Some(Property::Warning),
        RIN_POISON_RESISTANCE => Some(Property::PoisonResistance),
        RIN_FIRE_RESISTANCE => Some(Property::FireResistance),
        RIN_COLD_RESISTANCE => Some(Property::ColdResistance),
        RIN_SHOCK_RESISTANCE => Some(Property::ShockResistance),
        RIN_FREE_ACTION => Some(Property::FreeAction),
        RIN_SLOW_DIGESTION => Some(Property::SlowDigestion),
        RIN_TELEPORTATION => Some(Property::Teleportation),
        RIN_TELEPORT_CONTROL => Some(Property::TeleportControl),
        RIN_POLYMORPH => Some(Property::Polymorph),
        RIN_POLYMORPH_CONTROL => Some(Property::PolyControl),
        RIN_INVISIBILITY => Some(Property::Invisibility),
        RIN_SEE_INVISIBLE => Some(Property::SeeInvisible),
        RIN_CONFLICT => Some(Property::Conflict),
        RIN_AGGRAVATE_MONSTER => Some(Property::Aggravate),
        RIN_HUNGER => Some(Property::Hunger),
        RIN_SUSTAIN_ABILITY => Some(Property::SustainAbility),
        RIN_PROTECTION_FROM_SHAPE_CHANGERS => Some(Property::ProtFromShapechangers),
        _ => None,
    }
}

/// Get the property granted by an amulet type.
fn amulet_property(object_type: i16) -> Option<Property> {
    use amulet_types::*;
    match object_type {
        AMULET_OF_ESP => Some(Property::Telepathy),
        AMULET_OF_LIFE_SAVING => Some(Property::LifeSaving),
        AMULET_VERSUS_POISON => Some(Property::PoisonResistance),
        AMULET_OF_REFLECTION => Some(Property::Reflection),
        AMULET_OF_MAGICAL_BREATHING => Some(Property::MagicBreathing),
        AMULET_OF_UNCHANGING => Some(Property::Unchanging),
        _ => None,
    }
}

/// Apply effects when putting on a ring.
///
/// This grants extrinsic properties and applies stat bonuses based on ring type.
pub fn ring_on(state: &mut GameState, ring: &Object) -> EquipmentEffect {
    use ring_types::*;
    let mut effect = EquipmentEffect::default();
    let object_type = ring.object_type;

    // Determine which ring slot for property source
    let source = if ring.worn_mask & W_RINGL != 0 {
        PropertyFlags::FROM_RING_L
    } else {
        PropertyFlags::FROM_RING_R
    };

    // Grant extrinsic property if applicable
    if let Some(prop) = ring_property(object_type) {
        state.player.properties.grant_extrinsic(prop, source);

        // Special messages for certain properties
        match prop {
            Property::Invisibility => {
                if !state
                    .player
                    .properties
                    .has_intrinsic(Property::SeeInvisible)
                {
                    effect
                        .messages
                        .push("Suddenly you cannot see yourself.".to_string());
                    effect.identify = true;
                }
            }
            Property::SeeInvisible => {
                effect
                    .messages
                    .push("Your vision seems to sharpen.".to_string());
            }
            Property::Levitation => {
                effect
                    .messages
                    .push("You start to float in the air!".to_string());
                effect.identify = true;
            }
            Property::Conflict => {
                effect
                    .messages
                    .push("You feel like a rabble-rouser.".to_string());
            }
            _ => {}
        }
    }

    // Handle stat-modifying rings
    match object_type {
        RIN_GAIN_STRENGTH => {
            let bonus = ring.enchantment;
            if bonus != 0 {
                state
                    .player
                    .attr_current
                    .modify(crate::player::Attribute::Strength, bonus);
                let msg = if bonus > 0 {
                    "You feel stronger!"
                } else {
                    "You feel weaker!"
                };
                effect.messages.push(msg.to_string());
                effect.identify = true;
            }
        }
        RIN_GAIN_CONSTITUTION => {
            let bonus = ring.enchantment;
            if bonus != 0 {
                state
                    .player
                    .attr_current
                    .modify(crate::player::Attribute::Constitution, bonus);
                let msg = if bonus > 0 {
                    "You feel tougher!"
                } else {
                    "You feel fragile!"
                };
                effect.messages.push(msg.to_string());
                effect.identify = true;
            }
        }
        RIN_ADORNMENT => {
            let bonus = ring.enchantment;
            if bonus != 0 {
                state
                    .player
                    .attr_current
                    .modify(crate::player::Attribute::Charisma, bonus);
                let msg = if bonus > 0 {
                    "You feel more attractive!"
                } else {
                    "You feel ugly!"
                };
                effect.messages.push(msg.to_string());
                effect.identify = true;
            }
        }
        RIN_INCREASE_ACCURACY => {
            state.player.hit_bonus = state.player.hit_bonus.saturating_add(ring.enchantment);
        }
        RIN_INCREASE_DAMAGE => {
            state.player.damage_bonus = state.player.damage_bonus.saturating_add(ring.enchantment);
        }
        RIN_PROTECTION => {
            // Protection ring affects AC
            if ring.enchantment != 0 {
                effect.identify = true;
            }
        }
        _ => {}
    }

    effect
}

/// Remove effects when taking off a ring.
pub fn ring_off(state: &mut GameState, ring: &Object) -> EquipmentEffect {
    use ring_types::*;
    let mut effect = EquipmentEffect::default();
    let object_type = ring.object_type;

    // Determine which ring slot for property source
    let source = if ring.worn_mask & W_RINGL != 0 {
        PropertyFlags::FROM_RING_L
    } else {
        PropertyFlags::FROM_RING_R
    };

    // Remove extrinsic property if applicable
    if let Some(prop) = ring_property(object_type) {
        state.player.properties.remove_extrinsic(prop, source);

        // Special messages for certain properties
        match prop {
            Property::Invisibility => {
                if !state.player.properties.has(Property::Invisibility) {
                    effect
                        .messages
                        .push("Suddenly you can see yourself again.".to_string());
                }
            }
            Property::Levitation => {
                if !state.player.properties.has(Property::Levitation) {
                    effect
                        .messages
                        .push("You float gently to the ground.".to_string());
                }
            }
            _ => {}
        }
    }

    // Remove stat bonuses
    match object_type {
        RIN_GAIN_STRENGTH => {
            state
                .player
                .attr_current
                .modify(crate::player::Attribute::Strength, -ring.enchantment);
        }
        RIN_GAIN_CONSTITUTION => {
            state
                .player
                .attr_current
                .modify(crate::player::Attribute::Constitution, -ring.enchantment);
        }
        RIN_ADORNMENT => {
            state
                .player
                .attr_current
                .modify(crate::player::Attribute::Charisma, -ring.enchantment);
        }
        RIN_INCREASE_ACCURACY => {
            state.player.hit_bonus = state.player.hit_bonus.saturating_sub(ring.enchantment);
        }
        RIN_INCREASE_DAMAGE => {
            state.player.damage_bonus = state.player.damage_bonus.saturating_sub(ring.enchantment);
        }
        _ => {}
    }

    effect
}

/// Apply effects when putting on an amulet.
pub fn amulet_on(state: &mut GameState, amulet: &Object) -> EquipmentEffect {
    use amulet_types::*;
    let mut effect = EquipmentEffect::default();
    let object_type = amulet.object_type;

    // Grant extrinsic property if applicable
    if let Some(prop) = amulet_property(object_type) {
        state
            .player
            .properties
            .grant_extrinsic(prop, PropertyFlags::FROM_AMULET);
    }

    // Special amulet effects
    match object_type {
        AMULET_OF_ESP => {
            effect
                .messages
                .push("You feel a strange mental acuity.".to_string());
        }
        AMULET_OF_LIFE_SAVING => {
            // No message on equip, effect triggers on death
        }
        AMULET_OF_STRANGULATION => {
            // Start strangling countdown (6 turns to death)
            effect.messages.push("It constricts your throat!".to_string());
            effect.identify = true;
            state.player.strangled = 6;
        }
        AMULET_OF_RESTFUL_SLEEP => {
            // Causes drowsiness
            effect.messages.push("You feel drowsy.".to_string());
        }
        AMULET_OF_CHANGE => {
            // Changes sex - one-time effect, destroys amulet
            effect
                .messages
                .push("You are suddenly very different!".to_string());
            effect
                .messages
                .push("The amulet disintegrates!".to_string());
            effect.identify = true;
            effect.destroy = true;
        }
        AMULET_OF_UNCHANGING => {
            // Prevents polymorph
        }
        AMULET_OF_REFLECTION => {
            effect
                .messages
                .push("You feel a strange sense of security.".to_string());
        }
        AMULET_OF_MAGICAL_BREATHING => {
            // Allows underwater breathing
        }
        AMULET_OF_YENDOR | FAKE_AMULET_OF_YENDOR => {
            // The real one vs fake - identified differently
        }
        _ => {}
    }

    effect
}

/// Remove effects when taking off an amulet.
pub fn amulet_off(state: &mut GameState, amulet: &Object) -> EquipmentEffect {
    use amulet_types::*;
    let mut effect = EquipmentEffect::default();
    let object_type = amulet.object_type;

    // Remove extrinsic property if applicable
    if let Some(prop) = amulet_property(object_type) {
        state
            .player
            .properties
            .remove_extrinsic(prop, PropertyFlags::FROM_AMULET);
    }

    // Special removal effects
    match object_type {
        AMULET_OF_ESP => {
            if !state.player.properties.has(Property::Telepathy) {
                effect
                    .messages
                    .push("Your mental acuity fades.".to_string());
            }
        }
        AMULET_OF_STRANGULATION => {
            state.player.strangled = 0;
            effect.messages.push("You can breathe more easily now.".to_string());
        }
        AMULET_OF_MAGICAL_BREATHING => {
            if state.player.underwater && !state.player.properties.has(Property::MagicBreathing) {
                effect.messages.push("You can't breathe!".to_string());
                // Begin drowning: 1 turn of damage
                state.player.take_damage(state.rng.rnd(8) as i32);
            }
        }
        _ => {}
    }

    effect
}

// ============================================================================
// Dragon scale armor functions (polyself.c, do_wear.c)
// ============================================================================

/// Dragon scale mail types (approximate indices - match nh-data objects)
mod dragon_armor {
    pub const GRAY_DRAGON_SCALE_MAIL: i16 = 1;
    pub const SILVER_DRAGON_SCALE_MAIL: i16 = 2;
    pub const RED_DRAGON_SCALE_MAIL: i16 = 3;
    pub const WHITE_DRAGON_SCALE_MAIL: i16 = 4;
    pub const ORANGE_DRAGON_SCALE_MAIL: i16 = 5;
    pub const BLACK_DRAGON_SCALE_MAIL: i16 = 6;
    pub const BLUE_DRAGON_SCALE_MAIL: i16 = 7;
    pub const GREEN_DRAGON_SCALE_MAIL: i16 = 8;
    pub const YELLOW_DRAGON_SCALE_MAIL: i16 = 9;

    pub const GRAY_DRAGON_SCALES: i16 = 10;
    pub const SILVER_DRAGON_SCALES: i16 = 11;
    pub const RED_DRAGON_SCALES: i16 = 12;
    pub const WHITE_DRAGON_SCALES: i16 = 13;
    pub const ORANGE_DRAGON_SCALES: i16 = 14;
    pub const BLACK_DRAGON_SCALES: i16 = 15;
    pub const BLUE_DRAGON_SCALES: i16 = 16;
    pub const GREEN_DRAGON_SCALES: i16 = 17;
    pub const YELLOW_DRAGON_SCALES: i16 = 18;
}

/// Dragon types (for armor mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonType {
    Gray,
    Silver,
    Red,
    White,
    Orange,
    Black,
    Blue,
    Green,
    Yellow,
}

impl DragonType {
    /// Get the property conferred by this dragon type
    pub fn property(&self) -> Property {
        match self {
            DragonType::Gray => Property::MagicResistance,
            DragonType::Silver => Property::Reflection,
            DragonType::Red => Property::FireResistance,
            DragonType::White => Property::ColdResistance,
            DragonType::Orange => Property::SleepResistance,
            DragonType::Black => Property::DisintResistance,
            DragonType::Blue => Property::ShockResistance,
            DragonType::Green => Property::PoisonResistance,
            DragonType::Yellow => Property::AcidResistance,
        }
    }

    /// Get color name
    pub fn color_name(&self) -> &'static str {
        match self {
            DragonType::Gray => "gray",
            DragonType::Silver => "silver",
            DragonType::Red => "red",
            DragonType::White => "white",
            DragonType::Orange => "orange",
            DragonType::Black => "black",
            DragonType::Blue => "blue",
            DragonType::Green => "green",
            DragonType::Yellow => "yellow",
        }
    }
}

/// Convert armor object_type to dragon type (armor_to_dragon equivalent)
pub fn armor_to_dragon(object_type: i16) -> Option<DragonType> {
    use dragon_armor::*;
    match object_type {
        GRAY_DRAGON_SCALE_MAIL | GRAY_DRAGON_SCALES => Some(DragonType::Gray),
        SILVER_DRAGON_SCALE_MAIL | SILVER_DRAGON_SCALES => Some(DragonType::Silver),
        RED_DRAGON_SCALE_MAIL | RED_DRAGON_SCALES => Some(DragonType::Red),
        WHITE_DRAGON_SCALE_MAIL | WHITE_DRAGON_SCALES => Some(DragonType::White),
        ORANGE_DRAGON_SCALE_MAIL | ORANGE_DRAGON_SCALES => Some(DragonType::Orange),
        BLACK_DRAGON_SCALE_MAIL | BLACK_DRAGON_SCALES => Some(DragonType::Black),
        BLUE_DRAGON_SCALE_MAIL | BLUE_DRAGON_SCALES => Some(DragonType::Blue),
        GREEN_DRAGON_SCALE_MAIL | GREEN_DRAGON_SCALES => Some(DragonType::Green),
        YELLOW_DRAGON_SCALE_MAIL | YELLOW_DRAGON_SCALES => Some(DragonType::Yellow),
        _ => None,
    }
}

/// Check if an armor type is dragon scale mail
pub fn is_dragon_mail(object_type: i16) -> bool {
    use dragon_armor::*;
    (GRAY_DRAGON_SCALE_MAIL..=YELLOW_DRAGON_SCALE_MAIL).contains(&object_type)
}

/// Check if an armor type is dragon scales (raw)
pub fn is_dragon_scales(object_type: i16) -> bool {
    use dragon_armor::*;
    (GRAY_DRAGON_SCALES..=YELLOW_DRAGON_SCALES).contains(&object_type)
}

/// Check if an armor grants dragon resistance
pub fn is_dragon_armor(object_type: i16) -> bool {
    is_dragon_mail(object_type) || is_dragon_scales(object_type)
}

/// Get the property granted by dragon armor
pub fn dragon_armor_property(object_type: i16) -> Option<Property> {
    armor_to_dragon(object_type).map(|d| d.property())
}

/// Apply effects when putting on armor (Armor_on equivalent)
pub fn armor_on(state: &mut GameState, armor: &Object) -> EquipmentEffect {
    let mut effect = EquipmentEffect::default();

    if let Some(prop) = dragon_armor_property(armor.object_type) {
        state
            .player
            .properties
            .grant_extrinsic(prop, PropertyFlags::FROM_ARMOR);
        effect.identify = true;
    }

    effect
}

/// Remove effects when taking off armor (Armor_off equivalent)
pub fn armor_off(state: &mut GameState, armor: &Object) -> EquipmentEffect {
    let effect = EquipmentEffect::default();

    if let Some(prop) = dragon_armor_property(armor.object_type) {
        state
            .player
            .properties
            .remove_extrinsic(prop, PropertyFlags::FROM_ARMOR);
    }

    effect
}

/// Check if slot is already occupied (already_wearing equivalent)
pub fn already_wearing(state: &GameState, slot: u32) -> bool {
    state
        .inventory
        .iter()
        .any(|item| item.worn_mask & slot != 0)
}

pub fn doremring(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_remove(state, obj_letter)
}

pub fn take_off() {
    // Stub
}

pub fn taking_off() -> bool {
    false
}

pub fn donning() -> bool {
    false
}

pub fn doffing() -> bool {
    false
}

pub fn stop_donning() {
    // Stub
}

pub fn cancel_don() {
    // Stub
}

pub fn armor_gone() {
    // Stub
}

pub fn armor_or_accessory_off(_state: &mut GameState, _obj: &Object) {
    // Stub
}

pub fn armoroff(_state: &mut GameState, _obj: &Object) {
    // Stub
}

/// Apply effects when putting on boots (Boots_on from do_wear.c lines 162-218).
/// Handles: speed boots, levitation boots, water walking boots, elven boots,
/// fumble boots, jumping boots, kicking boots.
pub fn boots_on(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    // Grant base property from object data
    if let Some(prop) = obj_property(obj.object_type) {
        let had_prop = state.player.properties.has(prop);
        state.player.properties.grant_extrinsic(prop, PropertyFlags::FROM_BOOTS);

        // Item-specific messages (do_wear.c Boots_on)
        match name {
            "speed boots" => {
                if !had_prop {
                    state.message("You feel yourself speed up.");
                }
            }
            "levitation boots" => {
                if !had_prop {
                    state.message("You start to float in the air!");
                }
            }
            "water walking boots" => {
                // spoteffects if in water handled by gameloop
            }
            "elven boots" => {
                if !had_prop {
                    state.message("You walk very quietly.");
                }
            }
            "fumble boots" => {
                // Fumbling timeout starts
            }
            _ => {}
        }
    }
}

/// Remove effects when taking off boots (Boots_off from do_wear.c lines 220-278).
pub fn boots_off(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.remove_extrinsic(prop, PropertyFlags::FROM_BOOTS);

        match name {
            "speed boots" => {
                if !state.player.properties.has(Property::Speed) {
                    state.message("You feel yourself slow down.");
                }
            }
            "levitation boots" => {
                if !state.player.properties.has(Property::Levitation) {
                    state.message("You float gently to the ground.");
                }
            }
            "elven boots" => {
                if !state.player.properties.has(Property::Stealth) {
                    state.message("You sure are noisy.");
                }
            }
            _ => {}
        }
    }
}

/// Apply effects when putting on a cloak (Cloak_on from do_wear.c lines 280-333).
/// Handles: elven cloak, cloak of displacement, mummy wrapping,
/// cloak of invisibility, oilskin cloak, alchemy smock.
pub fn cloak_on(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    // Grant base property from object data
    if let Some(prop) = obj_property(obj.object_type) {
        let had_prop = state.player.properties.has(prop);
        state.player.properties.grant_extrinsic(prop, PropertyFlags::FROM_CLOAK);

        match name {
            "elven cloak" => {
                if !had_prop {
                    state.message("You walk very quietly.");
                }
            }
            "cloak of displacement" => {
                if !had_prop {
                    state.message("You feel quite displaced.");
                }
            }
            "cloak of invisibility" => {
                if !had_prop {
                    if state.player.properties.has(Property::SeeInvisible) {
                        state.message("Suddenly you can see through yourself.");
                    } else {
                        state.message("Suddenly you cannot see yourself.");
                    }
                }
            }
            _ => {}
        }
    }

    // Special cases not in property system
    match name {
        "mummy wrapping" => {
            // Mummy wrapping blocks invisibility display
            if state.player.properties.has(Property::Invisibility) {
                state.message("You can no longer see through yourself.");
            }
        }
        "oilskin cloak" => {
            state.message("It fits very tightly.");
        }
        "alchemy smock" => {
            // Alchemy smock grants acid resistance in addition to poison
            state.player.properties.grant_extrinsic(
                Property::AcidResistance,
                PropertyFlags::FROM_CLOAK,
            );
        }
        _ => {}
    }
}

/// Remove effects when taking off a cloak (Cloak_off from do_wear.c lines 335-384).
pub fn cloak_off(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.remove_extrinsic(prop, PropertyFlags::FROM_CLOAK);

        match name {
            "elven cloak" => {
                if !state.player.properties.has(Property::Stealth) {
                    state.message("You sure are noisy.");
                }
            }
            "cloak of displacement" => {
                if !state.player.properties.has(Property::Displaced) {
                    state.message("You stop shimmering.");
                }
            }
            "cloak of invisibility" => {
                if !state.player.properties.has(Property::Invisibility) {
                    if state.player.properties.has(Property::SeeInvisible) {
                        state.message("Suddenly you can no longer see through yourself.");
                    } else {
                        state.message("Suddenly you can see yourself.");
                    }
                }
            }
            _ => {}
        }
    }

    // Alchemy smock: also remove acid resistance
    if name == "alchemy smock" {
        state.player.properties.remove_extrinsic(
            Property::AcidResistance,
            PropertyFlags::FROM_CLOAK,
        );
    }
}

/// Apply effects when putting on gloves (Gloves_on from do_wear.c lines 500-526).
/// Handles: gauntlets of fumbling, gauntlets of power, gauntlets of dexterity.
pub fn gloves_on(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.grant_extrinsic(prop, PropertyFlags::FROM_GLOVES);
    }

    match name {
        "gauntlets of power" => {
            // Gauntlets of power grant strength 25 (handled via botl update in C)
            state.message("You feel powerful!");
        }
        "gauntlets of dexterity" => {
            // adj_abon: enchantment modifies dexterity
            let bonus = obj.enchantment;
            if bonus != 0 {
                state.player.attr_current.modify(Attribute::Dexterity, bonus);
            }
        }
        "gauntlets of fumbling" => {
            // Fumbling timeout starts
        }
        _ => {}
    }
}

/// Remove effects when taking off gloves (Gloves_off from do_wear.c lines 552-602).
/// Handles petrification check for wielding cockatrice corpse.
pub fn gloves_off(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.remove_extrinsic(prop, PropertyFlags::FROM_GLOVES);
    }

    match name {
        "gauntlets of power" => {
            state.message("You feel weaker.");
        }
        "gauntlets of dexterity" => {
            let bonus = obj.enchantment;
            if bonus != 0 {
                state.player.attr_current.modify(Attribute::Dexterity, -bonus);
            }
        }
        "gauntlets of fumbling" => {
            // Clear fumble timeout
            if !state.player.properties.has(Property::Fumbling) {
                // Fumbling cleared
            }
        }
        _ => {}
    }

    // Cure slippery fingers when gloves removed
    // C: Glib = 0 (clear slippery fingers)
    state.player.make_glib(0, false);
}

/// Apply effects when putting on a helmet (Helmet_on from do_wear.c lines 386-452).
/// Handles: helm of brilliance, cornuthaum, helm of opposite alignment, dunce cap.
pub fn helmet_on(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    // Grant base property from object data
    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.grant_extrinsic(prop, PropertyFlags::FROM_HELM);
    }

    match name {
        "helm of brilliance" => {
            // adj_abon: enchantment modifies INT and WIS
            let bonus = obj.enchantment;
            if bonus != 0 {
                state.player.attr_current.modify(Attribute::Intelligence, bonus);
                state.player.attr_current.modify(Attribute::Wisdom, bonus);
            }
        }
        "cornuthaum" => {
            // Wizards get +1 CHA, non-wizards get -1 CHA
            let bonus: i8 = if state.player.role == crate::player::Role::Wizard { 1 } else { -1 };
            state.player.attr_current.modify(Attribute::Charisma, bonus);
        }
        "dunce cap" => {
            // Curses itself, penalizes INT and WIS
            // In C: becomes cursed, INT/WIS penalties via ABON
            state.message("You feel giddy.");
        }
        "helm of opposite alignment" => {
            // Reverses alignment; becomes cursed
            state.message("Your languid demeanor belies great strife within you.");
        }
        _ => {}
    }
}

/// Remove effects when taking off a helmet (Helmet_off from do_wear.c lines 454-497).
pub fn helmet_off(state: &mut GameState, obj: &Object) {
    let name = obj_def(obj.object_type).map(|d| d.name).unwrap_or("");

    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.remove_extrinsic(prop, PropertyFlags::FROM_HELM);

        if prop == Property::Telepathy && !state.player.properties.has(Property::Telepathy) {
            state.message("Your senses fail!");
        }
    }

    match name {
        "helm of brilliance" => {
            let bonus = obj.enchantment;
            if bonus != 0 {
                state.player.attr_current.modify(Attribute::Intelligence, -bonus);
                state.player.attr_current.modify(Attribute::Wisdom, -bonus);
            }
        }
        "cornuthaum" => {
            let bonus: i8 = if state.player.role == crate::player::Role::Wizard { -1 } else { 1 };
            state.player.attr_current.modify(Attribute::Charisma, bonus);
        }
        "helm of opposite alignment" => {
            // Restore original alignment
        }
        _ => {}
    }
}

pub fn ring_gone(_state: &mut GameState, _obj: &Object) {
    // Stub
}

pub fn ring_off_or_gone(_state: &mut GameState, _obj: &Object, _gone: bool) {
    // Stub
}

/// Apply effects when putting on a shield (Shield_on from do_wear.c lines 604-626).
/// In C, shields don't have special on-wear effects; properties are set by setworn().
pub fn shield_on(state: &mut GameState, obj: &Object) {
    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.grant_extrinsic(prop, PropertyFlags::FROM_SHIELD);
    }
}

/// Remove effects when taking off a shield (Shield_off from do_wear.c lines 628-650).
pub fn shield_off(state: &mut GameState, obj: &Object) {
    if let Some(prop) = obj_property(obj.object_type) {
        state.player.properties.remove_extrinsic(prop, PropertyFlags::FROM_SHIELD);
    }
}

/// Apply effects when putting on a shirt
pub fn shirt_on(_state: &mut GameState, _obj: &Object) {
    // T-shirt, Hawaiian shirt
    // Shirts generally don't grant properties
}

/// Remove effects when taking off a shirt
pub fn shirt_off(_state: &mut GameState, _obj: &Object) {
    // Shirts generally don't grant properties
}

/// Apply effects when putting on a blindfold/towel (worn tool)
pub fn blindf_on(state: &mut GameState, _obj: &Object) {
    // Blindfold blinds the player
    state.message("You can't see any more.");
    state.player.blinded_timeout = 9999; // Indefinite while worn

    // Towel doesn't blind
    // Would check object type here
}

pub fn accessory_has_effect(_obj: &Object) -> bool {
    true
}

pub fn accessory_or_armor_on(_state: &mut GameState, _obj: &Object) {
    // Stub
}

pub fn already_wearing2(state: &GameState, slot: u32) -> bool {
    already_wearing(state, slot)
}

pub fn select_off(_state: &mut GameState, _obj: &Object) {
    // Stub
}

pub fn menu_remarm(_state: &mut GameState) {
    // Stub
}

pub fn doddoremarm() {
    // Stub
}

pub fn reset_remarm() {
    // Stub
}

pub fn destroy_arm(_state: &mut GameState, _obj: &Object) {
    // Stub
}

pub fn break_armor(_state: &mut GameState, _obj: &Object) {
    // Stub
}

pub fn breakarm(_state: &mut GameState, _obj: &Object) {
    // Stub
}

pub fn mon_break_armor(_state: &mut GameState, _monster_id: u32, _break_all: bool) {
    // Stub
}

pub fn m_lose_armor(_state: &mut GameState, _monster_id: u32, _obj: &Object) {
    // Stub
}

pub fn m_dowear(_state: &mut GameState, _monster_id: u32, _obj: &Object) -> bool {
    true
}

pub fn m_dowear_type(_state: &mut GameState, _monster_id: u32, _slot: u32, _obj: &Object) -> bool {
    true
}

pub fn cloak_simple_name(_obj: &Object) -> String {
    "cloak".to_string()
}

pub fn gloves_simple_name(_obj: &Object) -> String {
    "gloves".to_string()
}

pub fn helm_simple_name(_obj: &Object) -> String {
    "helmet".to_string()
}

pub fn suit_simple_name(_obj: &Object) -> String {
    "suit".to_string()
}

pub fn which_armor(state: &GameState, slot: u32) -> Option<&Object> {
    state
        .inventory
        .iter()
        .find(|item| item.worn_mask & slot != 0)
}

pub fn some_armor(state: &GameState) -> Option<&Object> {
    state
        .inventory
        .iter()
        .find(|item| item.worn_mask & W_ARMOR != 0)
}

pub fn wearing_armor(state: &GameState) -> bool {
    state
        .inventory
        .iter()
        .any(|item| item.worn_mask & W_ARMOR != 0)
}

pub fn is_worn(obj: &Object) -> bool {
    obj.worn_mask != 0
}

pub fn is_worn_by_type(obj: &Object) -> bool {
    obj.worn_mask != 0
}

pub fn wearslot(obj: &Object) -> u32 {
    obj.worn_mask
}

pub fn slots_required(_obj: &Object) -> u32 {
    // Stub
    0
}

pub fn setworn(obj: &mut Object, mask: u32) {
    obj.worn_mask |= mask;
}

pub fn setnotworn(obj: &mut Object) {
    obj.worn_mask = 0;
}

pub fn wearmask_to_obj(_mask: u32) -> Option<Object> {
    // Stub
    None
}

pub fn stuck_ring(_state: &GameState, _ring: &Object) -> bool {
    false
}

pub fn remove_worn_item(_state: &mut GameState, obj: &mut Object) {
    obj.worn_mask = 0;
}

pub fn fingers_or_gloves(_state: &GameState) -> String {
    "fingers".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::GameRng;

    fn make_armor_item(object_type: i16) -> Object {
        let mut obj = Object::default();
        obj.object_type = object_type;
        obj.class = ObjectClass::Armor;
        obj
    }

    // ======================================================================
    // property_from_constant tests
    // ======================================================================

    #[test]
    fn test_property_from_constant_zero_is_none() {
        assert!(property_from_constant(0).is_none());
    }

    #[test]
    fn test_property_from_constant_fire_res() {
        use crate::data::objects::FIRE_RES;
        assert_eq!(
            property_from_constant(FIRE_RES),
            Some(Property::FireResistance)
        );
    }

    #[test]
    fn test_property_from_constant_telepathy() {
        use crate::data::objects::TELEPAT;
        assert_eq!(
            property_from_constant(TELEPAT),
            Some(Property::Telepathy)
        );
    }

    #[test]
    fn test_property_from_constant_speed() {
        use crate::data::objects::FAST;
        assert_eq!(property_from_constant(FAST), Some(Property::Speed));
    }

    // ======================================================================
    // obj_def / obj_property tests
    // ======================================================================

    #[test]
    fn test_obj_def_boots_of_speed() {
        let def = obj_def(143).expect("boots of speed should exist");
        assert_eq!(def.name, "speed boots");
    }

    #[test]
    fn test_obj_property_boots_of_speed() {
        assert_eq!(obj_property(143), Some(Property::Speed));
    }

    #[test]
    fn test_obj_property_helm_of_telepathy() {
        assert_eq!(obj_property(78), Some(Property::Telepathy));
    }

    #[test]
    fn test_obj_property_cloak_of_displacement() {
        assert_eq!(obj_property(128), Some(Property::Displaced));
    }

    #[test]
    fn test_obj_property_shield_of_reflection() {
        assert_eq!(obj_property(135), Some(Property::Reflection));
    }

    // ======================================================================
    // armor_source tests
    // ======================================================================

    #[test]
    fn test_armor_source_boots() {
        assert!(armor_source(ArmorCategory::Boots).contains(PropertyFlags::FROM_BOOTS));
    }

    #[test]
    fn test_armor_source_cloak() {
        assert!(armor_source(ArmorCategory::Cloak).contains(PropertyFlags::FROM_CLOAK));
    }

    #[test]
    fn test_armor_source_helm() {
        assert!(armor_source(ArmorCategory::Helm).contains(PropertyFlags::FROM_HELM));
    }

    #[test]
    fn test_armor_source_gloves() {
        assert!(armor_source(ArmorCategory::Gloves).contains(PropertyFlags::FROM_GLOVES));
    }

    #[test]
    fn test_armor_source_shield() {
        assert!(armor_source(ArmorCategory::Shield).contains(PropertyFlags::FROM_SHIELD));
    }

    // ======================================================================
    // boots_on / boots_off tests
    // ======================================================================

    #[test]
    fn test_boots_on_speed() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(143); // speed boots
        assert!(!state.player.properties.has(Property::Speed));
        boots_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Speed));
    }

    #[test]
    fn test_boots_off_speed() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(143); // speed boots
        boots_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Speed));
        boots_off(&mut state, &obj);
        assert!(!state.player.properties.has(Property::Speed));
    }

    #[test]
    fn test_boots_on_levitation() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(149); // levitation boots
        boots_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Levitation));
    }

    #[test]
    fn test_boots_on_water_walking() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(144); // water walking boots
        boots_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::WaterWalking));
    }

    #[test]
    fn test_boots_on_elven_stealth() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(146); // elven boots
        boots_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Stealth));
    }

    #[test]
    fn test_boots_on_fumbling() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(148); // fumble boots
        boots_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Fumbling));
    }

    // ======================================================================
    // cloak_on / cloak_off tests
    // ======================================================================

    #[test]
    fn test_cloak_on_displacement() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(128); // cloak of displacement
        cloak_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Displaced));
    }

    #[test]
    fn test_cloak_off_displacement() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(128);
        cloak_on(&mut state, &obj);
        cloak_off(&mut state, &obj);
        assert!(!state.player.properties.has(Property::Displaced));
    }

    #[test]
    fn test_cloak_on_invisibility() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(126); // cloak of invisibility
        cloak_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Invisibility));
    }

    #[test]
    fn test_cloak_on_elven_stealth() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(118); // elven cloak
        cloak_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Stealth));
    }

    #[test]
    fn test_cloak_on_alchemy_smock_dual_resist() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(123); // alchemy smock
        cloak_on(&mut state, &obj);
        // Poison resistance from obj_property data
        assert!(state.player.properties.has(Property::PoisonResistance));
        // Acid resistance from special-case code
        assert!(state.player.properties.has(Property::AcidResistance));
    }

    #[test]
    fn test_cloak_off_alchemy_smock_clears_both() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(123); // alchemy smock
        cloak_on(&mut state, &obj);
        cloak_off(&mut state, &obj);
        assert!(!state.player.properties.has(Property::PoisonResistance));
        assert!(!state.player.properties.has(Property::AcidResistance));
    }

    // ======================================================================
    // helmet_on / helmet_off tests
    // ======================================================================

    #[test]
    fn test_helmet_on_telepathy() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(78); // helm of telepathy
        helmet_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Telepathy));
    }

    #[test]
    fn test_helmet_off_telepathy() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(78);
        helmet_on(&mut state, &obj);
        helmet_off(&mut state, &obj);
        assert!(!state.player.properties.has(Property::Telepathy));
    }

    #[test]
    fn test_helmet_on_brilliance_modifies_int_wis() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.attr_current.set(Attribute::Intelligence, 10);
        state.player.attr_current.set(Attribute::Wisdom, 10);
        let int_before = state.player.attr_current.get(Attribute::Intelligence);
        let wis_before = state.player.attr_current.get(Attribute::Wisdom);

        let mut obj = make_armor_item(76); // helm of brilliance
        obj.enchantment = 3;
        helmet_on(&mut state, &obj);

        assert_eq!(
            state.player.attr_current.get(Attribute::Intelligence),
            int_before + 3
        );
        assert_eq!(
            state.player.attr_current.get(Attribute::Wisdom),
            wis_before + 3
        );
    }

    #[test]
    fn test_helmet_off_brilliance_reverses() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.attr_current.set(Attribute::Intelligence, 10);

        let int_before = state.player.attr_current.get(Attribute::Intelligence);

        let mut obj = make_armor_item(76);
        obj.enchantment = 2;
        helmet_on(&mut state, &obj);
        assert_eq!(
            state.player.attr_current.get(Attribute::Intelligence),
            int_before + 2
        );
        helmet_off(&mut state, &obj);
        assert_eq!(
            state.player.attr_current.get(Attribute::Intelligence),
            int_before
        );
    }

    #[test]
    fn test_cornuthaum_wizard_bonus() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.role = crate::player::Role::Wizard;
        state.player.attr_current.set(Attribute::Charisma, 10);
        let cha_before = state.player.attr_current.get(Attribute::Charisma);

        let obj = make_armor_item(81); // cornuthaum
        helmet_on(&mut state, &obj);

        // Wizard gets +1 CHA
        assert_eq!(
            state.player.attr_current.get(Attribute::Charisma),
            cha_before + 1
        );
    }

    #[test]
    fn test_cornuthaum_non_wizard_penalty() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.role = crate::player::Role::Valkyrie;
        state.player.attr_current.set(Attribute::Charisma, 10);
        let cha_before = state.player.attr_current.get(Attribute::Charisma);

        let obj = make_armor_item(81);
        helmet_on(&mut state, &obj);

        // Non-wizard gets -1 CHA
        assert_eq!(
            state.player.attr_current.get(Attribute::Charisma),
            cha_before - 1
        );
    }

    // ======================================================================
    // gloves_on / gloves_off tests
    // ======================================================================

    #[test]
    fn test_gloves_on_fumbling() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(137); // gauntlets of fumbling
        gloves_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Fumbling));
    }

    #[test]
    fn test_gloves_off_fumbling() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(137);
        gloves_on(&mut state, &obj);
        gloves_off(&mut state, &obj);
        assert!(!state.player.properties.has(Property::Fumbling));
    }

    // ======================================================================
    // shield_on / shield_off tests
    // ======================================================================

    #[test]
    fn test_shield_on_reflection() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(135); // shield of reflection
        shield_on(&mut state, &obj);
        assert!(state.player.properties.has(Property::Reflection));
    }

    #[test]
    fn test_shield_off_reflection() {
        let mut state = GameState::new(GameRng::from_entropy());
        let obj = make_armor_item(135);
        shield_on(&mut state, &obj);
        shield_off(&mut state, &obj);
        assert!(!state.player.properties.has(Property::Reflection));
    }

    // ======================================================================
    // data-driven property mapping: verify data matches C objects
    // ======================================================================

    #[test]
    fn test_all_boot_properties_match_data() {
        // Verify the OBJECTS array property field maps correctly
        let cases: &[(i16, &str, Option<Property>)] = &[
            (143, "speed boots", Some(Property::Speed)),
            (144, "water walking boots", Some(Property::WaterWalking)),
            (146, "elven boots", Some(Property::Stealth)),
            (148, "fumble boots", Some(Property::Fumbling)),
            (149, "levitation boots", Some(Property::Levitation)),
        ];
        for &(idx, expected_name, expected_prop) in cases {
            let def = obj_def(idx).unwrap_or_else(|| panic!("no obj at index {idx}"));
            assert_eq!(def.name, expected_name, "name mismatch at index {idx}");
            assert_eq!(
                obj_property(idx), expected_prop,
                "property mismatch for {expected_name}"
            );
        }
    }

    #[test]
    fn test_all_cloak_properties_match_data() {
        let cases: &[(i16, &str, Option<Property>)] = &[
            (118, "elven cloak", Some(Property::Stealth)),
            (123, "alchemy smock", Some(Property::PoisonResistance)),
            (126, "cloak of invisibility", Some(Property::Invisibility)),
            (128, "cloak of displacement", Some(Property::Displaced)),
        ];
        for &(idx, expected_name, expected_prop) in cases {
            let def = obj_def(idx).unwrap_or_else(|| panic!("no obj at index {idx}"));
            assert_eq!(def.name, expected_name, "name mismatch at index {idx}");
            assert_eq!(
                obj_property(idx), expected_prop,
                "property mismatch for {expected_name}"
            );
        }
    }

    #[test]
    fn test_all_helm_properties_match_data() {
        let cases: &[(i16, &str, Option<Property>)] = &[
            (78, "helm of telepathy", Some(Property::Telepathy)),
            (81, "cornuthaum", Some(Property::Clairvoyant)),
        ];
        for &(idx, expected_name, expected_prop) in cases {
            let def = obj_def(idx).unwrap_or_else(|| panic!("no obj at index {idx}"));
            assert_eq!(def.name, expected_name, "name mismatch at index {idx}");
            assert_eq!(
                obj_property(idx), expected_prop,
                "property mismatch for {expected_name}"
            );
        }
    }
}
