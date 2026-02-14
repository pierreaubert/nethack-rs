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
    Rusted,
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
        (
            TrapType::Hole,
            if ctx.allow_holes && difficulty > 3 {
                4
            } else {
                0
            },
        ),
        (
            TrapType::TrapDoor,
            if ctx.allow_holes && difficulty > 4 {
                4
            } else {
                0
            },
        ),
        (
            TrapType::Teleport,
            if ctx.allow_magic && difficulty > 2 {
                5
            } else {
                0
            },
        ),
        (
            TrapType::LevelTeleport,
            if ctx.allow_magic && difficulty > 6 {
                3
            } else {
                0
            },
        ),
        (TrapType::Web, 6),
        (
            TrapType::MagicTrap,
            if ctx.allow_magic && difficulty > 3 {
                4
            } else {
                0
            },
        ),
        (
            TrapType::AntiMagic,
            if ctx.allow_magic && difficulty > 5 {
                3
            } else {
                0
            },
        ),
        (
            TrapType::Polymorph,
            if ctx.allow_magic && difficulty > 7 {
                2
            } else {
                0
            },
        ),
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
    let once = matches!(trap_type, TrapType::LandMine);
    Trap {
        x,
        y,
        trap_type,
        activated: false,
        seen: false,
        once,
        madeby_u: false,
        launch_oid: None,
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
        TrapType::Arrow => (1, 6),          // 1d6
        TrapType::Dart => (1, 4),           // 1d4 + poison
        TrapType::RockFall => (2, 6),       // 2d6
        TrapType::BearTrap => (2, 4),       // 2d4
        TrapType::LandMine => (4, 6),       // 4d6
        TrapType::RollingBoulder => (3, 6), // 3d6
        TrapType::FireTrap => (2, 6),       // 2d6 fire
        TrapType::Pit => (2, 6),            // 2d6 fall
        TrapType::SpikedPit => (3, 6),      // 3d6 fall + spikes
        TrapType::Hole => (1, 1),           // Fall damage varies
        TrapType::TrapDoor => (1, 1),       // Fall damage varies
        _ => (0, 0),                        // No direct damage
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
                TrapEffect::Trapped {
                    turns: (rng.rnd(5) + 3) as i32,
                }
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
            TrapEffect::Fall { depth: 1, damage }
        }
        TrapType::Hole | TrapType::TrapDoor => TrapEffect::Fall {
            depth: 1,
            damage: rng.rnd(5) as i32 + 1,
        },
        TrapType::Teleport => {
            // Random location on same level
            TrapEffect::Teleport {
                x: (rng.rn2(77) + 1) as i8,
                y: (rng.rn2(19) + 1) as i8,
            }
        }
        TrapType::LevelTeleport => TrapEffect::LevelTeleport { up: rng.one_in(2) },
        TrapType::MagicPortal => {
            // Special handling needed
            TrapEffect::None
        }
        TrapType::Web => TrapEffect::Trapped {
            turns: (rng.rnd(10) + 5) as i32,
        },
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
        TrapType::Squeaky => 50, // Easy to spot
        TrapType::Web => 60,     // Visible
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
            result
                .messages
                .push("An arrow shoots out at you!".to_string());
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
            result
                .messages
                .push("A little dart shoots out at you!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);

            if rng.rn2(20) < resistances.dexterity as u32 / 3 {
                result.messages.push("You dodge the dart.".to_string());
            } else {
                result.messages.push("It hits you!".to_string());
                result.damage = damage;

                // Poison check
                if rng.one_in(3) {
                    if resistances.poison_resistant {
                        result
                            .messages
                            .push("The dart was poisoned, but you resist!".to_string());
                    } else {
                        result.messages.push("The dart was poisoned!".to_string());
                        result.status = Some(StatusEffect::Poisoned);
                    }
                }
            }
        }

        TrapType::RockFall => {
            result
                .messages
                .push("A rock falls on your head!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);
            result.damage = damage;
            result
                .messages
                .push(format!("WHANG! You suffer {} damage.", damage));
        }

        TrapType::Squeaky => {
            result
                .messages
                .push("A board beneath you squeaks loudly.".to_string());
            // Wake up nearby monsters (handled by caller)
            result.identified = true;
        }

        TrapType::BearTrap => {
            if resistances.levitating {
                result
                    .messages
                    .push("You float over a bear trap.".to_string());
            } else {
                result
                    .messages
                    .push("SNAP! A bear trap closes on your foot!".to_string());
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result.held_turns = (rng.rnd(5) + 3) as i32;
                result
                    .messages
                    .push(format!("You are caught! (for {} turns)", result.held_turns));
            }
        }

        TrapType::LandMine => {
            result
                .messages
                .push("KAABLAMM!!! You step on a land mine!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);
            result.damage = damage;
            result.trap_destroyed = true;
            result
                .messages
                .push(format!("The explosion deals {} damage!", damage));
            // Could also scatter inventory, stun, etc.
        }

        TrapType::RollingBoulder => {
            result
                .messages
                .push("Click! You trigger a rolling boulder trap!".to_string());
            if rng.one_in(4) {
                result
                    .messages
                    .push("Fortunately, the boulder misses you.".to_string());
            } else {
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result
                    .messages
                    .push(format!("You are hit by a boulder for {} damage!", damage));
            }
        }

        TrapType::SleepingGas => {
            result
                .messages
                .push("A cloud of gas billows up around you!".to_string());
            if resistances.sleep_resistant {
                result.messages.push("You don't feel sleepy.".to_string());
            } else {
                result.messages.push("You fall asleep!".to_string());
                result.status = Some(StatusEffect::Asleep);
                result.held_turns = (rng.rnd(25) + 10) as i32;
            }
        }

        TrapType::RustTrap => {
            result
                .messages
                .push("A gush of water hits you!".to_string());
            // Rust effect: caller should erode iron equipment (erosion1 += 1)
            result.status = Some(StatusEffect::Rusted);
        }

        TrapType::FireTrap => {
            result
                .messages
                .push("A tower of flame erupts around you!".to_string());
            let damage = roll_trap_damage(rng, trap.trap_type);

            if resistances.fire_resistant {
                result.messages.push("But you resist the fire!".to_string());
                result.damage = damage / 2;
            } else {
                result.damage = damage;
                result
                    .messages
                    .push(format!("You are scorched for {} damage!", damage));
            }
        }

        TrapType::Pit => {
            if already_trapped {
                result
                    .messages
                    .push("You are still in the pit.".to_string());
            } else {
                result.messages.push("You fall into a pit!".to_string());
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result.held_turns = (rng.rnd(6) + 2) as i32;
            }
        }

        TrapType::SpikedPit => {
            if already_trapped {
                result
                    .messages
                    .push("You are still in the spiked pit.".to_string());
            } else {
                result
                    .messages
                    .push("You fall into a pit of spikes!".to_string());
                let damage = roll_trap_damage(rng, trap.trap_type);
                result.damage = damage;
                result.held_turns = (rng.rnd(8) + 3) as i32;

                if !resistances.poison_resistant && rng.one_in(6) {
                    result
                        .messages
                        .push("Some of the spikes were poisoned!".to_string());
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
                result.messages.push(format!(
                    "You {} over a {}.",
                    if resistances.flying { "fly" } else { "float" },
                    name
                ));
            } else {
                result
                    .messages
                    .push(format!("You fall through a {}!", name));
                result.fell_through = true;
                result.damage = (rng.rnd(6) + 1) as i32;
            }
        }

        TrapType::Teleport => {
            result
                .messages
                .push("You are suddenly teleported!".to_string());

            if resistances.teleport_control {
                result
                    .messages
                    .push("You have control over where you land.".to_string());
                // Caller handles destination selection
            }

            // Generate random destination
            let x = (rng.rn2(77) + 1) as i8;
            let y = (rng.rn2(19) + 1) as i8;
            result.teleport = Some((x, y));
        }

        TrapType::LevelTeleport => {
            if resistances.magic_resistant {
                result
                    .messages
                    .push("You shudder for a moment, but resist the magic.".to_string());
            } else {
                result
                    .messages
                    .push("You are teleported to another level!".to_string());
                result.fell_through = true; // Use this to indicate level change

                if resistances.teleport_control {
                    result
                        .messages
                        .push("You have some control over where you land.".to_string());
                }
            }
        }

        TrapType::MagicPortal => {
            result
                .messages
                .push("You feel a strange sensation...".to_string());
            result.fell_through = true;
            // Portal destination is fixed, handled by caller
        }

        TrapType::Web => {
            if resistances.phasing {
                result
                    .messages
                    .push("You pass right through the web.".to_string());
            } else {
                result
                    .messages
                    .push("You stumble into a spider web!".to_string());
                result.held_turns = (rng.rnd(10) + 5) as i32;
                result
                    .messages
                    .push(format!("You are stuck! (for {} turns)", result.held_turns));
            }
        }

        TrapType::Statue => {
            result
                .messages
                .push("The statue comes to life!".to_string());
            // Monster creation handled by caller
        }

        TrapType::MagicTrap => {
            result
                .messages
                .push("You are enveloped in a magical light!".to_string());

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
            result
                .messages
                .push("You feel your magical energy draining away!".to_string());
            // Energy drain handled by caller
            result.identified = true;
        }

        TrapType::Polymorph => {
            if resistances.magic_resistant {
                result
                    .messages
                    .push("You feel momentarily different, but resist.".to_string());
            } else {
                result
                    .messages
                    .push("You feel a change coming over you...".to_string());
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
pub fn resistances_from_properties<F>(has_property: F, dexterity: i8) -> TrapResistances
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

// ============================================================================
// Container traps (b_trapped, chest_trap)
// ============================================================================

/// Container trap type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerTrap {
    /// No trap
    None,
    /// Poison needle
    Poison,
    /// Paralysis gas
    Paralysis,
    /// Explosion (destroys contents)
    Explosion,
    /// Summon monsters
    Summon,
    /// Electric shock
    Electric,
}

/// Check if a container (box/chest) is trapped.
///
/// In NetHack, trapped containers have the `otrapped` flag set.
/// This is the `b_trapped` function equivalent.
///
/// # Arguments
/// * `trapped` - Whether the container has the trapped flag set
/// * `rng` - Random number generator (for trap type)
///
/// # Returns
/// The type of trap if trapped, None otherwise
pub fn b_trapped(trapped: bool, rng: &mut GameRng) -> ContainerTrap {
    if !trapped {
        return ContainerTrap::None;
    }

    // Random trap type based on weights
    let roll = rng.rn2(100);
    match roll {
        0..=24 => ContainerTrap::Poison,     // 25%
        25..=44 => ContainerTrap::Paralysis, // 20%
        45..=64 => ContainerTrap::Explosion, // 20%
        65..=84 => ContainerTrap::Summon,    // 20%
        _ => ContainerTrap::Electric,        // 15%
    }
}

/// Result of triggering a container trap
#[derive(Debug, Clone)]
pub struct ContainerTrapResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Damage dealt
    pub damage: i32,
    /// Status effect (poison, paralysis)
    pub status: Option<StatusEffect>,
    /// Whether container contents are destroyed
    pub contents_destroyed: bool,
    /// Whether to summon monsters
    pub summon_monsters: bool,
}

impl Default for ContainerTrapResult {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            damage: 0,
            status: None,
            contents_destroyed: false,
            summon_monsters: false,
        }
    }
}

/// Trigger a container trap (chest_trap equivalent).
///
/// # Arguments
/// * `rng` - Random number generator
/// * `trap_type` - The type of container trap
/// * `resistances` - Player's resistances
///
/// # Returns
/// The result of the trap triggering
pub fn chest_trap(
    rng: &mut GameRng,
    trap_type: ContainerTrap,
    resistances: &TrapResistances,
) -> ContainerTrapResult {
    let mut result = ContainerTrapResult::default();

    match trap_type {
        ContainerTrap::None => {}
        ContainerTrap::Poison => {
            result
                .messages
                .push("A small needle pricks you!".to_string());
            if resistances.poison_resistant {
                result
                    .messages
                    .push("It doesn't seem to affect you.".to_string());
            } else {
                result.messages.push("You feel very sick!".to_string());
                result.status = Some(StatusEffect::Poisoned);
                result.damage = rng.dice(1, 6) as i32;
            }
        }
        ContainerTrap::Paralysis => {
            result.messages.push("A puff of gas escapes!".to_string());
            if resistances.sleep_resistant {
                result.messages.push("You hold your breath.".to_string());
            } else {
                result.messages.push("You are frozen!".to_string());
                result.status = Some(StatusEffect::Paralyzed);
            }
        }
        ContainerTrap::Explosion => {
            result
                .messages
                .push("KABOOM! The container explodes!".to_string());
            result.damage = rng.dice(4, 6) as i32;
            result.contents_destroyed = true;
        }
        ContainerTrap::Summon => {
            result
                .messages
                .push("You trigger a magical trap!".to_string());
            result
                .messages
                .push("Monsters appear around you!".to_string());
            result.summon_monsters = true;
        }
        ContainerTrap::Electric => {
            result
                .messages
                .push("ZAP! You get an electric shock!".to_string());
            // Electric damage - could check for shock resistance
            result.damage = rng.dice(2, 6) as i32;
        }
    }

    result
}

// ============================================================================
// Statue trap activation
// ============================================================================

/// Result of activating a statue trap
#[derive(Debug, Clone)]
pub struct StatueTrapResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// The monster type index that should emerge (if any)
    pub monster_type: Option<i16>,
    /// Whether the statue is destroyed
    pub statue_destroyed: bool,
    /// Whether the trap triggered successfully
    pub triggered: bool,
}

