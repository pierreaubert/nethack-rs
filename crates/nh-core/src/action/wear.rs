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

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::{Object, ObjectClass};
use crate::player::{Property, PropertyFlags};

// ============================================================================
// Property Effects from Equipment
// ============================================================================

/// Determine what properties an item grants when worn
fn get_property_effects(object_type: i16) -> Vec<Property> {
    // These are approximate - should match actual item definitions
    match object_type {
        // Ring of Protection
        1001..=1005 => vec![Property::Protection],
        // Ring of Fire Resistance
        1010..=1015 => vec![Property::FireResistance],
        // Ring of Cold Resistance
        1020..=1025 => vec![Property::ColdResistance],
        // Ring of Shock Resistance
        1030..=1035 => vec![Property::ShockResistance],
        // Ring of Poison Resistance
        1040..=1045 => vec![Property::PoisonResistance],
        // Ring of Invisibility
        1050..=1055 => vec![Property::Invisibility],
        // Ring of See Invisible
        1060..=1065 => vec![Property::SeeInvisible],
        // Ring of Telepathy
        1070..=1075 => vec![Property::Telepathy],
        // Ring of Teleportation
        1080..=1085 => vec![Property::Teleportation],
        // Amulet of Reflection
        2000..=2005 => vec![Property::Reflection],
        // Amulet of Life Saving
        2010..=2015 => vec![Property::LifeSaving],
        // Amulet of ESP
        2020..=2025 => vec![Property::Telepathy],
        // Dragon Scale Mail (fire dragon)
        3001..=3010 => vec![Property::FireResistance],
        // Dragon Scale Mail (cold dragon)
        3011..=3020 => vec![Property::ColdResistance],
        // Dragon Scale Mail (lightning dragon)
        3021..=3030 => vec![Property::ShockResistance],
        // Helm of Telepathy
        4001..=4005 => vec![Property::Telepathy],
        // Cloak of Magic Resistance
        5001..=5005 => vec![Property::MagicResistance],
        // Cloak of Invisibility
        5010..=5015 => vec![Property::Invisibility],
        // Boots of Levitation
        6001..=6005 => vec![Property::Levitation],
        // Boots of Speed
        6010..=6015 => vec![Property::Speed],
        _ => vec![],
    }
}

/// Apply property effects when wearing armor
fn apply_wear_properties(state: &mut GameState, object_type: i16) {
    for property in get_property_effects(object_type) {
        state.player.properties.grant_intrinsic(property);
    }
}

/// Remove property effects when unequipping armor
fn remove_wear_properties(state: &mut GameState, object_type: i16) {
    for property in get_property_effects(object_type) {
        // Only remove if no other equipped item grants it
        state.player.properties.remove_intrinsic(property);
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

/// Determine which armor slot an item should use based on object_type
fn armor_slot(object_type: i16) -> u32 {
    // Object type ranges from nh-data/objects.rs
    // These are approximate - should match actual object definitions
    match object_type {
        // Body armor (suits, mail, etc.) - types ~1-30
        1..=30 => W_ARM,
        // Cloaks - types ~31-45
        31..=45 => W_ARMC,
        // Helmets - types ~46-60
        46..=60 => W_ARMH,
        // Gloves - types ~61-70
        61..=70 => W_ARMG,
        // Shields - types ~71-80
        71..=80 => W_ARMS,
        // Boots - types ~81-95
        81..=95 => W_ARMF,
        // Shirts - types ~96-100
        96..=100 => W_ARMU,
        _ => W_ARM, // Default to body armor
    }
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

    // Apply any property effects from the armor
    apply_wear_properties(state, obj_type);

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

    // Remove any property effects from the armor
    remove_wear_properties(state, obj_type);

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

pub fn armor_or_accessory_off(state: &mut GameState, obj: &Object) {
    // Stub
}

pub fn armoroff(state: &mut GameState, obj: &Object) {
    // Stub
}

/// Apply effects when putting on boots
pub fn boots_on(state: &mut GameState, obj: &Object) {
    // Speed boots, levitation boots, etc.
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .grant_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::Levitation => state.message("You start to float!"),
            Property::Speed => state.message("Your feet feel quick!"),
            Property::Jumping => state.message("You feel like jumping around."),
            Property::Swimming => state.message("You feel confident in water."),
            _ => {}
        }
    }
}

/// Remove effects when taking off boots
pub fn boots_off(state: &mut GameState, obj: &Object) {
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .remove_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::Levitation if !state.player.properties.has(Property::Levitation) => {
                state.message("You float gently to the ground.");
            }
            Property::Speed => state.message("You feel slower."),
            _ => {}
        }
    }
}

/// Apply effects when putting on a cloak
pub fn cloak_on(state: &mut GameState, obj: &Object) {
    // Cloak of invisibility, magic resistance, etc.
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .grant_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::Invisibility => {
                if !state
                    .player
                    .properties
                    .has_intrinsic(Property::SeeInvisible)
                {
                    state.message("Suddenly you cannot see yourself.");
                }
            }
            Property::MagicResistance => state.message("You feel resistive to magic."),
            Property::Stealth => state.message("You feel stealthy."),
            _ => {}
        }
    }
}

