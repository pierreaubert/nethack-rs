//! Quaffing potions (potion.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::magic::potion::quaff_potion;
use crate::object::{Object, ObjectClass};

/// Quaff a potion from inventory
pub fn do_quaff(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Get the potion from inventory
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if obj.class != ObjectClass::Potion {
        return ActionResult::Failed("That's not something you can drink.".to_string());
    }

    // Apply potion effects
    let result = quaff_potion(&obj, &mut state.player, &mut state.rng);

    // Display messages
    for msg in result.messages {
        state.message(msg);
    }

    // Consume the potion if it was used
    if result.consumed {
        state.remove_from_inventory(obj_letter);
    }

    if result.player_died {
        return ActionResult::Died("poisoned".to_string());
    }

    ActionResult::Success
}

pub fn dodrink(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_quaff(state, obj_letter)
}

pub fn dopotion(state: &mut GameState, obj: &Object) {
    // Apply potion effects directly (without quaffing mechanics like consuming?)
    let result = quaff_potion(obj, &mut state.player, &mut state.rng);
    for msg in result.messages {
        state.message(msg);
    }
}

pub fn peffects(state: &mut GameState, obj: &Object) {
    dopotion(state, obj);
}

pub fn potionhit(state: &mut GameState, obj: &Object) {
    state.message("The potion shatters!");
}

/// Dip an object into a potion (holy water blessing, etc.)
pub fn h2_opotion_dip(state: &mut GameState, obj: &mut Object, potion: &Object) {
    use crate::object::BucStatus;

    state.message("You dip the object into the potion.");

    // Check for holy/unholy water effects
    if potion.object_type == crate::magic::potion::PotionType::Water as i16 {
        match potion.buc {
            BucStatus::Blessed => {
                // Holy water - bless the object
                if obj.buc == BucStatus::Cursed {
                    obj.buc = BucStatus::Uncursed;
                    state.message("The object glows with a soft light.");
                } else if obj.buc == BucStatus::Uncursed {
                    obj.buc = BucStatus::Blessed;
                    state.message("The object glows with a bright light!");
                } else {
                    state.message("The object seems slightly brighter.");
                }
            }
            BucStatus::Cursed => {
                // Unholy water - curse the object
                if obj.buc == BucStatus::Blessed {
                    obj.buc = BucStatus::Uncursed;
                    state.message("The object's glow fades.");
                } else if obj.buc == BucStatus::Uncursed {
                    obj.buc = BucStatus::Cursed;
                    state.message("The object turns dark.");
                } else {
                    state.message("The object seems slightly darker.");
                }
            }
            BucStatus::Uncursed => {
                state.message("The water evaporates.");
            }
        }
    }
}

/// Dip an object into a potion
pub fn dodip(state: &mut GameState, obj_letter: char, potion_letter: char) -> ActionResult {
    use crate::object::BucStatus;

    // Get both objects
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    let potion = match state.get_inventory_item(potion_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that potion.".to_string()),
    };

    if potion.class != ObjectClass::Potion {
        return ActionResult::Failed("That's not a potion.".to_string());
    }

    // Can't dip potion into itself
    if obj_letter == potion_letter {
        return ActionResult::Failed("You can't dip something into itself.".to_string());
    }

    // Handle dipping based on what we're dipping
    if obj.class == ObjectClass::Potion {
        // Mixing potions
        let mix_result = mixtype(&obj, &potion, &mut state.rng);
        if mix_result != 0 {
            state.message("The potions mix together!");
            // In full implementation, would create new potion
        } else {
            state.message("The potions fizz briefly but nothing happens.");
        }
    } else {
        // Dipping non-potion object - inline the dip logic to avoid borrow issues
        state.message("You dip the object into the potion.");

        if potion.object_type == crate::magic::potion::PotionType::Water as i16 {
            // Get the object mutably and apply water effects
            // Capture the message to send after the borrow is released
            let effect_msg = if let Some(obj_mut) = state.get_inventory_item_mut(obj_letter) {
                match potion.buc {
                    BucStatus::Blessed => {
                        // Holy water - bless the object
                        if obj_mut.buc == BucStatus::Cursed {
                            obj_mut.buc = BucStatus::Uncursed;
                            Some("The object glows with a soft light.")
                        } else if obj_mut.buc == BucStatus::Uncursed {
                            obj_mut.buc = BucStatus::Blessed;
                            Some("The object glows with a bright light!")
                        } else {
                            Some("The object seems slightly brighter.")
                        }
                    }
                    BucStatus::Cursed => {
                        // Unholy water - curse the object
                        if obj_mut.buc == BucStatus::Blessed {
                            obj_mut.buc = BucStatus::Uncursed;
                            Some("The object's glow fades.")
                        } else if obj_mut.buc == BucStatus::Uncursed {
                            obj_mut.buc = BucStatus::Cursed;
                            Some("The object turns dark.")
                        } else {
                            Some("The object seems slightly darker.")
                        }
                    }
                    BucStatus::Uncursed => Some("The water evaporates."),
                }
            } else {
                None
            };
            // Send message after the borrow is released
            if let Some(msg) = effect_msg {
                state.message(msg);
            }
        }
    }

    // Consume the potion
    state.remove_from_inventory(potion_letter);

    ActionResult::Success
}