impl Default for StatueTrapResult {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            monster_type: None,
            statue_destroyed: false,
            triggered: false,
        }
    }
}

/// Activate a statue trap, causing a monster to emerge.
///
/// This is the `activate_statue_trap` function equivalent.
/// In NetHack, statue traps contain a monster that emerges when
/// the trap is triggered (usually by approaching or touching the statue).
///
/// # Arguments
/// * `rng` - Random number generator
/// * `statue_monster_type` - The monster type index stored in the statue (from corpsenm)
/// * `historic` - Whether the statue is historic (cannot activate)
///
/// # Returns
/// The result of the trap activation
pub fn activate_statue_trap(
    rng: &mut GameRng,
    statue_monster_type: Option<i16>,
    historic: bool,
) -> StatueTrapResult {
    let mut result = StatueTrapResult::default();

    // Historic statues cannot be activated
    if historic {
        result
            .messages
            .push("The historic statue remains motionless.".to_string());
        return result;
    }

    // Need a valid monster type
    let monster_type = match statue_monster_type {
        Some(mt) if mt >= 0 => mt,
        _ => {
            // No monster stored, trap fizzles
            result
                .messages
                .push("The statue crumbles to dust!".to_string());
            result.statue_destroyed = true;
            return result;
        }
    };

    // Small chance the statue just crumbles
    if rng.one_in(10) {
        result.messages.push("The statue crumbles!".to_string());
        result.statue_destroyed = true;
        return result;
    }

    // Monster emerges!
    result
        .messages
        .push("The statue comes to life!".to_string());
    result.monster_type = Some(monster_type);
    result.statue_destroyed = true;
    result.triggered = true;

    result
}

