//! Monster creation (makemon.c)
//!
//! Handles spawning monsters with proper type initialization from PerMonst
//! templates, difficulty-scaled HP, initial equipment, group spawning,
//! and position validation.
//!
//! Functions accept `&[PerMonst]` for the monster database since nh-core
//! cannot depend on nh-data (the dependency runs the other way).

#[cfg(not(feature = "std"))]
use crate::compat::*;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use super::{Monster, MonsterId, MonsterFlags, MonsterState, PerMonst};
use crate::dungeon::Level;
use crate::rng::GameRng;

// ============================================================================
// Gen-flag constants (must match nh-data/src/monsters.rs)
// ============================================================================

/// Bottom 3 bits of gen_flags encode spawn frequency (0-7)
const G_FREQ_MASK: u16 = 0x0007;
/// Can be genocided
const G_GENO: u16 = 0x0020;
/// Appears in large groups
const G_LGROUP: u16 = 0x0040;
/// Appears in small groups
const G_SGROUP: u16 = 0x0080;
/// Not randomly generated (quest/special only)
const G_NOGEN: u16 = 0x0200;
/// Only generated in Gehennom
const G_HELL: u16 = 0x0400;
/// Never generated in Gehennom
const G_NOHELL: u16 = 0x0800;
/// Unique monster (only one per game)
const G_UNIQ: u16 = 0x1000;

// ============================================================================
// Types
// ============================================================================

bitflags! {
    /// Flags controlling monster creation (MM_* in C)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MakeMonFlags: u32 {
        /// Don't give the monster inventory/weapons
        const NO_MINVENT    = 0x0001;
        /// Don't spawn group members
        const NO_GRP        = 0x0002;
        /// Force hostile regardless of template
        const ANGRY         = 0x0004;
        /// Force peaceful regardless of template
        const PEACEFUL      = 0x0008;
        /// Don't increment born count
        const NOCOUNTBIRTH  = 0x0010;
        /// Skip goodpos check (caller validated)
        const NO_GOODPOS    = 0x0020;
    }
}

/// Birth/death/genocide tracking per monster type (mvitals[] in C)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonsterVitals {
    /// Number of this type that have been created
    pub born: u16,
    /// Number of this type that have died
    pub died: u16,
    /// Whether this type has been genocided
    pub genocided: bool,
}

/// Result of a makemon call with additional info
#[derive(Debug, Clone, Copy)]
pub struct MakeMonResult {
    /// ID of the created monster
    pub id: MonsterId,
    /// How many total monsters were created (including group members)
    pub total_created: usize,
}

// ============================================================================
// Gen-flag helpers
// ============================================================================

/// Get spawn frequency from gen_flags (0-7)
fn gen_frequency(gen_flags: u16) -> u16 {
    gen_flags & G_FREQ_MASK
}

/// Check if monster type is generated in Gehennom only
fn is_hell_only(gen_flags: u16) -> bool {
    gen_flags & G_HELL != 0
}

/// Check if monster type is never generated in Gehennom
fn is_nohell(gen_flags: u16) -> bool {
    gen_flags & G_NOHELL != 0
}

/// Check if monster type should not be randomly generated
fn is_nogen(gen_flags: u16) -> bool {
    gen_flags & G_NOGEN != 0
}

/// Check if monster type is unique
fn is_unique(gen_flags: u16) -> bool {
    gen_flags & G_UNIQ != 0
}

/// Check if monster type spawns in small groups
fn is_sgroup(gen_flags: u16) -> bool {
    gen_flags & G_SGROUP != 0
}

/// Check if monster type spawns in large groups
fn is_lgroup(gen_flags: u16) -> bool {
    gen_flags & G_LGROUP != 0
}

/// Check if monster type can be genocided
pub fn is_genocidable(gen_flags: u16) -> bool {
    gen_flags & G_GENO != 0
}

// ============================================================================
// HP calculation
// ============================================================================

/// Compute HP for a new monster based on its template.
///
/// Matches C `newmonhp()` in makemon.c:
/// - Level 0 monsters get rnd(4) HP
/// - Others get d(m_lev, 8)
/// - Minimum 1 HP
pub fn new_mon_hp(pm: &PerMonst, rng: &mut GameRng) -> (i32, i32) {
    let level = pm.level.max(0) as u32;

    let hp = if level == 0 {
        rng.rnd(4) as i32
    } else {
        rng.dice(level, 8) as i32
    };

    let hp = hp.max(1);
    (hp, hp)
}

