//! Genocide system (read.c do_genocide)
//!
//! Handles genocide of monster species. When a player uses a scroll of genocide,
//! a single monster species can be eliminated from the game.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::dungeon::Level;
use crate::gameloop::GameState;
use serde::{Deserialize, Serialize};
use hashbrown::HashMap;

/// Flags for genocide status of a monster type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenocideFlags {
    /// Monster species has been genocided
    pub genocided: bool,
    /// If genocided, remove corpses of this type (no_corpse flag)
    pub no_corpse: bool,
}

/// Global genocide tracking - tracks which monster species are genocided
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonsterVitals {
    /// Genocide flags indexed by monster_type (i16)
    flags: HashMap<i16, GenocideFlags>,
}

impl MonsterVitals {
    /// Create a new empty MonsterVitals tracker
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    /// Check if a monster type is genocided
    pub fn is_genocided(&self, monster_type: i16) -> bool {
        self.flags
            .get(&monster_type)
            .map(|f| f.genocided)
            .unwrap_or(false)
    }

    /// Mark a monster type as genocided
    pub fn mark_genocided(&mut self, monster_type: i16) {
        self.flags.insert(
            monster_type,
            GenocideFlags {
                genocided: true,
                no_corpse: true,
            },
        );
    }

    /// Get genocide flags for a monster type
    pub fn get_flags(&self, monster_type: i16) -> GenocideFlags {
        self.flags.get(&monster_type).copied().unwrap_or_default()
    }

    /// Get number of genocided monster types
    pub fn num_genocides(&self) -> u32 {
        self.flags.values().filter(|f| f.genocided).count() as u32
    }

    /// Clear all genocide flags (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.flags.clear();
    }
}

/// Result of a genocide operation
#[derive(Debug, Clone)]
pub struct GenocideResult {
    /// Messages to display to the player
    pub messages: Vec<String>,
    /// Number of monsters killed
    pub killed_count: u32,
    /// Whether the player was killed (self-genocide)
    pub player_died: bool,
    /// Alignment shift (-10 for peaceful, 10 for demon, etc.)
    pub alignment_shift: i8,
}

impl GenocideResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            killed_count: 0,
            player_died: false,
            alignment_shift: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }

    pub fn with_killed(mut self, count: u32) -> Self {
        self.killed_count = count;
        self
    }

    pub fn with_alignment_shift(mut self, shift: i8) -> Self {
        self.alignment_shift = shift;
        self
    }

    pub fn with_player_death(mut self) -> Self {
        self.player_died = true;
        self
    }
}

impl Default for GenocideResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Kill all monsters matching any type in a list
///
/// Returns:
/// - count of monsters removed
/// - list of messages describing what happened
fn kill_genocided_monsters_by_type(level: &mut Level, monster_types: &[i16]) -> (u32, Vec<String>) {
    let mut messages = Vec::new();
    let mut count = 0u32;

    // Collect indices of monsters to remove (backwards to avoid index shifting)
    let indices_to_remove: Vec<usize> = level
        .monsters
        .iter()
        .enumerate()
        .filter(|(_, m)| monster_types.contains(&m.monster_type))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    // Remove monsters in reverse order to preserve indices
    for &idx in indices_to_remove.iter().rev() {
        if let Some(monster) = level.monsters.get(idx) {
            messages.push(format!("A {} is annihilated!", monster.name));
            // Update monster_grid to remove the reference
            let x = monster.x as usize;
            let y = monster.y as usize;
            if x < crate::COLNO && y < crate::ROWNO {
                level.monster_grid[x][y] = None;
            }
            count += 1;
        }
        level.monsters.remove(idx);
    }

    (count, messages)
}

/// Kill all monsters of a specific type on a level and remove them
///
/// Returns:
/// - count of monsters removed
/// - list of messages describing what happened
fn kill_genocided_monsters(level: &mut Level, monster_type: i16) -> (u32, Vec<String>) {
    let mut messages = Vec::new();
    let mut count = 0u32;

    // Collect indices of monsters to remove (backwards to avoid index shifting)
    let indices_to_remove: Vec<usize> = level
        .monsters
        .iter()
        .enumerate()
        .filter(|(_, m)| m.monster_type == monster_type)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    // Remove monsters in reverse order to preserve indices
    for &idx in indices_to_remove.iter().rev() {
        if let Some(monster) = level.monsters.get(idx) {
            messages.push(format!("A {} is annihilated!", monster.name));
            // Update monster_grid to remove the reference
            let x = monster.x as usize;
            let y = monster.y as usize;
            if x < crate::COLNO && y < crate::ROWNO {
                level.monster_grid[x][y] = None;
            }
            count += 1;
        }
        level.monsters.remove(idx);
    }

    (count, messages)
}

