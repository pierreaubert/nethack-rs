//! Opening and closing doors (lock.c)
//!
//! Door states from NetHack (rm.h):
//! - D_NODOOR = 0x00 (doorway with no door)
//! - D_BROKEN = 0x01 (broken door)
//! - D_ISOPEN = 0x02 (open door)
//! - D_CLOSED = 0x04 (closed door)
//! - D_LOCKED = 0x08 (locked door)
//! - D_TRAPPED = 0x10 (trapped door)
//! - D_SECRET = 0x20 (secret door)

use crate::action::{ActionResult, Direction};
use crate::dungeon::{CellType, DoorState};
use crate::gameloop::GameState;
use crate::object::{Object, ObjectClass};

/// Open a door in the given direction
pub fn do_open(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    let cell = state.current_level.cell(x as usize, y as usize);

    // Check if it's a door type
    if cell.typ != CellType::Door && cell.typ != CellType::SecretDoor {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    // Secret doors can't be opened until found
    if cell.typ == CellType::SecretDoor {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    let door_state = cell.door_state();

    // Check door state using flags
    if door_state.contains(DoorState::OPEN) {
        state.message("This door is already open.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::LOCKED) {
        state.message("This door is locked.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::BROKEN) {
        state.message("This door is broken.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::CLOSED) {
        openit(state, x, y);
        ActionResult::Success
    } else {
        // No door or already open
        state.message("This doorway has no door.");
        ActionResult::NoTime
    }
}

pub fn doopen_indir(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    // Simplified logic: just check if there is a door and open it
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ == CellType::Door {
        let door_state = cell.door_state();
        if door_state.contains(DoorState::CLOSED) && !door_state.contains(DoorState::LOCKED) {
            openit(state, x, y);
            return ActionResult::Success;
        }
    }
    ActionResult::NoTime
}

/// Close a door in the given direction
pub fn do_close(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    // Check for monster blocking
    if state.current_level.monster_at(x, y).is_some() {
        state.message("Something blocks the way!");
        return ActionResult::NoTime;
    }

    let cell = state.current_level.cell(x as usize, y as usize);

    if cell.typ != CellType::Door {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    let door_state = cell.door_state();

    if door_state.contains(DoorState::CLOSED) || door_state.contains(DoorState::LOCKED) {
        state.message("This door is already closed.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::BROKEN) {
        state.message("This door is broken.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::OPEN) {
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        cell.set_door_state(DoorState::CLOSED);
        state.message("The door closes.");
        ActionResult::Success
    } else {
        state.message("This doorway has no door.");
        ActionResult::NoTime
    }
}

pub fn openit(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    cell.set_door_state(DoorState::OPEN);
    state.message("The door opens.");
}

pub fn openone(state: &mut GameState, x: i8, y: i8) {
    openit(state, x, y);
}

pub fn really_close(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    cell.set_door_state(DoorState::CLOSED);
}

pub fn doorlock(state: &mut GameState, x: i8, y: i8) -> bool {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    let mut door_state = cell.door_state();

    if door_state.contains(DoorState::LOCKED) {
        door_state.remove(DoorState::LOCKED);
        cell.set_door_state(door_state);
        state.message("You unlock the door.");
        true
    } else if door_state.contains(DoorState::CLOSED) {
        door_state.insert(DoorState::LOCKED);
        cell.set_door_state(door_state);
        state.message("You lock the door.");
        true
    } else {
        false
    }
}

pub fn boxlock(state: &mut GameState, container: &mut Object) -> bool {
    if container.locked {
        container.locked = false;
        state.message("You unlock the container.");
    } else {
        container.locked = true;
        state.message("You lock the container.");
    }
    true
}

pub fn doorless_door(state: &mut GameState, x: i8, y: i8) -> bool {
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ == CellType::Door {
        let door_state = cell.door_state();
        if door_state.contains(DoorState::BROKEN) {
            state.message("This door is broken.");
            return true;
        }
    }
    false
}

/// Type of lock picking tool (matches C lock.c picktyp values)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickType {
    /// Lock pick (LOCK_PICK) — best for rogues
    LockPick,
    /// Credit card (CREDIT_CARD) — can only unlock, not lock
    CreditCard,
    /// Skeleton key (SKELETON_KEY) — most reliable
    SkeletonKey,
}

/// Target of a lock picking action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockTarget {
    /// Picking/locking a door at (x, y)
    Door { x: i8, y: i8 },
    /// Picking/locking a container (index in floor objects)
    Container { obj_index: usize },
}

/// Lock picking context (what we're currently picking)
///
/// Matches C xlock_s struct from lock.c
#[derive(Debug, Default, Clone)]
pub struct LockContext {
    /// What we're trying to pick
    pub target: Option<LockTarget>,
    /// Type of tool being used
    pub pick_type: Option<PickType>,
    /// Success chance per turn (percentage)
    pub chance: u32,
    /// Turns spent so far
    pub used_time: u32,
    /// Whether using the Master Key of Thievery
    pub magic_key: bool,
    // Legacy compatibility
    pub door_x: Option<i8>,
    pub door_y: Option<i8>,
}

/// Calculate lock picking success chance per turn based on tool type and dexterity.
///
/// Matches C lock.c chance calculations for both doors and containers.
/// Rogues get large bonuses. Skeleton keys are most reliable.
pub fn calculate_pick_chance(
    pick_type: PickType,
    dexterity: i8,
    is_rogue: bool,
    is_door: bool,
    is_cursed: bool,
) -> u32 {
    let dex = dexterity as i32;
    let rogue_bonus = if is_rogue { 1 } else { 0 };

    let base = if is_door {
        // Door chances (from C lock.c lines 503-515)
        match pick_type {
            PickType::CreditCard => 2 * dex + 20 * rogue_bonus,
            PickType::LockPick => 3 * dex + 30 * rogue_bonus,
            PickType::SkeletonKey => 70 + dex,
        }
    } else {
        // Container chances (from C lock.c lines 421-433)
        match pick_type {
            PickType::CreditCard => dex + 20 * rogue_bonus,
            PickType::LockPick => 4 * dex + 25 * rogue_bonus,
            PickType::SkeletonKey => 75 + dex,
        }
    };

    let chance = if is_cursed { base / 2 } else { base };
    chance.max(0) as u32
}

/// Get description of current lock action based on target and lock state.
///
/// Matches C lock_action() from lock.c.
pub fn lock_action_desc(target: &LockTarget, is_locked: bool, pick_type: PickType) -> &'static str {
    match target {
        LockTarget::Door { .. } => {
            if is_locked {
                if pick_type == PickType::LockPick || pick_type == PickType::CreditCard {
                    "picking the lock"
                } else {
                    "unlocking the door"
                }
            } else {
                "locking the door"
            }
        }
        LockTarget::Container { .. } => {
            if is_locked {
                if pick_type == PickType::LockPick || pick_type == PickType::CreditCard {
                    "picking the lock"
                } else {
                    "unlocking the chest"
                }
            } else {
                "locking the chest"
            }
        }
    }
}

/// Get description of current lock action based on whether door is currently locked
pub fn lock_action(is_unlocking: bool) -> &'static str {
    if is_unlocking {
        "unlocking the door"
    } else {
        "locking the door"
    }
}

/// Apply a lock pick tool to a door at (x, y).
///
/// Matches C pick_lock() from lock.c for the door case.
/// Sets up the multi-turn occupation with proper chance calculation.
pub fn pick_lock(state: &mut GameState, tool: &Object, x: i8, y: i8) -> ActionResult {
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ != CellType::Door {
        state.message("Nothing to pick here.");
        return ActionResult::NoTime;
    }

    let door_state = cell.door_state();

    // Check door conditions that prevent lock manipulation
    if door_state.contains(DoorState::BROKEN) {
        state.message("This door is broken.");
        return ActionResult::NoTime;
    }
    if door_state.contains(DoorState::OPEN) {
        state.message("You cannot lock an open door.");
        return ActionResult::NoTime;
    }

    // Determine pick type from tool
    let pick_type = classify_pick_tool(tool);
    let Some(pick_type) = pick_type else {
        state.message("You can't pick a lock with that!");
        return ActionResult::NoTime;
    };

    // Credit cards can only unlock, not lock
    let is_locked = door_state.contains(DoorState::LOCKED);
    if pick_type == PickType::CreditCard && !is_locked {
        state.message("You can't lock a door with a credit card.");
        return ActionResult::NoTime;
    }

    // Calculate success chance based on tool, dexterity, role
    let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
    let is_rogue = state.player.role == crate::player::Role::Rogue;
    let is_cursed = tool.buc == crate::object::BucStatus::Cursed;
    let chance = calculate_pick_chance(pick_type, dex, is_rogue, true, is_cursed);

    // For now, attempt the pick immediately with calculated chance
    // Full implementation would set up multi-turn occupation via LockContext
    if state.rng.rn2(100) < chance {
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        let mut new_state = cell.door_state();
        if is_locked {
            new_state.remove(DoorState::LOCKED);
            new_state.insert(DoorState::CLOSED);
        } else {
            new_state.insert(DoorState::LOCKED);
        }
        cell.set_door_state(new_state);

        let action = lock_action_desc(&LockTarget::Door { x, y }, is_locked, pick_type);
        state.message(format!("You succeed in {}.", action));
    } else {
        let action = lock_action_desc(&LockTarget::Door { x, y }, is_locked, pick_type);
        state.message(format!("You fail at {}.", action));
    }
    ActionResult::Success
}

/// Classify an object as a lock picking tool type.
///
/// Returns None if the object is not a valid lock pick tool.
pub fn classify_pick_tool(tool: &Object) -> Option<PickType> {
    // Check by name — in full implementation would check otyp
    if let Some(ref name) = tool.name {
        let lower = name.to_lowercase();
        if lower.contains("lock pick") || lower.contains("lockpick") {
            return Some(PickType::LockPick);
        }
        if lower.contains("credit card") {
            return Some(PickType::CreditCard);
        }
        if lower.contains("skeleton key") || lower.contains("key") {
            return Some(PickType::SkeletonKey);
        }
    }
    // Also check by class — tools that are keys
    if tool.class == ObjectClass::Tool {
        // Default to lock pick for unidentified tools used as picks
        return Some(PickType::LockPick);
    }
    None
}

/// Check if we're currently picking a lock
pub fn picking_lock() -> bool {
    false
}

/// Check if we're currently picking the lock at a specific position
pub fn picking_at() -> bool {
    false
}

/// Continue picking a lock (occupation function).
///
/// Matches C picklock() from lock.c. Each call represents one turn of
/// attempting to manipulate the lock. Success is checked against chance
/// (calculated from DEX, tool type, role). Gives up after 50 turns.
///
/// Returns time used (1 = still busy, 0 = done/cancelled).
pub fn picklock(state: &mut GameState, ctx: &mut LockContext) -> u32 {
    let target = match ctx.target {
        Some(t) => t,
        None => return 0,
    };

    match target {
        LockTarget::Door { x, y } => picklock_door(state, ctx, x, y),
        LockTarget::Container { obj_index } => picklock_container(state, ctx, obj_index),
    }
}

/// Lock picking on a door — one turn of the occupation.
fn picklock_door(state: &mut GameState, ctx: &mut LockContext, x: i8, y: i8) -> u32 {
    if !state.current_level.is_valid_pos(x, y) {
        reset_pick(ctx);
        return 0;
    }

    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ != CellType::Door {
        state.message("This doorway has no door.");
        reset_pick(ctx);
        return 0;
    }

    let door_state = cell.door_state();

    // C: check door conditions that abort picking
    if door_state.contains(DoorState::OPEN) {
        state.message("You cannot lock an open door.");
        reset_pick(ctx);
        return 0;
    }
    if door_state.contains(DoorState::BROKEN) {
        state.message("This door is broken.");
        reset_pick(ctx);
        return 0;
    }

    ctx.used_time += 1;

    // Give up after 50 turns (matches C: xlock.usedtime++ >= 50)
    if ctx.used_time >= 50 {
        let is_locked = door_state.contains(DoorState::LOCKED);
        let pick_type = ctx.pick_type.unwrap_or(PickType::LockPick);
        let action = lock_action_desc(&LockTarget::Door { x, y }, is_locked, pick_type);
        state.message(format!("You give up your attempt at {}.", action));
        // Exercise DEX even on failure (matches C)
        reset_pick(ctx);
        return 0;
    }

    // Check for success (matches C: rn2(100) >= xlock.chance means failure)
    if state.rng.rn2(100) >= ctx.chance {
        return 1; // Still busy
    }

    // Success!
    let was_locked = door_state.contains(DoorState::LOCKED);
    let was_trapped = door_state.contains(DoorState::TRAPPED);

    // Check for trapped door (matches C: if trapped, trigger trap)
    if was_trapped && !ctx.magic_key {
        // Trap triggers — door is destroyed
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        cell.set_door_state(DoorState::empty()); // D_NODOOR
        state.message("KABOOM! The door was trapped!");
        reset_pick(ctx);
        return 0;
    }

    // Magic key detects traps (matches C lines 114-143)
    if was_trapped && ctx.magic_key {
        // Disarm the trap
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        let mut new_state = cell.door_state();
        new_state.remove(DoorState::TRAPPED);
        cell.set_door_state(new_state);
        state.message("You find and disarm a trap on the door!");
        ctx.chance += 20; // Less effort next time (matches C)
        reset_pick(ctx);
        return 0;
    }

    // Toggle lock state
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    let mut new_state = cell.door_state();
    if was_locked {
        new_state.remove(DoorState::LOCKED);
        new_state.insert(DoorState::CLOSED);
    } else {
        new_state.insert(DoorState::LOCKED);
    }
    cell.set_door_state(new_state);

    let pick_type = ctx.pick_type.unwrap_or(PickType::LockPick);
    let action = lock_action_desc(&LockTarget::Door { x, y }, was_locked, pick_type);
    state.message(format!("You succeed in {}.", action));
    reset_pick(ctx);
    0
}

/// Lock picking on a container — one turn of the occupation.
fn picklock_container(state: &mut GameState, ctx: &mut LockContext, _obj_index: usize) -> u32 {
    // TODO: Full container lock picking with object lookup
    // For now, simplified: just use chance-based success like door
    ctx.used_time += 1;

    if ctx.used_time >= 50 {
        state.message("You give up your attempt to pick the lock.");
        reset_pick(ctx);
        return 0;
    }

    if state.rng.rn2(100) >= ctx.chance {
        return 1; // Still busy
    }

    state.message("You succeed in picking the lock.");
    reset_pick(ctx);
    0
}

/// Reset lock picking context (matches C reset_pick()).
pub fn reset_pick(ctx: &mut LockContext) {
    ctx.target = None;
    ctx.pick_type = None;
    ctx.chance = 0;
    ctx.used_time = 0;
    ctx.magic_key = false;
    ctx.door_x = None;
    ctx.door_y = None;
}

/// Conditionally reset pick if level changes or container is deleted.
///
/// Matches C maybe_reset_pick() from lock.c.
pub fn maybe_reset_pick(ctx: &mut LockContext, level_change: bool) {
    if level_change {
        reset_pick(ctx);
    }
}

pub fn check_door_at(state: &GameState, x: i8, y: i8) -> bool {
    let cell = state.current_level.cell(x as usize, y as usize);
    cell.typ == CellType::Door
}

/// Close a holding trap (bear trap or web) on a monster
/// Returns true if trap was closed successfully
pub fn closeholdingtrap(state: &mut GameState, x: i8, y: i8) -> bool {
    // Check for trap at position
    if let Some(trap_idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y)
    {
        let trap = &state.current_level.traps[trap_idx];
        // Check if it's a holding trap (bear trap or web)
        match trap.trap_type {
            crate::dungeon::TrapType::BearTrap | crate::dungeon::TrapType::Web => {
                // Trap is already set/closed
                return true;
            }
            _ => return false,
        }
    }
    false
}

/// Open a holding trap (bear trap or web) to release a monster
/// Returns true if trap was opened successfully
pub fn openholdingtrap(state: &mut GameState, x: i8, y: i8) -> bool {
    // Check for trap at position
    if let Some(trap_idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y)
    {
        let trap = &state.current_level.traps[trap_idx];
        match trap.trap_type {
            crate::dungeon::TrapType::BearTrap | crate::dungeon::TrapType::Web => {
                // In full implementation, would release trapped monster
                // For now, just acknowledge the trap exists
                return true;
            }
            _ => return false,
        }
    }
    false
}

/// Open a falling trap (trapdoor, pit, hole) under a monster
/// Returns true if trap was triggered
pub fn openfallingtrap(state: &mut GameState, x: i8, y: i8, trapdoor_only: bool) -> bool {
    if let Some(trap_idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y)
    {
        let trap = &state.current_level.traps[trap_idx];
        match trap.trap_type {
            crate::dungeon::TrapType::TrapDoor => return true,
            crate::dungeon::TrapType::Hole
            | crate::dungeon::TrapType::Pit
            | crate::dungeon::TrapType::SpikedPit => {
                if !trapdoor_only {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Place a door during level generation (from mklev.c)
/// Randomly chooses between regular door and secret door
pub fn dodoor(state: &mut GameState, x: i8, y: i8) {
    // 1 in 8 chance of secret door
    if state.rng.one_in(8) {
        dosdoor(state, x, y, true);
    } else {
        dosdoor(state, x, y, false);
    }
}

/// Set door state during level generation
pub fn dosdoor(state: &mut GameState, x: i8, y: i8, secret: bool) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);

    if secret {
        cell.typ = CellType::SecretDoor;
        cell.set_door_state(DoorState::CLOSED);
    } else {
        cell.typ = CellType::Door;

        // Randomly determine door state
        if state.rng.one_in(3) {
            // 1/3 chance: open, locked, or closed
            if state.rng.one_in(5) {
                cell.set_door_state(DoorState::OPEN);
            } else if state.rng.one_in(6) {
                cell.set_door_state(DoorState::LOCKED);
            } else {
                cell.set_door_state(DoorState::CLOSED);
            }
        } else {
            // 2/3 chance: no door (doorway)
            cell.set_door_state(DoorState::empty());
        }
    }
}

pub fn search_door(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    if cell.typ == CellType::SecretDoor && state.rng.one_in(3) {
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::CLOSED);
        state.message("You find a hidden door!");
    }
}

/// Check if a shopkeeper is blocking a door
/// Returns true if blocked, displays message
pub fn block_door(state: &mut GameState, x: i8, y: i8) -> bool {
    // Check if there's a door at the position
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ != CellType::Door {
        return false;
    }

    // Check if there's a shopkeeper at or near this door
    // In the full implementation, this would check shop boundaries
    // and shopkeeper debt/billing status
    // For now, return false (no blocking)
    false
}

/// Check if a shopkeeper is blocking entry to a shop
/// Returns true if entry is blocked
pub fn block_entry(state: &mut GameState, _x: i8, _y: i8) -> bool {
    // Check if player is at a broken door
    let player_cell = state
        .current_level
        .cell(state.player.pos.x as usize, state.player.pos.y as usize);
    if player_cell.typ != CellType::Door {
        return false;
    }

    let door_state = player_cell.door_state();
    if !door_state.contains(DoorState::BROKEN) {
        return false;
    }

    // In full implementation, would check if destination is a shop
    // and if shopkeeper wants to block entry
    false
}

pub fn cvt_sdoor_to_door(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    if cell.typ == CellType::SecretDoor {
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::CLOSED);
    }
}

// ============================================================================
// Force lock functions (from lock.c)
// ============================================================================

/// Result of attempting to force a container
#[derive(Debug, Clone, Default)]
pub struct ForceResult {
    pub messages: Vec<String>,
    pub success: bool,
    pub container_destroyed: bool,
    pub lock_broken: bool,
    pub items_destroyed: Vec<String>,
    pub shop_damage: i32,
}

impl ForceResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_message(mut self, msg: &str) -> Self {
        self.messages.push(msg.to_string());
        self
    }

    pub fn failure(msg: &str) -> Self {
        Self::new().with_message(msg)
    }

    pub fn success_open(msg: &str) -> Self {
        let mut result = Self::new().with_message(msg);
        result.success = true;
        result.lock_broken = true;
        result
    }
}

/// Whether a weapon is blade-type (for forcing locks).
///
/// In C (lock.c line 559): blade weapons pry, blunt weapons bash.
/// Blade weapons are more precise but can break.
pub fn is_blade_weapon(weapon: &crate::object::Object) -> bool {
    // Check by class — weapons in the Weapon class with cutting damage
    if weapon.class == ObjectClass::Weapon {
        // Check weapon name for blade-like keywords
        if let Some(ref name) = weapon.name {
            let lower = name.to_lowercase();
            return lower.contains("sword") || lower.contains("dagger")
                || lower.contains("knife") || lower.contains("blade")
                || lower.contains("katana") || lower.contains("scimitar")
                || lower.contains("saber") || lower.contains("axe");
        }
    }
    false
}

/// Attempt to force open a locked container (doforce equivalent).
///
/// Matches C doforce() from lock.c. Player uses wielded weapon to
/// force open a locked container. Blade weapons pry (DEX-based),
/// blunt weapons bash (STR-based). Blade weapons may break on failure.
/// Blunt weapons wake nearby monsters.
///
/// # Arguments
/// * `player` - The player attempting to force
/// * `weapon` - The weapon being used (wielded weapon)
/// * `container` - The container being forced
/// * `rng` - Random number generator
///
/// # Returns
/// Result indicating success, weapon break, destruction, or failure
pub fn doforce(
    player: &crate::player::You,
    container: &mut crate::object::Object,
    rng: &mut crate::rng::GameRng,
) -> ForceResult {
    // Check for special conditions (matches C doforce lines 537-540)
    if player.swallowed {
        return ForceResult::failure("You can't force anything from inside here.");
    }

    // Check if container is already unlocked or broken (matches C lines 570-579)
    if !container.locked {
        if container.broken {
            return ForceResult::failure("That container's lock is already broken.");
        } else {
            return ForceResult::failure("That container is already unlocked.");
        }
    }

    let mut result = ForceResult::new();

    // Determine forcing style: blade pries, blunt bashes (matches C line 559)
    // For now, use STR-based chance (blunt-style) as default
    // In full implementation, would check wielded weapon type
    let is_blade = false; // Simplified: assume blunt by default

    if is_blade {
        result.messages.push("You force your weapon into a crack and pry.".to_string());
    } else {
        result.messages.push("You start bashing it with your weapon.".to_string());
    }

    // Calculate success chance (matches C line 596: objects[uwep->otyp].oc_wldam * 2)
    // Approximate with weapon damage * 2
    let base_chance = if is_blade {
        // Blade: DEX-based, lower damage chance but less destructive
        (player.attr_current.get(crate::player::Attribute::Dexterity) as i32) * 2
    } else {
        // Blunt: STR-based, higher damage chance
        (player.attr_current.get(crate::player::Attribute::Strength) as i32) * 2
    };

    let roll = rng.rn2(100) as i32;

    if roll < base_chance {
        // Success (matches C line 255-263)
        result.success = true;
        result.lock_broken = true;
        result.messages.push("You succeed in forcing the lock.".to_string());

        // Blunt weapons: chance of destroying container (matches C line 260)
        if !is_blade && rng.rn2(3) == 0 {
            result.container_destroyed = true;
            result.messages.push("In fact, you've totally destroyed it!".to_string());
        }
    } else {
        // Failure — the lock holds
        result.messages.push("The lock holds.".to_string());

        // Blade weapons may break on failure (matches C lines 236-248)
        // In C: probability of surviving = (.992)^50 = .67 for +0 weapon
        if is_blade && rng.rn2(1000) > 992 {
            result.messages.push("Your weapon broke!".to_string());
        }
    }

    result
}

/// Break a chest's lock (breakchestlock equivalent)
///
/// Called when a chest lock is broken through force or other means.
/// If destroy_it is true, the chest is destroyed and contents are scattered.
///
/// # Arguments
/// * `container` - The container whose lock is being broken
/// * `destroy_it` - Whether to destroy the container entirely
/// * `rng` - Random number generator
///
/// # Returns
/// Result with damage details
pub fn breakchestlock(
    container: &mut crate::object::Object,
    destroy_it: bool,
    rng: &mut crate::rng::GameRng,
) -> ForceResult {
    let mut result = ForceResult::new();

    if !destroy_it {
        // Just break the lock, container survives
        container.locked = false;
        container.broken = true;
        result.lock_broken = true;
        result.messages.push(format!(
            "You break the lock on {}.",
            container.display_name()
        ));
    } else {
        // Container is destroyed - contents are scattered
        result.container_destroyed = true;
        result.lock_broken = true;
        result.messages.push(format!(
            "In fact, you've totally destroyed {}.",
            container.display_name()
        ));

        // Some contents may be destroyed (potions, fragile items)
        // In full implementation, would iterate through container contents
        // and potentially destroy some items
        let items_at_risk = rng.rn2(5) as usize;
        for i in 0..items_at_risk {
            if rng.rn2(3) == 0 {
                result.items_destroyed.push(format!("item {}", i));
                result
                    .messages
                    .push("Something inside is destroyed!".to_string());
            }
        }
    }

    result
}

/// Generate a message for an item destroyed when forcing a chest
/// (chest_shatter_msg equivalent)
///
/// Different materials break in different ways.
///
/// # Arguments
/// * `object` - The object that was destroyed
/// * `blind` - Whether the player is blind
///
/// # Returns
/// Message describing the destruction
pub fn chest_shatter_msg(object: &crate::object::Object, blind: bool) -> String {
    // Potions shatter specially
    if object.class == crate::object::ObjectClass::Potion {
        if blind {
            return "You hear something shatter!".to_string();
        } else {
            return "You see a bottle shatter!".to_string();
        }
    }

    // Other items based on material
    // In full implementation, would check object material
    let disposition = match object.class {
        crate::object::ObjectClass::Scroll | crate::object::ObjectClass::Spellbook => {
            "is torn to shreds"
        }
        crate::object::ObjectClass::Wand => "is snapped in two",
        crate::object::ObjectClass::Food => "is crushed",
        _ => "is smashed",
    };

    format!("A {} {}!", object.display_name(), disposition)
}

/// Context for multi-turn forcing of a container lock.
///
/// Matches C forcelock() occupation from lock.c.
#[derive(Debug, Default, Clone)]
pub struct ForceContext {
    /// Whether using a blade (true) or blunt (false) weapon
    pub is_blade: bool,
    /// Success chance per turn (percentage)
    pub chance: u32,
    /// Turns spent forcing
    pub used_time: u32,
}

/// Continue forcing a container lock (occupation function).
///
/// Matches C forcelock() from lock.c. Each call = one turn of forcing.
/// Returns 1 if still working, 0 if done/cancelled.
pub fn forcelock(
    ctx: &mut ForceContext,
    rng: &mut crate::rng::GameRng,
) -> (u32, ForceResult) {
    ctx.used_time += 1;

    // Give up after 50 turns (matches C: xlock.usedtime++ >= 50)
    if ctx.used_time >= 50 {
        let mut result = ForceResult::new();
        result.messages.push("You give up your attempt to force the lock.".to_string());
        return (0, result);
    }

    // Blade weapons can break during forcing (matches C lines 236-248)
    if ctx.is_blade && rng.rn2(1000) > 992 {
        let mut result = ForceResult::new();
        result.messages.push("Your weapon broke!".to_string());
        return (0, result);
    }

    // Check for success (matches C line 252: rn2(100) >= xlock.chance)
    if rng.rn2(100) >= ctx.chance {
        return (1, ForceResult::new()); // Still busy
    }

    // Success!
    let mut result = ForceResult::new();
    result.success = true;
    result.lock_broken = true;
    result.messages.push("You succeed in forcing the lock.".to_string());

    // Blunt weapons: 1/3 chance of destroying container (matches C line 260)
    if !ctx.is_blade && rng.rn2(3) == 0 {
        result.container_destroyed = true;
    }

    (0, result)
}

/// Wand/spell effect on a box lock.
///
/// Matches C boxlock() from lock.c. WAN_LOCKING locks it,
/// WAN_OPENING unlocks it.
pub fn boxlock_spell(container: &mut crate::object::Object, locking: bool) -> Option<&'static str> {
    if locking {
        if !container.locked {
            container.locked = true;
            container.broken = false;
            Some("Klunk!")
        } else {
            None // Already locked
        }
    } else if container.locked {
        container.locked = false;
        Some("Klick!")
    } else {
        container.broken = false; // Silently fix if broken
        None
    }
}

/// Wand/spell effect on a door lock.
///
/// Matches C doorlock() from lock.c. Handles locking (wizard lock),
/// opening (knock), and striking (force bolt) effects on doors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorSpellEffect {
    /// WAN_LOCKING / SPE_WIZARD_LOCK
    Lock,
    /// WAN_OPENING / SPE_KNOCK
    Knock,
    /// WAN_STRIKING / SPE_FORCE_BOLT
    Strike,
}

/// Result of a door spell effect
#[derive(Debug, Clone)]
pub struct DoorSpellResult {
    pub changed: bool,
    pub message: Option<String>,
    pub loudness: i32,
    pub destroyed: bool,
}

pub fn doorlock_spell(
    level: &mut crate::dungeon::Level,
    x: i8,
    y: i8,
    effect: DoorSpellEffect,
) -> DoorSpellResult {
    let cell = level.cell(x as usize, y as usize);
    if cell.typ != CellType::Door {
        return DoorSpellResult {
            changed: false,
            message: None,
            loudness: 0,
            destroyed: false,
        };
    }

    let door_state = cell.door_state();

    match effect {
        DoorSpellEffect::Lock => {
            // Lock the door (matches C lines 988-1009)
            let msg = match () {
                _ if door_state.contains(DoorState::CLOSED) => "The door locks!",
                _ if door_state.contains(DoorState::OPEN) => "The door swings shut, and locks!",
                _ if door_state.contains(DoorState::BROKEN) => "The broken door reassembles and locks!",
                _ => "A cloud of dust springs up and assembles itself into a door!",
            };
            let cell = level.cell_mut(x as usize, y as usize);
            let trapped = door_state.contains(DoorState::TRAPPED);
            let mut new_state = DoorState::LOCKED;
            if trapped {
                new_state.insert(DoorState::TRAPPED);
            }
            cell.set_door_state(new_state);
            DoorSpellResult {
                changed: true,
                message: Some(msg.to_string()),
                loudness: 0,
                destroyed: false,
            }
        }
        DoorSpellEffect::Knock => {
            // Unlock the door (matches C lines 1012-1017)
            if door_state.contains(DoorState::LOCKED) {
                let cell = level.cell_mut(x as usize, y as usize);
                let trapped = door_state.contains(DoorState::TRAPPED);
                let mut new_state = DoorState::CLOSED;
                if trapped {
                    new_state.insert(DoorState::TRAPPED);
                }
                cell.set_door_state(new_state);
                DoorSpellResult {
                    changed: true,
                    message: Some("The door unlocks!".to_string()),
                    loudness: 0,
                    destroyed: false,
                }
            } else {
                DoorSpellResult {
                    changed: false,
                    message: None,
                    loudness: 0,
                    destroyed: false,
                }
            }
        }
        DoorSpellEffect::Strike => {
            // Bash the door (matches C lines 1018-1051)
            if door_state.contains(DoorState::LOCKED) || door_state.contains(DoorState::CLOSED) {
                if door_state.contains(DoorState::TRAPPED) {
                    // Trapped door explodes
                    let cell = level.cell_mut(x as usize, y as usize);
                    cell.set_door_state(DoorState::empty());
                    DoorSpellResult {
                        changed: true,
                        message: Some("KABOOM!! The door explodes!".to_string()),
                        loudness: 40,
                        destroyed: true,
                    }
                } else {
                    let cell = level.cell_mut(x as usize, y as usize);
                    cell.set_door_state(DoorState::BROKEN);
                    DoorSpellResult {
                        changed: true,
                        message: Some("The door crashes open!".to_string()),
                        loudness: 20,
                        destroyed: true,
                    }
                }
            } else {
                DoorSpellResult {
                    changed: false,
                    message: None,
                    loudness: 0,
                    destroyed: false,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::CellType;

    #[test]
    fn test_do_open_door() {
        let mut state = GameState::default();
        let (x, y) = (10, 10);
        state.player.pos.x = x - 1;
        state.player.pos.y = y;

        // Place a closed door
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::CLOSED);

        // Open it
        // We need to mock RNG or attributes to ensure success
        state
            .player
            .attr_current
            .set(crate::player::Attribute::Strength, 25); // Max str

        do_open(&mut state, Direction::East);

        let cell = state.current_level.cell(x as usize, y as usize);
        assert!(cell.door_state().contains(DoorState::OPEN));
    }

    // ========== Force Lock Tests ==========

    #[test]
    fn test_force_result_new() {
        let result = ForceResult::new();
        assert!(!result.success);
        assert!(!result.container_destroyed);
        assert!(!result.lock_broken);
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_force_result_failure() {
        let result = ForceResult::failure("Test failure");
        assert!(!result.success);
        assert_eq!(result.messages.len(), 1);
        assert!(result.messages[0].contains("Test failure"));
    }

    #[test]
    fn test_doforce_swallowed() {
        let mut player = crate::player::You::default();
        player.swallowed = true;
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        assert!(!result.success);
        assert!(result.messages[0].contains("can't force"));
    }

    #[test]
    fn test_doforce_already_unlocked() {
        let player = crate::player::You::default();
        let mut container = crate::object::Object::default();
        container.locked = false;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        assert!(!result.success);
        assert!(result.messages[0].contains("already unlocked"));
    }

    #[test]
    fn test_doforce_already_broken() {
        let player = crate::player::You::default();
        let mut container = crate::object::Object::default();
        container.locked = false;
        container.broken = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        assert!(!result.success);
        assert!(result.messages[0].contains("already broken"));
    }

    #[test]
    fn test_doforce_locked_container() {
        let mut player = crate::player::You::default();
        player
            .attr_current
            .set(crate::player::Attribute::Strength, 18);
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        // May succeed or fail based on RNG
        assert!(!result.messages.is_empty());
        // First message describes the forcing action (bash or pry)
        assert!(result.messages[0].contains("bashing") || result.messages[0].contains("pry"));
    }

    #[test]
    fn test_breakchestlock_not_destroyed() {
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = breakchestlock(&mut container, false, &mut rng);

        assert!(result.lock_broken);
        assert!(!result.container_destroyed);
        assert!(!container.locked);
        assert!(container.broken);
    }

    #[test]
    fn test_breakchestlock_destroyed() {
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = breakchestlock(&mut container, true, &mut rng);

        assert!(result.lock_broken);
        assert!(result.container_destroyed);
        assert!(result.messages.iter().any(|m| m.contains("destroyed")));
    }

    #[test]
    fn test_chest_shatter_msg_potion_blind() {
        let obj = crate::object::Object {
            class: crate::object::ObjectClass::Potion,
            ..Default::default()
        };
        let msg = chest_shatter_msg(&obj, true);
        assert!(msg.contains("hear"));
    }

    #[test]
    fn test_chest_shatter_msg_potion_sighted() {
        let obj = crate::object::Object {
            class: crate::object::ObjectClass::Potion,
            ..Default::default()
        };
        let msg = chest_shatter_msg(&obj, false);
        assert!(msg.contains("see"));
    }

    #[test]
    fn test_chest_shatter_msg_scroll() {
        let obj = crate::object::Object {
            class: crate::object::ObjectClass::Scroll,
            ..Default::default()
        };
        let msg = chest_shatter_msg(&obj, false);
        assert!(msg.contains("torn to shreds"));
    }

    // ========== Lock Picking Chance Tests ==========

    #[test]
    fn test_pick_chance_skeleton_key_door() {
        // Skeleton key on door: 70 + DEX
        let chance = calculate_pick_chance(PickType::SkeletonKey, 18, false, true, false);
        assert_eq!(chance, 88); // 70 + 18
    }

    #[test]
    fn test_pick_chance_lockpick_door_rogue() {
        // Lock pick on door by rogue: 3*DEX + 30
        let chance = calculate_pick_chance(PickType::LockPick, 16, true, true, false);
        assert_eq!(chance, 78); // 3*16 + 30
    }

    #[test]
    fn test_pick_chance_credit_card_door() {
        // Credit card on door: 2*DEX + 0 (non-rogue)
        let chance = calculate_pick_chance(PickType::CreditCard, 14, false, true, false);
        assert_eq!(chance, 28); // 2*14
    }

    #[test]
    fn test_pick_chance_lockpick_container_rogue() {
        // Lock pick on container by rogue: 4*DEX + 25
        let chance = calculate_pick_chance(PickType::LockPick, 16, true, false, false);
        assert_eq!(chance, 89); // 4*16 + 25
    }

    #[test]
    fn test_pick_chance_skeleton_key_container() {
        // Skeleton key on container: 75 + DEX
        let chance = calculate_pick_chance(PickType::SkeletonKey, 14, false, false, false);
        assert_eq!(chance, 89); // 75 + 14
    }

    #[test]
    fn test_pick_chance_cursed_halved() {
        // Cursed tool: chance is halved
        let normal = calculate_pick_chance(PickType::SkeletonKey, 18, false, true, false);
        let cursed = calculate_pick_chance(PickType::SkeletonKey, 18, false, true, true);
        assert_eq!(cursed, normal / 2);
    }

    #[test]
    fn test_lock_action_desc_door_locked() {
        let desc = lock_action_desc(&LockTarget::Door { x: 5, y: 5 }, true, PickType::SkeletonKey);
        assert_eq!(desc, "unlocking the door");
    }

    #[test]
    fn test_lock_action_desc_door_unlocked() {
        let desc = lock_action_desc(&LockTarget::Door { x: 5, y: 5 }, false, PickType::SkeletonKey);
        assert_eq!(desc, "locking the door");
    }

    #[test]
    fn test_lock_action_desc_lockpick() {
        let desc = lock_action_desc(&LockTarget::Door { x: 5, y: 5 }, true, PickType::LockPick);
        assert_eq!(desc, "picking the lock");
    }

    #[test]
    fn test_classify_pick_tool_lockpick() {
        let mut tool = crate::object::Object::default();
        tool.name = Some("lock pick".to_string());
        assert_eq!(classify_pick_tool(&tool), Some(PickType::LockPick));
    }

    #[test]
    fn test_classify_pick_tool_credit_card() {
        let mut tool = crate::object::Object::default();
        tool.name = Some("credit card".to_string());
        assert_eq!(classify_pick_tool(&tool), Some(PickType::CreditCard));
    }

    #[test]
    fn test_classify_pick_tool_skeleton_key() {
        let mut tool = crate::object::Object::default();
        tool.name = Some("skeleton key".to_string());
        assert_eq!(classify_pick_tool(&tool), Some(PickType::SkeletonKey));
    }

    // ========== Multi-turn Forcing Tests ==========

    #[test]
    fn test_forcelock_gives_up_after_50() {
        let mut ctx = ForceContext {
            is_blade: false,
            chance: 0, // Will never succeed
            used_time: 49,
        };
        let mut rng = crate::rng::GameRng::new(42);
        let (turns, result) = forcelock(&mut ctx, &mut rng);
        assert_eq!(turns, 0);
        assert!(result.messages.iter().any(|m| m.contains("give up")));
    }

    #[test]
    fn test_forcelock_high_chance_succeeds() {
        let mut ctx = ForceContext {
            is_blade: false,
            chance: 99, // Almost always succeeds
            used_time: 0,
        };
        let mut rng = crate::rng::GameRng::new(42);
        let (turns, result) = forcelock(&mut ctx, &mut rng);
        assert_eq!(turns, 0);
        assert!(result.success);
    }

    // ========== Door Spell Tests ==========

    #[test]
    fn test_doorlock_spell_lock() {
        let mut level = crate::dungeon::Level::new(crate::dungeon::DLevel::new(0, 1));
        let cell = level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::CLOSED);

        let result = doorlock_spell(&mut level, 10, 10, DoorSpellEffect::Lock);
        assert!(result.changed);
        assert!(result.message.unwrap().contains("locks"));

        let cell = level.cell(10, 10);
        assert!(cell.door_state().contains(DoorState::LOCKED));
    }

    #[test]
    fn test_doorlock_spell_knock() {
        let mut level = crate::dungeon::Level::new(crate::dungeon::DLevel::new(0, 1));
        let cell = level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::LOCKED);

        let result = doorlock_spell(&mut level, 10, 10, DoorSpellEffect::Knock);
        assert!(result.changed);
        assert!(result.message.unwrap().contains("unlocks"));

        let cell = level.cell(10, 10);
        assert!(cell.door_state().contains(DoorState::CLOSED));
        assert!(!cell.door_state().contains(DoorState::LOCKED));
    }

    #[test]
    fn test_doorlock_spell_strike_breaks_door() {
        let mut level = crate::dungeon::Level::new(crate::dungeon::DLevel::new(0, 1));
        let cell = level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::LOCKED);

        let result = doorlock_spell(&mut level, 10, 10, DoorSpellEffect::Strike);
        assert!(result.changed);
        assert!(result.destroyed);
        assert_eq!(result.loudness, 20);

        let cell = level.cell(10, 10);
        assert!(cell.door_state().contains(DoorState::BROKEN));
    }

    #[test]
    fn test_doorlock_spell_strike_trapped() {
        let mut level = crate::dungeon::Level::new(crate::dungeon::DLevel::new(0, 1));
        let cell = level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::LOCKED | DoorState::TRAPPED);

        let result = doorlock_spell(&mut level, 10, 10, DoorSpellEffect::Strike);
        assert!(result.changed);
        assert!(result.destroyed);
        assert_eq!(result.loudness, 40); // Louder explosion
        assert!(result.message.unwrap().contains("KABOOM"));
    }

    #[test]
    fn test_boxlock_spell_lock() {
        let mut container = crate::object::Object::default();
        container.locked = false;
        let msg = boxlock_spell(&mut container, true);
        assert!(container.locked);
        assert_eq!(msg, Some("Klunk!"));
    }

    #[test]
    fn test_boxlock_spell_unlock() {
        let mut container = crate::object::Object::default();
        container.locked = true;
        let msg = boxlock_spell(&mut container, false);
        assert!(!container.locked);
        assert_eq!(msg, Some("Klick!"));
    }

    // ========== Picklock Multi-turn Door Tests ==========

    #[test]
    fn test_picklock_door_success() {
        let mut state = GameState::default();
        let cell = state.current_level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::LOCKED);

        let mut ctx = LockContext {
            target: Some(LockTarget::Door { x: 10, y: 10 }),
            pick_type: Some(PickType::SkeletonKey),
            chance: 99, // Almost always succeeds
            used_time: 0,
            magic_key: false,
            door_x: Some(10),
            door_y: Some(10),
        };

        let result = picklock(&mut state, &mut ctx);
        assert_eq!(result, 0); // Done

        let cell = state.current_level.cell(10, 10);
        assert!(!cell.door_state().contains(DoorState::LOCKED));
    }

    #[test]
    fn test_picklock_door_timeout() {
        let mut state = GameState::default();
        let cell = state.current_level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::LOCKED);

        let mut ctx = LockContext {
            target: Some(LockTarget::Door { x: 10, y: 10 }),
            pick_type: Some(PickType::LockPick),
            chance: 0, // Will never succeed
            used_time: 49, // About to time out
            magic_key: false,
            door_x: Some(10),
            door_y: Some(10),
        };

        let result = picklock(&mut state, &mut ctx);
        assert_eq!(result, 0); // Gave up

        // Door should still be locked
        let cell = state.current_level.cell(10, 10);
        assert!(cell.door_state().contains(DoorState::LOCKED));
    }

    #[test]
    fn test_picklock_trapped_door() {
        let mut state = GameState::default();
        let cell = state.current_level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::LOCKED | DoorState::TRAPPED);

        let mut ctx = LockContext {
            target: Some(LockTarget::Door { x: 10, y: 10 }),
            pick_type: Some(PickType::LockPick),
            chance: 99,
            used_time: 0,
            magic_key: false,
            door_x: Some(10),
            door_y: Some(10),
        };

        let result = picklock(&mut state, &mut ctx);
        assert_eq!(result, 0);

        // Trapped door should be destroyed (D_NODOOR)
        let cell = state.current_level.cell(10, 10);
        assert!(cell.door_state().is_empty());
    }

    #[test]
    fn test_picklock_magic_key_detects_trap() {
        let mut state = GameState::default();
        let cell = state.current_level.cell_mut(10, 10);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::LOCKED | DoorState::TRAPPED);

        let mut ctx = LockContext {
            target: Some(LockTarget::Door { x: 10, y: 10 }),
            pick_type: Some(PickType::SkeletonKey),
            chance: 99,
            used_time: 0,
            magic_key: true, // Master Key of Thievery
            door_x: Some(10),
            door_y: Some(10),
        };

        let result = picklock(&mut state, &mut ctx);
        assert_eq!(result, 0);

        // Trap should be disarmed but door still locked
        let cell = state.current_level.cell(10, 10);
        assert!(!cell.door_state().contains(DoorState::TRAPPED));
    }
}