/// Check if a statue at a location is a trap (has a monster inside)
pub fn is_statue_trap(has_corpsenm: bool, monster_type: Option<i16>) -> bool {
    has_corpsenm && monster_type.map(|t| t >= 0).unwrap_or(false)
}

/// Get the message when a statue trap is revealed
pub fn statue_trap_reveal_msg() -> &'static str {
    "You notice a peculiar cavity inside the statue."
}

// ============================================================================
// Additional trap helpers
// ============================================================================

/// Check if a door is trapped (mb_trapped equivalent for door traps)
pub fn mb_trapped(trapped: bool) -> bool {
    trapped
}

/// Generate a random trap for a container
pub fn rndtrap_container(rng: &mut GameRng) -> ContainerTrap {
    let roll = rng.rn2(5);
    match roll {
        0 => ContainerTrap::Poison,
        1 => ContainerTrap::Paralysis,
        2 => ContainerTrap::Explosion,
        3 => ContainerTrap::Summon,
        _ => ContainerTrap::Electric,
    }
}

/// Check if player can avoid triggering a container trap
pub fn avoid_container_trap(rng: &mut GameRng, dexterity: i8, luck: i8) -> bool {
    // Base 10% chance to avoid, modified by dex and luck
    let chance = 10 + (dexterity as i32 - 10) * 2 + luck as i32;
    (rng.rn2(100) as i32) < chance.clamp(1, 50)
}