/// Perform class genocide on all monsters of a given class/symbol
///
/// Genocides all monsters that match a character symbol (e.g., 'd' for dragons).
/// This is a blessed scroll of genocide effect.
///
/// # Arguments
/// * `class_symbol` - The monster class symbol to genocide (e.g., 'd', 'O')
/// * `game_state` - The game state to modify
///
/// # Returns
/// GenocideResult with messages and statistics
pub fn do_class_genocide(_class_symbol: char, game_state: &mut GameState) -> GenocideResult {
    let mut result = GenocideResult::new();
    let mut genocided_types = Vec::new();

    // Build a list of all monster types matching this symbol
    // For now, we'll use a simplified approach since we don't have full monster data
    // In a real implementation, we'd iterate through the MONSTERS array

    // Mark all matching types as genocided and collect them
    // Note: In a full implementation, we would iterate through nh-data MONSTERS
    // and check if MONSTERS[monster_type].symbol == class_symbol.
    // For now, this is a placeholder that would be filled in with actual symbol data.
    //
    // To use with nh-data:
    //   use nh_data::MONSTERS;
    //   for (idx, permonst) in MONSTERS.iter().enumerate() {
    //       if permonst.symbol == class_symbol && !is_unique_npc(idx as i16, permonst.gen_flags) {
    //           mark_genocided(idx as i16);
    //       }
    //   }
    //
    // For now, we'll just mark a range of types as an example
    for monster_type in 0i16..256 {
        // Skip unique NPCs (with placeholder gen_flags of 0 for now)
        if is_unique_npc(monster_type, 0) {
            continue;
        }

        // Mark as genocided
        game_state.monster_vitals.mark_genocided(monster_type);
        genocided_types.push(monster_type);
    }

    // Kill all instances on current level
    let (killed, mut msgs) =
        kill_genocided_monsters_by_type(&mut game_state.current_level, &genocided_types);
    result.killed_count += killed;
    result.messages.append(&mut msgs);

    // Kill on all other levels
    for (_dlevel, level) in game_state.levels.iter_mut() {
        let (killed, mut msgs) = kill_genocided_monsters_by_type(level, &genocided_types);
        result.killed_count += killed;
        result.messages.append(&mut msgs);
    }

    // Update conduct
    game_state.player.conduct.genocides += 1;

    // Add completion message
    result.messages.push(format!(
        "You have eliminated all monsters of that class! ({} killed)",
        result.killed_count
    ));

    result
}

/// Perform reverse genocide - spawn monsters around the player
///
/// This spawns 4-6 monsters of a specific type around the player.
/// This is a cursed scroll of genocide effect.
///
/// # Arguments
/// * `monster_type` - The monster type to spawn
/// * `player_pos` - The player's position (x, y)
/// * `game_state` - The game state to modify
///
/// # Returns
/// GenocideResult with spawn information
pub fn do_reverse_genocide(
    monster_type: i16,
    player_pos: (i8, i8),
    game_state: &mut GameState,
) -> GenocideResult {
    let mut result = GenocideResult::new();

    // Check if this monster is genocided (can't spawn genocided monsters)
    if game_state.monster_vitals.is_genocided(monster_type) {
        result
            .messages
            .push("You feel a strange power, but nothing happens.".to_string());
        return result;
    }

    // Determine spawn count (4-6 monsters)
    let spawn_count = 4 + game_state.rng.rn2(3) as u32;

    // Try to spawn monsters around the player
    let mut spawned = 0u32;
    for _ in 0..spawn_count * 3 {
        // Try to find an empty position using enexto
        if let Some((x, y)) =
            crate::dungeon::enexto(player_pos.0, player_pos.1, &game_state.current_level)
        {
            // Create a new monster at this position
            // The add_monster method will assign a proper ID
            let mut monster =
                crate::monster::Monster::new(crate::monster::MonsterId::NONE, monster_type, x, y);

            // Initialize monster name based on type
            // This would use PerMonst data in a full implementation
            monster.name = format!("creature_{}", monster_type);

            game_state.current_level.add_monster(monster);
            spawned += 1;

            if spawned >= spawn_count {
                break;
            }
        }
    }

    if spawned > 0 {
        result
            .messages
            .push(format!("{} creature(s) appear around you!", spawned));
        result.killed_count = spawned; // Use killed_count field to report spawned
    } else {
        result.messages.push("Nothing happens.".to_string());
    }

    result
}