// ============================================================================
// Position validation
// ============================================================================

/// Check if a position is valid for monster placement.
///
/// Matches C `goodpos()` in makemon.c. Position must be:
/// - In bounds
/// - Passable (walkable, or monster can fly/pass walls)
/// - Not already occupied by another monster
pub fn goodpos(level: &Level, x: i8, y: i8, pm: Option<&PerMonst>) -> bool {
    if !level.is_valid_pos(x, y) {
        return false;
    }

    // Check if cell is passable for this monster type.
    // In C: flies() helps with pools/moats, but only passes_walls() lets
    // a monster exist in solid rock.
    let passable = if let Some(pm) = pm {
        level.is_walkable(x, y) || pm.passes_walls()
    } else {
        level.is_walkable(x, y)
    };

    if !passable {
        return false;
    }

    // Not occupied by another monster
    level.monster_at(x, y).is_none()
}

/// Find a nearby valid position for monster placement.
///
/// Matches C `enexto()` in makemon.c. Searches outward in expanding
/// squares from (x, y) until a valid position is found.
pub fn enexto(
    level: &Level,
    x: i8,
    y: i8,
    pm: Option<&PerMonst>,
) -> Option<(i8, i8)> {
    // Try the original position first
    if goodpos(level, x, y, pm) {
        return Some((x, y));
    }

    // Search in expanding squares (distance 1..20)
    for dist in 1..=20i8 {
        for dx in -dist..=dist {
            for dy in -dist..=dist {
                // Only check the outer ring of each square
                if dx.abs() != dist && dy.abs() != dist {
                    continue;
                }
                let nx = x.saturating_add(dx);
                let ny = y.saturating_add(dy);
                if goodpos(level, nx, ny, pm) {
                    return Some((nx, ny));
                }
            }
        }
    }

    None
}

// ============================================================================
// Monster type selection
// ============================================================================

/// Pick a random monster type appropriate for the given depth.
///
/// Matches C `rndmonst()` in makemon.c. Uses difficulty bands and
/// frequency weighting:
/// - `min_level = depth / 6`
/// - `max_level = (depth + player_level) / 2 + 1`
/// - Weighted by gen_flags frequency
///
/// Returns the monster type index into the `monsters` array, or None if
/// no suitable monster exists.
pub fn rndmonst(
    monsters: &[PerMonst],
    depth: i32,
    player_level: i32,
    is_hell: bool,
    vitals: &[MonsterVitals],
    rng: &mut GameRng,
) -> Option<i16> {
    let min_level = (depth / 6) as i8;
    let max_level = ((depth + player_level) / 2 + 1).max(1) as i8;

    let mut candidates: Vec<(i16, u16)> = Vec::new();

    for (i, pm) in monsters.iter().enumerate() {
        if !is_candidate(pm, i, is_hell, vitals) {
            continue;
        }
        if pm.level < min_level || pm.level > max_level {
            continue;
        }
        let freq = gen_frequency(pm.gen_flags);
        if freq == 0 {
            continue;
        }
        candidates.push((i as i16, freq));
    }

    if candidates.is_empty() {
        // Fallback: any valid monster whose level <= max_level
        for (i, pm) in monsters.iter().enumerate() {
            if !is_candidate(pm, i, is_hell, vitals) {
                continue;
            }
            if pm.level > max_level {
                continue;
            }
            let freq = gen_frequency(pm.gen_flags);
            if freq > 0 {
                candidates.push((i as i16, freq));
            }
        }
    }

    weighted_pick(&candidates, rng)
}

/// Pick a random monster of a given class (by display symbol).
///
/// Matches C `mkclass()` in makemon.c. Picks from all generable
/// monsters with the specified symbol, weighted by frequency.
pub fn mkclass(
    monsters: &[PerMonst],
    class_char: char,
    vitals: &[MonsterVitals],
    rng: &mut GameRng,
) -> Option<i16> {
    let mut candidates: Vec<(i16, u16)> = Vec::new();

    for (i, pm) in monsters.iter().enumerate() {
        if pm.symbol != class_char {
            continue;
        }
        if !is_candidate(pm, i, false, vitals) {
            continue;
        }
        let freq = gen_frequency(pm.gen_flags);
        if freq == 0 {
            continue;
        }
        candidates.push((i as i16, freq));
    }

    weighted_pick(&candidates, rng)
}