/// Select a random trap type (for special levels)
/// Matches C's rndtrap() from sp_lev.c
///
/// This function excludes certain dangerous traps that shouldn't appear randomly in special levels.
/// It keeps retrying until it finds a valid trap type.
///
/// # Arguments
/// * `rng` - Random number generator
/// * `level` - Current level (for context checks)
///
/// # Returns
/// A random TrapType that is valid for placement in the current context
pub fn rndtrap(rng: &mut GameRng, level: &crate::dungeon::level::Level) -> TrapType {
    use crate::dungeon::level::{Level, TrapType as LevelTrapType};

    // Define the pool of valid trap types for random placement
    let trap_pool = vec![
        LevelTrapType::Arrow,
        LevelTrapType::Dart,
        LevelTrapType::RockFall,
        LevelTrapType::Squeaky,
        LevelTrapType::BearTrap,
        LevelTrapType::LandMine,
        LevelTrapType::RollingBoulder,
        LevelTrapType::SleepingGas,
        LevelTrapType::RustTrap,
        LevelTrapType::FireTrap,
        LevelTrapType::Pit,
        LevelTrapType::SpikedPit,
        LevelTrapType::Web,
        LevelTrapType::Teleport,
        LevelTrapType::MagicTrap,
        LevelTrapType::AntiMagic,
        LevelTrapType::Polymorph,
    ];

    loop {
        let idx = rng.rn2(trap_pool.len() as u32) as usize;
        let trap_type = trap_pool[idx];

        // Exclude certain traps based on level conditions
        let valid = match trap_type {
            // Never place holes randomly on special levels (they're planned)
            LevelTrapType::Hole => false,
            // Vibrating square is special (endgame only)
            LevelTrapType::TrapDoor => {
                // Can only place trapdoors if we can dig down
                level.dlevel.dungeon_num != 1 // Not in Gehennom
            }
            // Level teleport disallowed on no-teleport levels
            LevelTrapType::LevelTeleport | LevelTrapType::Teleport => !level.flags.no_teleport,
            // Boulders and rocks not in endgame
            LevelTrapType::RollingBoulder => level.dlevel.dungeon_num != 7,
            // Everything else is allowed
            _ => true,
        };

        if valid {
            return trap_type;
        }
    }
}

