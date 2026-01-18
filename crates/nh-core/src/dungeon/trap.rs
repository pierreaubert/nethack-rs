//! Trap system (trap.c)
//!
//! Functions for trap creation, triggering, and effects.

use crate::dungeon::level::{Trap, TrapType};
use crate::player::Property;
use crate::rng::GameRng;

/// Trap effect result
#[derive(Debug, Clone)]
pub enum TrapEffect {
    /// No effect (trap missed or resisted)
    None,
    /// Damage dealt
    Damage(i32),
    /// Status effect applied
    Status(StatusEffect),
    /// Teleported to new location
    Teleport { x: i8, y: i8 },
    /// Level teleport
    LevelTeleport { up: bool },
    /// Fell into pit/hole
    Fall { depth: i32, damage: i32 },
    /// Trapped (bear trap, web, etc.)
    Trapped { turns: i32 },
    /// Item affected (rust, etc.)
    ItemDamage,
    /// Polymorph
    Polymorph,
    /// Magic effect (random)
    MagicEffect,
}

/// Status effects from traps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffect {
    Poisoned,
    Asleep,
    Confused,
    Blind,
    Stunned,
    Paralyzed,
}

/// Trap creation context
pub struct TrapContext {
    /// Difficulty level for trap generation
    pub difficulty: i32,
    /// Whether traps can be deadly (holes, trapdoors)
    pub allow_holes: bool,
    /// Whether magic traps are allowed
    pub allow_magic: bool,
}

impl Default for TrapContext {
    fn default() -> Self {
        Self {
            difficulty: 1,
            allow_holes: true,
            allow_magic: true,
        }
    }
}

/// Create a random trap type based on difficulty
pub fn random_trap_type(rng: &mut GameRng, ctx: &TrapContext) -> TrapType {
    let difficulty = ctx.difficulty;

    // Weight different trap types based on difficulty
    let choices: Vec<(TrapType, i32)> = vec![
        (TrapType::Arrow, 10),
        (TrapType::Dart, 10),
        (TrapType::RockFall, 8),
        (TrapType::Squeaky, 5),
        (TrapType::BearTrap, 8),
        (TrapType::Pit, 10),
        (TrapType::SpikedPit, if difficulty > 3 { 8 } else { 2 }),
        (TrapType::SleepingGas, if difficulty > 2 { 6 } else { 2 }),
        (TrapType::RustTrap, 5),
        (TrapType::FireTrap, if difficulty > 4 { 6 } else { 2 }),
        (TrapType::LandMine, if difficulty > 5 { 5 } else { 0 }),
        (TrapType::RollingBoulder, if difficulty > 4 { 4 } else { 0 }),
        (TrapType::Hole, if ctx.allow_holes && difficulty > 3 { 4 } else { 0 }),
        (TrapType::TrapDoor, if ctx.allow_holes && difficulty > 4 { 4 } else { 0 }),
        (TrapType::Teleport, if ctx.allow_magic && difficulty > 2 { 5 } else { 0 }),
        (TrapType::LevelTeleport, if ctx.allow_magic && difficulty > 6 { 3 } else { 0 }),
        (TrapType::Web, 6),
        (TrapType::MagicTrap, if ctx.allow_magic && difficulty > 3 { 4 } else { 0 }),
        (TrapType::AntiMagic, if ctx.allow_magic && difficulty > 5 { 3 } else { 0 }),
        (TrapType::Polymorph, if ctx.allow_magic && difficulty > 7 { 2 } else { 0 }),
    ];

    let total_weight: i32 = choices.iter().map(|(_, w)| w).sum();
    if total_weight == 0 {
        return TrapType::Pit; // Fallback
    }

    let mut roll = rng.rn2(total_weight as u32) as i32;
    for (trap_type, weight) in choices {
        if roll < weight {
            return trap_type;
        }
        roll -= weight;
    }

    TrapType::Pit // Fallback
}

/// Create a new trap at the given location
pub fn create_trap(x: i8, y: i8, trap_type: TrapType) -> Trap {
    Trap {
        x,
        y,
        trap_type,
        activated: false,
        seen: false,
    }
}

/// Create a random trap at the given location
pub fn create_random_trap(rng: &mut GameRng, x: i8, y: i8, ctx: &TrapContext) -> Trap {
    let trap_type = random_trap_type(rng, ctx);
    create_trap(x, y, trap_type)
}

