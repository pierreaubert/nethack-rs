//! Game state snapshots for C vs Rust comparison.
//!
//! Snapshots capture a normalized view of game state that can be compared
//! across both engines regardless of internal representation differences.

use serde::{Deserialize, Serialize};

/// Complete game state snapshot at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub turn: u64,
    pub player: PlayerSnapshot,
    pub inventory: Vec<ItemSnapshot>,
    pub monsters: Vec<MonsterSnapshot>,
    /// Source engine identifier ("rust" or "c")
    pub source: String,
}

/// Player character state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub energy: i32,
    pub energy_max: i32,
    pub armor_class: i32,
    pub gold: i32,
    pub exp_level: i32,
    pub nutrition: i32,
    pub strength: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub charisma: i32,
    pub alive: bool,
    pub dungeon_level: i32,
    pub dungeon_num: i32,
    /// Active status effects (e.g. "confused", "stunned", "blind")
    pub status_effects: Vec<String>,
}

/// Inventory item snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSnapshot {
    /// Object type index
    pub object_type: i16,
    /// Object class name (e.g. "Weapon", "Armor")
    pub class: String,
    pub quantity: i32,
    pub enchantment: i8,
    pub buc: String,
    pub weight: u32,
}

/// Monster snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSnapshot {
    /// Monster type index
    pub monster_type: i16,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub peaceful: bool,
    pub sleeping: bool,
    pub alive: bool,
}

/// RNG trace entry for comparing random number generation sequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngTraceEntry {
    pub seq: u64,
    pub func: String,
    pub arg: u64,
    pub result: u64,
}
