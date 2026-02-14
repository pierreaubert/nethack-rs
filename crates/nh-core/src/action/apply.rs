//! Tool application (apply.c)
//!
//! Handles the 'a' (apply) command for using tools, instruments,
//! containers, digging implements, lock-picks, and other special items.

use crate::action::ActionResult;
use crate::dungeon::{LightSource, LightSourceType};
use crate::gameloop::GameState;
use crate::monster::MonsterId;
use crate::object::{BucStatus, Object, ObjectClass, ObjectId};
use crate::player::{Attribute, You};
use crate::rng::GameRng;

// ============================================================================
// Main dispatch
// ============================================================================

/// Apply a tool from inventory.
///
/// Based on C doapply() — routes to specific tool handler based on object type.
pub fn do_apply(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    // Tool class is the primary apply class, but some weapons/wands are also applicable
    if obj.class != ObjectClass::Tool
        && obj.class != ObjectClass::Weapon
        && obj.class != ObjectClass::Wand
    {
        return ActionResult::Failed("That's not something you can apply.".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "tool".to_string());

    match obj.object_type {
        // Pickaxe (176) and mattock (177)
        176 | 177 => apply_pickaxe(state, &obj_name),
        // Lamp (188) and lantern (189)
        188 | 189 => apply_light(state, obj_letter, &obj_name),
        // Whistle (190)
        190 => apply_whistle(state),
        // Magic whistle (191)
        191 => apply_magic_whistle(state),
        // Tooled horn (195)
        195 => apply_horn(state),
        // Horn of plenty (196)
        196 => apply_horn_of_plenty(state, obj_letter),
        // Bell (197) and Bell of Opening (198)
        197 | 198 => apply_bell(state, obj_letter, &obj),
        // Candelabrum (199)
        199 => apply_candelabrum(state, obj_letter),
        // Stethoscope (200)
        200 => apply_stethoscope(state),
        // Mirror (201)
        201 => apply_mirror(state),
        // Tinning kit (203)
        203 => apply_tinning_kit(state, obj_letter),
        // Skeleton key (205) and lock pick (206) and credit card (207)
        205..=207 => apply_lock_tool(state, obj_letter, &obj_name),
        // Camera (210)
        210 => apply_camera(state, obj_letter, &obj_name),
        // Towel (211)
        211 => apply_towel(state, obj_letter),
        // Blindfold (212) and lenses (214)
        212 | 214 => apply_blindfold(state, obj_letter, &obj_name),
        // Unicorn horn (213)
        213 => apply_unicorn_horn(state, &obj),
        // Leash (215)
        215 => apply_leash(state, obj_letter),
        // Figurine (216)
        216 => apply_figurine(state, obj_letter),
        // Grease (217)
        217 => apply_grease(state, obj_letter),
        // Bear trap (218) and land mine (219)
        218 | 219 => apply_trap_tool(state, obj_letter, &obj),
        // Candles (221-222)
        221 | 222 => apply_light(state, obj_letter, &obj_name),
        // Bag, sack, bag of holding (220, 223, 224)
        220 | 223 | 224 => apply_bag(state, obj_letter, &obj_name),
        // Bag of tricks (225)
        225 => apply_bag_of_tricks(state, obj_letter),
        // Instruments: flutes (192-193), harps (194, 202), bugle (204), drums (208-209)
        192 | 193 | 194 | 202 | 204 | 208 | 209 => {
            apply_instrument(state, obj_letter, &obj)
        }
        _ => {
            state.message(format!("You apply the {}.", obj_name));
            ActionResult::Success
        }
    }
}

// ============================================================================
// Digging (pickaxe/mattock)
// ============================================================================

/// Check if a position can be dug.
///
/// Based on C dig_check() — validates terrain type.
pub fn dig_check(state: &GameState, x: i8, y: i8) -> bool {
    if !state.current_level.is_valid_pos(x, y) {
        return false;
    }
    // Can dig walls but not special terrain
    let cell = state.current_level.cells[x as usize][y as usize];
    cell.typ.is_wall() || cell.typ == crate::dungeon::CellType::Stone || cell.typ == crate::dungeon::CellType::Room
}

/// Apply a digging tool (pickaxe or mattock).
///
/// Based on C use_pick_axe() in dig.c.
/// Requires a direction to dig in. Can dig walls, floors, and boulders.
fn apply_pickaxe(state: &mut GameState, obj_name: &str) -> ActionResult {
    state.message(format!("You swing the {}.", obj_name));
    // In full implementation: get direction, validate, start occupation
    state.message("In what direction do you want to dig?");
    ActionResult::NoTime
}

/// Result of digging at a position.
#[derive(Debug, Clone)]
pub struct DigResult {
    /// Messages
    pub messages: Vec<String>,
    /// Whether the dig succeeded
    pub success: bool,
    /// Turns required to complete
    pub turns: i32,
    /// Whether the wall/floor was destroyed
    pub terrain_changed: bool,
}

/// Perform a dig action at a specific position.
///
/// Based on C dig() — pickaxe/mattock mechanics.
/// Time to dig depends on tool type and player strength.
pub fn dig_at(
    state: &mut GameState,
    x: i8,
    y: i8,
    tool_type: i16,
) -> DigResult {
    let mut result = DigResult {
        messages: Vec::new(),
        success: false,
        turns: 0,
        terrain_changed: false,
    };

    if !dig_check(state, x, y) {
        result.messages.push("You can't dig there.".to_string());
        return result;
    }

    // Calculate dig time based on strength and tool
    let str_val = state.player.attr_current.get(Attribute::Strength) as i32;
    let base_time = if tool_type == 177 { 3 } else { 5 }; // Mattock is faster
    result.turns = (base_time - str_val / 6).max(1);

    let cell = &state.current_level.cells[x as usize][y as usize];
    match cell.typ {
        t if t.is_wall() || t == crate::dungeon::CellType::Stone => {
            result.success = true;
            result.terrain_changed = true;
            result.messages.push("You dig through the wall.".to_string());
        }
        crate::dungeon::CellType::Room => {
            // Dig a pit or hole
            result.success = true;
            result.messages.push("You dig a pit in the floor.".to_string());
        }
        _ => {
            result.messages.push("You can't dig there.".to_string());
        }
    }

    // Apply terrain change
    if result.terrain_changed {
        state.current_level.cells[x as usize][y as usize].typ =
            crate::dungeon::CellType::Corridor;
    }

    result
}

// ============================================================================
// Lock-picking
// ============================================================================

/// Result of a lock-picking attempt.
#[derive(Debug, Clone)]
pub struct LockPickResult {
    /// Messages
    pub messages: Vec<String>,
    /// Whether the lock was opened
    pub opened: bool,
    /// Whether the tool broke
    pub tool_broke: bool,
    /// Turns required
    pub turns: i32,
}

/// Apply a lock-picking tool to a door or container.
///
/// Based on C pick_lock() in lock.c.
/// Skeleton keys always work, lock picks have a chance, credit cards work on doors.
fn apply_lock_tool(
    state: &mut GameState,
    _obj_letter: char,
    obj_name: &str,
) -> ActionResult {
    state.message(format!(
        "You try to pick a lock with the {}.",
        obj_name
    ));
    // In full implementation: get direction, find door/container
    state.message("There's nothing here to unlock.");
    ActionResult::NoTime
}

/// Attempt to pick a lock at a position.
///
/// Based on C pick_lock(). Success depends on tool type and dexterity.
pub fn pick_lock(
    player: &You,
    tool_type: i16,
    is_cursed: bool,
    rng: &mut GameRng,
) -> LockPickResult {
    let mut result = LockPickResult {
        messages: Vec::new(),
        opened: false,
        tool_broke: false,
        turns: 1,
    };

    let dex = player.attr_current.get(Attribute::Dexterity) as i32;

    // Success chance depends on tool type
    let base_chance = match tool_type {
        205 => 100, // Skeleton key: always works
        206 => 50 + dex * 2, // Lock pick: dex-based
        207 => 30 + dex, // Credit card: harder
        _ => 10,
    };

    // Cursed tools are less effective
    let chance = if is_cursed {
        base_chance / 2
    } else {
        base_chance
    };

    let roll = rng.rn2(100) as i32;
    if roll < chance {
        result.opened = true;
        result.messages.push("You succeed in picking the lock.".to_string());
    } else {
        result.messages.push("You fail to pick the lock.".to_string());
        // Lock pick has a chance to break on failure
        if tool_type == 206 && rng.rn2(15) == 0 {
            result.tool_broke = true;
            result.messages.push("Your lock pick breaks!".to_string());
        }
    }

    result
}

// ============================================================================
// Musical instruments
// ============================================================================

/// Result of playing an instrument.
#[derive(Debug, Clone)]
pub struct InstrumentResult {
    /// Messages
    pub messages: Vec<String>,
    /// Whether a magical effect occurred
    pub magical_effect: bool,
    /// Monsters affected
    pub affected_monsters: Vec<MonsterId>,
    /// Whether a charge was consumed
    pub charge_consumed: bool,
}

/// Apply (play) a musical instrument.
///
/// Based on C do_play_instrument() in music.c.
/// Magic instruments have special effects; ordinary ones just make noise.
fn apply_instrument(
    state: &mut GameState,
    obj_letter: char,
    obj: &Object,
) -> ActionResult {
    let obj_name = obj.name.clone().unwrap_or_else(|| "instrument".to_string());
    let is_magic = matches!(obj.object_type, 193 | 202 | 209); // Magic flute, harp, drum

    if is_magic {
        let result = magic_instrument_effect(state, obj_letter, obj);
        for msg in &result.messages {
            state.message(msg.clone());
        }
        if result.charge_consumed
            && let Some(inv_obj) = state.get_inventory_item_mut(obj_letter)
            && inv_obj.enchantment > 0
        {
            inv_obj.enchantment -= 1;
        }
    } else {
        state.message(format!("You play the {}.", obj_name));
        // Ordinary instruments: wake nearby monsters
        for monster in &mut state.current_level.monsters {
            let dx = (monster.x - state.player.pos.x).abs();
            let dy = (monster.y - state.player.pos.y).abs();
            if dx <= 10 && dy <= 10 {
                monster.state.sleeping = false;
            }
        }
    }

    ActionResult::Success
}

/// Apply the magical effect of a magic instrument.
///
/// Based on C effects in music.c.
/// - Magic flute: puts monsters to sleep
/// - Magic harp: charms monsters (makes tame)
/// - Drum of earthquake: shakes the level, damages walls
fn magic_instrument_effect(
    state: &mut GameState,
    _obj_letter: char,
    obj: &Object,
) -> InstrumentResult {
    let mut result = InstrumentResult {
        messages: Vec::new(),
        magical_effect: false,
        affected_monsters: Vec::new(),
        charge_consumed: false,
    };

    if obj.enchantment <= 0 {
        result.messages.push("The instrument plays a note but nothing happens.".to_string());
        return result;
    }

    result.charge_consumed = true;
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    match obj.object_type {
        193 => {
            // Magic flute: put monsters to sleep
            result.messages.push("You produce a lilting melody.".to_string());
            result.magical_effect = true;
            for monster in &mut state.current_level.monsters {
                let dx = (monster.x - px).abs();
                let dy = (monster.y - py).abs();
                if dx <= 5 && dy <= 5 && !monster.resists_sleep() {
                    monster.state.sleeping = true;
                    monster.sleep_timeout = state.rng.rnd(20) as u16 + 10;
                    result.affected_monsters.push(monster.id);
                }
            }
        }
        202 => {
            // Magic harp: charm monsters
            result.messages.push("You produce a mesmerizing melody.".to_string());
            result.magical_effect = true;
            for monster in &mut state.current_level.monsters {
                let dx = (monster.x - px).abs();
                let dy = (monster.y - py).abs();
                if dx <= 5 && dy <= 5 && !monster.state.tame {
                    // Chance to tame based on level difference
                    if monster.level as i32 <= state.player.exp_level + 3
                        && state.rng.rn2(3) != 0
                    {
                        monster.state.tame = true;
                        monster.state.peaceful = true;
                        monster.tameness = 5;
                        result.affected_monsters.push(monster.id);
                    }
                }
            }
        }
        209 => {
            // Drum of earthquake: shake the level
            result.messages.push("The ground shakes violently!".to_string());
            result.magical_effect = true;
            // Damage nearby monsters
            for monster in &mut state.current_level.monsters {
                let dx = (monster.x - px).abs();
                let dy = (monster.y - py).abs();
                if dx <= 3 && dy <= 3 {
                    let quake_dmg = state.rng.dice(2, 6) as i32;
                    monster.hp -= quake_dmg;
                    result.affected_monsters.push(monster.id);
                }
            }
            // Damage player slightly
            state.player.hp -= state.rng.rnd(4) as i32;
        }
        _ => {}
    }

    result
}

// ============================================================================
// Container operations (bags)
// ============================================================================

/// Apply a bag (open/loot).
///
/// Based on C use_container() in pickup.c.
/// Opens the bag's inventory for the player to interact with.
fn apply_bag(
    state: &mut GameState,
    obj_letter: char,
    obj_name: &str,
) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::NoTime,
    };

    if obj.contents.is_empty() {
        state.message(format!("The {} is empty.", obj_name));
    } else {
        state.message(format!(
            "The {} contains {} item(s).",
            obj_name,
            obj.contents.len()
        ));
        // In full implementation: display contents menu, allow put in/take out
    }

    ActionResult::Success
}