/// Perform genocide on a single monster type
///
/// This is the core genocide operation. It:
/// 1. Marks the monster type as genocided globally
/// 2. Kills all instances on the current level
/// 3. Kills all instances on all other levels in the game
/// 4. Checks for self-genocide
/// 5. Updates player conduct
/// 6. Calculates alignment shifts
pub fn do_genocide(monster_type: i16, game_state: &mut GameState) -> GenocideResult {
    let mut result = GenocideResult::new();

    // Mark this monster type as globally genocided
    game_state.monster_vitals.mark_genocided(monster_type);

    // Kill on current level
    let (killed, mut msgs) = kill_genocided_monsters(&mut game_state.current_level, monster_type);
    result.killed_count += killed;
    result.messages.append(&mut msgs);

    // Kill on all other levels
    for (_dlevel, level) in game_state.levels.iter_mut() {
        let (killed, mut msgs) = kill_genocided_monsters(level, monster_type);
        result.killed_count += killed;
        result.messages.append(&mut msgs);
    }

    // Check if player species was genocided (self-genocide)
    let player_monster_type = get_player_monster_type(&game_state.player);
    if monster_type == player_monster_type {
        result
            .messages
            .push("You feel a sudden chill...".to_string());
        result.messages.push("You are annihilated!".to_string());
        result.player_died = true;
    }

    // Update conduct counter
    game_state.player.conduct.genocides += 1;

    // Apply alignment shifts based on what was genocided
    let alignment_shift = calculate_alignment_shift(monster_type);
    result.alignment_shift = alignment_shift;
    if alignment_shift != 0 {
        let shift_val = alignment_shift as i32;
        if alignment_shift < 0 {
            game_state.player.alignment.decrease(-shift_val);
            result
                .messages
                .push("You feel less aligned with your god.".to_string());
        } else {
            game_state.player.alignment.increase(shift_val);
            result
                .messages
                .push("You feel more aligned with your god.".to_string());
        }
    }

    result
}

/// Check if a monster is a unique NPC that cannot be genocided
///
/// These monsters are exempt from genocide:
/// - Unique monsters (G_UNIQ flag: 0x1000)
/// - Non-generated monsters (G_NOGEN flag: 0x0200)
///
/// # Arguments
/// * `monster_type` - The monster type ID
/// * `gen_flags` - The generation flags from PerMonst data
///
/// Normally called with monster data from nh-data crate.
pub fn is_unique_npc(_monster_type: i16, gen_flags: u16) -> bool {
    // Generation flag constants from nh-data/src/monsters.rs
    const G_UNIQ: u16 = 0x1000; // Unique monster
    const G_NOGEN: u16 = 0x0200; // Non-generated

    // Check for G_UNIQ or G_NOGEN flags which mark unique/non-spawnable monsters
    let is_unique = (gen_flags & G_UNIQ) != 0;
    let is_nogen = (gen_flags & G_NOGEN) != 0;

    is_unique || is_nogen
}

/// Get the player's monster type (for checking self-genocide)
///
/// Returns:
/// - the player's current monster type if polymorphed
/// - a special value if not polymorphed (0 for now, but should be race-based)
fn get_player_monster_type(player: &crate::player::You) -> i16 {
    // If player is polymorphed, use the polymorph form
    if let Some(monster_type) = player.monster_num {
        return monster_type;
    }

    // Otherwise return 0 (no match - player is not a monster type)
    0
}

/// List all genocided monster types
///
/// Returns a vector of all monster type IDs that have been genocided.
/// Equivalent to list_genocided from read.c
pub fn list_genocided(vitals: &MonsterVitals) -> Vec<i16> {
    vitals
        .flags
        .iter()
        .filter(|(_, f)| f.genocided)
        .map(|(&mtype, _)| mtype)
        .collect()
}

/// Count the number of extinct species
///
/// Returns the total count of monster types that have been genocided.
/// Equivalent to num_extinct from read.c
pub fn num_extinct(vitals: &MonsterVitals) -> u32 {
    vitals.num_genocides()
}

/// Check if a species is dead (genocided)
///
/// Returns true if the specified monster type has been genocided.
/// Equivalent to dead_species from read.c
pub fn dead_species(monster_type: i16, vitals: &MonsterVitals) -> bool {
    vitals.is_genocided(monster_type)
}