/// Get base damage for a trap type
pub fn trap_base_damage(trap_type: TrapType) -> (i32, i32) {
    match trap_type {
        TrapType::Arrow => (1, 6),      // 1d6
        TrapType::Dart => (1, 4),       // 1d4 + poison
        TrapType::RockFall => (2, 6),   // 2d6
        TrapType::BearTrap => (2, 4),   // 2d4
        TrapType::LandMine => (4, 6),   // 4d6
        TrapType::RollingBoulder => (3, 6), // 3d6
        TrapType::FireTrap => (2, 6),   // 2d6 fire
        TrapType::Pit => (2, 6),        // 2d6 fall
        TrapType::SpikedPit => (3, 6),  // 3d6 fall + spikes
        TrapType::Hole => (1, 1),       // Fall damage varies
        TrapType::TrapDoor => (1, 1),   // Fall damage varies
        _ => (0, 0),                    // No direct damage
    }
}

/// Roll damage for a trap
pub fn roll_trap_damage(rng: &mut GameRng, trap_type: TrapType) -> i32 {
    let (dice, sides) = trap_base_damage(trap_type);
    if dice == 0 || sides == 0 {
        return 0;
    }

    let mut total = 0;
    for _ in 0..dice {
        total += rng.rnd(sides as u32) as i32;
    }
    total
}

/// Check if a trap type can hold a creature
pub fn is_holding_trap(trap_type: TrapType) -> bool {
    matches!(
        trap_type,
        TrapType::BearTrap | TrapType::Pit | TrapType::SpikedPit | TrapType::Web
    )
}

/// Check if a trap type is a pit
pub fn is_pit(trap_type: TrapType) -> bool {
    matches!(trap_type, TrapType::Pit | TrapType::SpikedPit)
}

/// Check if a trap type is a hole (leads to lower level)
pub fn is_hole(trap_type: TrapType) -> bool {
    matches!(trap_type, TrapType::Hole | TrapType::TrapDoor)
}

/// Check if a trap type is magical
pub fn is_magic_trap(trap_type: TrapType) -> bool {
    matches!(
        trap_type,
        TrapType::Teleport
            | TrapType::LevelTeleport
            | TrapType::MagicPortal
            | TrapType::MagicTrap
            | TrapType::AntiMagic
            | TrapType::Polymorph
    )
}

/// Check if a trap can be disarmed
pub fn can_disarm(trap_type: TrapType) -> bool {
    !matches!(
        trap_type,
        TrapType::MagicPortal | TrapType::Statue | TrapType::Hole | TrapType::TrapDoor
    )
}

/// Get the difficulty to disarm a trap (higher = harder)
pub fn disarm_difficulty(trap_type: TrapType) -> i32 {
    match trap_type {
        TrapType::Squeaky => 5,
        TrapType::Arrow | TrapType::Dart => 10,
        TrapType::BearTrap => 15,
        TrapType::Pit | TrapType::SpikedPit => 10,
        TrapType::Web => 5,
        TrapType::RockFall => 20,
        TrapType::LandMine => 25,
        TrapType::RollingBoulder => 20,
        TrapType::SleepingGas => 15,
        TrapType::RustTrap => 15,
        TrapType::FireTrap => 20,
        TrapType::Teleport => 20,
        TrapType::LevelTeleport => 25,
        TrapType::MagicTrap => 25,
        TrapType::AntiMagic => 20,
        TrapType::Polymorph => 25,
        _ => 30, // Very hard or impossible
    }
}

/// Get the display character for a trap
pub fn trap_glyph(trap_type: TrapType, seen: bool) -> char {
    if !seen {
        return '.'; // Hidden trap looks like floor
    }
    match trap_type {
        TrapType::Arrow => '^',
        TrapType::Dart => '^',
        TrapType::RockFall => '^',
        TrapType::Squeaky => '^',
        TrapType::BearTrap => '^',
        TrapType::LandMine => '^',
        TrapType::RollingBoulder => '^',
        TrapType::SleepingGas => '^',
        TrapType::RustTrap => '^',
        TrapType::FireTrap => '^',
        TrapType::Pit => '^',
        TrapType::SpikedPit => '^',
        TrapType::Hole => '^',
        TrapType::TrapDoor => '^',
        TrapType::Teleport => '^',
        TrapType::LevelTeleport => '^',
        TrapType::MagicPortal => '\\',
        TrapType::Web => '"',
        TrapType::Statue => '`',
        TrapType::MagicTrap => '^',
        TrapType::AntiMagic => '^',
        TrapType::Polymorph => '^',
    }
}