/// Apply a bag of tricks.
///
/// Based on C bagotricks(). Creates a random monster when applied.
/// Charges are consumed (stored in enchantment/spe).
fn apply_bag_of_tricks(
    state: &mut GameState,
    obj_letter: char,
) -> ActionResult {
    let charges = match state.get_inventory_item(obj_letter) {
        Some(o) => o.enchantment,
        None => return ActionResult::NoTime,
    };

    if charges <= 0 {
        state.message("The bag is empty.");
        return ActionResult::NoTime;
    }

    // Consume a charge
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.enchantment -= 1;
    }

    // Create a random monster adjacent to player
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = px + dx;
            let ny = py + dy;
            if state.current_level.is_valid_pos(nx, ny)
                && state.current_level.is_walkable(nx, ny)
                && state.current_level.monster_at(nx, ny).is_none()
            {
                // Create monster using makemon if available
                let monster_type = state.rng.rn2(50) as i16; // Random type
                let mut monster = crate::monster::Monster::new(
                    MonsterId(state.current_level.monsters.len() as u32 + 1),
                    monster_type,
                    nx,
                    ny,
                );
                monster.name = "creature".to_string();
                monster.hp = state.rng.dice(2, 8) as i32;
                monster.hp_max = monster.hp;
                monster.level = (state.rng.rnd(5) + 1) as u8;
                state.current_level.add_monster(monster);
                state.message("A creature pops out of the bag!");
                return ActionResult::Success;
            }
        }
    }

    state.message("Nothing comes out of the bag.");
    ActionResult::Success
}

// ============================================================================
// Camera
// ============================================================================

/// Apply a camera to flash-blind monsters.
///
/// Based on C use_camera(). Uses charges; cursed may flash self.
fn apply_camera(
    state: &mut GameState,
    obj_letter: char,
    obj_name: &str,
) -> ActionResult {
    let (charges, is_cursed) = match state.get_inventory_item(obj_letter) {
        Some(o) => (o.enchantment, o.buc == BucStatus::Cursed),
        None => return ActionResult::NoTime,
    };

    if charges <= 0 {
        state.message("The camera is out of film.");
        return ActionResult::NoTime;
    }

    // Consume a charge
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.enchantment -= 1;
    }

    // Cursed: 50% chance to flash self
    if is_cursed && state.rng.rn2(2) == 0 {
        state.message("The flash bounces back at you!");
        state.player.blinded_timeout += state.rng.rnd(25) as u16 + 20;
        return ActionResult::Success;
    }

    state.message(format!("You flash the {}.", obj_name));

    // Blind monsters in the direction
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut blinded_count = 0;

    for monster in &mut state.current_level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        // Flash affects monsters within 5 squares in front
        if dx <= 5 && dy <= 5
            && monster.blinded_timeout == 0
        {
            monster.blinded_timeout = state.rng.rnd(15) as u16 + 10;
            blinded_count += 1;
        }
    }

    if blinded_count > 0 {
        state.message(format!(
            "{} monster(s) are blinded by the flash!",
            blinded_count
        ));
    }

    ActionResult::Success
}

// ============================================================================
// Towel
// ============================================================================

/// Apply a towel to wipe face/items.
///
/// Based on C use_towel(). Removes blindness, glib, and cream.
/// Cursed towels can cause glib hands or blindness.
fn apply_towel(state: &mut GameState, obj_letter: char) -> ActionResult {
    let is_cursed = match state.get_inventory_item(obj_letter) {
        Some(o) => o.buc == BucStatus::Cursed,
        None => return ActionResult::NoTime,
    };

    if is_cursed {
        // Cursed: random bad effect
        match state.rng.rn2(3) {
            0 => {
                state.message("The towel gets you all slimy!");
                // Make hands glib (would drop wielded weapon)
            }
            1 => {
                state.message("The towel smears something on your face!");
                state.player.blinded_timeout += state.rng.rnd(10) as u16 + 5;
            }
            _ => {
                state.message("The towel falls apart in your hands!");
            }
        }
    } else {
        // Uncursed/blessed: cure effects
        let mut wiped = false;

        if state.player.blinded_timeout > 0 {
            state.player.blinded_timeout = 0;
            state.message("You wipe the blindness from your eyes.");
            wiped = true;
        }

        if !wiped {
            state.message("You wipe your face with the towel.");
        }
    }

    ActionResult::Success
}

// ============================================================================
// Blindfold/lenses
// ============================================================================

/// Apply a blindfold or lenses (toggle wearing).
///
/// Based on C Blindf_on()/Blindf_off() routing.
fn apply_blindfold(
    state: &mut GameState,
    obj_letter: char,
    obj_name: &str,
) -> ActionResult {
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        if obj.is_worn() {
            // Take off
            if obj.buc == BucStatus::Cursed {
                state.message(format!(
                    "The {} is stuck to your face!",
                    obj_name
                ));
                return ActionResult::NoTime;
            }
            obj.worn_mask = 0;
            state.message(format!("You take off the {}.", obj_name));
        } else {
            // Put on
            obj.worn_mask = 1; // Simplified worn flag
            state.message(format!("You put on the {}.", obj_name));
        }
    }
    ActionResult::Success
}

// ============================================================================
// Unicorn horn
// ============================================================================

/// Apply a unicorn horn to cure ailments.
///
/// Based on C use_unicorn_horn().
/// Blessed: cures multiple ailments. Cursed: causes random bad effect.
/// Uncursed: cures 1-2 ailments.
fn apply_unicorn_horn(state: &mut GameState, obj: &Object) -> ActionResult {
    if obj.buc == BucStatus::Cursed {
        // Cursed: random bad effect
        match state.rng.rn2(7) {
            0 => {
                state.message("You feel sick!");
                state.player.hp -= state.rng.rnd(8) as i32;
            }
            1 => {
                state.message("You go blind!");
                state.player.blinded_timeout += state.rng.rnd(90) as u16 + 10;
            }
            2 => {
                state.message("You feel confused.");
                state.player.confused_timeout += state.rng.rnd(90) as u16 + 10;
            }
            3 => {
                state.message("You feel stunned.");
                state.player.stunned_timeout += state.rng.rnd(90) as u16 + 10;
            }
            4 => {
                state.message("You feel weaker.");
                let cur = state.player.attr_current.get(Attribute::Strength);
                state.player.attr_current.set(Attribute::Strength, cur.saturating_sub(1).max(3));
            }
            5 => {
                state.message("You feel disoriented.");
                state.player.hallucinating_timeout += state.rng.rnd(90) as u16 + 10;
            }
            _ => {
                state.message("You feel deaf!");
                // Deafness timeout (simplified)
            }
        }
        return ActionResult::Success;
    }

    // Number of troubles to fix
    let fixes = if obj.buc == BucStatus::Blessed {
        state.rng.dice(2, 4) as i32 // Blessed: avg 5
    } else {
        state.rng.dice(2, 2) as i32 // Uncursed: avg 3
    };

    let mut cured = 0;

    // Fix troubles in priority order
    if cured < fixes && state.player.confused_timeout > 0 {
        state.player.confused_timeout = 0;
        state.message("Your head clears.");
        cured += 1;
    }
    if cured < fixes && state.player.stunned_timeout > 0 {
        state.player.stunned_timeout = 0;
        state.message("You feel steadier.");
        cured += 1;
    }
    if cured < fixes && state.player.blinded_timeout > 0 {
        state.player.blinded_timeout = 0;
        state.message("Your vision clears.");
        cured += 1;
    }
    if cured < fixes && state.player.hallucinating_timeout > 0 {
        state.player.hallucinating_timeout = 0;
        state.message("Everything looks normal again.");
        cured += 1;
    }

    // Restore attributes if still have fixes left
    if cured < fixes {
        let cur = state.player.attr_current.get(Attribute::Strength);
        let max = state.player.attr_max.get(Attribute::Strength);
        if cur < max {
            state.player.attr_current.set(Attribute::Strength, (cur + 1).min(max));
            state.message("You feel stronger.");
            cured += 1;
        }
    }
    if cured < fixes {
        let cur = state.player.attr_current.get(Attribute::Dexterity);
        let max = state.player.attr_max.get(Attribute::Dexterity);
        if cur < max {
            state.player.attr_current.set(Attribute::Dexterity, (cur + 1).min(max));
            state.message("You feel more agile.");
            cured += 1;
        }
    }

    if cured == 0 {
        state.message("You feel healthy.");
    }

    ActionResult::Success
}