/// Dip an object into a fountain
pub fn dipfountain(state: &mut GameState, obj: &mut Object) {
    state.message("You dip the object into the fountain.");

    // Random fountain effects
    let roll = state.rng.rn2(30);
    match roll {
        0..=2 => {
            // Water demon might appear
            state.message("A water demon appears!");
        }
        3..=5 => {
            // Object might rust
            if obj.class == ObjectClass::Weapon || obj.class == ObjectClass::Armor {
                if obj.erosion1 < 3 && !obj.erosion_proof {
                    obj.erosion1 += 1;
                    state.message("The object rusts!");
                }
            }
        }
        6..=8 => {
            // Fountain might dry up
            state.message("The fountain dries up!");
            // In full implementation, would change cell type
        }
        _ => {
            state.message("The water splashes.");
        }
    }
}

/// Drink from a fountain
pub fn drinkfountain(state: &mut GameState) {
    state.message("You drink from the fountain.");

    // Random fountain effects
    let roll = state.rng.rn2(30);
    match roll {
        0 => {
            state.message("This water is foul! You feel sick.");
            // Could apply sickness
        }
        1..=3 => {
            state.message("The water is cool and refreshing.");
            state.player.hp = state.player.hp.saturating_add(1);
        }
        4..=5 => {
            state.message("You see an image of someone very ugly!");
            // Self reflection effect
        }
        6 => {
            state.message("A water nymph appears!");
        }
        7 => {
            state.message("You feel a strange tingling.");
            // Could grant see invisible
        }
        8..=10 => {
            state.message("The fountain dries up!");
        }
        _ => {
            state.message("The water tastes normal.");
        }
    }
}

/// Drink from a sink
pub fn drinksink(state: &mut GameState) {
    state.message("You drink from the sink.");

    let roll = state.rng.rn2(20);
    match roll {
        0 => {
            state.message("Yuk, this water tastes awful!");
            state.player.nutrition = state.player.nutrition.saturating_sub(10);
        }
        1..=2 => {
            state.message("Gag! This water is foul!");
        }
        3..=4 => {
            state.message("A black ooze flows from the faucet!");
        }
        5 => {
            state.message("A ring comes up from the drain!");
            // Would drop a ring here
        }
        6 => {
            state.message("You hear a gurgling noise.");
        }
        _ => {
            state.message("You drink some tap water.");
        }
    }
}

