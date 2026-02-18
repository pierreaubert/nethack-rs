//! Convergence reporting — aggregates diffs into human-readable and
//! machine-readable reports.

use crate::diff::{Severity, StateDiff};
use serde::{Deserialize, Serialize};

/// Summary of a convergence comparison session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceReport {
    /// Descriptive label for this report (e.g. "rest-only seed=42 1000 turns").
    pub label: String,
    /// Seed used.
    pub seed: u64,
    /// Total turns executed.
    pub total_turns: u64,
    /// Turn at which first critical diff appeared (None = no critical diffs).
    pub first_critical_turn: Option<u64>,
    /// Total number of per-turn snapshots that had any diff.
    pub turns_with_diffs: u64,
    /// Aggregate diff counts by severity.
    pub critical_count: u64,
    pub major_count: u64,
    pub minor_count: u64,
    /// Per-turn diff details (turn number, diffs).
    pub turn_diffs: Vec<TurnDiffEntry>,
}

/// Diffs for a single turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnDiffEntry {
    pub turn: u64,
    pub diffs: Vec<StateDiff>,
}

impl ConvergenceReport {
    /// Create a new empty report.
    pub fn new(label: String, seed: u64) -> Self {
        Self {
            label,
            seed,
            total_turns: 0,
            first_critical_turn: None,
            turns_with_diffs: 0,
            critical_count: 0,
            major_count: 0,
            minor_count: 0,
            turn_diffs: Vec::new(),
        }
    }

    /// Record diffs for a given turn.
    pub fn add_turn(&mut self, turn: u64, diffs: Vec<StateDiff>) {
        self.total_turns = turn + 1;

        if diffs.is_empty() {
            return;
        }

        self.turns_with_diffs += 1;

        for d in &diffs {
            match d.severity {
                Severity::Critical => {
                    self.critical_count += 1;
                    if self.first_critical_turn.is_none() {
                        self.first_critical_turn = Some(turn);
                    }
                }
                Severity::Major => self.major_count += 1,
                Severity::Minor => self.minor_count += 1,
            }
        }

        self.turn_diffs.push(TurnDiffEntry { turn, diffs });
    }

    /// True if no critical diffs were found.
    pub fn passed(&self) -> bool {
        self.critical_count == 0
    }

    /// Print a human-readable summary to stdout.
    pub fn print_summary(&self) {
        println!("\n============================================================");
        println!("Convergence Report: {}", self.label);
        println!("Seed: {}, Turns: {}", self.seed, self.total_turns);
        println!(
            "Result: {}",
            if self.passed() { "PASS" } else { "FAIL" }
        );
        println!(
            "Diffs: {} critical, {} major, {} minor",
            self.critical_count, self.major_count, self.minor_count
        );
        println!(
            "Turns with diffs: {}/{}",
            self.turns_with_diffs, self.total_turns
        );

        if let Some(t) = self.first_critical_turn {
            println!("First critical diff at turn {}", t);
        }

        // Print first few diff entries
        let show = self.turn_diffs.len().min(10);
        if show > 0 {
            println!("\nFirst {} turns with diffs:", show);
            for entry in &self.turn_diffs[..show] {
                println!("  Turn {}:", entry.turn);
                for d in &entry.diffs {
                    println!("    {}", d);
                }
            }
            if self.turn_diffs.len() > show {
                println!("  ... and {} more turns with diffs", self.turn_diffs.len() - show);
            }
        }

        println!("============================================================\n");
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }
}