/// Get the name of a trap type
pub fn trap_name(trap_type: TrapType) -> &'static str {
    match trap_type {
        TrapType::Arrow => "arrow trap",
        TrapType::Dart => "dart trap",
        TrapType::RockFall => "falling rock trap",
        TrapType::Squeaky => "squeaky board",
        TrapType::BearTrap => "bear trap",
        TrapType::LandMine => "land mine",
        TrapType::RollingBoulder => "rolling boulder trap",
        TrapType::SleepingGas => "sleeping gas trap",
        TrapType::RustTrap => "rust trap",
        TrapType::FireTrap => "fire trap",
        TrapType::Pit => "pit",
        TrapType::SpikedPit => "spiked pit",
        TrapType::Hole => "hole",
        TrapType::TrapDoor => "trap door",
        TrapType::Teleport => "teleportation trap",
        TrapType::LevelTeleport => "level teleporter",
        TrapType::MagicPortal => "magic portal",
        TrapType::Web => "web",
        TrapType::Statue => "statue trap",
        TrapType::MagicTrap => "magic trap",
        TrapType::AntiMagic => "anti-magic field",
        TrapType::Polymorph => "polymorph trap",
    }
}

/// Trigger a trap and return its effect
pub fn trigger_trap(rng: &mut GameRng, trap: &mut Trap) -> TrapEffect {
    trap.activated = true;
    trap.seen = true;

    match trap.trap_type {
        TrapType::Arrow | TrapType::Dart => {
            let damage = roll_trap_damage(rng, trap.trap_type);
            if trap.trap_type == TrapType::Dart && rng.one_in(2) {
                TrapEffect::Status(StatusEffect::Poisoned)
            } else {
                TrapEffect::Damage(damage)
            }
        }
        TrapType::RockFall | TrapType::RollingBoulder => {
            let damage = roll_trap_damage(rng, trap.trap_type);
            TrapEffect::Damage(damage)
        }
        TrapType::Squeaky => {
            // Makes noise, alerts monsters
            TrapEffect::None
        }
        TrapType::BearTrap => {
            let damage = roll_trap_damage(rng, trap.trap_type);
            if damage > 0 {
                TrapEffect::Trapped { turns: (rng.rnd(5) + 3) as i32 }
            } else {
                TrapEffect::Damage(damage)
            }
        }
        TrapType::LandMine => {
            let damage = roll_trap_damage(rng, trap.trap_type);
            TrapEffect::Damage(damage)
        }
        TrapType::SleepingGas => TrapEffect::Status(StatusEffect::Asleep),
        TrapType::RustTrap => TrapEffect::ItemDamage,
        TrapType::FireTrap => {
            let damage = roll_trap_damage(rng, trap.trap_type);
            TrapEffect::Damage(damage)
        }
        TrapType::Pit | TrapType::SpikedPit => {
            let damage = roll_trap_damage(rng, trap.trap_type);
            TrapEffect::Fall {
                depth: 1,
                damage,
            }
        }
        TrapType::Hole | TrapType::TrapDoor => {
            TrapEffect::Fall {
                depth: 1,
                damage: rng.rnd(5) as i32 + 1,
            }
        }
        TrapType::Teleport => {
            // Random location on same level
            TrapEffect::Teleport {
                x: (rng.rn2(77) + 1) as i8,
                y: (rng.rn2(19) + 1) as i8,
            }
        }
        TrapType::LevelTeleport => {
            TrapEffect::LevelTeleport {
                up: rng.one_in(2),
            }
        }
        TrapType::MagicPortal => {
            // Special handling needed
            TrapEffect::None
        }
        TrapType::Web => TrapEffect::Trapped { turns: (rng.rnd(10) + 5) as i32 },
        TrapType::Statue => {
            // Animate statue
            TrapEffect::None
        }
        TrapType::MagicTrap => TrapEffect::MagicEffect,
        TrapType::AntiMagic => {
            // Drains magic
            TrapEffect::None
        }
        TrapType::Polymorph => TrapEffect::Polymorph,
    }
}

