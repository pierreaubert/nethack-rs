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

/// Worn mask constants matching NetHack
pub mod worn_mask {
    pub const W_ARM: u32 = 0x00000001;   // Body armor
    pub const W_ARMC: u32 = 0x00000002;  // Cloak
    pub const W_ARMH: u32 = 0x00000004;  // Helmet
    pub const W_ARMS: u32 = 0x00000008;  // Shield
    pub const W_ARMG: u32 = 0x00000010;  // Gloves
    pub const W_ARMF: u32 = 0x00000020;  // Boots
    pub const W_ARMU: u32 = 0x00000040;  // Undershirt
    pub const W_WEP: u32 = 0x00000100;   // Wielded weapon
    pub const W_SWAPWEP: u32 = 0x00000200; // Secondary weapon
    pub const W_QUIVER: u32 = 0x00000400; // Quivered ammo
    pub const W_AMUL: u32 = 0x00010000;  // Amulet
    pub const W_RINGL: u32 = 0x00020000; // Left ring
    pub const W_RINGR: u32 = 0x00040000; // Right ring
    pub const W_TOOL: u32 = 0x00080000;  // Worn tool
    
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

    // Second pass: actually wear it
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.worn_mask |= slot;
    }

    state.message(format!("You put on {}.", obj_name));
    ActionResult::Success
}

/// Take off armor
pub fn do_takeoff(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (obj_name, worn_mask, is_cursed) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };
        (obj.display_name(), obj.worn_mask, obj.is_cursed())
    };

    if worn_mask & W_ARMOR == 0 {
        return ActionResult::Failed("You're not wearing that.".to_string());
    }

    if is_cursed {
        state.message("You can't. It is cursed.");
        return ActionResult::Failed("You can't remove it, it's cursed!".to_string());
    }

    // Second pass: actually remove it
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.worn_mask &= !W_ARMOR;
    }

    state.message(format!("You take off {}.", obj_name));
    ActionResult::Success
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
                if !state.player.properties.has_intrinsic(Property::SeeInvisible) {
                    effect.messages.push("Suddenly you cannot see yourself.".to_string());
                    effect.identify = true;
                }
            }
            Property::SeeInvisible => {
                effect.messages.push("Your vision seems to sharpen.".to_string());
            }
            Property::Levitation => {
                effect.messages.push("You start to float in the air!".to_string());
                effect.identify = true;
            }
            Property::Conflict => {
                effect.messages.push("You feel like a rabble-rouser.".to_string());
            }
            _ => {}
        }
    }

    // Handle stat-modifying rings
    match object_type {
        RIN_GAIN_STRENGTH => {
            let bonus = ring.enchantment;
            if bonus != 0 {
                state.player.attr_current.modify(crate::player::Attribute::Strength, bonus);
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
                state.player.attr_current.modify(crate::player::Attribute::Constitution, bonus);
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
                state.player.attr_current.modify(crate::player::Attribute::Charisma, bonus);
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
                    effect.messages.push("Suddenly you can see yourself again.".to_string());
                }
            }
            Property::Levitation => {
                if !state.player.properties.has(Property::Levitation) {
                    effect.messages.push("You float gently to the ground.".to_string());
                }
            }
            _ => {}
        }
    }

    // Remove stat bonuses
    match object_type {
        RIN_GAIN_STRENGTH => {
            state.player.attr_current.modify(crate::player::Attribute::Strength, -ring.enchantment);
        }
        RIN_GAIN_CONSTITUTION => {
            state.player.attr_current.modify(crate::player::Attribute::Constitution, -ring.enchantment);
        }
        RIN_ADORNMENT => {
            state.player.attr_current.modify(crate::player::Attribute::Charisma, -ring.enchantment);
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
        state.player.properties.grant_extrinsic(prop, PropertyFlags::FROM_AMULET);
    }

    // Special amulet effects
    match object_type {
        AMULET_OF_ESP => {
            effect.messages.push("You feel a strange mental acuity.".to_string());
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
            effect.messages.push("You are suddenly very different!".to_string());
            effect.messages.push("The amulet disintegrates!".to_string());
            effect.identify = true;
            effect.destroy = true;
        }
        AMULET_OF_UNCHANGING => {
            // Prevents polymorph
        }
        AMULET_OF_REFLECTION => {
            effect.messages.push("You feel a strange sense of security.".to_string());
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
        state.player.properties.remove_extrinsic(prop, PropertyFlags::FROM_AMULET);
    }

    // Special removal effects
    match object_type {
        AMULET_OF_ESP => {
            if !state.player.properties.has(Property::Telepathy) {
                effect.messages.push("Your mental acuity fades.".to_string());
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
