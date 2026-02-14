//! Reading scrolls (read.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::magic::genocide::{do_class_genocide, do_genocide, do_reverse_genocide};
use crate::magic::scroll::{ScrollType, read_scroll};
use crate::object::{Object, ObjectClass};

/// Read a scroll from inventory
pub fn do_read(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Get the scroll from inventory
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    // Check item type
    match obj.class {
        ObjectClass::Scroll => {
            // Proceed with reading scroll
        }
        ObjectClass::Spellbook => {
            return study_book(state, &obj);
        }
        ObjectClass::Armor => {
            // Check for T-shirt or Apron (using object_type placeholders)
            // T-shirt: 10, Apron: 11 (stub values)
            if obj.object_type == 10 {
                let msg = tshirt_text(&obj);
                state.message(format!("It reads: \"{}\"", msg));
                return ActionResult::Success;
            } else if obj.object_type == 11 {
                let msg = apron_text(&obj);
                state.message(format!("It reads: \"{}\"", msg));
                return ActionResult::Success;
            }
            return ActionResult::Failed("That's not something you can read.".to_string());
        }
        _ => {
            return ActionResult::Failed("That's not something you can read.".to_string());
        }
    }

    // Special handling for genocide scroll - needs GameState for cross-level operations
    if let Some(ScrollType::Genocide) = ScrollType::from_object_type(obj.object_type) {
        let blessed = obj.is_blessed();
        let cursed = obj.is_cursed();

        if cursed {
            state.message("A thunderous voice booms: FOOL!");
            state.message("You have angered the gods!");

            // Reverse genocide - spawn monsters around player
            // Use a default monster type (type 10 as example, could be randomized)
            let monster_type = 10;
            let result = do_reverse_genocide(
                monster_type,
                (state.player.pos.x, state.player.pos.y),
                state,
            );
            for msg in result.messages {
                state.message(msg);
            }
        } else if blessed {
            state.message("What class of monsters do you wish to genocide?");

            // Blessed scroll - genocide entire class
            // Uses a default class symbol 'd' (dragons) - in a real UI, player would select
            // For now, this demonstrates the full genocide capability
            let result = do_class_genocide('d', state);
            for msg in result.messages {
                state.message(msg);
            }
        } else {
            state.message("What monster do you wish to genocide?");

            // Normal scroll - genocide single monster type
            // Would normally get player input to select a specific monster
            // For now, if polymorphed, use that type; otherwise use default
            let target_type = if let Some(player_monster_type) = state.player.monster_num {
                player_monster_type
            } else {
                // Default to monster type 5 if not polymorphed
                5
            };

            let result = do_genocide(target_type, state);
            for msg in result.messages {
                state.message(msg);
            }

            if result.player_died {
                state.message("You have been annihilated!");
                return ActionResult::Failed("You have committed self-genocide!".to_string());
            }
        }

        // Consume the scroll
        state.remove_from_inventory(obj_letter);
        return ActionResult::Success;
    }

    // Apply scroll effects for all other scrolls
    let result = read_scroll(
        &obj,
        &mut state.player,
        &mut state.current_level,
        &mut state.rng,
    );

    // Display messages
    for msg in result.messages {
        state.message(msg);
    }

    // Consume the scroll if it was used
    if result.consumed {
        state.remove_from_inventory(obj_letter);
    }

    ActionResult::Success
}

pub fn doread(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_read(state, obj_letter)
}

pub fn seffects(state: &mut GameState, obj: &Object) {
    // Stub for applying scroll effects directly
    let result = read_scroll(
        obj,
        &mut state.player,
        &mut state.current_level,
        &mut state.rng,
    );
    for msg in result.messages {
        state.message(msg);
    }
}

/// Study a spellbook to learn its spell
pub fn study_book(state: &mut GameState, book: &Object) -> ActionResult {
    use crate::magic::spell::{KnownSpell, SpellType};

    // Check if player is confused
    if state.player.is_confused() {
        if confused_book(state, book) {
            // Book was destroyed
            return ActionResult::Success;
        }
        return ActionResult::Success;
    }

    // Check if book is cursed
    if book.is_cursed() {
        cursed_book(state, book);
        return ActionResult::Success;
    }

    // Check for blank spellbook
    if book.object_type == 0 {
        // SPE_BLANK_PAPER placeholder
        state.message("This spellbook is all blank.");
        return ActionResult::Success;
    }

    state.message("You begin to study the spellbook...");

    // Try to learn the spell
    // Convert book object_type to SpellType (simplified mapping)
    if let Some(spell_type) = spell_type_from_book(book.object_type) {
        // Check if already known
        let already_known = state
            .player
            .known_spells
            .iter()
            .any(|s| s.spell_type == spell_type);

        if already_known {
            // Refresh spell memory
            if let Some(spell) = state
                .player
                .known_spells
                .iter_mut()
                .find(|s| s.spell_type == spell_type)
            {
                spell.turns_remaining = 20000; // ~500 game turns worth
                state.message("You refresh your memory of the spell.");
            }
        } else {
            // Learn new spell
            let new_spell = KnownSpell {
                spell_type,
                turns_remaining: 20000,
                times_cast: 0,
                times_failed: 0,
            };
            state.player.known_spells.push(new_spell);
            state.message(format!(
                "You have learned the spell of {}!",
                spell_type.name()
            ));
        }
    } else {
        state.message("You read the strange runes but cannot decipher them.");
    }

    ActionResult::Success
}