/// Check if player can detect a trap (based on search skill, etc.)
pub fn can_detect_trap(rng: &mut GameRng, search_skill: i32, trap_type: TrapType) -> bool {
    let base_chance = match trap_type {
        TrapType::Squeaky => 50,  // Easy to spot
        TrapType::Web => 60,      // Visible
        TrapType::Pit | TrapType::SpikedPit => 40,
        TrapType::BearTrap => 30,
        TrapType::Arrow | TrapType::Dart => 25,
        TrapType::MagicPortal => 80, // Usually visible
        _ => 20,
    };

    let chance = base_chance + search_skill * 5;
    (rng.rn2(100) as i32) < chance
}

/// Attempt to disarm a trap
pub fn try_disarm(rng: &mut GameRng, trap: &Trap, dex: i32, skill: i32) -> bool {
    if !can_disarm(trap.trap_type) {
        return false;
    }

    let difficulty = disarm_difficulty(trap.trap_type);
    let chance = 50 + (dex - 10) * 3 + skill * 5 - difficulty;
    let roll = rng.rn2(100) as i32;

    roll < chance.max(5).min(95)
}

// ============================================================================
// Full trap triggering effects (dotrap)
// ============================================================================

/// Player properties that affect trap interaction
pub struct TrapResistances {
    /// Player is flying (avoids ground traps)
    pub flying: bool,
    /// Player is levitating (avoids ground traps)
    pub levitating: bool,
    /// Player has fire resistance
    pub fire_resistant: bool,
    /// Player has poison resistance
    pub poison_resistant: bool,
    /// Player has sleep resistance
    pub sleep_resistant: bool,
    /// Player has teleport control
    pub teleport_control: bool,
    /// Player has magic resistance
    pub magic_resistant: bool,
    /// Player is phasing through walls
    pub phasing: bool,
    /// Player's dexterity (affects dodging)
    pub dexterity: i8,
}

impl Default for TrapResistances {
    fn default() -> Self {
        Self {
            flying: false,
            levitating: false,
            fire_resistant: false,
            poison_resistant: false,
            sleep_resistant: false,
            teleport_control: false,
            magic_resistant: false,
            phasing: false,
            dexterity: 10,
        }
    }
}

/// Result of stepping into a trap
#[derive(Debug, Clone)]
pub struct DotrapResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Damage dealt (if any)
    pub damage: i32,
    /// Status effect to apply (if any)
    pub status: Option<StatusEffect>,
    /// Turns player is held in trap (0 = not held)
    pub held_turns: i32,
    /// Teleport destination (if teleported)
    pub teleport: Option<(i8, i8)>,
    /// Whether player fell to a lower level
    pub fell_through: bool,
    /// Whether trap was identified
    pub identified: bool,
    /// Whether trap should be removed after triggering
    pub trap_destroyed: bool,
}

impl Default for DotrapResult {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            damage: 0,
            status: None,
            held_turns: 0,
            teleport: None,
            fell_through: false,
            identified: true, // Most traps are identified when triggered
            trap_destroyed: false,
        }
    }
}