/// Remove effects when taking off a cloak
pub fn cloak_off(state: &mut GameState, obj: &Object) {
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .remove_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::Invisibility if !state.player.properties.has(Property::Invisibility) => {
                state.message("Suddenly you can see yourself.");
            }
            Property::Stealth => state.message("You feel less stealthy."),
            _ => {}
        }
    }
}

/// Apply effects when putting on gloves
pub fn gloves_on(state: &mut GameState, obj: &Object) {
    // Gauntlets of power, dexterity, fumbling
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .grant_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::HalfPhysDamage => state.message("You feel stronger!"), // Power gauntlets
            Property::Fumbling => state.message("Your hands feel clumsy."),
            _ => {}
        }
    }
}

/// Remove effects when taking off gloves
pub fn gloves_off(state: &mut GameState, obj: &Object) {
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .remove_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::HalfPhysDamage => state.message("You feel weaker."),
            Property::Fumbling => state.message("Your hands feel more dextrous."),
            _ => {}
        }
    }
}

/// Apply effects when putting on a helmet
pub fn helmet_on(state: &mut GameState, obj: &Object) {
    // Helm of telepathy, brilliance, etc.
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .grant_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::Telepathy => state.message("You feel a strange mental acuity."),
            _ => {}
        }
    }
}

/// Remove effects when taking off a helmet
pub fn helmet_off(state: &mut GameState, obj: &Object) {
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .remove_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::Telepathy if !state.player.properties.has(Property::Telepathy) => {
                state.message("Your mind feels clouded.");
            }
            _ => {}
        }
    }
}

pub fn ring_gone(state: &mut GameState, obj: &Object) {
    // Stub
}

pub fn ring_off_or_gone(state: &mut GameState, obj: &Object, gone: bool) {
    // Stub
}

/// Apply effects when putting on a shield
pub fn shield_on(state: &mut GameState, obj: &Object) {
    // Shield of reflection, etc.
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .grant_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
        match prop {
            Property::Reflection => state.message("You feel a strange sense of security."),
            _ => {}
        }
    }
}

/// Remove effects when taking off a shield
pub fn shield_off(state: &mut GameState, obj: &Object) {
    if let Some(prop) = get_property_effects(obj.object_type).first() {
        state
            .player
            .properties
            .remove_extrinsic(*prop, PropertyFlags::FROM_ARMOR);
    }
}

/// Apply effects when putting on a shirt
pub fn shirt_on(state: &mut GameState, obj: &Object) {
    // T-shirt, Hawaiian shirt
    // Shirts generally don't grant properties
}

/// Remove effects when taking off a shirt
pub fn shirt_off(state: &mut GameState, _obj: &Object) {
    // Shirts generally don't grant properties
}

/// Apply effects when putting on a blindfold/towel (worn tool)
pub fn blindf_on(state: &mut GameState, obj: &Object) {
    // Blindfold blinds the player
    state.message("You can't see any more.");
    state.player.blinded_timeout = 9999; // Indefinite while worn

    // Towel doesn't blind
    // Would check object type here
}

pub fn accessory_has_effect(obj: &Object) -> bool {
    true
}

pub fn accessory_or_armor_on(state: &mut GameState, obj: &Object) {
    // Stub
}

pub fn already_wearing2(state: &GameState, slot: u32) -> bool {
    already_wearing(state, slot)
}

pub fn select_off(state: &mut GameState, obj: &Object) {
    // Stub
}

pub fn menu_remarm(state: &mut GameState) {
    // Stub
}

pub fn doddoremarm() {
    // Stub
}

pub fn reset_remarm() {
    // Stub
}

pub fn destroy_arm(state: &mut GameState, obj: &Object) {
    // Stub
}

pub fn break_armor(state: &mut GameState, obj: &Object) {
    // Stub
}

pub fn breakarm(state: &mut GameState, obj: &Object) {
    // Stub
}

pub fn mon_break_armor(state: &mut GameState, monster_id: u32, break_all: bool) {
    // Stub
}

pub fn m_lose_armor(state: &mut GameState, monster_id: u32, obj: &Object) {
    // Stub
}

pub fn m_dowear(state: &mut GameState, monster_id: u32, obj: &Object) -> bool {
    true
}

pub fn m_dowear_type(state: &mut GameState, monster_id: u32, slot: u32, obj: &Object) -> bool {
    true
}

pub fn cloak_simple_name(obj: &Object) -> String {
    "cloak".to_string()
}

pub fn gloves_simple_name(obj: &Object) -> String {
    "gloves".to_string()
}

pub fn helm_simple_name(obj: &Object) -> String {
    "helmet".to_string()
}

pub fn suit_simple_name(obj: &Object) -> String {
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

pub fn slots_required(obj: &Object) -> u32 {
    // Stub
    0
}

pub fn setworn(obj: &mut Object, mask: u32) {
    obj.worn_mask |= mask;
}

pub fn setnotworn(obj: &mut Object) {
    obj.worn_mask = 0;
}

pub fn wearmask_to_obj(mask: u32) -> Option<Object> {
    // Stub
    None
}

pub fn stuck_ring(state: &GameState, ring: &Object) -> bool {
    false
}

pub fn remove_worn_item(state: &mut GameState, obj: &mut Object) {
    obj.worn_mask = 0;
}

pub fn fingers_or_gloves(state: &GameState) -> String {
    "fingers".to_string()
}