/// Determine what mixing two potions produces (potion.c:1789 mixtype)
/// Returns the resulting potion object_type or 0 for no reaction.
/// Port of C's mixtype() with the canonical swap normalization.
pub fn mixtype(obj1: &Object, obj2: &Object, rng: &mut crate::rng::GameRng) -> i16 {
    use crate::magic::potion::PotionType;

    let mut o1typ = obj1.object_type;
    let mut o2typ = obj2.object_type;

    // C normalization: swap so that the "catalyst" potions are always o2
    if obj1.class == ObjectClass::Potion {
        match PotionType::from_object_type(o2typ) {
            Some(
                PotionType::GainLevel
                | PotionType::GainEnergy
                | PotionType::Healing
                | PotionType::ExtraHealing
                | PotionType::FullHealing
                | PotionType::Enlightenment
                | PotionType::FruitJuice,
            ) => {
                std::mem::swap(&mut o1typ, &mut o2typ);
            }
            _ => {}
        }
    }

    let p1 = PotionType::from_object_type(o1typ);
    let p2 = PotionType::from_object_type(o2typ);

    match p1 {
        Some(PotionType::Healing) => {
            if p2 == Some(PotionType::Speed) {
                return PotionType::ExtraHealing as i16;
            }
            // Fall through to healing chain
            match p2 {
                Some(PotionType::GainLevel | PotionType::GainEnergy) => {
                    return PotionType::ExtraHealing as i16;
                }
                Some(PotionType::Sickness) => return PotionType::FruitJuice as i16,
                Some(PotionType::Hallucination | PotionType::Blindness | PotionType::Confusion) => {
                    return PotionType::Water as i16;
                }
                _ => {}
            }
        }
        Some(PotionType::ExtraHealing) => {
            match p2 {
                Some(PotionType::GainLevel | PotionType::GainEnergy) => {
                    return PotionType::FullHealing as i16;
                }
                Some(PotionType::Sickness) => return PotionType::FruitJuice as i16,
                Some(PotionType::Hallucination | PotionType::Blindness | PotionType::Confusion) => {
                    return PotionType::Water as i16;
                }
                _ => {}
            }
        }
        Some(PotionType::FullHealing) => {
            match p2 {
                Some(PotionType::GainLevel | PotionType::GainEnergy) => {
                    return PotionType::GainAbility as i16;
                }
                Some(PotionType::Sickness) => return PotionType::FruitJuice as i16,
                Some(PotionType::Hallucination | PotionType::Blindness | PotionType::Confusion) => {
                    return PotionType::Water as i16;
                }
                _ => {}
            }
        }
        Some(PotionType::GainLevel | PotionType::GainEnergy) => {
            match p2 {
                Some(PotionType::Confusion) => {
                    // 2/3 chance booze, 1/3 enlightenment
                    return if rng.rn2(3) != 0 {
                        PotionType::Booze as i16
                    } else {
                        PotionType::Enlightenment as i16
                    };
                }
                Some(PotionType::Healing) => return PotionType::ExtraHealing as i16,
                Some(PotionType::ExtraHealing) => return PotionType::FullHealing as i16,
                Some(PotionType::FullHealing) => return PotionType::GainAbility as i16,
                Some(PotionType::FruitJuice) => return PotionType::SeeInvisible as i16,
                Some(PotionType::Booze) => return PotionType::Hallucination as i16,
                _ => {}
            }
        }
        Some(PotionType::FruitJuice) => {
            match p2 {
                Some(PotionType::Sickness) => return PotionType::Sickness as i16,
                Some(PotionType::Enlightenment | PotionType::Speed) => {
                    return PotionType::Booze as i16;
                }
                Some(PotionType::GainLevel | PotionType::GainEnergy) => {
                    return PotionType::SeeInvisible as i16;
                }
                _ => {}
            }
        }
        Some(PotionType::Enlightenment) => {
            match p2 {
                Some(PotionType::Levitation) => {
                    if rng.rn2(3) != 0 {
                        return PotionType::GainLevel as i16;
                    }
                }
                Some(PotionType::FruitJuice) => return PotionType::Booze as i16,
                Some(PotionType::Booze) => return PotionType::Confusion as i16,
                _ => {}
            }
        }
        _ => {}
    }

    0 // No special mixture
}

pub fn ghost_from_bottle(state: &mut GameState) {
    state.message("A ghost rises from the bottle!");
}

pub fn djinni_from_bottle(state: &mut GameState) {
    state.message("A djinni rises from the bottle!");
}

pub fn bottlename() -> String {
    "bottle".to_string()
}

pub fn mquaffmsg(state: &mut GameState, monster_name: &str, potion_name: &str) {
    state.message(format!("{} drinks a {}.", monster_name, potion_name));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass, ObjectId};
    use crate::rng::GameRng;

    #[test]
    fn test_quaff_potion() {
        let mut state = GameState::new(GameRng::from_entropy());
        let mut obj = Object::default();
        obj.id = ObjectId(1);
        obj.class = ObjectClass::Potion;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        let result = do_quaff(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
    }
}