/// Process a trap being triggered by the player.
///
/// This is the main function for handling trap effects, equivalent to
/// NetHack's dotrap() function. It checks resistances, calculates damage,
/// applies effects, and generates appropriate messages.
///
/// # Arguments
/// * `rng` - Random number generator
/// * `trap` - The trap being triggered
/// * `resistances` - Player's resistances and properties
/// * `already_trapped` - Whether player is already in a trap (e.g., pit)
///
/// # Returns
/// A DotrapResult describing what happened
pub fn dotrap(
    rng: &mut GameRng,
    trap: &mut Trap,
    resistances: &TrapResistances,
    already_trapped: bool,
) -> DotrapResult {
    let mut result = DotrapResult::default();
    let trap_name = trap_name(trap.trap_type);

    // Mark trap as seen and activated
    trap.seen = true;
    trap.activated = true;

    // Check for flying/levitation avoiding ground traps
    if (resistances.flying || resistances.levitating)
        && is_ground_trap(trap.trap_type)
        && !already_trapped
    {
        result.messages.push(format!(
            "You {} over {}.",
            if resistances.flying { "fly" } else { "float" },
            trap_name
        ));
        return result;
    }

    // Process trap by type
    match trap.trap_type {
        TrapType::Arrow => {
            result.messages.push("An arrow shoots out at you!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);

            // Dexterity check to dodge
            if rng.rn2(20) < resistances.dexterity as u32 / 3 {
                result.messages.push("You dodge the arrow.".to_string());
            } else {
                result.messages.push("It hits you!".to_string());
                result.damage = damage;
            }
        }

        TrapType::Dart => {
            result.messages.push("A little dart shoots out at you!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);

            if rng.rn2(20) < resistances.dexterity as u32 / 3 {
                result.messages.push("You dodge the dart.".to_string());
            } else {
                result.messages.push("It hits you!".to_string());
                result.damage = damage;

                // Poison check
                if rng.one_in(3) {
                    if resistances.poison_resistant {
                        result.messages.push("The dart was poisoned, but you resist!".to_string());
                    } else {
                        result.messages.push("The dart was poisoned!".to_string());
                        result.status = Some(StatusEffect::Poisoned);
                    }
                }
            }
        }

        TrapType::RockFall => {
            result.messages.push("A rock falls on your head!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);
            result.damage = damage;
            result.messages.push(format!("WHANG! You suffer {} damage.", damage));
        }

        TrapType::Squeaky => {
            result.messages.push("A board beneath you squeaks loudly.".to_string());
            // Wake up nearby monsters (handled by caller)
            result.identified = true;
        }

        TrapType::BearTrap => {
            if resistances.levitating {
                result.messages.push("You float over a bear trap.".to_string());
            } else {
                result.messages.push("SNAP! A bear trap closes on your foot!".to_string());
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result.held_turns = (rng.rnd(5) + 3) as i32;
                result.messages.push(format!("You are caught! (for {} turns)", result.held_turns));
            }
        }

        TrapType::LandMine => {
            result.messages.push("KAABLAMM!!! You step on a land mine!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);
            result.damage = damage;
            result.trap_destroyed = true;
            result.messages.push(format!("The explosion deals {} damage!", damage));
            // Could also scatter inventory, stun, etc.
        }

        TrapType::RollingBoulder => {
            result.messages.push("Click! You trigger a rolling boulder trap!".to_string());
            if rng.one_in(4) {
                result.messages.push("Fortunately, the boulder misses you.".to_string());
            } else {
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result.messages.push(format!("You are hit by a boulder for {} damage!", damage));
            }
        }

        TrapType::SleepingGas => {
            result.messages.push("A cloud of gas billows up around you!".to_string());
            if resistances.sleep_resistant {
                result.messages.push("You don't feel sleepy.".to_string());
            } else {
                result.messages.push("You fall asleep!".to_string());
                result.status = Some(StatusEffect::Asleep);
                result.held_turns = (rng.rnd(25) + 10) as i32;
            }
        }

        TrapType::RustTrap => {
            result.messages.push("A gush of water hits you!".to_string());
            // Would rust iron armor, handled by caller
            result.messages.push("Your equipment may be affected!".to_string());
            result.status = Some(StatusEffect::Stunned); // Proxy for item damage
        }

        TrapType::FireTrap => {
            result.messages.push("A tower of flame erupts around you!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);

            if resistances.fire_resistant {
                result.messages.push("But you resist the fire!".to_string());
                result.damage = damage / 2;
            } else {
                result.damage = damage;
                result.messages.push(format!("You are scorched for {} damage!", damage));
            }
        }

        TrapType::Pit => {
            if already_trapped {
                result.messages.push("You are still in the pit.".to_string());
            } else {
                result.messages.push("You fall into a pit!".to_string());
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result.held_turns = (rng.rnd(6) + 2) as i32;
            }
        }

        TrapType::SpikedPit => {
            if already_trapped {
                result.messages.push("You are still in the spiked pit.".to_string());
            } else {
                result.messages.push("You fall into a pit of spikes!".to_string());
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result.held_turns = (rng.rnd(8) + 3) as i32;

                if !resistances.poison_resistant && rng.one_in(6) {
                    result.messages.push("Some of the spikes were poisoned!".to_string());
                    result.status = Some(StatusEffect::Poisoned);
                }
            }
        }

        TrapType::Hole | TrapType::TrapDoor => {
            let name = if trap.trap_type == TrapType::Hole {
                "hole"
            } else {
                "trap door"
            };

            if resistances.flying || resistances.levitating {
                result.messages.push(format!("You {} over a {}.",
                    if resistances.flying { "fly" } else { "float" },
                    name
                ));
            } else {
                result.messages.push(format!("You fall through a {}!", name));
                result.fell_through = true;
                result.damage = (rng.rnd(6) + 1) as i32;
            }
        }

        TrapType::Teleport => {
            result.messages.push("You are suddenly teleported!".to_string());

            if resistances.teleport_control {
                result.messages.push("You have control over where you land.".to_string());
                // Caller handles destination selection
            }

            // Generate random destination
            let x = (rng.rn2(77) + 1) as i8;
            let y = (rng.rn2(19) + 1) as i8;
            result.teleport = Some((x, y));
        }

        TrapType::LevelTeleport => {
            if resistances.magic_resistant {
                result.messages.push("You shudder for a moment, but resist the magic.".to_string());
            } else {
                result.messages.push("You are teleported to another level!".to_string());
                result.fell_through = true; // Use this to indicate level change

                if resistances.teleport_control {
                    result.messages.push("You have some control over where you land.".to_string());
                }
            }
        }

        TrapType::MagicPortal => {
            result.messages.push("You feel a strange sensation...".to_string());
            result.fell_through = true;
            // Portal destination is fixed, handled by caller
        }

        TrapType::Web => {
            if resistances.phasing {
                result.messages.push("You pass right through the web.".to_string());
            } else {
                result.messages.push("You stumble into a spider web!".to_string());
                result.held_turns = (rng.rnd(10) + 5) as i32;
                result.messages.push(format!("You are stuck! (for {} turns)", result.held_turns));
            }
        }

        TrapType::Statue => {
            result.messages.push("The statue comes to life!".to_string());
            // Monster creation handled by caller
        }

        TrapType::MagicTrap => {
            result.messages.push("You are enveloped in a magical light!".to_string());

            // Random magic effect
            let effect = rng.rn2(10);
            match effect {
                0..=2 => {
                    result.messages.push("You feel disoriented.".to_string());
                    result.status = Some(StatusEffect::Confused);
                }
                3..=4 => {
                    result.messages.push("A tower of flame erupts!".to_string());
                    if !resistances.fire_resistant {
                        result.damage = rng.dice(4, 4) as i32;
                    }
                }
                5..=6 => {
                    result.messages.push("You feel drained.".to_string());
                    // Energy drain handled by caller
                }
                7 => {
                    result.messages.push("You feel lucky!".to_string());
                    // Luck increase handled by caller
                }
                8 => {
                    result.messages.push("You feel unlucky.".to_string());
                    // Luck decrease handled by caller
                }
                _ => {
                    result.messages.push("Nothing happens.".to_string());
                }
            }
        }

        TrapType::AntiMagic => {
            result.messages.push("You feel your magical energy draining away!".to_string());
            // Energy drain handled by caller
            result.identified = true;
        }

        TrapType::Polymorph => {
            if resistances.magic_resistant {
                result.messages.push("You feel momentarily different, but resist.".to_string());
            } else {
                result.messages.push("You feel a change coming over you...".to_string());
                // Polymorph handled by caller
            }
        }
    }

    result
}

/// Check if a trap type is a ground trap (affected by flying/levitation)
pub fn is_ground_trap(trap_type: TrapType) -> bool {
    matches!(
        trap_type,
        TrapType::BearTrap
            | TrapType::Pit
            | TrapType::SpikedPit
            | TrapType::Hole
            | TrapType::TrapDoor
            | TrapType::Web
            | TrapType::Squeaky
    )
}

/// Build TrapResistances from player properties.
///
/// Helper function to create resistance info from the player's property set.
pub fn resistances_from_properties<F>(
    has_property: F,
    dexterity: i8,
) -> TrapResistances
where
    F: Fn(Property) -> bool,
{
    TrapResistances {
        flying: has_property(Property::Flying),
        levitating: has_property(Property::Levitation),
        fire_resistant: has_property(Property::FireResistance),
        poison_resistant: has_property(Property::PoisonResistance),
        sleep_resistant: has_property(Property::SleepResistance),
        teleport_control: has_property(Property::TeleportControl),
        magic_resistant: has_property(Property::MagicResistance),
        phasing: false, // Phasing (passes through walls) not yet implemented
        dexterity,
    }
}

/// Get escape message when player gets out of a holding trap
pub fn escape_trap_message(trap_type: TrapType) -> &'static str {
    match trap_type {
        TrapType::BearTrap => "You pull free of the bear trap.",
        TrapType::Pit => "You climb out of the pit.",
        TrapType::SpikedPit => "You climb out of the spiked pit.",
        TrapType::Web => "You tear through the web.",
        _ => "You escape from the trap.",
    }
}

/// Check if player can escape from a holding trap this turn
pub fn try_escape_trap(rng: &mut GameRng, trap_type: TrapType, strength: i8) -> bool {
    let base_chance = match trap_type {
        TrapType::BearTrap => 25,
        TrapType::Pit | TrapType::SpikedPit => 40,
        TrapType::Web => 35,
        _ => 50,
    };

    // Strength bonus
    let chance = base_chance + (strength as i32 - 10) * 3;
    (rng.rn2(100) as i32) < chance.clamp(5, 95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_trap() {
        let trap = create_trap(5, 10, TrapType::BearTrap);
        assert_eq!(trap.x, 5);
        assert_eq!(trap.y, 10);
        assert_eq!(trap.trap_type, TrapType::BearTrap);
        assert!(!trap.activated);
        assert!(!trap.seen);
    }

    #[test]
    fn test_random_trap_type() {
        let mut rng = GameRng::from_entropy();
        let ctx = TrapContext::default();

        // Generate several traps and ensure variety
        let mut types: Vec<TrapType> = Vec::new();
        for _ in 0..100 {
            let t = random_trap_type(&mut rng, &ctx);
            if !types.contains(&t) {
                types.push(t);
            }
        }
        assert!(types.len() > 5, "Should generate variety of trap types");
    }

    #[test]
    fn test_is_pit() {
        assert!(is_pit(TrapType::Pit));
        assert!(is_pit(TrapType::SpikedPit));
        assert!(!is_pit(TrapType::BearTrap));
    }

    #[test]
    fn test_is_hole() {
        assert!(is_hole(TrapType::Hole));
        assert!(is_hole(TrapType::TrapDoor));
        assert!(!is_hole(TrapType::Pit));
    }

    #[test]
    fn test_is_holding_trap() {
        assert!(is_holding_trap(TrapType::BearTrap));
        assert!(is_holding_trap(TrapType::Web));
        assert!(is_holding_trap(TrapType::Pit));
        assert!(!is_holding_trap(TrapType::Arrow));
    }

    #[test]
    fn test_trap_damage() {
        let mut rng = GameRng::from_entropy();

        // Arrow trap should do 1-6 damage
        let damage = roll_trap_damage(&mut rng, TrapType::Arrow);
        assert!(damage >= 1 && damage <= 6);

        // Land mine should do more damage
        let damage = roll_trap_damage(&mut rng, TrapType::LandMine);
        assert!(damage >= 4 && damage <= 24);
    }

    #[test]
    fn test_trigger_trap() {
        let mut rng = GameRng::from_entropy();
        let mut trap = create_trap(5, 5, TrapType::BearTrap);

        let effect = trigger_trap(&mut rng, &mut trap);
        assert!(trap.activated);
        assert!(trap.seen);
        assert!(matches!(effect, TrapEffect::Trapped { .. } | TrapEffect::Damage(_)));
    }

    #[test]
    fn test_trap_name() {
        assert_eq!(trap_name(TrapType::BearTrap), "bear trap");
        assert_eq!(trap_name(TrapType::Pit), "pit");
        assert_eq!(trap_name(TrapType::Teleport), "teleportation trap");
    }

    #[test]
    fn test_can_disarm() {
        assert!(can_disarm(TrapType::BearTrap));
        assert!(can_disarm(TrapType::Arrow));
        assert!(!can_disarm(TrapType::MagicPortal));
        assert!(!can_disarm(TrapType::Hole));
    }

    #[test]
    fn test_disarm_difficulty() {
        // Squeaky board should be easy
        assert!(disarm_difficulty(TrapType::Squeaky) < disarm_difficulty(TrapType::LandMine));
        // Land mine should be hard
        assert!(disarm_difficulty(TrapType::LandMine) > 20);
    }
}