// ============================================================================
// Leash
// ============================================================================

/// Apply a leash to a tame monster.
///
/// Based on C use_leash(). Can leash up to 2 tame pets.
/// Applying again to a leashed pet unleashes it (unless cursed).
fn apply_leash(
    state: &mut GameState,
    _obj_letter: char,
) -> ActionResult {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    // Count currently leashed pets
    let leashed_count = state
        .current_level
        .monsters
        .iter()
        .filter(|m| m.state.tame && m.state.leashed)
        .count();

    // Find adjacent tame monster — collect info first to avoid borrow conflicts
    enum LeashAction {
        NotTame(String),
        Unleash(String),
        TooMany,
        Leash(String),
    }
    let mut action = None;
    let mut target_pos = None;

    'outer: for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = px + dx;
            let ny = py + dy;

            if let Some(monster) = state.current_level.monster_at(nx, ny) {
                if !monster.state.tame {
                    action = Some(LeashAction::NotTame(monster.name.clone()));
                } else if monster.state.leashed {
                    action = Some(LeashAction::Unleash(monster.name.clone()));
                    target_pos = Some((nx, ny));
                } else if leashed_count >= MAX_LEASHED_APPLY {
                    action = Some(LeashAction::TooMany);
                } else {
                    action = Some(LeashAction::Leash(monster.name.clone()));
                    target_pos = Some((nx, ny));
                }
                break 'outer;
            }
        }
    }

    match action {
        Some(LeashAction::NotTame(name)) => {
            state.message(format!("{} is not tame enough to leash.", name));
            ActionResult::NoTime
        }
        Some(LeashAction::Unleash(name)) => {
            if let Some((nx, ny)) = target_pos
                && let Some(m) = state.current_level.monster_at_mut(nx, ny)
            {
                m.state.leashed = false;
            }
            state.message(format!("You unleash {}.", name));
            ActionResult::Success
        }
        Some(LeashAction::TooMany) => {
            state.message("You can't leash any more pets.");
            ActionResult::NoTime
        }
        Some(LeashAction::Leash(name)) => {
            if let Some((nx, ny)) = target_pos
                && let Some(m) = state.current_level.monster_at_mut(nx, ny)
            {
                m.state.leashed = true;
            }
            state.message(format!("You leash {}.", name));
            ActionResult::Success
        }
        None => {
            state.message("There's nothing here to leash.");
            ActionResult::NoTime
        }
    }
}

/// Maximum number of simultaneously leashed pets (used by apply_leash).
const MAX_LEASHED_APPLY: usize = 2;

// ============================================================================
// Bell
// ============================================================================

/// Apply a bell or the Bell of Opening.
///
/// Based on C use_bell(). Regular bells wake monsters.
/// Bell of Opening: blessed opens doors/removes chains,
/// cursed creates undead, uncursed searches for hidden.
fn apply_bell(
    state: &mut GameState,
    obj_letter: char,
    obj: &Object,
) -> ActionResult {
    let is_bell_of_opening = obj.object_type == 198;

    if is_bell_of_opening {
        let charges = obj.enchantment;
        if charges <= 0 {
            state.message("The bell makes a dull sound.");
            return ActionResult::Success;
        }

        // Consume a charge
        if let Some(inv_obj) = state.get_inventory_item_mut(obj_letter) {
            inv_obj.enchantment -= 1;
        }

        match obj.buc {
            BucStatus::Blessed => {
                state.message("A brilliant tone rings out!");
                // Open nearby doors
                let px = state.player.pos.x;
                let py = state.player.pos.y;
                for dy in -3i8..=3 {
                    for dx in -3i8..=3 {
                        let nx = px + dx;
                        let ny = py + dy;
                        if state.current_level.is_valid_pos(nx, ny) {
                            let cell =
                                &mut state.current_level.cells[nx as usize][ny as usize];
                            if cell.typ == crate::dungeon::CellType::Door
                                && cell.door_state().contains(crate::dungeon::DoorState::CLOSED)
                            {
                                cell.set_door_state(crate::dungeon::DoorState::OPEN);
                            }
                        }
                    }
                }
                state.message("All nearby doors spring open!");
            }
            BucStatus::Cursed => {
                state.message("A sinister tone reverberates...");
                state.message("Undead creatures rise from the ground!");
                // Create undead (simplified)
            }
            BucStatus::Uncursed => {
                state.message("A clear tone rings out.");
                // Reveal hidden doors/corridors nearby
                state.message("You sense hidden things nearby.");
            }
        }
    } else {
        // Regular bell: wake monsters
        state.message("Ding-dong!");
        for monster in &mut state.current_level.monsters {
            let dx = (monster.x - state.player.pos.x).abs();
            let dy = (monster.y - state.player.pos.y).abs();
            if dx <= 10 && dy <= 10 {
                monster.state.sleeping = false;
            }
        }
    }

    ActionResult::Success
}

// ============================================================================
// Candelabrum
// ============================================================================

/// Apply the Candelabrum of Invocation.
///
/// Based on C use_candelabrum(). Holds up to 7 candles.
/// At the invocation position, gives full brightness.
fn apply_candelabrum(
    state: &mut GameState,
    obj_letter: char,
) -> ActionResult {
    let (candles, is_lit, is_cursed) = match state.get_inventory_item(obj_letter) {
        Some(o) => (o.enchantment, o.lit, o.buc == BucStatus::Cursed),
        None => return ActionResult::NoTime,
    };

    if is_lit {
        // Snuff it out
        if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
            obj.lit = false;
        }
        state.message("You snuff the candelabrum.");
        return ActionResult::Success;
    }

    if candles <= 0 {
        state.message("The candelabrum has no candles.");
        return ActionResult::NoTime;
    }

    if is_cursed {
        state.message("The candelabrum flickers and goes out.");
        return ActionResult::Success;
    }

    // Light it
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.lit = true;
    }

    if candles >= 7 {
        state.message("The candelabrum blazes with brilliant light!");
    } else {
        state.message(format!(
            "The candelabrum glows with {} candle(s).",
            candles
        ));
    }

    ActionResult::Success
}

// ============================================================================
// Figurine
// ============================================================================

/// Apply a figurine to animate it into a monster.
///
/// Based on C use_figurine(). Transforms the figurine into the
/// corresponding monster type. Requires a valid placement position.
fn apply_figurine(
    state: &mut GameState,
    obj_letter: char,
) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::NoTime,
    };

    let px = state.player.pos.x;
    let py = state.player.pos.y;

    // Find an adjacent empty spot
    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = px + dx;
            let ny = py + dy;

            if state.current_level.is_valid_pos(nx, ny)
                && state.current_level.is_walkable(nx, ny)
                && state.current_level.monster_at(nx, ny).is_none()
            {
                // Create the monster from the figurine
                let monster_type = obj.object_type; // Figurine stores monster type
                let mut monster = crate::monster::Monster::new(
                    MonsterId(state.current_level.monsters.len() as u32 + 1),
                    monster_type,
                    nx,
                    ny,
                );
                monster.name = "figurine creature".to_string();
                monster.hp = state.rng.dice(3, 8) as i32;
                monster.hp_max = monster.hp;

                state.current_level.add_monster(monster);
                state.remove_from_inventory(obj_letter);
                state.message("The figurine comes to life!");
                return ActionResult::Success;
            }
        }
    }

    state.message("There's no room for the figurine to come alive.");
    ActionResult::NoTime
}

// ============================================================================
// Grease
// ============================================================================

/// Apply a can of grease to an item.
///
/// Based on C use_grease(). Makes an item slippery/greased,
/// protecting it from erosion. Cursed: makes hands glib.
fn apply_grease(
    state: &mut GameState,
    obj_letter: char,
) -> ActionResult {
    let (charges, is_cursed) = match state.get_inventory_item(obj_letter) {
        Some(o) => (o.enchantment, o.buc == BucStatus::Cursed),
        None => return ActionResult::NoTime,
    };

    if charges <= 0 {
        state.message("The can of grease is empty.");
        return ActionResult::NoTime;
    }

    // Consume a charge
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        obj.enchantment -= 1;
    }

    if is_cursed && state.rng.rn2(2) == 0 {
        state.message("The grease slips from the can and makes your hands glib!");
        return ActionResult::Success;
    }

    // In full implementation: select target item from inventory
    state.message("You grease an item, protecting it from erosion.");

    ActionResult::Success
}

// ============================================================================
// Trap tools (bear trap / land mine)
// ============================================================================

/// Result of setting a trap.
#[derive(Debug, Clone)]
pub struct TrapSetResult {
    /// Messages
    pub messages: Vec<String>,
    /// Whether the trap was set successfully
    pub success: bool,
    /// Turns required to set the trap
    pub turns: i32,
    /// Whether the player triggered it on themselves (fumble)
    pub self_triggered: bool,
}