/// Check if a monster type is eligible for random generation
fn is_candidate(
    pm: &PerMonst,
    index: usize,
    is_hell: bool,
    vitals: &[MonsterVitals],
) -> bool {
    // Skip genocided types
    if index < vitals.len() && vitals[index].genocided {
        return false;
    }
    // Skip non-generable types
    if is_nogen(pm.gen_flags) {
        return false;
    }
    // Skip unique types already born
    if is_unique(pm.gen_flags) && index < vitals.len() && vitals[index].born > 0 {
        return false;
    }
    // Check Hell restrictions
    if is_hell && is_nohell(pm.gen_flags) {
        return false;
    }
    if !is_hell && is_hell_only(pm.gen_flags) {
        return false;
    }
    true
}

/// Weighted random selection from (index, weight) pairs
fn weighted_pick(candidates: &[(i16, u16)], rng: &mut GameRng) -> Option<i16> {
    if candidates.is_empty() {
        return None;
    }

    let total_weight: u32 = candidates.iter().map(|(_, w)| *w as u32).sum();
    if total_weight == 0 {
        return None;
    }
    let mut roll = rng.rn2(total_weight);

    for &(idx, weight) in candidates {
        if roll < weight as u32 {
            return Some(idx);
        }
        roll -= weight as u32;
    }

    // Shouldn't reach here, but return last candidate
    Some(candidates.last().unwrap().0)
}

// ============================================================================
// Main monster creation
// ============================================================================

/// Create a fully initialized monster from a PerMonst template.
///
/// Matches C `makemon()` in makemon.c. Sets HP, AC, speed, attacks,
/// resistances, and flags from the template. Optionally equips weapons
/// and inventory (unless `NO_MINVENT`), and spawns group members
/// (unless `NO_GRP`).
///
/// Returns the MonsterId of the leader (first created monster), or None
/// if the position is invalid.
pub fn makemon(
    level: &mut Level,
    monsters: &[PerMonst],
    monster_type: i16,
    x: i8,
    y: i8,
    mm_flags: MakeMonFlags,
    vitals: &mut Vec<MonsterVitals>,
    rng: &mut GameRng,
) -> Option<MonsterId> {
    let mtype_idx = monster_type as usize;
    if mtype_idx >= monsters.len() {
        return None;
    }

    let pm = &monsters[mtype_idx];

    // Find valid position
    let (px, py) = if mm_flags.contains(MakeMonFlags::NO_GOODPOS) {
        (x, y)
    } else {
        enexto(level, x, y, Some(pm))?
    };

    // Build the monster instance
    let mon = init_monster(monster_type, px, py, pm, mm_flags, rng);

    // Track birth
    if !mm_flags.contains(MakeMonFlags::NOCOUNTBIRTH) {
        ensure_vitals(vitals, mtype_idx);
        vitals[mtype_idx].born += 1;
    }

    // Add to level
    let id = level.add_monster(mon);

    // Spawn group members (prevent recursive groups)
    if !mm_flags.contains(MakeMonFlags::NO_GRP) {
        if is_sgroup(pm.gen_flags) {
            let count = rng.rnd(3) as usize + 1; // 2-4
            m_initgrp(level, monsters, monster_type, px, py, count, mm_flags, vitals, rng);
        } else if is_lgroup(pm.gen_flags) {
            let count = rng.rnd(6) as usize + 3; // 4-9
            m_initgrp(level, monsters, monster_type, px, py, count, mm_flags, vitals, rng);
        }
    }

    Some(id)
}

