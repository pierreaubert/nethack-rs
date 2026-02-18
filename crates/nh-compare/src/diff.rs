//! Snapshot diffing and RNG trace comparison.
//!
//! Compares two `GameSnapshot`s field-by-field, producing a list of
//! `StateDiff` entries with severity classification.

use crate::snapshot::{GameSnapshot, ItemSnapshot, MonsterSnapshot, RngTraceEntry};
use serde::{Deserialize, Serialize};

/// How important a difference is for convergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Minor,
    Major,
    Critical,
}

impl core::fmt::Display for Severity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Severity::Minor => write!(f, "MINOR"),
            Severity::Major => write!(f, "MAJOR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A single difference between two snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub severity: Severity,
    pub field: String,
    pub rust_value: String,
    pub c_value: String,
}

impl core::fmt::Display for StateDiff {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[{}] {}: rust={}, c={}",
            self.severity, self.field, self.rust_value, self.c_value
        )
    }
}

/// Compare two game snapshots and return all differences.
pub fn diff_snapshots(rust: &GameSnapshot, c: &GameSnapshot) -> Vec<StateDiff> {
    let mut diffs = Vec::new();

    // Turn count
    if rust.turn != c.turn {
        diffs.push(StateDiff {
            severity: Severity::Major,
            field: "turn".into(),
            rust_value: rust.turn.to_string(),
            c_value: c.turn.to_string(),
        });
    }

    // Player state (critical fields)
    diff_field(
        &mut diffs,
        Severity::Critical,
        "player.x",
        rust.player.x,
        c.player.x,
    );
    diff_field(
        &mut diffs,
        Severity::Critical,
        "player.y",
        rust.player.y,
        c.player.y,
    );
    diff_field(
        &mut diffs,
        Severity::Critical,
        "player.alive",
        rust.player.alive,
        c.player.alive,
    );
    diff_field(
        &mut diffs,
        Severity::Critical,
        "player.hp",
        rust.player.hp,
        c.player.hp,
    );
    diff_field(
        &mut diffs,
        Severity::Critical,
        "player.hp_max",
        rust.player.hp_max,
        c.player.hp_max,
    );

    // Minor player fields
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.energy",
        rust.player.energy,
        c.player.energy,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.energy_max",
        rust.player.energy_max,
        c.player.energy_max,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.armor_class",
        rust.player.armor_class,
        c.player.armor_class,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.gold",
        rust.player.gold,
        c.player.gold,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.exp_level",
        rust.player.exp_level,
        c.player.exp_level,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.nutrition",
        rust.player.nutrition,
        c.player.nutrition,
    );

    // Attributes (minor)
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.strength",
        rust.player.strength,
        c.player.strength,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.intelligence",
        rust.player.intelligence,
        c.player.intelligence,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.wisdom",
        rust.player.wisdom,
        c.player.wisdom,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.dexterity",
        rust.player.dexterity,
        c.player.dexterity,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.constitution",
        rust.player.constitution,
        c.player.constitution,
    );
    diff_field(
        &mut diffs,
        Severity::Minor,
        "player.charisma",
        rust.player.charisma,
        c.player.charisma,
    );

    // Status effects (minor)
    if rust.player.status_effects != c.player.status_effects {
        diffs.push(StateDiff {
            severity: Severity::Minor,
            field: "player.status_effects".into(),
            rust_value: format!("{:?}", rust.player.status_effects),
            c_value: format!("{:?}", c.player.status_effects),
        });
    }

    // Inventory (major)
    diff_field(
        &mut diffs,
        Severity::Major,
        "inventory.count",
        rust.inventory.len() as i32,
        c.inventory.len() as i32,
    );
    diff_items(&mut diffs, &rust.inventory, &c.inventory);

    // Monsters (major)
    diff_field(
        &mut diffs,
        Severity::Major,
        "monsters.count",
        rust.monsters.len() as i32,
        c.monsters.len() as i32,
    );
    diff_monsters(&mut diffs, &rust.monsters, &c.monsters);

    diffs
}

fn diff_field<T: PartialEq + core::fmt::Display>(
    diffs: &mut Vec<StateDiff>,
    severity: Severity,
    field: &str,
    rust_val: T,
    c_val: T,
) {
    if rust_val != c_val {
        diffs.push(StateDiff {
            severity,
            field: field.into(),
            rust_value: rust_val.to_string(),
            c_value: c_val.to_string(),
        });
    }
}

fn diff_items(diffs: &mut Vec<StateDiff>, rust: &[ItemSnapshot], c: &[ItemSnapshot]) {
    let count = rust.len().min(c.len());
    for i in 0..count {
        let prefix = format!("inventory[{}]", i);
        if rust[i].object_type != c[i].object_type {
            diffs.push(StateDiff {
                severity: Severity::Major,
                field: format!("{}.object_type", prefix),
                rust_value: rust[i].object_type.to_string(),
                c_value: c[i].object_type.to_string(),
            });
        }
        if rust[i].enchantment != c[i].enchantment {
            diffs.push(StateDiff {
                severity: Severity::Major,
                field: format!("{}.enchantment", prefix),
                rust_value: rust[i].enchantment.to_string(),
                c_value: c[i].enchantment.to_string(),
            });
        }
        if rust[i].buc != c[i].buc {
            diffs.push(StateDiff {
                severity: Severity::Major,
                field: format!("{}.buc", prefix),
                rust_value: rust[i].buc.clone(),
                c_value: c[i].buc.clone(),
            });
        }
    }
}