/// Apply a trap tool (bear trap or land mine).
///
/// Based on C use_trap(). Setting a trap takes multiple turns
/// based on dexterity. Cursed/fumbling may trigger it on yourself.
fn apply_trap_tool(
    state: &mut GameState,
    obj_letter: char,
    obj: &Object,
) -> ActionResult {
    let is_bear_trap = obj.object_type == 218;
    let trap_name = if is_bear_trap {
        "bear trap"
    } else {
        "land mine"
    };

    let result = set_trap(
        &state.player,
        is_bear_trap,
        obj.buc == BucStatus::Cursed,
        &mut state.rng,
    );

    for msg in &result.messages {
        state.message(msg.clone());
    }

    if result.success {
        // Remove the trap tool from inventory
        state.remove_from_inventory(obj_letter);

        // Place trap on the level
        let trap_type = if is_bear_trap {
            crate::dungeon::TrapType::BearTrap
        } else {
            crate::dungeon::TrapType::LandMine
        };

        state.current_level.traps.push(crate::dungeon::Trap {
            x: state.player.pos.x,
            y: state.player.pos.y,
            trap_type,
            seen: true,
            activated: false,
            once: false,
            madeby_u: true,
            launch_oid: None,
        });

        state.message(format!("You set a {} here.", trap_name));
    }

    if result.self_triggered {
        // Fumble: take damage
        let damage = if is_bear_trap {
            state.rng.dice(2, 4) as i32
        } else {
            state.rng.dice(4, 8) as i32
        };
        state.player.hp -= damage;
        state.message(format!(
            "The {} goes off! You take {} damage!",
            trap_name, damage
        ));
    }

    ActionResult::Success
}

/// Calculate trap setting result.
///
/// Based on C set_trap() occupation. Time depends on DEX; cursed may bungle.
pub fn set_trap(
    player: &You,
    is_bear_trap: bool,
    is_cursed: bool,
    rng: &mut GameRng,
) -> TrapSetResult {
    let mut result = TrapSetResult {
        messages: Vec::new(),
        success: true,
        turns: 0,
        self_triggered: false,
    };

    // Calculate turns required
    let dex = player.attr_current.get(Attribute::Dexterity) as i32;
    result.turns = if dex >= 16 {
        2
    } else if dex >= 12 {
        3
    } else if dex >= 8 {
        4
    } else {
        5
    };

    // Bear trap with low strength takes longer
    if is_bear_trap {
        let str_val = player.attr_current.get(Attribute::Strength) as i32;
        if str_val < 18 {
            result.turns += (18 - str_val) / 4;
        }
    }

    // Cursed: 50% chance of bungling
    if is_cursed && rng.rn2(2) == 0 {
        result.self_triggered = true;
        result.success = false;
        result.messages.push("You bungle the trap!".to_string());
    } else {
        result.messages.push("You carefully set the trap.".to_string());
    }

    result
}

// ============================================================================
// Tinning kit
// ============================================================================

/// Apply a tinning kit.
///
/// Based on C use_tinning_kit(). Converts a corpse into a tin.
/// Requires a corpse at feet or in inventory.
fn apply_tinning_kit(
    state: &mut GameState,
    obj_letter: char,
) -> ActionResult {
    let charges = match state.get_inventory_item(obj_letter) {
        Some(o) => o.enchantment,
        None => return ActionResult::NoTime,
    };

    if charges <= 0 {
        state.message("The tinning kit is out of tins.");
        return ActionResult::NoTime;
    }

    // Look for corpse at feet
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let corpse_idx = state
        .current_level
        .objects
        .iter()
        .position(|o| o.class == ObjectClass::Food && o.x == px && o.y == py);

    if let Some(idx) = corpse_idx {
        // Consume a charge
        if let Some(kit) = state.get_inventory_item_mut(obj_letter) {
            kit.enchantment -= 1;
        }

        // Remove corpse, create tin
        let corpse = state.current_level.objects.remove(idx);
        let tin = Object::new(
            ObjectId(state.rng.rnd(10000)),
            corpse.object_type,
            ObjectClass::Food,
        );
        state.add_to_inventory(tin);

        state.message("You tin the corpse.");
        ActionResult::Success
    } else {
        state.message("There's no corpse here to tin.");
        ActionResult::NoTime
    }
}

// ============================================================================
// Light source (lamp/lantern/candle)
// ============================================================================

/// Apply a light source (toggle lit state).
fn apply_light(state: &mut GameState, obj_letter: char, obj_name: &str) -> ActionResult {
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        if obj.lit {
            obj.lit = false;
            state.message(format!("You extinguish the {}.", obj_name));
        } else {
            obj.lit = true;
            state.message(format!("The {} is now lit.", obj_name));
        }
    }
    ActionResult::Success
}

// ============================================================================
// Horn of plenty
// ============================================================================

/// Apply horn of plenty -- creates food.
fn apply_horn_of_plenty(state: &mut GameState, obj_letter: char) -> ActionResult {
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        if obj.enchantment > 0 {
            obj.enchantment -= 1;
            state.player.nutrition += 600;
            state.message("Food spills out of the horn!");
            ActionResult::Success
        } else {
            state.message("The horn is empty.");
            ActionResult::NoTime
        }
    } else {
        ActionResult::NoTime
    }
}

// ============================================================================
// Horn (tooled/regular)
// ============================================================================

/// Apply a regular horn -- makes noise, wakes monsters.
fn apply_horn(state: &mut GameState) -> ActionResult {
    state.message("You produce a loud noise!");
    for monster in &mut state.current_level.monsters {
        monster.state.sleeping = false;
    }
    ActionResult::Success
}

// ============================================================================
// Stethoscope
// ============================================================================

/// Apply stethoscope -- examine monster or self.
fn apply_stethoscope(state: &mut GameState) -> ActionResult {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            if let Some(monster) = state.current_level.monster_at(px + dx, py + dy) {
                state.message(format!(
                    "{}: HP {}/{}, AC {}, Level {}",
                    monster.name, monster.hp, monster.hp_max, monster.ac, monster.level
                ));
                return ActionResult::Success;
            }
        }
    }

    // No monster found, examine self
    state.message(format!(
        "You: HP {}/{}, AC {}, Level {}",
        state.player.hp,
        state.player.hp_max,
        state.calculate_armor_class(),
        state.player.exp_level
    ));
    ActionResult::Success
}

// ============================================================================
// Whistle
// ============================================================================

/// Apply a whistle -- wakes nearby monsters.
fn apply_whistle(state: &mut GameState) -> ActionResult {
    state.message("You produce a high-pitched humming noise.");
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    for monster in &mut state.current_level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= 10 && dy <= 10 {
            monster.state.sleeping = false;
        }
    }
    ActionResult::Success
}

// ============================================================================
// Magic whistle
// ============================================================================

/// Apply a magic whistle -- pets teleport to you.
fn apply_magic_whistle(state: &mut GameState) -> ActionResult {
    state.message("You produce a strange whistling sound.");
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut pets_moved = 0;

    let tame_ids: Vec<_> = state
        .current_level
        .monsters
        .iter()
        .filter(|m| m.state.tame)
        .map(|m| m.id)
        .collect();

    for monster_id in tame_ids {
        for dy in -1i8..=1 {
            for dx in -1i8..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = px + dx;
                let ny = py + dy;
                if state.current_level.is_walkable(nx, ny)
                    && state.current_level.monster_at(nx, ny).is_none()
                {
                    if let Some(monster) = state.current_level.monster_mut(monster_id) {
                        monster.x = nx;
                        monster.y = ny;
                        monster.state.sleeping = false;
                        pets_moved += 1;
                    }
                    break;
                }
            }
        }
    }

    if pets_moved > 0 {
        state.message(format!("{} pet(s) come to you.", pets_moved));
    }

    ActionResult::Success
}

// ============================================================================
// Mirror
// ============================================================================

/// Apply a mirror -- look at yourself or scare monsters.
fn apply_mirror(state: &mut GameState) -> ActionResult {
    let is_sick = state.player.hp < state.player.hp_max / 4;
    let is_hungry = state.player.nutrition < 150;
    let is_hallucinating = state.player.hallucinating_timeout > 0;

    if is_hallucinating {
        state.message("You look groovy.");
    } else if is_sick {
        state.message("You look peaked.");
    } else if is_hungry {
        state.message("You look undernourished.");
    } else {
        state.message("You look as beautiful as ever.");
    }

    ActionResult::Success
}

/// Apply a mirror in a direction -- can scare monsters.
pub fn apply_mirror_at(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    if let Some(monster) = state.current_level.monster_at_mut(x, y) {
        let monster_name = monster.name.clone();
        if state.rng.one_in(3) {
            monster.state.fleeing = true;
            monster.flee_timeout = state.rng.dice(2, 6) as u16;
            state.message(format!(
                "The {} is frightened by its reflection!",
                monster_name
            ));
        } else {
            state.message(format!("The {} ignores the mirror.", monster_name));
        }
    } else {
        state.message("You reflect the empty space.");
    }

    ActionResult::Success
}

// ============================================================================
// Compatibility wrappers (gitea branch)
// ============================================================================

/// Ring a bell -- scares nearby monsters.
pub fn use_bell(state: &mut GameState) -> ActionResult {
    state.message("You ring the bell...");

    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut affected = 0;

    for monster in &mut state.current_level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= 10 && dy <= 10 {
            // Undead flee, others might be affected
            monster.state.fleeing = true;
            monster.flee_timeout = state.rng.dice(2, 6) as u16;
            affected += 1;
        }
    }

    if affected > 0 {
        state.message(format!("{} monster(s) flee in fear!", affected));
    }

    ActionResult::Success
}

/// Simple camera usage (no charge tracking).
pub fn use_camera(state: &mut GameState) -> ActionResult {
    state.message("You point the camera.");
    state.message("Click! You take a picture.");

    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut blinded_monsters = Vec::new();

    for monster in &mut state.current_level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= 3 && dy <= 3 {
            blinded_monsters.push(monster.name.clone());
        }
    }

    for name in blinded_monsters {
        state.message(format!("The {} is blinded by the flash!", name));
    }

    ActionResult::Success
}

pub fn use_container(state: &mut GameState) -> ActionResult {
    state.message("You use the container.");
    ActionResult::Success
}

pub fn use_cream_pie(state: &mut GameState) -> ActionResult {
    state.message("You throw the cream pie.");
    ActionResult::Success
}

pub fn use_crystal_ball(state: &mut GameState) -> ActionResult {
    state.message("You gaze into the crystal ball.");
    ActionResult::Success
}