/// Helper to convert book object_type to SpellType
fn spell_type_from_book(book_type: i16) -> Option<crate::magic::spell::SpellType> {
    use crate::magic::spell::SpellType;
    // Simplified mapping - in full implementation would use ObjClassDef
    match book_type {
        1 => Some(SpellType::ForceBolt),
        2 => Some(SpellType::MagicMissile),
        3 => Some(SpellType::Fireball),
        4 => Some(SpellType::ConeOfCold),
        5 => Some(SpellType::Sleep),
        6 => Some(SpellType::FingerOfDeath),
        7 => Some(SpellType::Drain),
        8 => Some(SpellType::Healing),
        9 => Some(SpellType::ExtraHealing),
        10 => Some(SpellType::CureBlindness),
        11 => Some(SpellType::CureSickness),
        12 => Some(SpellType::CreateMonster),
        13 => Some(SpellType::Confuse),
        14 => Some(SpellType::Slow),
        15 => Some(SpellType::Haste),
        16 => Some(SpellType::Invisibility),
        17 => Some(SpellType::DetectMonsters),
        18 => Some(SpellType::Clairvoyance),
        19 => Some(SpellType::DetectFood),
        20 => Some(SpellType::DetectUnseen),
        _ => None,
    }
}

/// Reading a spellbook while confused may destroy it
/// Returns true if the book was destroyed
pub fn confused_book(state: &mut GameState, book: &Object) -> bool {
    state.message("Being confused you have difficulties controlling your actions.");

    // 1 in 3 chance to tear the book
    if state.rng.one_in(3) {
        state.message("You accidentally tear the spellbook to pieces!");
        return true;
    }

    state.message("You find yourself reading the first line over and over again.");
    false
}

/// Effects of reading a cursed spellbook
pub fn cursed_book(state: &mut GameState, book: &Object) {
    use crate::player::Attribute;

    state.message("The book is cursed!");

    // Random bad effects
    let effect = state.rng.rn2(5);
    match effect {
        0 => {
            state.message("You feel a surge of pain!");
            state.player.take_damage(state.rng.dice(2, 6) as i32);
        }
        1 => {
            state.message("Your mind reels!");
            state.player.make_confused(30, true);
        }
        2 => {
            state.message("A cloud of noxious gas surrounds you!");
            state.player.make_stunned(20, true);
        }
        3 => {
            state.message("You feel weaker...");
            let current_str = state.player.attr_current.get(Attribute::Strength);
            state
                .player
                .attr_current
                .set(Attribute::Strength, current_str.saturating_sub(1));
        }
        _ => {
            state.message("You shudder for a moment.");
        }
    }
}

pub fn book_cursed(state: &mut GameState, book: &Object) {
    cursed_book(state, book);
}

pub fn book_disappears(state: &mut GameState, book: &Object) {
    state.message("The book disappears in a puff of smoke!");
}

/// Substitute a different book (polymorphed book effect)
pub fn book_substitution(state: &mut GameState, _book: &Object) {
    state.message("The book seems different somehow...");
}

/// The book's description changes (appearance randomization)
pub fn new_book_description(state: &mut GameState, _book: &Object) {
    state.message("You notice new markings on the cover.");
}

pub fn learnscroll(state: &mut GameState, obj: &Object) {
    state.message(format!("You learn about {}.", obj.display_name()));
}

pub fn learnscrolltyp(_scroll_type: i16) {
    // Called to mark a scroll type as known - handled by scroll identification system
}

/// Special effects for The Book of the Dead
pub fn deadbook(state: &mut GameState, _book: &Object) {
    state.message("You turn the pages of the Book of the Dead...");

    // Random chance to summon undead
    let roll = state.rng.rn2(7);
    match roll {
        0..=1 => {
            state.message("Spirits rise from the pages!");
            // Would summon ghosts here
        }
        2..=3 => {
            state.message("You hear the rattling of bones!");
            // Would summon skeletons here
        }
        4 => {
            state.message("A chill wind emanates from the book.");
            state.player.take_damage(state.rng.dice(1, 10) as i32);
        }
        5 => {
            state.message("Dark whispers fill your mind...");
            state.player.make_confused(15, true);
        }
        _ => {
            state.message("The book hums with dark energy.");
        }
    }
}

pub fn lookup_novel(state: &mut GameState, book: &Object) {
    state.message("You read the novel.");
}

pub fn noveltitle(book: &Object) -> String {
    "A Tale of Two Cities".to_string()
}

pub fn tshirt_text(tshirt: &Object) -> String {
    let msgs = [
        "I explored the Dungeons of Doom and all I got was this lousy T-shirt!",
        "Is that Mjollnir in your pocket or are you just happy to see me?",
        "Don't Panic",
    ];
    let idx = (tshirt.id.0 as usize) % msgs.len();
    msgs[idx].to_string()
}

pub fn apron_text(apron: &Object) -> String {
    let msgs = [
        "Kiss the cook",
        "I'm making SCIENCE!",
        "Don't mess with the chef",
    ];
    let idx = (apron.id.0 as usize) % msgs.len();
    msgs[idx].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass, ObjectId};
    use crate::rng::GameRng;

    #[test]
    fn test_read_scroll() {
        let mut state = GameState::new(GameRng::from_entropy());
        let mut obj = Object::default();
        obj.id = ObjectId(1);
        obj.class = ObjectClass::Scroll;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        // This will likely fail or do nothing if the scroll type is 0 (Mail) or similar
        // But do_read should handle it gracefully
        let result = do_read(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
    }
}