fn diff_monsters(diffs: &mut Vec<StateDiff>, rust: &[MonsterSnapshot], c: &[MonsterSnapshot]) {
    let count = rust.len().min(c.len());
    for i in 0..count {
        let prefix = format!("monster[{}]", i);
        if rust[i].monster_type != c[i].monster_type {
            diffs.push(StateDiff {
                severity: Severity::Major,
                field: format!("{}.type", prefix),
                rust_value: rust[i].monster_type.to_string(),
                c_value: c[i].monster_type.to_string(),
            });
        }
        if rust[i].x != c[i].x || rust[i].y != c[i].y {
            diffs.push(StateDiff {
                severity: Severity::Major,
                field: format!("{}.pos", prefix),
                rust_value: format!("({},{})", rust[i].x, rust[i].y),
                c_value: format!("({},{})", c[i].x, c[i].y),
            });
        }
        if rust[i].hp != c[i].hp {
            diffs.push(StateDiff {
                severity: Severity::Major,
                field: format!("{}.hp", prefix),
                rust_value: rust[i].hp.to_string(),
                c_value: c[i].hp.to_string(),
            });
        }
    }
}

/// Point of divergence in RNG traces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngDivergence {
    /// Call index where divergence first occurs.
    pub call_index: usize,
    /// Context: entries around the divergence point from the Rust trace.
    pub rust_context: Vec<RngTraceEntry>,
    /// Context: entries around the divergence point from the C trace.
    pub c_context: Vec<RngTraceEntry>,
    /// Description of what differs.
    pub description: String,
}

/// Compare two RNG traces and find the first point of divergence.
///
/// Returns `None` if traces match for their shared length.
pub fn compare_rng_traces(
    rust: &[RngTraceEntry],
    c: &[RngTraceEntry],
) -> Option<RngDivergence> {
    let len = rust.len().min(c.len());

    for i in 0..len {
        let r = &rust[i];
        let cv = &c[i];

        if r.func != cv.func || r.arg != cv.arg || r.result != cv.result {
            let context_start = i.saturating_sub(5);
            let context_end = (i + 6).min(len);

            let description = if r.func != cv.func {
                format!(
                    "Function mismatch at call {}: rust={}({}), c={}({})",
                    i, r.func, r.arg, cv.func, cv.arg
                )
            } else if r.arg != cv.arg {
                format!(
                    "Argument mismatch at call {}: {}(rust={}, c={})",
                    i, r.func, r.arg, cv.arg
                )
            } else {
                format!(
                    "Result mismatch at call {}: {}({}) -> rust={}, c={}",
                    i, r.func, r.arg, r.result, cv.result
                )
            };

            return Some(RngDivergence {
                call_index: i,
                rust_context: rust[context_start..context_end].to_vec(),
                c_context: c[context_start..context_end].to_vec(),
                description,
            });
        }
    }

    // Check for length mismatch
    if rust.len() != c.len() {
        return Some(RngDivergence {
            call_index: len,
            rust_context: rust[len.saturating_sub(3)..rust.len().min(len + 3)].to_vec(),
            c_context: c[len.saturating_sub(3)..c.len().min(len + 3)].to_vec(),
            description: format!(
                "Trace length mismatch: rust={} calls, c={} calls",
                rust.len(),
                c.len()
            ),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::*;

    fn empty_player() -> PlayerSnapshot {
        PlayerSnapshot {
            x: 10,
            y: 5,
            hp: 16,
            hp_max: 16,
            energy: 4,
            energy_max: 4,
            armor_class: 10,
            gold: 0,
            exp_level: 1,
            nutrition: 900,
            strength: 16,
            intelligence: 10,
            wisdom: 10,
            dexterity: 12,
            constitution: 14,
            charisma: 8,
            alive: true,
            dungeon_level: 1,
            dungeon_num: 0,
            status_effects: vec![],
        }
    }

    #[test]
    fn test_identical_snapshots_no_diffs() {
        let snap = GameSnapshot {
            turn: 1,
            player: empty_player(),
            inventory: vec![],
            monsters: vec![],
            source: "rust".into(),
        };
        let diffs = diff_snapshots(&snap, &snap);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_position_diff_is_critical() {
        let mut rust = GameSnapshot {
            turn: 1,
            player: empty_player(),
            inventory: vec![],
            monsters: vec![],
            source: "rust".into(),
        };
        let mut c = rust.clone();
        c.source = "c".into();
        rust.player.x = 10;
        c.player.x = 11;

        let diffs = diff_snapshots(&rust, &c);
        assert!(!diffs.is_empty());
        assert_eq!(diffs[0].severity, Severity::Critical);
        assert_eq!(diffs[0].field, "player.x");
    }

    #[test]
    fn test_rng_trace_match() {
        let trace = vec![
            RngTraceEntry { seq: 0, func: "rn2".into(), arg: 6, result: 3 },
            RngTraceEntry { seq: 1, func: "rn2".into(), arg: 10, result: 7 },
        ];
        assert!(compare_rng_traces(&trace, &trace).is_none());
    }

    #[test]
    fn test_rng_trace_divergence() {
        let rust_trace = vec![
            RngTraceEntry { seq: 0, func: "rn2".into(), arg: 6, result: 3 },
            RngTraceEntry { seq: 1, func: "rn2".into(), arg: 10, result: 7 },
        ];
        let c_trace = vec![
            RngTraceEntry { seq: 0, func: "rn2".into(), arg: 6, result: 3 },
            RngTraceEntry { seq: 1, func: "rnd".into(), arg: 10, result: 7 },
        ];
        let div = compare_rng_traces(&rust_trace, &c_trace).unwrap();
        assert_eq!(div.call_index, 1);
        assert!(div.description.contains("Function mismatch"));
    }
}