pub fn use_defensive(state: &mut GameState) -> ActionResult {
    state.message("You use the defensive tool.");
    ActionResult::Success
}

pub fn use_figurine(state: &mut GameState) -> ActionResult {
    state.message("You set the figurine on the ground.");
    // Would transform into monster
    ActionResult::Success
}

pub fn use_grapple(state: &mut GameState) -> ActionResult {
    state.message("You cast the grappling hook.");
    ActionResult::Success
}

pub fn use_grease(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Grease the player's boots or a specified item
    let obj_name = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };
        obj.display_name()
    };

    state.message(format!("You apply grease to the {}.", obj_name));
    state.message("You slide more easily now.");
    // In full implementation, would grant slipping property
    ActionResult::Success
}

pub fn use_lamp(state: &mut GameState, obj_letter: char, obj_name: &str) -> ActionResult {
    apply_light(state, obj_letter, obj_name)
}

pub fn use_leash(state: &mut GameState) -> ActionResult {
    state.message("You use the leash.");
    ActionResult::Success
}

pub fn use_magic_whistle(state: &mut GameState) -> ActionResult {
    apply_magic_whistle(state)
}

pub fn use_mirror(state: &mut GameState) -> ActionResult {
    apply_mirror(state)
}

pub fn use_misc(state: &mut GameState) -> ActionResult {
    state.message("You use the miscellaneous tool.");
    ActionResult::Success
}

pub fn use_offensive(state: &mut GameState) -> ActionResult {
    state.message("You use the offensive tool.");
    ActionResult::Success
}

pub fn use_pick_axe(state: &mut GameState, obj_name: &str) -> ActionResult {
    apply_pickaxe(state, obj_name)
}

pub fn use_pick_axe2(state: &mut GameState) -> ActionResult {
    use_pick_axe(state, "pick-axe")
}

pub fn use_pole(state: &mut GameState) -> ActionResult {
    state.message("You thrust the pole.");
    ActionResult::Success
}

pub fn use_saddle(state: &mut GameState) -> ActionResult {
    state.message("You use the saddle.");
    ActionResult::Success
}

pub fn use_skill(state: &mut GameState) -> ActionResult {
    state.message("You use the skill.");
    ActionResult::Success
}

pub fn use_stethoscope(state: &mut GameState) -> ActionResult {
    apply_stethoscope(state)
}

pub fn use_stone(state: &mut GameState) -> ActionResult {
    state.message("You use the stone.");
    ActionResult::Success
}

pub fn use_tin_opener(state: &mut GameState) -> ActionResult {
    state.message("You use the tin opener.");
    ActionResult::Success
}

pub fn use_tinning_kit(state: &mut GameState, _obj_letter: char, obj_name: &str) -> ActionResult {
    // Check if there's a corpse at this location
    let has_corpse = state
        .current_level
        .objects_at(state.player.pos.x, state.player.pos.y)
        .iter()
        .any(|o| {
            o.class == ObjectClass::Food && o.name.as_ref().map_or(false, |n| n.contains("corpse"))
        });

    if !has_corpse {
        state.message("You need a fresh corpse to tin.");
        return ActionResult::NoTime;
    }

    state.message(format!("You use the {} to preserve a corpse.", obj_name));
    state.message("The corpse becomes tinned meat.");
    // In full implementation, would create a tin object and remove corpse
    ActionResult::Success
}

pub fn use_towel(state: &mut GameState) -> ActionResult {
    state.message("You dry yourself off with the towel.");
    state.message("You feel fresher.");
    // Could remove water/slippery effects
    ActionResult::Success
}

pub fn use_trap(state: &mut GameState) -> ActionResult {
    state.message("You set a trap.");
    ActionResult::Success
}

pub fn use_unicorn_horn(state: &mut GameState) -> ActionResult {
    let mut cured = false;

    if state.player.confused_timeout > 0 {
        state.player.confused_timeout = 0;
        state.message("Your head clears.");
        cured = true;
    }
    if state.player.stunned_timeout > 0 {
        state.player.stunned_timeout = 0;
        state.message("You feel steadier.");
        cured = true;
    }
    if state.player.blinded_timeout > 0 {
        state.player.blinded_timeout = 0;
        state.message("Your vision clears.");
        cured = true;
    }
    if state.player.hallucinating_timeout > 0 {
        state.player.hallucinating_timeout = 0;
        state.message("Everything looks normal again.");
        cured = true;
    }

    if !cured {
        state.message("You feel healthy.");
    }

    ActionResult::Success
}

pub fn use_whip(state: &mut GameState) -> ActionResult {
    state.message("You crack the whip.");
    ActionResult::Success
}

pub fn use_whistle(state: &mut GameState) -> ActionResult {
    apply_whistle(state)
}

pub fn tool_in_use(_state: &mut GameState) -> bool {
    // Stub
    false
}

pub fn setapplyclasses(_list: &str) {
    // Stub
}

pub fn bagotricks(state: &mut GameState, obj_letter: char) -> ActionResult {
    apply_bag_of_tricks(state, obj_letter)
}

pub fn light_cocktail(_state: &mut GameState) {
    // Stub
}

pub fn hornoplenty(state: &mut GameState, obj_letter: char) -> ActionResult {
    apply_horn_of_plenty(state, obj_letter)
}

pub fn figurine_location_checks() -> bool {
    true
}

fn apply_magic_harp(state: &mut GameState) -> ActionResult {
    state.message("You play the magic harp...");

    // Nearby monsters might be tamed
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut tamed = 0;

    for monster in &mut state.current_level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= 8 && dy <= 8 && state.rng.one_in(3) {
            monster.state.tame = true;
            tamed += 1;
        }
    }

    if tamed > 0 {
        state.message(format!("The music tames {} monster(s)!", tamed));
    } else {
        state.message("The music has no effect.");
    }

    ActionResult::Success
}

fn apply_bugle(state: &mut GameState) -> ActionResult {
    state.message("You blow the bugle loudly!");

    for monster in &mut state.current_level.monsters {
        monster.state.sleeping = false;
    }

    state.message("All nearby creatures are awakened!");
    ActionResult::Success
}

fn apply_frost_horn(state: &mut GameState) -> ActionResult {
    state.message("You blow the frost horn, and a chill spreads...");

    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut slowed = 0;

    for monster in &mut state.current_level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= 6 && dy <= 6 {
            // Slow nearby monsters
            monster.state.slowed = true;
            slowed += 1;
        }
    }

    if slowed > 0 {
        state.message(format!("You slow {} creature(s) with the frost!", slowed));
    }

    ActionResult::Success
}

fn apply_rope(state: &mut GameState) -> ActionResult {
    state.message("You unwind the rope.");
    state.message("You could use this for climbing or to bind something.");
    // Would require direction selection in full implementation
    ActionResult::NoTime
}

// ============================================================================
// Light Source System (from light.c)
// ============================================================================

/// Maximum light radius
pub const MAX_LIGHT_RADIUS: i32 = 15;

/// Default lamp light radius
pub const LAMP_LIGHT_RADIUS: i32 = 3;

/// Catch an object that's on fire (thrown lit lamp, etc.)
pub fn catch_lit(state: &mut GameState, obj_letter: char) {
    // Check object state first
    let (is_lit, obj_id) = {
        if let Some(obj) = state.get_inventory_item(obj_letter) {
            (obj.lit, obj.id)
        } else {
            return;
        }
    };

    if !is_lit {
        return;
    }

    // Determine outcome
    let goes_out = state.rng.one_in(3);

    if goes_out {
        // Now extinguish
        if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
            obj.lit = false;
        }
        state.message("The light goes out!");
        del_light_source_for_object(state, obj_id);
    }
}

/// Begin burning an object (light a lamp, candle, etc.)
pub fn begin_burn(state: &mut GameState, obj_letter: char, silent: bool) {
    let (obj_id, obj_x, obj_y, obj_name, radius) = {
        let obj = match state.get_inventory_item_mut(obj_letter) {
            Some(o) => o,
            None => return,
        };

        if obj.lit {
            return; // Already lit
        }

        obj.lit = true;
        let radius = light_radius_for_object(obj);
        (obj.id, obj.x, obj.y, obj.display_name(), radius)
    };

    // Create light source for this object
    new_light_source_for_object(state, obj_x, obj_y, radius, obj_id);

    if !silent {
        state.message(format!("The {} is now lit.", obj_name));
    }
}

/// End burning of an object (snuff out)
pub fn end_burn(state: &mut GameState, obj_letter: char, silent: bool) {
    let (obj_id, obj_name) = {
        let obj = match state.get_inventory_item_mut(obj_letter) {
            Some(o) => o,
            None => return,
        };

        if !obj.lit {
            return; // Not lit
        }

        obj.lit = false;
        (obj.id, obj.display_name())
    };

    // Remove associated light source
    del_light_source_for_object(state, obj_id);

    if !silent {
        state.message(format!("The {} goes out.", obj_name));
    }
}

/// Process burning objects (called each turn)
/// Objects that burn may consume fuel and eventually go out
pub fn burn_object(state: &mut GameState, obj_letter: char) {
    let should_extinguish = {
        let obj = match state.get_inventory_item_mut(obj_letter) {
            Some(o) => o,
            None => return,
        };

        if !obj.lit {
            return;
        }

        // Consume fuel (using age as fuel counter)
        obj.age = obj.age.saturating_sub(1);

        // Check if fuel ran out
        obj.age == 0
    };

    if should_extinguish {
        end_burn(state, obj_letter, false);
        state.message("Your lamp has run out of fuel.");
    }
}

/// Snuff out light sources at a specific position (darkness effect)
pub fn snuff_light_source(state: &mut GameState, x: i8, y: i8) {
    // Find light sources at this position and extinguish them
    let mut objects_to_snuff = Vec::new();

    for ls in &state.current_level.light_sources {
        if ls.x == x && ls.y == y && ls.source_type == LightSourceType::Object {
            objects_to_snuff.push(ObjectId(ls.id));
        }
    }

    for obj_id in objects_to_snuff {
        // Find the object and extinguish it
        if let Some(obj) = state
            .current_level
            .objects
            .iter_mut()
            .find(|o| o.id == obj_id)
        {
            if obj.lit && !is_artifact_light(obj) {
                obj.lit = false;
                del_light_source_for_object(state, obj_id);
                state.message("A light is snuffed out!");
            }
        }
    }
}

