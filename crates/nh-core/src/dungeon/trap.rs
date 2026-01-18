//! Trap system (trap.c)
//!
//! Functions for trap creation, triggering, and effects.

use crate::dungeon::level::{Trap, TrapType};
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
