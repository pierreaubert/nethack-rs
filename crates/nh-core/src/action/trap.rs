//! Trap mechanics (trap.c)

use crate::action::ActionResult;
use crate::dungeon::TrapType;
use crate::dungeon::trap::{
    self, is_holding_trap, roll_trap_damage, trap_name,
};
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterResistances};
use crate::rng::GameRng;

/// Check for and trigger traps at a position
pub fn check_trap(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    let trap_type = match state.current_level.trap_at(x, y) {
        Some(t) => t.trap_type,
        None => return ActionResult::NoTime,
    };

    // Build player resistances for dotrap
    let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
    let resistances = trap::resistances_from_properties(
        |prop| state.player.properties.has(prop),
        dex,
    );

    // Trigger the trap
    if let Some(trap) = state.current_level.trap_at_mut(x, y) {
        let result = trap::dotrap(&mut state.rng, trap, &resistances, false);

        for msg in &result.messages {
            state.message(msg.clone());
        }

        if result.damage > 0 {
            state.player.take_damage(result.damage);
        }

        if result.held_turns > 0 {
            state.player.utrap = result.held_turns;
            state.player.utraptype = Some(trap_type);
        }

        if result.trap_destroyed {
            state.current_level.remove_trap(x, y);
        }
    }

    if state.player.hp <= 0 {
        ActionResult::Died("killed by a trap".to_string())
    } else {
        ActionResult::Success
    }
}

/// Trigger a specific trap type on the player (convenience wrapper for tests/gameloop)
pub fn trigger_trap(state: &mut GameState, trap_type: TrapType) -> ActionResult {
    let mut temp_trap = crate::dungeon::trap::create_trap(
        state.player.pos.x,
        state.player.pos.y,
        trap_type,
    );
    let effect = crate::dungeon::trap::trigger_trap(&mut state.rng, &mut temp_trap);

    match effect {
        crate::dungeon::trap::TrapEffect::Damage(d) => {
            state.player.take_damage(d);
            if state.player.hp <= 0 {
                ActionResult::Died("killed by a trap".to_string())
            } else {
                ActionResult::Success
            }
        }
        crate::dungeon::trap::TrapEffect::Status(s) => {
            state.message(format!("You are affected by {:?}!", s));
            ActionResult::Success
        }
        crate::dungeon::trap::TrapEffect::Trapped { turns } => {
            state.player.utrap = turns;
            state.player.utraptype = Some(trap_type);
            ActionResult::Success
        }
        crate::dungeon::trap::TrapEffect::Teleport { x, y } => {
            state.player.pos.x = x;
            state.player.pos.y = y;
            ActionResult::Success
        }
        crate::dungeon::trap::TrapEffect::Fall { damage, .. } => {
            state.player.take_damage(damage);
            if state.player.hp <= 0 {
                ActionResult::Died("killed by a fall".to_string())
            } else {
                ActionResult::Success
            }
        }
        _ => ActionResult::Success,
    }
}

/// Search for traps and secret doors
pub fn do_search(state: &mut GameState) -> ActionResult {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    // Check adjacent cells for hidden traps
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let tx = px + dx;
            let ty = py + dy;
            if let Some(trap) = state.current_level.trap_at_mut(tx, ty)
                && !trap.seen
            {
                // Search skill check
                let search_bonus = state.player.exp_level;
                if state.rng.rn2(20) < (search_bonus as u32).min(18) {
                    trap.seen = true;
                    let name = trap_name(trap.trap_type);
                    state.message(format!("You find a {}.", name));
                }
            }
        }
    }

    state.message("You search for traps and secret doors.");
    ActionResult::Success
}

// ============================================================================
// mintrap — monster triggers a trap (trap.c)
// ============================================================================

/// Result of a monster stepping into a trap
#[derive(Debug, Clone, Default)]
pub struct MintrapResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Damage dealt to the monster
    pub damage: i32,
    /// Monster is held in trap (turns)
    pub held_turns: i32,
    /// Monster was teleported to new position
    pub teleport: Option<(i8, i8)>,
    /// Monster fell through to a lower level
    pub fell_through: bool,
    /// Trap should be destroyed
    pub trap_destroyed: bool,
    /// Trap was avoided entirely
    pub avoided: bool,
}