/// Initialize a Monster instance from a PerMonst template.
///
/// Sets all fields from the template data: name, HP, AC, level, speed,
/// alignment, attacks, resistances, flags, gender, and behavior state.
fn init_monster(
    monster_type: i16,
    x: i8,
    y: i8,
    pm: &PerMonst,
    mm_flags: MakeMonFlags,
    rng: &mut GameRng,
) -> Monster {
    let mut mon = Monster::new(MonsterId(0), monster_type, x, y);

    // Name from template
    mon.name = pm.name.to_string();

    // HP from template
    let (hp, hp_max) = new_mon_hp(pm, rng);
    mon.hp = hp;
    mon.hp_max = hp_max;

    // AC from template
    mon.ac = pm.armor_class;

    // Level from template
    mon.level = pm.level.max(0) as u8;

    // Speed from template
    mon.base_speed = pm.move_speed as i32;

    // Alignment from template
    mon.alignment = pm.alignment;

    // Attacks from template
    mon.attacks = pm.attacks;

    // Resistances from template
    mon.resistances = pm.resistances;

    // Flags from template
    mon.flags = pm.flags;

    // Behavior state: ANGRY flag or template hostility
    if mm_flags.contains(MakeMonFlags::ANGRY) {
        mon.state = MonsterState::active();
    } else if mm_flags.contains(MakeMonFlags::PEACEFUL) {
        mon.state = MonsterState::peaceful();
    } else if pm.flags.contains(MonsterFlags::DOMESTIC) {
        mon.state = MonsterState::tame();
    } else if pm.is_peaceful() {
        mon.state = MonsterState::peaceful();
    } else {
        mon.state = MonsterState::active();
    }

    // Gender
    if pm.flags.contains(MonsterFlags::FEMALE) {
        mon.female = true;
    } else if pm.flags.contains(MonsterFlags::MALE) {
        mon.female = false;
    } else if !pm.flags.contains(MonsterFlags::NEUTER) {
        mon.female = rng.rn2(2) == 0;
    }

    // NODIAG: grid bugs can only move cardinally (C: hack.h:444)
    // PM_GRID_BUG = 115
    if monster_type == 115 {
        mon.no_diagonal_move = true;
    }

    // Equip weapons and inventory
    if !mm_flags.contains(MakeMonFlags::NO_MINVENT) {
        m_initweap(&mut mon, pm, rng);
        m_initinv(&mut mon, pm, rng);
    }

    mon
}

/// Ensure the vitals array is large enough for the given index.
fn ensure_vitals(vitals: &mut Vec<MonsterVitals>, index: usize) {
    if vitals.len() <= index {
        vitals.resize_with(index + 1, MonsterVitals::default);
    }
}

// ============================================================================
// Equipment initialization
// ============================================================================

/// Equip a monster with initial weapons based on its type.
///
/// Matches C `m_initweap()` in makemon.c. Race/class-specific:
/// - Giants: boulders (ROCKTHROW flag)
/// - Mercenaries: military weapons/armor
/// - Orcs: orcish weapons
/// - Elves: elven weapons
/// - Dwarves: dwarvish weapons
///
/// Full object creation requires the Objects database (nh-data), so this
/// is a framework for future expansion.
fn m_initweap(_mon: &mut Monster, _pm: &PerMonst, _rng: &mut GameRng) {
    // Weapon assignment requires Object creation from nh-data OBJECTS table.
    // The framework is in place; callers should provide an object factory
    // callback or use a higher-level wrapper with access to OBJECTS.
    //
    // Expected mapping: Giant→boulder, Mercenary→long sword/shield/mail,
    // Orc→orcish weapons, Elf→elven weapons, Dwarf→dwarvish weapons,
    // Angel→saber/long sword, Demon→trident/broadsword
}

/// Give a monster its initial inventory.
///
/// Matches C `m_initinv()` in makemon.c. Class-specific:
/// - Shopkeepers: keys, gold
/// - Nymphs: mirrors
/// - Leprechauns: gold
/// - Mercenaries: food rations
///
/// Full object creation requires the Objects database (nh-data).
fn m_initinv(_mon: &mut Monster, _pm: &PerMonst, _rng: &mut GameRng) {
    // Inventory assignment requires Object creation from nh-data OBJECTS table.
    // Shopkeepers→keys/gold, Nymphs→mirrors, Leprechauns→gold, Mercenaries→rations.
}

// ============================================================================
// Group spawning
// ============================================================================

/// Spawn a group of monsters around a leader position.
///
/// Matches C `m_initgrp()` in makemon.c. Creates `count` monsters of
/// the same type adjacent to (`x`, `y`), each with `NO_GRP` to prevent
/// recursive group spawning.
fn m_initgrp(
    level: &mut Level,
    monsters: &[PerMonst],
    monster_type: i16,
    x: i8,
    y: i8,
    count: usize,
    base_flags: MakeMonFlags,
    vitals: &mut Vec<MonsterVitals>,
    rng: &mut GameRng,
) {
    let grp_flags = base_flags | MakeMonFlags::NO_GRP;

    for _ in 0..count {
        let dx = rng.rn2(3) as i8 - 1; // -1, 0, 1
        let dy = rng.rn2(3) as i8 - 1;
        let gx = x.saturating_add(dx);
        let gy = y.saturating_add(dy);

        let pm = monsters.get(monster_type as usize);
        if goodpos(level, gx, gy, pm) {
            makemon(level, monsters, monster_type, gx, gy, grp_flags, vitals, rng);
        }
    }
}

// ============================================================================
// Utility
// ============================================================================