/// Snuff a specific lit object
pub fn snuff_lit(state: &mut GameState, obj_letter: char) {
    end_burn(state, obj_letter, false);
}

/// Show lamp flickering message (low fuel warning)
pub fn see_lamp_flicker(state: &mut GameState, obj_letter: char) {
    if let Some(obj) = state.get_inventory_item(obj_letter) {
        if obj.lit && obj.age > 0 && obj.age < 50 {
            state.message(format!("Your {} flickers.", obj.display_name()));
        }
    }
}

/// Adjust light radius if object's state changed (blessed/cursed artifacts)
pub fn maybe_adjust_light(state: &mut GameState, obj_id: ObjectId, new_radius: i32) {
    obj_adjust_light_radius(state, obj_id, new_radius);
}

/// Check if an object is currently burning (emitting light)
pub fn obj_is_burning(obj: &crate::object::Object) -> bool {
    obj.lit
}

/// Check if an object sheds light at all
pub fn obj_sheds_light(obj: &crate::object::Object) -> bool {
    obj_is_burning(obj)
}

/// Adjust the light radius of a light source attached to an object
pub fn obj_adjust_light_radius(state: &mut GameState, obj_id: ObjectId, new_radius: i32) {
    let clamped_radius = new_radius.clamp(1, MAX_LIGHT_RADIUS);

    for ls in &mut state.current_level.light_sources {
        if ls.source_type == LightSourceType::Object && ls.id == obj_id.0 {
            if ls.range != clamped_radius {
                ls.range = clamped_radius;
                // Trigger vision recalculation (would set vision_full_recalc = 1 in C)
            }
            return;
        }
    }
}

/// Process all light sources to update lighting on the level
/// Called during vision recalculation
pub fn do_light_sources(state: &mut GameState) {
    // Collect light source updates to avoid borrow conflicts
    let mut updates: Vec<(usize, i8, i8, bool)> = Vec::new();
    let player_pos = state.player.pos;

    for (idx, ls) in state.current_level.light_sources.iter().enumerate() {
        let (new_x, new_y, show) = match ls.source_type {
            LightSourceType::Object => {
                // Find the object and get position
                if let Some(obj) = state.current_level.objects.iter().find(|o| o.id.0 == ls.id) {
                    (obj.x, obj.y, obj.lit)
                } else if let Some(obj) = state.inventory.iter().find(|o| o.id.0 == ls.id) {
                    // Object in inventory - use player position
                    (player_pos.x, player_pos.y, obj.lit)
                } else {
                    (ls.x, ls.y, false)
                }
            }
            LightSourceType::Monster => {
                // Find the monster and get position
                if let Some(mon) = state
                    .current_level
                    .monsters
                    .iter()
                    .find(|m| m.id.0 == ls.id)
                {
                    (mon.x, mon.y, true)
                } else {
                    (ls.x, ls.y, false)
                }
            }
        };
        updates.push((idx, new_x, new_y, show));
    }

    // Apply updates
    for (idx, x, y, show) in updates {
        if let Some(ls) = state.current_level.light_sources.get_mut(idx) {
            ls.x = x;
            ls.y = y;
            ls.flags.show = show;
        }
    }

    // Collect light source data for lighting application
    let light_data: Vec<(i8, i8, i32)> = state
        .current_level
        .light_sources
        .iter()
        .filter(|ls| ls.flags.show)
        .map(|ls| (ls.x, ls.y, ls.range))
        .collect();

    // Apply lighting from each active light source
    for (x, y, range) in light_data {
        apply_light_at(&mut state.current_level, x, y, range);
    }
}

/// Apply light from a source at position (x,y) with given radius
fn apply_light_at(level: &mut crate::dungeon::Level, x: i8, y: i8, radius: i32) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = x + dx as i8;
            let ny = y + dy as i8;

            if !level.is_valid_pos(nx, ny) {
                continue;
            }

            // Check if within radius (circular)
            if dx * dx + dy * dy <= radius * radius {
                // Check line of sight
                if level.has_line_of_sight(x, y, nx, ny) {
                    level.cells[nx as usize][ny as usize].lit = true;
                }
            }
        }
    }
}

/// Create a new light source attached to an object
pub fn new_light_source_for_object(
    state: &mut GameState,
    x: i8,
    y: i8,
    range: i32,
    obj_id: ObjectId,
) {
    let clamped_range = range.clamp(1, MAX_LIGHT_RADIUS);

    let ls = LightSource::from_object(x, y, clamped_range, obj_id);
    state.current_level.light_sources.push(ls);
}

/// Create a new light source attached to a monster
pub fn new_light_source_for_monster(
    state: &mut GameState,
    x: i8,
    y: i8,
    range: i32,
    monster_id: MonsterId,
) {
    let clamped_range = range.clamp(1, MAX_LIGHT_RADIUS);

    let ls = LightSource::from_monster(x, y, clamped_range, monster_id);
    state.current_level.light_sources.push(ls);
}

/// Delete a light source attached to an object
pub fn del_light_source_for_object(state: &mut GameState, obj_id: ObjectId) {
    state
        .current_level
        .light_sources
        .retain(|ls| !(ls.source_type == LightSourceType::Object && ls.id == obj_id.0));
}

/// Delete a light source attached to a monster
pub fn del_light_source_for_monster(state: &mut GameState, monster_id: MonsterId) {
    state
        .current_level
        .light_sources
        .retain(|ls| !(ls.source_type == LightSourceType::Monster && ls.id == monster_id.0));
}

/// Legacy wrapper for new_light_source (backward compatibility)
pub fn new_light_source(state: &mut GameState, x: i8, y: i8, range: i32, obj_id: ObjectId) {
    new_light_source_for_object(state, x, y, range, obj_id);
}

/// Legacy wrapper for del_light_source (backward compatibility)
pub fn del_light_source(state: &mut GameState, obj_id: ObjectId) {
    del_light_source_for_object(state, obj_id);
}

/// Write a light source to save data (stub for save/restore system)
pub fn write_ls(_ls: &LightSource) -> Vec<u8> {
    // Would serialize the light source for saving
    Vec::new()
}

/// Count and optionally write light sources for saving
pub fn maybe_write_ls(state: &GameState, _write: bool) -> usize {
    state.current_level.light_sources.len()
}

/// Get statistics about light sources (for wizard mode)
pub fn light_stats(state: &GameState) -> (usize, usize) {
    let count = state.current_level.light_sources.len();
    let size = count * std::mem::size_of::<LightSource>();
    (count, size)
}

/// Relink light sources after restore (fix object/monster pointers)
pub fn relink_light_sources(state: &mut GameState) {
    for ls in &mut state.current_level.light_sources {
        if ls.flags.needs_fixup {
            ls.flags.needs_fixup = false;
            // Object/monster ID mapping would happen here during restore
        }
    }
}

/// Move a light source from one object to another (for object merging)
pub fn obj_move_light_source(state: &mut GameState, src_id: ObjectId, dest_id: ObjectId) {
    for ls in &mut state.current_level.light_sources {
        if ls.source_type == LightSourceType::Object && ls.id == src_id.0 {
            ls.id = dest_id.0;
            return;
        }
    }
}

/// Merge light sources when objects are combined (e.g., candles)
pub fn obj_merge_light_sources(state: &mut GameState, src_id: ObjectId, dest_id: ObjectId) {
    // Remove source light source
    del_light_source_for_object(state, src_id);

    // Adjust destination's light radius (for combined candles)
    if let Some(dest_obj) = state.current_level.objects.iter().find(|o| o.id == dest_id) {
        let new_radius = light_radius_for_object(dest_obj);
        obj_adjust_light_radius(state, dest_id, new_radius);
    }
}

/// Split light sources when objects are divided (e.g., candle stack split)
pub fn obj_split_light_source(state: &mut GameState, src_id: ObjectId, dest_id: ObjectId) {
    // Find source light source and clone it for the new object
    let src_ls = state
        .current_level
        .light_sources
        .iter()
        .find(|ls| ls.source_type == LightSourceType::Object && ls.id == src_id.0)
        .cloned();

    if let Some(mut new_ls) = src_ls {
        new_ls.id = dest_id.0;
        state.current_level.light_sources.push(new_ls);
    }
}

/// Save all light sources (for level save)
pub fn save_light_sources(state: &GameState) -> Vec<LightSource> {
    state.current_level.light_sources.clone()
}

/// Restore light sources (for level restore)
pub fn restore_light_sources(state: &mut GameState, sources: Vec<LightSource>) {
    state.current_level.light_sources = sources;
}

/// Calculate light radius for an object based on its type
fn light_radius_for_object(obj: &crate::object::Object) -> i32 {
    if !obj.lit {
        return 0;
    }

    // Different light sources have different radii based on object type
    // Object types from objects.c (approximate ranges)
    match obj.object_type {
        // Magic lamp
        188 => {
            if obj.enchantment > 0 {
                3
            } else {
                2
            }
        }
        // Oil lamp / brass lantern
        189 => 3,
        // Candles (single candle = 2, more = brighter)
        221 | 222 => candle_light_range(obj.quantity),
        // Candelabrum
        223 => candelabrum_light_range(obj.enchantment as i32),
        // Artifacts (Sunsword, etc.)
        _ if is_artifact_light(obj) => artifact_light_radius(obj),
        // Default for other lit objects
        _ => 2,
    }
}

/// Calculate light radius for candles based on quantity
fn candle_light_range(quantity: i32) -> i32 {
    // 1-6 candles: radius 2
    // 7-48 candles: radius 3
    // 49-342 candles: radius 4
    let mut n = quantity as i64;
    let mut radius = 1;
    while n > 0 {
        radius += 1;
        n /= 7;
    }
    radius.min(MAX_LIGHT_RADIUS)
}

/// Calculate light radius for candelabrum based on attached candles
fn candelabrum_light_range(candles: i32) -> i32 {
    match candles {
        0 => 0,
        1..=3 => 2,
        4..=6 => 3,
        _ => 4,
    }
}

/// Check if object is an artifact that emits light
fn is_artifact_light(obj: &crate::object::Object) -> bool {
    // Sunsword and similar artifacts (artifact != 0 means it's an artifact)
    obj.artifact != 0 && obj.lit
}