/// Count hidden traps surrounding a location
/// Matches C's count_surround_traps() from artifact.c
///
/// This counts traps in a 3x3 area around the given coordinates that are NOT currently
/// visible to the player (hidden traps). Visible traps (already discovered) are skipped.
/// This is used for effects like the Mark of the Thief warning.
///
/// # Arguments
/// * `level` - Current level
/// * `x` - X coordinate (center of search area)
/// * `y` - Y coordinate (center of search area)
///
/// # Returns
/// Number of hidden traps in surrounding 3x3 area
pub fn count_surround_traps(level: &crate::dungeon::level::Level, x: i8, y: i8) -> usize {
    use crate::{COLNO, ROWNO};

    let mut count = 0;

    // Check 3x3 area around position
    for dx in -1..=1 {
        for dy in -1..=1 {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;

            // Skip out-of-bounds
            if nx >= COLNO || ny >= ROWNO {
                continue;
            }

            // Count non-visible traps
            let trap_count = level
                .traps
                .iter()
                .filter(|trap| {
                    trap.x as usize == nx && trap.y as usize == ny && !trap.seen // Only count unseen traps
                })
                .count();

            if trap_count > 0 {
                count += 1; // Count location, not individual traps
            }
        }
    }

    count
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
        assert!(matches!(
            effect,
            TrapEffect::Trapped { .. } | TrapEffect::Damage(_)
        ));
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

    #[test]
    fn test_b_trapped() {
        let mut rng = GameRng::from_entropy();

        // Not trapped should return None
        assert_eq!(b_trapped(false, &mut rng), ContainerTrap::None);

        // Trapped should return some trap type
        let trap = b_trapped(true, &mut rng);
        assert_ne!(trap, ContainerTrap::None);
    }

    #[test]
    fn test_chest_trap() {
        let mut rng = GameRng::from_entropy();
        let resistances = TrapResistances::default();

        // Poison trap
        let result = chest_trap(&mut rng, ContainerTrap::Poison, &resistances);
        assert!(!result.messages.is_empty());

        // Explosion destroys contents
        let result = chest_trap(&mut rng, ContainerTrap::Explosion, &resistances);
        assert!(result.contents_destroyed);
        assert!(result.damage > 0);

        // Summon trap
        let result = chest_trap(&mut rng, ContainerTrap::Summon, &resistances);
        assert!(result.summon_monsters);
    }

    #[test]
    fn test_activate_statue_trap() {
        let mut rng = GameRng::from_entropy();

        // Historic statue doesn't activate
        let result = activate_statue_trap(&mut rng, Some(10), true);
        assert!(!result.triggered);
        assert!(result.monster_type.is_none());

        // No monster type stored
        let result = activate_statue_trap(&mut rng, None, false);
        assert!(!result.triggered);
        assert!(result.statue_destroyed);
    }

    #[test]
    fn test_is_statue_trap() {
        assert!(is_statue_trap(true, Some(5)));
        assert!(!is_statue_trap(false, Some(5)));
        assert!(!is_statue_trap(true, Some(-1)));
        assert!(!is_statue_trap(true, None));
    }

    #[test]
    fn test_rndtrap() {
        use crate::dungeon::dlevel::DLevel;
        use crate::dungeon::level::Level;

        let mut rng = GameRng::from_entropy();
        let level = Level::new(DLevel::new(0, 5)); // Main dungeon

        // Should return a valid trap type
        for _ in 0..10 {
            let trap_type = rndtrap(&mut rng, &level);
            // Should not be Hole (always excluded)
            assert_ne!(trap_type, TrapType::Hole);
            // Should not be MagicPortal or VibratingSquare (excluded for special levels)
            assert_ne!(trap_type, TrapType::MagicPortal);
        }
    }

    #[test]
    fn test_rndtrap_gehennom() {
        use crate::dungeon::dlevel::DLevel;
        use crate::dungeon::level::Level;

        let mut rng = GameRng::from_entropy();
        let level = Level::new(DLevel::new(1, 5)); // Gehennom

        // Should not place trapdoors in Gehennom
        for _ in 0..20 {
            let trap_type = rndtrap(&mut rng, &level);
            assert_ne!(trap_type, TrapType::Hole);
            // Note: TrapDoor might appear, depending on random selection
        }
    }

    #[test]
    fn test_count_surround_traps() {
        use crate::dungeon::dlevel::DLevel;
        use crate::dungeon::level::{Level, Trap, TrapType};

        let mut level = Level::new(DLevel::new(0, 1));

        // Add a trap at (10, 10)
        let trap1 = Trap {
            x: 10,
            y: 10,
            trap_type: TrapType::Pit,
            activated: false,
            seen: false, // Not yet discovered
            once: false,
            madeby_u: false,
            launch_oid: None,
        };
        level.traps.push(trap1);

        // Add a discovered trap at (11, 11) (should not be counted)
        let trap2 = Trap {
            x: 11,
            y: 11,
            trap_type: TrapType::BearTrap,
            activated: false,
            seen: true, // Already discovered
            once: false,
            madeby_u: false,
            launch_oid: None,
        };
        level.traps.push(trap2);

        // Count from center position (10, 10)
        let count = count_surround_traps(&level, 10, 10);
        // Should count: trap at (10,10) is undiscovered, trap at (11,11) is discovered (not counted)
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_surround_traps_multiple() {
        use crate::dungeon::dlevel::DLevel;
        use crate::dungeon::level::{Level, Trap, TrapType};

        let mut level = Level::new(DLevel::new(0, 1));

        // Add traps around position (10, 10)
        for x in 9..=11 {
            for y in 9..=11 {
                if (x, y) != (10, 10) {
                    let trap = Trap {
                        x: x as i8,
                        y: y as i8,
                        trap_type: TrapType::Arrow,
                        activated: false,
                        seen: false,
                        once: false,
                        madeby_u: false,
                        launch_oid: None,
                    };
                    level.traps.push(trap);
                }
            }
        }

        // Should count 8 surrounding traps (3x3 grid minus center)
        let count = count_surround_traps(&level, 10, 10);
        assert_eq!(count, 8);
    }

    #[test]
    fn test_count_surround_traps_boundary() {
        use crate::dungeon::dlevel::DLevel;
        use crate::dungeon::level::{Level, Trap, TrapType};

        let mut level = Level::new(DLevel::new(0, 1));

        // Add trap at corner (1, 1)
        let trap = Trap {
            x: 1,
            y: 1,
            trap_type: TrapType::Pit,
            activated: false,
            seen: false,
            once: false,
            madeby_u: false,
            launch_oid: None,
        };
        level.traps.push(trap);

        // Count from corner - should handle boundary correctly
        let count = count_surround_traps(&level, 1, 1);
        assert_eq!(count, 1);

        // Count from slightly offset - should include corner trap
        let count = count_surround_traps(&level, 2, 2);
        assert_eq!(count, 1);
    }
}