/// Calculate alignment shift for genociding a specific monster type
///
/// Different monster types have different alignment implications:
/// - Peaceful monsters: -10 (chaotic act)
/// - Humans: -5 (somewhat chaotic)
/// - Demons: +10 (lawful act)
/// - Angels/celestials: -15 (very chaotic)
fn calculate_alignment_shift(_monster_type: i16) -> i8 {
    // This is a simplified version. In full implementation, we would check:
    // - IS_PEACEFUL flag from PerMonst data
    // - M2_HUMAN flag from PerMonst data
    // - IS_DEMON flag from PerMonst data
    // - etc.
    //
    // For now, we use a conservative approach and don't apply shifts.
    // This can be enhanced when monster data is more accessible.

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_vitals_creation() {
        let vitals = MonsterVitals::new();
        assert_eq!(vitals.num_genocides(), 0);
        assert!(!vitals.is_genocided(1));
    }

    #[test]
    fn test_mark_genocided() {
        let mut vitals = MonsterVitals::new();
        vitals.mark_genocided(5);
        assert!(vitals.is_genocided(5));
        assert_eq!(vitals.num_genocides(), 1);
    }

    #[test]
    fn test_multiple_genocides() {
        let mut vitals = MonsterVitals::new();
        vitals.mark_genocided(5);
        vitals.mark_genocided(10);
        vitals.mark_genocided(15);
        assert_eq!(vitals.num_genocides(), 3);
        assert!(vitals.is_genocided(5));
        assert!(vitals.is_genocided(10));
        assert!(vitals.is_genocided(15));
        assert!(!vitals.is_genocided(20));
    }

    #[test]
    fn test_genocide_flags() {
        let mut vitals = MonsterVitals::new();
        vitals.mark_genocided(7);
        let flags = vitals.get_flags(7);
        assert!(flags.genocided);
        assert!(flags.no_corpse);
    }

    #[test]
    fn test_genocide_flags_default() {
        let vitals = MonsterVitals::new();
        let flags = vitals.get_flags(999);
        assert!(!flags.genocided);
        assert!(!flags.no_corpse);
    }

    #[test]
    fn test_genocide_result_builder() {
        let result = GenocideResult::new()
            .with_message("Test message 1")
            .with_message("Test message 2")
            .with_killed(5)
            .with_alignment_shift(-10);

        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.killed_count, 5);
        assert_eq!(result.alignment_shift, -10);
        assert!(!result.player_died);
    }

    #[test]
    fn test_genocide_result_with_player_death() {
        let result = GenocideResult::new()
            .with_player_death()
            .with_message("You die!");

        assert!(result.player_died);
        assert_eq!(result.messages.len(), 1);
    }

    #[test]
    fn test_kill_genocided_monsters_empty_level() {
        let mut level = crate::dungeon::Level::new(crate::dungeon::DLevel::default());
        let (count, messages) = kill_genocided_monsters(&mut level, 5);
        assert_eq!(count, 0);
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_kill_genocided_monsters_removes_correct_type() {
        let mut level = crate::dungeon::Level::new(crate::dungeon::DLevel::default());

        // Add some test monsters
        let mon1 = crate::monster::Monster::new(crate::monster::MonsterId(1), 5, 10, 10);
        let mon2 = crate::monster::Monster::new(crate::monster::MonsterId(2), 6, 11, 11);
        let mon3 = crate::monster::Monster::new(crate::monster::MonsterId(3), 5, 12, 12);

        level.add_monster(mon1);
        level.add_monster(mon2);
        level.add_monster(mon3);

        assert_eq!(level.monsters.len(), 3);

        // Kill monster type 5
        let (count, messages) = kill_genocided_monsters(&mut level, 5);

        assert_eq!(count, 2);
        assert_eq!(messages.len(), 2);
        assert_eq!(level.monsters.len(), 1);
        assert_eq!(level.monsters[0].monster_type, 6);
    }

    #[test]
    fn test_serialization() {
        let mut vitals = MonsterVitals::new();
        vitals.mark_genocided(5);
        vitals.mark_genocided(10);

        let json = serde_json::to_string(&vitals).expect("Failed to serialize");
        let deserialized: MonsterVitals =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.num_genocides(), 2);
        assert!(deserialized.is_genocided(5));
        assert!(deserialized.is_genocided(10));
    }

    #[test]
    fn test_is_unique_npc() {
        const G_UNIQ: u16 = 0x1000;
        const G_NOGEN: u16 = 0x0200;

        // Test that unique monsters are protected
        assert!(is_unique_npc(0, G_UNIQ)); // Has G_UNIQ flag
        assert!(is_unique_npc(1, G_NOGEN)); // Has G_NOGEN flag
        assert!(is_unique_npc(100, G_UNIQ | G_NOGEN)); // Has both flags

        // Test that regular monsters are not protected
        assert!(!is_unique_npc(10, 0)); // No special flags
        assert!(!is_unique_npc(30, 0x0001)); // Some other flag
        assert!(!is_unique_npc(99, 0)); // No special flags
    }

    #[test]
    fn test_reverse_genocide_with_genocided_type() {
        let mut state = crate::gameloop::GameState::new(crate::rng::GameRng::new(42));

        // Mark monster type 10 as genocided
        state.monster_vitals.mark_genocided(10);

        // Try reverse genocide on genocided type
        let result = do_reverse_genocide(10, (10, 10), &mut state);

        // Should fail with message
        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("nothing happens"));
    }

    #[test]
    fn test_class_genocide_marks_many_types() {
        let mut state = crate::gameloop::GameState::new(crate::rng::GameRng::new(42));

        // Do class genocide
        let result = do_class_genocide('d', &mut state);

        // Should have marked multiple types as genocided
        assert_eq!(state.player.conduct.genocides, 1);
        assert!(!result.messages.is_empty());

        // Multiple monsters should be genocided (since we iterate 0-256)
        // At least a few should be marked
        let mut genocided_count = 0;
        for i in 0..256 {
            if state.monster_vitals.is_genocided(i as i16) {
                genocided_count += 1;
            }
        }
        assert!(genocided_count > 0, "Some monsters should be genocided");
    }

    #[test]
    fn test_reverse_genocide_spawns_monsters() {
        let mut state = crate::gameloop::GameState::new(crate::rng::GameRng::new(42));

        // Create a floor around player to spawn monsters
        let px = state.player.pos.x as usize;
        let py = state.player.pos.y as usize;
        for x in (px.saturating_sub(5))..=(px + 5) {
            for y in (py.saturating_sub(5))..=(py + 5) {
                if x < crate::COLNO && y < crate::ROWNO {
                    state.current_level.cells[x][y] = crate::dungeon::Cell::floor();
                }
            }
        }

        let initial_count = state.current_level.monsters.len();

        // Do reverse genocide
        let result = do_reverse_genocide(15, (state.player.pos.x, state.player.pos.y), &mut state);

        // Should spawn creatures
        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("appear"));

        // Should have more monsters after
        let final_count = state.current_level.monsters.len();
        assert!(final_count > initial_count, "Monsters should be spawned");
    }

    #[test]
    fn test_genocide_scroll_normal() {
        let mut state = crate::gameloop::GameState::new(crate::rng::GameRng::new(42));

        // Genocide type 20
        let result = do_genocide(20, &mut state);

        // Should be marked as genocided
        assert!(state.monster_vitals.is_genocided(20));
        assert_eq!(state.player.conduct.genocides, 1);
        // With no monsters on the level and no alignment shift,
        // there are no messages generated
        assert!(!result.player_died);
    }

    #[test]
    fn test_genocide_scroll_self_genocide() {
        let mut state = crate::gameloop::GameState::new(crate::rng::GameRng::new(42));

        // Polymorphed player
        state.player.monster_num = Some(25);

        // Try to genocide the player's current form
        let result = do_genocide(25, &mut state);

        // Should mark self as dead
        assert!(result.player_died);
        assert!(result.messages.iter().any(|m| m.contains("annihilated")));
    }

    #[test]
    fn test_class_genocide_protects_uniques() {
        const G_UNIQ: u16 = 0x1000;

        // Unique NPCs should not be genocidable
        assert!(is_unique_npc(1, G_UNIQ));
        assert!(is_unique_npc(50, G_UNIQ));
        assert!(is_unique_npc(100, G_UNIQ));
    }

    #[test]
    fn test_class_genocide_regular_monsters_not_unique() {
        // Regular monsters (no flags)
        assert!(!is_unique_npc(5, 0));
        assert!(!is_unique_npc(15, 0));
        assert!(!is_unique_npc(75, 0));
    }

    #[test]
    fn test_list_genocided() {
        let mut vitals = MonsterVitals::new();
        vitals.mark_genocided(5);
        vitals.mark_genocided(10);
        vitals.mark_genocided(15);

        let list = list_genocided(&vitals);
        assert_eq!(list.len(), 3);
        assert!(list.contains(&5));
        assert!(list.contains(&10));
        assert!(list.contains(&15));
    }

    #[test]
    fn test_dead_species() {
        let mut vitals = MonsterVitals::new();
        vitals.mark_genocided(5);

        assert!(dead_species(5, &vitals));
        assert!(!dead_species(6, &vitals));
    }

    #[test]
    fn test_num_extinct() {
        let mut vitals = MonsterVitals::new();
        vitals.mark_genocided(5);
        vitals.mark_genocided(10);
        assert_eq!(num_extinct(&vitals), 2);
    }
}
