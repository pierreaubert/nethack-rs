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
use crate::object::ObjectClass;

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