/// Check if a monster type should not appear on this elemental level.
///
/// Matches C `wrong_elem_type()` in makemon.c. Fire-resistant monsters
/// don't appear on the Water Plane, etc.
pub fn wrong_elem_type(_pm: &PerMonst, _depth: i32) -> bool {
    // Elemental plane restrictions not yet applicable: planes not implemented
    false
}

/// Record a monster death in the vitals tracking.
pub fn record_death(vitals: &mut Vec<MonsterVitals>, monster_type: i16) {
    let idx = monster_type as usize;
    ensure_vitals(vitals, idx);
    vitals[idx].died += 1;
}

/// Genocide a monster type, preventing future spawning.
pub fn genocide_type(vitals: &mut Vec<MonsterVitals>, monster_type: i16) {
    let idx = monster_type as usize;
    ensure_vitals(vitals, idx);
    vitals[idx].genocided = true;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{empty_attacks, Attack, AttackType, DamageType};
    use crate::dungeon::{DLevel, Level};
    use crate::monster::{MonsterResistances, MonsterSize, MonsterSound};
    use crate::rng::GameRng;

    /// Create a simple test PerMonst
    fn test_permonst(name: &'static str, symbol: char, level: i8, gen_flags: u16) -> PerMonst {
        PerMonst {
            name,
            symbol,
            level,
            move_speed: 12,
            armor_class: 5,
            magic_resistance: 0,
            alignment: 0,
            gen_flags,
            attacks: empty_attacks(),
            corpse_weight: 100,
            corpse_nutrition: 50,
            sound: MonsterSound::Silent,
            size: MonsterSize::Medium,
            resistances: MonsterResistances::empty(),
            conveys: MonsterResistances::empty(),
            flags: MonsterFlags::empty(),
            difficulty: level.max(0) as u8,
            color: 0,
        }
    }

    /// Create a test monster database
    fn test_monsters() -> Vec<PerMonst> {
        vec![
            test_permonst("grid bug", 'x', 0, G_GENO | 3),                         // 0
            test_permonst("newt", 'r', 0, G_GENO | 5),                              // 1
            test_permonst("jackal", 'd', 0, G_GENO | G_SGROUP | 3),                 // 2
            test_permonst("kobold", 'k', 1, G_GENO | G_SGROUP | 3),                 // 3
            test_permonst("goblin", 'o', 1, G_GENO | 2),                            // 4
            test_permonst("gnome", 'G', 2, G_GENO | 4),                             // 5
            test_permonst("giant ant", 'a', 2, G_GENO | G_SGROUP | 3),              // 6
            test_permonst("orc", 'o', 3, G_GENO | 2),                               // 7
            test_permonst("wolf", 'd', 5, G_GENO | G_SGROUP | 3),                   // 8
            test_permonst("ogre", 'O', 7, G_GENO | 2),                              // 9
            test_permonst("quest monster", 'Q', 5, G_NOGEN | 1),                     // 10 - NOGEN
            test_permonst("unique boss", 'V', 20, G_UNIQ | G_NOGEN | 1),            // 11 - unique
            test_permonst("hell hound", 'd', 12, G_GENO | G_HELL | 2),              // 12 - hell only
            test_permonst("pony", 'u', 3, G_GENO | G_NOHELL | 3),                   // 13 - no hell
        ]
    }

    /// Create a test level with a room
    fn test_level_with_room() -> Level {
        let mut level = Level::new(DLevel::default());
        // Create a small room (10x5 at position 5,3)
        for x in 5..15 {
            for y in 3..8 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }
        level
    }

    // ---- gen_frequency tests ----

    #[test]
    fn test_gen_frequency() {
        assert_eq!(gen_frequency(G_GENO | 3), 3);
        assert_eq!(gen_frequency(G_GENO | G_SGROUP | 5), 5);
        assert_eq!(gen_frequency(G_NOGEN | 0), 0);
        assert_eq!(gen_frequency(G_UNIQ | 7), 7);
    }

    #[test]
    fn test_gen_flag_helpers() {
        assert!(is_hell_only(G_HELL | 2));
        assert!(!is_hell_only(G_GENO | 2));
        assert!(is_nohell(G_NOHELL | 3));
        assert!(is_nogen(G_NOGEN | 1));
        assert!(is_unique(G_UNIQ | 1));
        assert!(is_sgroup(G_SGROUP | 3));
        assert!(is_lgroup(G_LGROUP | 2));
        assert!(is_genocidable(G_GENO | 3));
    }

    // ---- new_mon_hp tests ----

    #[test]
    fn test_new_mon_hp_level_zero() {
        let pm = test_permonst("newt", 'r', 0, G_GENO | 3);
        let mut rng = GameRng::new(42);
        let (hp, hp_max) = new_mon_hp(&pm, &mut rng);
        assert!(hp >= 1 && hp <= 4);
        assert_eq!(hp, hp_max);
    }

    #[test]
    fn test_new_mon_hp_higher_level() {
        let pm = test_permonst("ogre", 'O', 7, G_GENO | 2);
        let mut rng = GameRng::new(42);
        let (hp, hp_max) = new_mon_hp(&pm, &mut rng);
        // d(7, 8) = 7-56
        assert!(hp >= 1 && hp <= 56, "hp={hp} out of range");
        assert_eq!(hp, hp_max);
    }

    #[test]
    fn test_new_mon_hp_minimum_one() {
        // Negative level should be clamped to 0
        let pm = test_permonst("neg", 'x', -1, G_GENO | 1);
        let mut rng = GameRng::new(42);
        let (hp, _) = new_mon_hp(&pm, &mut rng);
        assert!(hp >= 1);
    }

    // ---- goodpos tests ----

    #[test]
    fn test_goodpos_empty_room() {
        let level = test_level_with_room();
        assert!(goodpos(&level, 7, 5, None));
    }

    #[test]
    fn test_goodpos_wall() {
        let level = test_level_with_room();
        assert!(!goodpos(&level, 0, 0, None)); // Stone
    }

    #[test]
    fn test_goodpos_out_of_bounds() {
        let level = test_level_with_room();
        assert!(!goodpos(&level, -1, 5, None));
        assert!(!goodpos(&level, 100, 5, None));
    }

    #[test]
    fn test_goodpos_occupied() {
        let mut level = test_level_with_room();
        let mon = Monster::new(MonsterId(0), 0, 7, 5);
        level.add_monster(mon);
        assert!(!goodpos(&level, 7, 5, None));
    }

    #[test]
    fn test_goodpos_flying_over_nonwalkable() {
        let level = test_level_with_room();
        // Stone at (0,0) is not walkable, but a flying monster could go there
        // Actually in NetHack flying doesn't let you go through walls
        // Only WALLWALK does. Stone is not passable.
        let mut pm = test_permonst("flyer", 'B', 3, G_GENO | 2);
        pm.flags = MonsterFlags::FLY;
        // Still can't stand on stone
        assert!(!goodpos(&level, 0, 0, Some(&pm)));
    }

    // ---- enexto tests ----

    #[test]
    fn test_enexto_direct() {
        let level = test_level_with_room();
        let result = enexto(&level, 7, 5, None);
        assert_eq!(result, Some((7, 5)));
    }

    #[test]
    fn test_enexto_adjacent() {
        let mut level = test_level_with_room();
        // Block the direct position
        let mon = Monster::new(MonsterId(0), 0, 7, 5);
        level.add_monster(mon);

        let result = enexto(&level, 7, 5, None);
        assert!(result.is_some());
        let (rx, ry) = result.unwrap();
        // Should be adjacent
        assert!((rx - 7).abs() <= 1 && (ry - 5).abs() <= 1);
        assert!(level.is_walkable(rx, ry));
    }

    // ---- rndmonst tests ----

    #[test]
    fn test_rndmonst_basic() {
        let monsters = test_monsters();
        let vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut rng = GameRng::new(42);

        let result = rndmonst(&monsters, 3, 1, false, &vitals, &mut rng);
        assert!(result.is_some());
        let idx = result.unwrap() as usize;
        // Should not pick NOGEN or UNIQ monsters
        assert_ne!(idx, 10); // quest monster
        assert_ne!(idx, 11); // unique boss
        assert_ne!(idx, 12); // hell only (not in hell)
    }

    #[test]
    fn test_rndmonst_hell() {
        let monsters = test_monsters();
        let vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut rng = GameRng::new(42);

        // In hell, depth 30, should NOT pick nohell monsters (idx 13)
        let mut saw_nohell = false;
        for _ in 0..200 {
            if let Some(idx) = rndmonst(&monsters, 30, 15, true, &vitals, &mut rng) {
                if idx == 13 {
                    saw_nohell = true;
                }
            }
        }
        assert!(!saw_nohell, "nohell monster picked in hell");
    }

    #[test]
    fn test_rndmonst_genocided() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        vitals[1].genocided = true; // Genocide newts
        let mut rng = GameRng::new(42);

        for _ in 0..100 {
            if let Some(idx) = rndmonst(&monsters, 1, 1, false, &vitals, &mut rng) {
                assert_ne!(idx, 1, "genocided newt was picked");
            }
        }
    }

    #[test]
    fn test_rndmonst_empty_db() {
        let monsters: Vec<PerMonst> = Vec::new();
        let vitals: Vec<MonsterVitals> = Vec::new();
        let mut rng = GameRng::new(42);

        let result = rndmonst(&monsters, 1, 1, false, &vitals, &mut rng);
        assert!(result.is_none());
    }

    // ---- mkclass tests ----

    #[test]
    fn test_mkclass_dogs() {
        let monsters = test_monsters();
        let vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut rng = GameRng::new(42);

        // 'd' class: jackal (2), wolf (8), hell_hound (12)
        let result = mkclass(&monsters, 'd', &vitals, &mut rng);
        assert!(result.is_some());
        let idx = result.unwrap();
        assert!(
            idx == 2 || idx == 8 || idx == 13,
            "got unexpected index {idx} for class 'd'"
        );
    }

    #[test]
    fn test_mkclass_nonexistent() {
        let monsters = test_monsters();
        let vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut rng = GameRng::new(42);

        let result = mkclass(&monsters, 'Z', &vitals, &mut rng);
        assert!(result.is_none());
    }

    // ---- makemon tests ----

    #[test]
    fn test_makemon_basic() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut level = test_level_with_room();
        let mut rng = GameRng::new(42);

        let result = makemon(
            &mut level,
            &monsters,
            5, // gnome
            7,
            5,
            MakeMonFlags::NO_GRP,
            &mut vitals,
            &mut rng,
        );

        assert!(result.is_some());
        let id = result.unwrap();
        let mon = level.monster(id).unwrap();
        assert_eq!(mon.name, "gnome");
        assert_eq!(mon.monster_type, 5);
        assert_eq!(mon.ac, 5);
        assert_eq!(mon.level, 2);
        assert!(mon.hp >= 1);
        assert_eq!(vitals[5].born, 1);
    }

    #[test]
    fn test_makemon_sets_template_data() {
        let mut monsters = test_monsters();
        // Give the orc some attacks and resistances
        monsters[7].attacks[0] = Attack::new(AttackType::Weapon, DamageType::Physical, 1, 8);
        monsters[7].resistances = MonsterResistances::POISON;
        monsters[7].flags = MonsterFlags::ORC | MonsterFlags::HOSTILE;

        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut level = test_level_with_room();
        let mut rng = GameRng::new(42);

        let id = makemon(
            &mut level,
            &monsters,
            7, // orc
            7,
            5,
            MakeMonFlags::NO_GRP,
            &mut vitals,
            &mut rng,
        )
        .unwrap();

        let mon = level.monster(id).unwrap();
        assert_eq!(mon.attacks[0].attack_type, AttackType::Weapon);
        assert_eq!(mon.attacks[0].dice_num, 1);
        assert_eq!(mon.attacks[0].dice_sides, 8);
        assert!(mon.resistances.contains(MonsterResistances::POISON));
        assert!(mon.flags.contains(MonsterFlags::ORC));
        assert!(mon.is_hostile());
    }

    #[test]
    fn test_makemon_hostile_by_flag() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut level = test_level_with_room();
        let mut rng = GameRng::new(42);

        let id = makemon(
            &mut level,
            &monsters,
            5, // gnome
            7,
            5,
            MakeMonFlags::NO_GRP | MakeMonFlags::ANGRY,
            &mut vitals,
            &mut rng,
        )
        .unwrap();

        let mon = level.monster(id).unwrap();
        assert!(mon.is_hostile());
    }

    #[test]
    fn test_makemon_peaceful_by_flag() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut level = test_level_with_room();
        let mut rng = GameRng::new(42);

        let id = makemon(
            &mut level,
            &monsters,
            5, // gnome
            7,
            5,
            MakeMonFlags::NO_GRP | MakeMonFlags::PEACEFUL,
            &mut vitals,
            &mut rng,
        )
        .unwrap();

        let mon = level.monster(id).unwrap();
        assert!(mon.is_peaceful());
    }

    #[test]
    fn test_makemon_invalid_type() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut level = test_level_with_room();
        let mut rng = GameRng::new(42);

        let result = makemon(
            &mut level,
            &monsters,
            999, // out of range
            7,
            5,
            MakeMonFlags::NO_GRP,
            &mut vitals,
            &mut rng,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_makemon_no_valid_position() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        // Level with no walkable cells
        let level_no_room = Level::new(DLevel::default());
        let mut level = level_no_room;
        let mut rng = GameRng::new(42);

        let result = makemon(
            &mut level,
            &monsters,
            5,
            40,
            10,
            MakeMonFlags::NO_GRP,
            &mut vitals,
            &mut rng,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_makemon_with_group() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut level = test_level_with_room();
        let mut rng = GameRng::new(42);

        // Jackal (idx 2) has G_SGROUP
        let result = makemon(
            &mut level,
            &monsters,
            2, // jackal
            7,
            5,
            MakeMonFlags::empty(),
            &mut vitals,
            &mut rng,
        );

        assert!(result.is_some());
        // Should have spawned extra group members (1 leader + 2-4 group)
        assert!(
            level.monsters.len() >= 2,
            "expected group, got {} monsters",
            level.monsters.len()
        );
        // All should be jackals
        for mon in &level.monsters {
            assert_eq!(mon.monster_type, 2);
            assert_eq!(mon.name, "jackal");
        }
    }

    #[test]
    fn test_makemon_nocountbirth() {
        let monsters = test_monsters();
        let mut vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut level = test_level_with_room();
        let mut rng = GameRng::new(42);

        makemon(
            &mut level,
            &monsters,
            5,
            7,
            5,
            MakeMonFlags::NO_GRP | MakeMonFlags::NOCOUNTBIRTH,
            &mut vitals,
            &mut rng,
        );

        assert_eq!(vitals[5].born, 0);
    }

    // ---- vitals tracking tests ----

    #[test]
    fn test_record_death() {
        let mut vitals: Vec<MonsterVitals> = Vec::new();
        record_death(&mut vitals, 5);
        assert_eq!(vitals[5].died, 1);
        record_death(&mut vitals, 5);
        assert_eq!(vitals[5].died, 2);
    }

    #[test]
    fn test_genocide_type() {
        let mut vitals: Vec<MonsterVitals> = Vec::new();
        genocide_type(&mut vitals, 3);
        assert!(vitals[3].genocided);
    }

    #[test]
    fn test_ensure_vitals_grows() {
        let mut vitals: Vec<MonsterVitals> = Vec::new();
        ensure_vitals(&mut vitals, 10);
        assert_eq!(vitals.len(), 11);
        // Previous entries should be default
        assert!(!vitals[0].genocided);
        assert_eq!(vitals[0].born, 0);
    }

    // ---- weighted_pick tests ----

    #[test]
    fn test_weighted_pick_single() {
        let candidates = vec![(42i16, 5u16)];
        let mut rng = GameRng::new(42);
        assert_eq!(weighted_pick(&candidates, &mut rng), Some(42));
    }

    #[test]
    fn test_weighted_pick_empty() {
        let candidates: Vec<(i16, u16)> = Vec::new();
        let mut rng = GameRng::new(42);
        assert_eq!(weighted_pick(&candidates, &mut rng), None);
    }

    #[test]
    fn test_weighted_pick_distribution() {
        // One heavy weight, one light weight
        let candidates = vec![(0i16, 100u16), (1i16, 1u16)];
        let mut rng = GameRng::new(42);
        let mut counts = [0u32; 2];
        for _ in 0..1000 {
            if let Some(idx) = weighted_pick(&candidates, &mut rng) {
                counts[idx as usize] += 1;
            }
        }
        // The heavy weight should dominate
        assert!(counts[0] > counts[1] * 10, "heavy={} light={}", counts[0], counts[1]);
    }

    // ---- mkclass distribution ----

    #[test]
    fn test_mkclass_excludes_hell_only_outside_hell() {
        let monsters = test_monsters();
        let vitals = vec![MonsterVitals::default(); monsters.len()];
        let mut rng = GameRng::new(42);

        // 'd' class: jackal(2), wolf(8), hell_hound(12-hell), pony(13-not pony, it's 'u')
        // Actually: 'd' = jackal(2), wolf(8), hell_hound(12)
        // mkclass doesn't check hell status — it passes is_hell=false to is_candidate
        for _ in 0..100 {
            if let Some(idx) = mkclass(&monsters, 'd', &vitals, &mut rng) {
                assert_ne!(idx, 12, "hell hound should not be picked outside hell");
            }
        }
    }
}