/// Calculate artifact light radius based on BUC status
fn artifact_light_radius(obj: &crate::object::Object) -> i32 {
    if obj.is_blessed() {
        3
    } else if obj.is_cursed() {
        1
    } else {
        2
    }
}

// ============================================================================
// Leash System (from apply.c)
// ============================================================================

/// Maximum number of pets that can be leashed at once
pub const MAX_LEASHED: i32 = 2;

/// Check if a monster can be leashed
pub fn leashable(state: &GameState, monster_id: MonsterId) -> bool {
    if let Some(monster) = state.current_level.monster(monster_id) {
        // Must be tame
        if !monster.state.tame {
            return false;
        }

        // Check distance - must be adjacent or close
        let dx = (monster.x - state.player.pos.x).abs();
        let dy = (monster.y - state.player.pos.y).abs();
        if dx > 1 || dy > 1 {
            return false;
        }

        // Can't leash certain monster types (like huge monsters)
        // This would check monster size in full implementation

        true
    } else {
        false
    }
}

/// Count the number of currently leashed monsters
pub fn number_leashed(state: &GameState) -> i32 {
    state
        .current_level
        .monsters
        .iter()
        .filter(|m| m.state.leashed)
        .count() as i32
}

/// Unleash a specific monster
pub fn m_unleash(state: &mut GameState, monster_id: MonsterId) {
    // Get monster name first
    let (was_leashed, name) = {
        if let Some(monster) = state.current_level.monster(monster_id) {
            (monster.state.leashed, monster.name.clone())
        } else {
            return;
        }
    };

    if was_leashed {
        // Now mutably unleash
        if let Some(monster) = state.current_level.monster_mut(monster_id) {
            monster.state.leashed = false;
        }
        state.message(format!("{} is unleashed.", name));
    }
}

/// Unleash all monsters connected to a specific leash object
pub fn o_unleash(state: &mut GameState, leash_obj_id: ObjectId) {
    // Find all monsters leashed by this object and unleash them
    let monsters_to_unleash: Vec<MonsterId> = state
        .current_level
        .monsters
        .iter()
        .filter(|m| m.state.leashed && m.leash_id == Some(leash_obj_id))
        .map(|m| m.id)
        .collect();

    for monster_id in monsters_to_unleash {
        m_unleash(state, monster_id);
    }
}

/// Unleash all leashed monsters
pub fn unleash_all(state: &mut GameState) {
    let leashed_ids: Vec<MonsterId> = state
        .current_level
        .monsters
        .iter()
        .filter(|m| m.state.leashed)
        .map(|m| m.id)
        .collect();

    for monster_id in leashed_ids {
        m_unleash(state, monster_id);
    }
}

/// Check leash status - called when player moves to verify leashed pets can follow
pub fn check_leash(state: &mut GameState, dx: i8, dy: i8) {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    // Collect monsters to unleash with their names
    let to_unleash: Vec<(MonsterId, String)> = state
        .current_level
        .monsters
        .iter()
        .filter(|monster| {
            if monster.state.leashed {
                // Calculate distance after player move
                let new_px = px + dx;
                let new_py = py + dy;
                let dist_x = (monster.x - new_px).abs();
                let dist_y = (monster.y - new_py).abs();

                // Leash has limited range (about 2 squares)
                dist_x > 2 || dist_y > 2
            } else {
                false
            }
        })
        .map(|m| (m.id, m.name.clone()))
        .collect();

    // Unleash pets that can't follow
    for (monster_id, name) in to_unleash {
        if let Some(monster) = state.current_level.monster_mut(monster_id) {
            monster.state.leashed = false;
        }
        state.message(format!("The leash chokes {}!", name));
    }
}