/// Monster triggers a trap (mintrap from trap.c).
///
/// Core logic for each trap type vs a monster:
/// - Flying/levitating monsters avoid ground traps
/// - Arrow/Dart: damage + dodge check
/// - BearTrap/Web: hold monster
/// - Pit/SpikedPit: damage + hold
/// - Teleport: random relocation
/// - FireTrap: fire damage (check resistance)
/// - SleepingGas: sleep monster
/// - LandMine: damage + destroy trap
///
/// Returns true if trap triggered, false if avoided.
pub fn mintrap(
    rng: &mut GameRng,
    monster: &Monster,
    trap_type: TrapType,
) -> MintrapResult {
    let mut result = MintrapResult::default();
    let mon_name = &monster.name;

    // Flying/levitating monsters avoid ground traps.
    // TODO: check MonsterFlags::FLY once PerMonst flags are available.
    let _is_flying = false;

    match trap_type {
        TrapType::Arrow => {
            // Dexterity-based dodge (simplified)
            let damage = roll_trap_damage(rng, trap_type);
            if rng.one_in(4) {
                result.messages.push(format!("An arrow misses the {}.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("An arrow hits the {}!", mon_name));
                result.damage = damage;
            }
        }

        TrapType::Dart => {
            let damage = roll_trap_damage(rng, trap_type);
            if rng.one_in(4) {
                result.messages.push(format!("A dart misses the {}.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("A dart hits the {}!", mon_name));
                result.damage = damage;
                // Poison check
                if rng.one_in(3) && !monster.resistances.contains(MonsterResistances::POISON) {
                    result.messages.push(format!("The {} is poisoned!", mon_name));
                    result.damage += rng.rnd(6) as i32;
                }
            }
        }

        TrapType::RockFall => {
            let damage = roll_trap_damage(rng, trap_type);
            result.messages.push(format!("A rock falls on the {}!", mon_name));
            result.damage = damage;
        }

        TrapType::Squeaky => {
            result.messages.push("A board beneath it squeaks loudly.".to_string());
            // Wakes up sleeping monsters nearby (handled by caller)
        }

        TrapType::BearTrap => {
            let damage = roll_trap_damage(rng, trap_type);
            result.messages.push(format!("The {} is caught in a bear trap!", mon_name));
            result.damage = damage;
            result.held_turns = (rng.rnd(5) + 3) as i32;
        }

        TrapType::LandMine => {
            let damage = roll_trap_damage(rng, trap_type);
            result.messages.push(format!("KAABLAMM!!! The {} triggers a land mine!", mon_name));
            result.damage = damage;
            result.trap_destroyed = true;
        }

        TrapType::RollingBoulder => {
            if rng.one_in(4) {
                result.messages.push(format!("A boulder misses the {}.", mon_name));
                result.avoided = true;
            } else {
                let damage = roll_trap_damage(rng, trap_type);
                result.messages.push(format!("A boulder hits the {}!", mon_name));
                result.damage = damage;
            }
        }

        TrapType::SleepingGas => {
            if monster.resistances.contains(MonsterResistances::SLEEP) {
                result.messages.push(format!("The {} resists the gas.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("The {} falls asleep!", mon_name));
                result.held_turns = (rng.rnd(25) + 10) as i32;
            }
        }

        TrapType::RustTrap => {
            result.messages.push(format!("A gush of water hits the {}!", mon_name));
            // Rust damage to monster's iron equipment (erosion +1)
            result.damage = 0; // No HP damage, just equipment degradation
        }

        TrapType::FireTrap => {
            let damage = roll_trap_damage(rng, trap_type);
            if monster.resistances.contains(MonsterResistances::FIRE) {
                result.messages.push(format!("The {} is unaffected by the fire.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("The {} is engulfed in flames!", mon_name));
                result.damage = damage;
            }
        }

        TrapType::Pit | TrapType::SpikedPit => {
            let damage = roll_trap_damage(rng, trap_type);
            let pit_name = trap_name(trap_type);
            result.messages.push(format!("The {} falls into a {}!", mon_name, pit_name));
            result.damage = damage;
            result.held_turns = (rng.rnd(6) + 2) as i32;

            if trap_type == TrapType::SpikedPit
                && rng.one_in(6)
                && !monster.resistances.contains(MonsterResistances::POISON)
            {
                result.messages.push("The spikes were poisoned!".to_string());
                result.damage += rng.rnd(8) as i32;
            }
        }

        TrapType::Hole | TrapType::TrapDoor => {
            result.messages.push(format!("The {} falls through!", mon_name));
            result.fell_through = true;
            result.damage = rng.rnd(6) as i32;
        }

        TrapType::Teleport => {
            result.messages.push(format!("The {} is teleported!", mon_name));
            let x = (rng.rn2(77) + 1) as i8;
            let y = (rng.rn2(19) + 1) as i8;
            result.teleport = Some((x, y));
        }

        TrapType::LevelTeleport => {
            if monster.resistances.contains(MonsterResistances::FIRE) {
                // Using FIRE as proxy for magic resistance check
                result.avoided = true;
            } else {
                result.messages.push(format!("The {} vanishes!", mon_name));
                result.fell_through = true;
            }
        }

        TrapType::Web => {
            result.messages.push(format!("The {} is caught in a web!", mon_name));
            result.held_turns = (rng.rnd(10) + 5) as i32;
        }

        TrapType::MagicTrap => {
            result.messages.push(format!("The {} is caught in a magical light!", mon_name));
            // Random minor effect
        }

        TrapType::AntiMagic => {
            // Drain energy from spellcasting monsters
            result.messages.push(format!("The {} shudders.", mon_name));
        }

        TrapType::Polymorph => {
            result.messages.push(format!("The {} undergoes a transformation!", mon_name));
            // Polymorph handled by caller
        }

        TrapType::MagicPortal | TrapType::Statue => {
            // Special handling by caller
            result.avoided = true;
        }
    }

    result
}

/// Trap perception: mark trap as seen if player has line of sight
pub fn seetrap(state: &mut GameState, x: i8, y: i8) {
    if state.player.blinded_timeout > 0 {
        return;
    }
    if let Some(trap) = state.current_level.trap_at_mut(x, y)
        && !trap.seen
    {
        trap.seen = true;
        let name = trap_name(trap.trap_type);
        state.message(format!("You see a {} here.", name));
    }
}

/// Trap perception when blind: feel the trap
pub fn feeltrap(state: &mut GameState, x: i8, y: i8) {
    if let Some(trap) = state.current_level.trap_at_mut(x, y)
        && !trap.seen
    {
        trap.seen = true;
        state.message("You feel a trap here.");
    }
}

/// Float up: escape from holding traps when gaining levitation/flying
pub fn float_up(state: &mut GameState) {
    if state.player.utrap > 0
        && let Some(trap_type) = state.player.utraptype
        && is_holding_trap(trap_type)
    {
        state.message("You float up, out of the trap.");
        state.player.utrap = 0;
        state.player.utraptype = None;
    }
    state.message("You start to float in the air!");
}

/// Float down: check current tile for trap, pool, lava when losing levitation
pub fn float_down(state: &mut GameState) {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    state.message("You float gently to the ground.");

    // Check for trap at current position
    if let Some(trap_type) = state.current_level.trap_at(px, py).map(|t| t.trap_type) {
        let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
        let resistances = trap::resistances_from_properties(
            |prop| state.player.properties.has(prop),
            dex,
        );
        if let Some(trap) = state.current_level.trap_at_mut(px, py) {
            let result = trap::dotrap(
                &mut state.rng,
                trap,
                &resistances,
                false,
            );

            for msg in &result.messages {
                state.message(msg.clone());
            }

            if result.damage > 0 {
                state.player.take_damage(result.damage);
            }

            if result.held_turns > 0 {
                state.player.utrap = result.held_turns;
                state.player.utraptype = Some(trap_type);
            }

            if result.trap_destroyed {
                state.current_level.remove_trap(px, py);
            }
        }
    }
}

/// Disarm a holding trap (bear trap -> iron chain)
pub fn disarm_holdingtrap(state: &mut GameState, x: i8, y: i8) -> bool {
    let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
    let trap_type = match state.current_level.trap_at(x, y) {
        Some(t) => t.trap_type,
        None => return false,
    };

    let difficulty = trap::disarm_difficulty(trap_type);
    let chance = 50 + (dex as i32 - 10) * 3 - difficulty;
    let roll = state.rng.rn2(100) as i32;

    if roll < chance.clamp(5, 95) {
        state.message("You disarm the trap.");
        state.current_level.remove_trap(x, y);
        true
    } else {
        state.message("You fail to disarm the trap.");
        false
    }
}

/// Disarm a shooting trap (arrow/dart -> get projectiles)
pub fn disarm_shooting_trap(state: &mut GameState, x: i8, y: i8) -> bool {
    let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
    let trap_type = match state.current_level.trap_at(x, y) {
        Some(t) => t.trap_type,
        None => return false,
    };

    let difficulty = trap::disarm_difficulty(trap_type);
    let chance = 50 + (dex as i32 - 10) * 3 - difficulty;
    let roll = state.rng.rn2(100) as i32;

    if roll < chance.clamp(5, 95) {
        let name = trap_name(trap_type);
        state.message(format!("You disarm the {}.", name));
        state.current_level.remove_trap(x, y);
        // Create projectile objects on the ground
        let quantity = (state.rng.rnd(5) + 1) as i32;
        let proj_name = if trap_type == TrapType::Arrow { "arrow" } else { "dart" };
        let mut proj = crate::object::Object::new(
            crate::object::ObjectId(state.rng.rn2(10000)),
            0,
            crate::object::ObjectClass::Weapon,
        );
        proj.name = Some(proj_name.to_string());
        proj.quantity = quantity;
        state.current_level.add_object(proj, x, y);
        state.message(format!("You find {} {}s.", quantity, proj_name));
        true
    } else {
        state.message("You fail to disarm the trap.");
        // Failing to disarm might trigger it
        if state.rng.one_in(3) {
            state.message("Oops! You triggered it!");
        }
        false
    }
}

/// Disarm a squeaky board (always succeeds)
pub fn disarm_squeaky_board(state: &mut GameState, x: i8, y: i8) -> bool {
    if let Some(trap) = state.current_level.trap_at(x, y)
        && trap.trap_type == TrapType::Squeaky
    {
        state.message("You silence the squeaky board.");
        state.current_level.remove_trap(x, y);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::{Monster, MonsterId, MonsterResistances};
    use crate::rng::GameRng;

    fn make_monster(name: &str) -> Monster {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.name = name.to_string();
        m
    }

    #[test]
    fn test_mintrap_arrow() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("goblin");
        let result = mintrap(&mut rng, &monster, TrapType::Arrow);
        // Either hits or misses
        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_mintrap_bear_trap_holds() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("gnome");
        let result = mintrap(&mut rng, &monster, TrapType::BearTrap);
        assert!(result.held_turns > 0);
        assert!(result.damage > 0);
    }

    #[test]
    fn test_mintrap_fire_resistant() {
        let mut rng = GameRng::from_entropy();
        let mut monster = make_monster("fire elemental");
        monster.resistances = MonsterResistances::FIRE;
        let result = mintrap(&mut rng, &monster, TrapType::FireTrap);
        assert!(result.avoided);
        assert_eq!(result.damage, 0);
    }

    #[test]
    fn test_mintrap_sleeping_gas_resistant() {
        let mut rng = GameRng::from_entropy();
        let mut monster = make_monster("elf");
        monster.resistances = MonsterResistances::SLEEP;
        let result = mintrap(&mut rng, &monster, TrapType::SleepingGas);
        assert!(result.avoided);
    }

    #[test]
    fn test_mintrap_land_mine_destroys() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("kobold");
        let result = mintrap(&mut rng, &monster, TrapType::LandMine);
        assert!(result.trap_destroyed);
        assert!(result.damage > 0);
    }

    #[test]
    fn test_mintrap_teleport() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("orc");
        let result = mintrap(&mut rng, &monster, TrapType::Teleport);
        assert!(result.teleport.is_some());
    }

    #[test]
    fn test_mintrap_pit_holds() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("dwarf");
        let result = mintrap(&mut rng, &monster, TrapType::Pit);
        assert!(result.held_turns > 0);
        assert!(result.damage > 0);
    }
}