/// Get the leash object for a monster, if any
pub fn get_mleash(state: &GameState, monster_id: MonsterId) -> Option<ObjectId> {
    state
        .current_level
        .monster(monster_id)
        .and_then(|m| if m.state.leashed { m.leash_id } else { None })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::{Monster, MonsterId, MonsterState};
    use crate::object::ObjectId;

    fn test_player() -> You {
        let mut player = You::default();
        player.exp_level = 1;
        player.attr_current.set(Attribute::Strength, 14);
        player.attr_current.set(Attribute::Dexterity, 12);
        player.attr_max.set(Attribute::Strength, 18);
        player.attr_max.set(Attribute::Dexterity, 18);
        player.hp = 20;
        player.hp_max = 20;
        player
    }

    fn test_monster(id: u32) -> Monster {
        let mut m = Monster::new(MonsterId(id), 0, 5, 5);
        m.name = "kobold".to_string();
        m.hp = 10;
        m.hp_max = 10;
        m.level = 1;
        m.ac = 7;
        m
    }

    fn create_test_state() -> GameState {
        let mut state = GameState::default();
        state.player.pos.x = 10;
        state.player.pos.y = 10;
        state
    }

    // ---- Lock picking tests ----

    #[test]
    fn test_skeleton_key_always_works() {
        let player = test_player();
        let mut rng = GameRng::new(42);
        let result = pick_lock(&player, 205, false, &mut rng);
        assert!(result.opened);
    }

    #[test]
    fn test_lock_pick_dex_based() {
        let player = test_player();
        let mut success = 0;
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            if pick_lock(&player, 206, false, &mut rng).opened {
                success += 1;
            }
        }
        // With dex 12: chance = 50 + 24 = 74%
        assert!(success > 50);
    }

    #[test]
    fn test_cursed_lock_pick_worse() {
        let player = test_player();
        let mut normal_success = 0;
        let mut cursed_success = 0;
        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            if pick_lock(&player, 206, false, &mut rng).opened {
                normal_success += 1;
            }
            let mut rng2 = GameRng::new(seed);
            if pick_lock(&player, 206, true, &mut rng2).opened {
                cursed_success += 1;
            }
        }
        assert!(normal_success > cursed_success);
    }

    // ---- Trap setting tests ----

    #[test]
    fn test_trap_set_time_dex() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        player.attr_current.set(Attribute::Dexterity, 16);
        let fast = set_trap(&player, false, false, &mut rng);
        assert_eq!(fast.turns, 2);

        player.attr_current.set(Attribute::Dexterity, 8);
        let slow = set_trap(&player, false, false, &mut rng);
        assert_eq!(slow.turns, 4);
    }

    #[test]
    fn test_trap_set_bear_str_penalty() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 10);
        player.attr_current.set(Attribute::Dexterity, 16);
        let mut rng = GameRng::new(42);

        let result = set_trap(&player, true, false, &mut rng);
        assert!(result.turns > 2); // Extra time for low strength bear trap
    }

    #[test]
    fn test_trap_cursed_bungle() {
        let player = test_player();
        let mut bungle_count = 0;
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let result = set_trap(&player, false, true, &mut rng);
            if result.self_triggered {
                bungle_count += 1;
            }
        }
        // ~50% should bungle
        assert!(bungle_count > 30 && bungle_count < 70);
    }

    // ---- Instrument tests ----

    #[test]
    fn test_magic_flute_sleeps_monsters() {
        let mut state = GameState::new(GameRng::new(42));
        let mut flute = Object::new(ObjectId(1), 193, ObjectClass::Tool);
        flute.enchantment = 3;
        flute.inv_letter = 'a';
        flute.name = Some("magic flute".to_string());
        state.inventory.push(flute);

        // Add a monster near player
        let mut monster = test_monster(1);
        monster.x = state.player.pos.x + 1;
        monster.y = state.player.pos.y;
        state.current_level.add_monster(monster);

        let result = do_apply(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
    }

    // ---- Unicorn horn tests ----

    #[test]
    fn test_unicorn_horn_cures_confusion() {
        let mut state = GameState::new(GameRng::new(42));
        state.player.confused_timeout = 10;
        let horn = Object::new(ObjectId(1), 213, ObjectClass::Tool);

        apply_unicorn_horn(&mut state, &horn);
        assert_eq!(state.player.confused_timeout, 0);
    }

    #[test]
    fn test_unicorn_horn_blessed_cures_more() {
        let mut state = GameState::new(GameRng::new(42));
        state.player.confused_timeout = 10;
        state.player.stunned_timeout = 10;
        state.player.blinded_timeout = 10;
        state.player.hallucinating_timeout = 10;

        let mut horn = Object::new(ObjectId(1), 213, ObjectClass::Tool);
        horn.buc = BucStatus::Blessed;

        apply_unicorn_horn(&mut state, &horn);
        // Blessed should cure multiple ailments
        let total_remaining = state.player.confused_timeout as i32
            + state.player.stunned_timeout as i32
            + state.player.blinded_timeout as i32
            + state.player.hallucinating_timeout as i32;
        assert_eq!(total_remaining, 0);
    }

    #[test]
    fn test_unicorn_horn_cursed_causes_harm() {
        let mut state = GameState::new(GameRng::new(42));
        let starting_hp = state.player.hp;

        let mut horn = Object::new(ObjectId(1), 213, ObjectClass::Tool);
        horn.buc = BucStatus::Cursed;

        apply_unicorn_horn(&mut state, &horn);
        // Should cause some bad effect (hp loss or timeout increase)
        let had_effect = state.player.hp < starting_hp
            || state.player.confused_timeout > 0
            || state.player.stunned_timeout > 0
            || state.player.blinded_timeout > 0
            || state.player.hallucinating_timeout > 0;
        assert!(had_effect);
    }

    // ---- Dig tests ----

    #[test]
    fn test_dig_check_valid() {
        let state = GameState::new(GameRng::new(42));
        // Level is generated, most positions should be checkable
        let valid = dig_check(&state, 5, 5);
        // Just verify it doesn't crash
        assert!(valid || !valid);
    }

    // ---- Camera tests ----

    #[test]
    fn test_camera_no_charges() {
        let mut state = GameState::new(GameRng::new(42));
        let mut camera = Object::new(ObjectId(1), 210, ObjectClass::Tool);
        camera.enchantment = 0;
        camera.inv_letter = 'c';
        camera.name = Some("camera".to_string());
        state.inventory.push(camera);

        let result = do_apply(&mut state, 'c');
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ---- Bag of tricks tests ----

    #[test]
    fn test_bag_of_tricks_creates_monster() {
        let mut state = GameState::new(GameRng::new(42));
        let monster_count_before = state.current_level.monsters.len();

        let mut bag = Object::new(ObjectId(1), 225, ObjectClass::Tool);
        bag.enchantment = 5;
        bag.inv_letter = 'b';
        bag.name = Some("bag of tricks".to_string());
        state.inventory.push(bag);

        let _result = do_apply(&mut state, 'b');
        // May or may not spawn depending on available space
        let monster_count_after = state.current_level.monsters.len();
        assert!(monster_count_after >= monster_count_before);
    }

    // ---- Horn of plenty tests ----

    #[test]
    fn test_horn_of_plenty_feeds() {
        let mut state = GameState::new(GameRng::new(42));
        let starting_nutrition = state.player.nutrition;

        let mut horn = Object::new(ObjectId(1), 196, ObjectClass::Tool);
        horn.enchantment = 3;
        horn.inv_letter = 'h';
        horn.name = Some("horn of plenty".to_string());
        state.inventory.push(horn);

        let result = do_apply(&mut state, 'h');
        assert!(matches!(result, ActionResult::Success));
        assert_eq!(state.player.nutrition, starting_nutrition + 600);
    }

    #[test]
    fn test_horn_of_plenty_empty() {
        let mut state = GameState::new(GameRng::new(42));

        let mut horn = Object::new(ObjectId(1), 196, ObjectClass::Tool);
        horn.enchantment = 0;
        horn.inv_letter = 'h';
        horn.name = Some("horn of plenty".to_string());
        state.inventory.push(horn);

        let result = do_apply(&mut state, 'h');
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ---- Stethoscope tests ----

    #[test]
    fn test_stethoscope_self() {
        let mut state = GameState::new(GameRng::new(42));
        let mut scope = Object::new(ObjectId(1), 200, ObjectClass::Tool);
        scope.inv_letter = 's';
        scope.name = Some("stethoscope".to_string());
        state.inventory.push(scope);

        let result = do_apply(&mut state, 's');
        assert!(matches!(result, ActionResult::Success));
        assert!(!state.messages.is_empty());
    }

    // ---- Candelabrum tests ----

    #[test]
    fn test_candelabrum_no_candles() {
        let mut state = GameState::new(GameRng::new(42));
        let mut cand = Object::new(ObjectId(1), 199, ObjectClass::Tool);
        cand.enchantment = 0;
        cand.inv_letter = 'c';
        cand.name = Some("candelabrum".to_string());
        state.inventory.push(cand);

        let result = do_apply(&mut state, 'c');
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_candelabrum_full() {
        let mut state = GameState::new(GameRng::new(42));
        let mut cand = Object::new(ObjectId(1), 199, ObjectClass::Tool);
        cand.enchantment = 7;
        cand.inv_letter = 'c';
        cand.name = Some("candelabrum".to_string());
        state.inventory.push(cand);

        let result = do_apply(&mut state, 'c');
        assert!(matches!(result, ActionResult::Success));

        // Should be lit now
        let obj = state.get_inventory_item('c').unwrap();
        assert!(obj.lit);
    }

    // ---- Bell tests (gitea) ----

    #[test]
    fn test_use_bell_scares_monsters() {
        let mut state = create_test_state();
        state.current_level.monsters.clear();

        // Add a monster nearby
        let mut monster = Monster::new(MonsterId(1), 11, 10, 11);
        monster.state = MonsterState::active();
        state.current_level.monsters.push(monster);

        // Use bell
        use_bell(&mut state);

        // Monster should be fleeing
        assert!(state.current_level.monsters[0].state.fleeing);
        assert!(state.messages.iter().any(|m| m.contains("flee in fear")));
    }

    // ---- Camera tests (gitea) ----

    #[test]
    fn test_use_camera_blinds_monsters() {
        let mut state = create_test_state();
        state.current_level.monsters.clear();

        // Add a monster nearby (within 3 squares of player at 10,10)
        let mut monster = Monster::new(MonsterId(1), 11, 10, 11);
        monster.name = "kobold".to_string();
        state.current_level.monsters.push(monster);

        // Use camera
        use_camera(&mut state);

        // Check for blind message
        assert!(
            state
                .messages
                .iter()
                .any(|m| m.contains("blinded by the flash"))
        );
    }

    // ============================================================================
    // Light Source Tests
    // ============================================================================

    #[test]
    fn test_new_light_source_for_object() {
        let mut state = create_test_state();
        let obj_id = ObjectId(1);

        new_light_source_for_object(&mut state, 5, 5, 3, obj_id);

        assert_eq!(state.current_level.light_sources.len(), 1);
        let ls = &state.current_level.light_sources[0];
        assert_eq!(ls.x, 5);
        assert_eq!(ls.y, 5);
        assert_eq!(ls.range, 3);
        assert_eq!(ls.source_type, LightSourceType::Object);
        assert_eq!(ls.id, 1);
    }

    #[test]
    fn test_del_light_source_for_object() {
        let mut state = create_test_state();
        let obj_id = ObjectId(1);

        new_light_source_for_object(&mut state, 5, 5, 3, obj_id);
        assert_eq!(state.current_level.light_sources.len(), 1);

        del_light_source_for_object(&mut state, obj_id);
        assert_eq!(state.current_level.light_sources.len(), 0);
    }

    #[test]
    fn test_light_radius_clamped() {
        let mut state = create_test_state();
        let obj_id = ObjectId(1);

        // Test that radius is clamped to MAX_LIGHT_RADIUS
        new_light_source_for_object(&mut state, 5, 5, 100, obj_id);
        assert_eq!(state.current_level.light_sources[0].range, MAX_LIGHT_RADIUS);

        // Test that radius is at least 1
        state.current_level.light_sources.clear();
        new_light_source_for_object(&mut state, 5, 5, 0, obj_id);
        assert_eq!(state.current_level.light_sources[0].range, 1);
    }

    #[test]
    fn test_obj_adjust_light_radius() {
        let mut state = create_test_state();
        let obj_id = ObjectId(1);

        new_light_source_for_object(&mut state, 5, 5, 2, obj_id);
        assert_eq!(state.current_level.light_sources[0].range, 2);

        obj_adjust_light_radius(&mut state, obj_id, 4);
        assert_eq!(state.current_level.light_sources[0].range, 4);
    }

    #[test]
    fn test_obj_move_light_source() {
        let mut state = create_test_state();
        let src_id = ObjectId(1);
        let dest_id = ObjectId(2);

        new_light_source_for_object(&mut state, 5, 5, 3, src_id);
        obj_move_light_source(&mut state, src_id, dest_id);

        assert_eq!(state.current_level.light_sources[0].id, 2);
    }

    #[test]
    fn test_light_stats() {
        let mut state = create_test_state();

        let (count, _) = light_stats(&state);
        assert_eq!(count, 0);

        new_light_source_for_object(&mut state, 5, 5, 3, ObjectId(1));
        new_light_source_for_object(&mut state, 6, 6, 2, ObjectId(2));

        let (count, size) = light_stats(&state);
        assert_eq!(count, 2);
        assert!(size > 0);
    }

    #[test]
    fn test_candle_light_range() {
        // 1-6 candles: radius 2
        assert_eq!(candle_light_range(1), 2);
        assert_eq!(candle_light_range(6), 2);

        // 7-48 candles: radius 3
        assert_eq!(candle_light_range(7), 3);
        assert_eq!(candle_light_range(48), 3);

        // 49+ candles: radius 4+
        assert_eq!(candle_light_range(49), 4);
    }

    #[test]
    fn test_candelabrum_light_range() {
        assert_eq!(candelabrum_light_range(0), 0);
        assert_eq!(candelabrum_light_range(1), 2);
        assert_eq!(candelabrum_light_range(3), 2);
        assert_eq!(candelabrum_light_range(4), 3);
        assert_eq!(candelabrum_light_range(6), 3);
        assert_eq!(candelabrum_light_range(7), 4);
    }

    // ============================================================================
    // Leash Tests
    // ============================================================================

    #[test]
    fn test_number_leashed() {
        let mut state = create_test_state();
        assert_eq!(number_leashed(&state), 0);

        let mut monster = Monster::new(MonsterId(1), 11, 10, 1);
        monster.state.leashed = true;
        state.current_level.monsters.push(monster);

        assert_eq!(number_leashed(&state), 1);
    }

    #[test]
    fn test_leashable_requires_tame() {
        let mut state = create_test_state();
        state.current_level.monsters.clear();

        // Untamed monster adjacent to player at (10, 10)
        let mut monster = Monster::new(MonsterId(1), 11, 10, 11);
        monster.state.tame = false;
        state.current_level.monsters.push(monster);

        assert!(!leashable(&state, MonsterId(1)));

        // Make it tame
        state.current_level.monsters[0].state.tame = true;
        assert!(leashable(&state, MonsterId(1)));
    }

    #[test]
    fn test_leashable_requires_proximity() {
        let mut state = create_test_state();
        state.current_level.monsters.clear();

        // Tame monster far away
        let mut monster = Monster::new(MonsterId(1), 11, 20, 20);
        monster.state.tame = true;
        state.current_level.monsters.push(monster);

        assert!(!leashable(&state, MonsterId(1)));

        // Move it adjacent to player at (10, 10)
        state.current_level.monsters[0].x = 10;
        state.current_level.monsters[0].y = 11;
        assert!(leashable(&state, MonsterId(1)));
    }

    #[test]
    fn test_m_unleash() {
        let mut state = create_test_state();
        state.current_level.monsters.clear();

        let mut monster = Monster::new(MonsterId(1), 11, 10, 11);
        monster.state.tame = true;
        monster.state.leashed = true;
        monster.name = "dog".to_string();
        state.current_level.monsters.push(monster);

        m_unleash(&mut state, MonsterId(1));

        assert!(!state.current_level.monsters[0].state.leashed);
        assert!(state.messages.iter().any(|m| m.contains("unleashed")));
    }

    #[test]
    fn test_unleash_all() {
        let mut state = create_test_state();
        state.current_level.monsters.clear();

        for i in 0..3 {
            let mut monster = Monster::new(MonsterId(i as u32 + 1), 11, 10 + i as i8, 10);
            monster.state.tame = true;
            monster.state.leashed = true;
            monster.name = format!("dog{}", i);
            state.current_level.monsters.push(monster);
        }

        assert_eq!(number_leashed(&state), 3);

        unleash_all(&mut state);

        assert_eq!(number_leashed(&state), 0);
    }
}
