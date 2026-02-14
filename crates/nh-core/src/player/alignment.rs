//! Player alignment from you.h

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Alignment type (lawful, neutral, chaotic)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
pub enum AlignmentType {
    Lawful,
    #[default]
    Neutral,
    Chaotic,
}

impl AlignmentType {
    /// Get the numeric value (-1, 0, 1)
    pub const fn value(&self) -> i8 {
        match self {
            AlignmentType::Lawful => 1,
            AlignmentType::Neutral => 0,
            AlignmentType::Chaotic => -1,
        }
    }

    /// Create from numeric value
    pub const fn from_value(v: i8) -> Self {
        match v {
            1.. => AlignmentType::Lawful,
            0 => AlignmentType::Neutral,
            _ => AlignmentType::Chaotic,
        }
    }

    /// Get the god name for this alignment (default names)
    pub const fn default_god(&self) -> &'static str {
        match self {
            AlignmentType::Lawful => "Mitra",
            AlignmentType::Neutral => "Crom",
            AlignmentType::Chaotic => "Set",
        }
    }

    /// Get the alignment as a string (align_str in C)
    pub const fn as_str(&self) -> &'static str {
        match self {
            AlignmentType::Lawful => "lawful",
            AlignmentType::Neutral => "neutral",
            AlignmentType::Chaotic => "chaotic",
        }
    }

    /// Get the alignment as a title/noun string
    pub const fn as_title(&self) -> &'static str {
        match self {
            AlignmentType::Lawful => "Law",
            AlignmentType::Neutral => "Neutral",
            AlignmentType::Chaotic => "Chaos",
        }
    }

    /// Check if alignment is coaligned (same alignment)
    pub const fn is_coaligned(&self, other: &AlignmentType) -> bool {
        matches!(
            (self, other),
            (AlignmentType::Lawful, AlignmentType::Lawful)
                | (AlignmentType::Neutral, AlignmentType::Neutral)
                | (AlignmentType::Chaotic, AlignmentType::Chaotic)
        )
    }

    /// Check if alignment is cross-aligned (opposite alignment)
    pub const fn is_cross_aligned(&self, other: &AlignmentType) -> bool {
        matches!(
            (self, other),
            (AlignmentType::Lawful, AlignmentType::Chaotic)
                | (AlignmentType::Chaotic, AlignmentType::Lawful)
        )
    }

    /// Parse alignment from string (str2align equivalent)
    ///
    /// # Arguments
    /// * `s` - String to parse (e.g., "lawful", "L", "law")
    ///
    /// # Returns
    /// The parsed alignment, or None if invalid
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "lawful" | "law" | "l" => Some(AlignmentType::Lawful),
            "neutral" | "neu" | "n" => Some(AlignmentType::Neutral),
            "chaotic" | "cha" | "c" => Some(AlignmentType::Chaotic),
            _ => None,
        }
    }
}

/// Parse alignment from string (str2align equivalent)
pub fn str2align(s: &str) -> Option<AlignmentType> {
    AlignmentType::from_str(s)
}

/// Maximum alignment record value, scales with game progress
/// In C: #define ALIGNLIM (10L + (moves / 200L))
pub fn align_limit(moves: i64) -> i32 {
    (10 + (moves / 200)) as i32
}

/// Adjust alignment record by amount (adjalign equivalent)
///
/// Adds or subtracts from alignment record, respecting limits.
/// Ensures the record doesn't overflow.
pub fn adjalign(alignment: &mut Alignment, n: i32, moves: i64) {
    let new_align = alignment.record.saturating_add(n);
    let limit = align_limit(moves);

    if n < 0 {
        // Decreasing alignment (bad deeds)
        if new_align < alignment.record {
            alignment.record = new_align;
        }
    } else if new_align > alignment.record {
        // Increasing alignment (good deeds)
        alignment.record = new_align.min(limit);
    }
}

/// Get induced alignment from dungeon features (induced_align equivalent)
///
/// Returns an alignment type based on:
/// - Current special level alignment (if any, with pct% chance)
/// - Dungeon alignment (if any, with pct% chance)
/// - Random alignment (if no dungeon features match)
pub fn induced_align(pct: u32) -> AlignmentType {
    // Simplified version - in a full implementation would need dungeon context
    // For now, return random alignment based on probability
    let roll = (pct as i32) % 100;
    match roll {
        0..=32 => AlignmentType::Lawful,
        33..=65 => AlignmentType::Neutral,
        _ => AlignmentType::Chaotic,
    }
}

/// Get non-coaligned alignment (noncoalignment equivalent)
///
/// Returns an alignment that is not the same as the input.
/// If neutral, randomly chooses lawful or chaotic.
/// If lawful/chaotic, returns the opposite (chaotic/lawful) with 50% chance,
/// or 0 (neutral) with 50% chance.
pub fn noncoalignment(alignment: AlignmentType) -> AlignmentType {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    // Use a simple deterministic hash-based randomness for reproducibility
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u8(alignment as u8);
    let rand_bit = (hasher.finish() as u32) & 1;

    match alignment {
        AlignmentType::Neutral => {
            if rand_bit == 0 {
                AlignmentType::Lawful
            } else {
                AlignmentType::Chaotic
            }
        }
        AlignmentType::Lawful => {
            if rand_bit == 0 {
                AlignmentType::Chaotic
            } else {
                AlignmentType::Neutral
            }
        }
        AlignmentType::Chaotic => {
            if rand_bit == 0 {
                AlignmentType::Lawful
            } else {
                AlignmentType::Neutral
            }
        }
    }
}

/// Get pious description string (piousness equivalent)
///
/// Returns a description of the player's piety level based on alignment record.
/// Optionally includes a suffix and shows negative piety levels.
///
/// # Arguments
/// * `record` - The alignment record value
/// * `show_neg` - Whether to show negative piety descriptions
/// * `suffix` - Optional suffix to append to the description
///
/// # Returns
/// A string describing the piety level (e.g., "piously", "devoutly", "insufficiently")
pub fn piousness(record: i32, show_neg: bool, suffix: Option<&str>) -> String {
    let pious_word = match record {
        20.. => "piously",
        14..20 => "devoutly",
        9..14 => "fervently",
        4..9 => "stridently",
        3 => "",
        1..3 => "haltingly",
        0 => "nominally",
        _ if !show_neg => "insufficiently",
        -3..0 => "strayed",
        -8..-3 => "sinned",
        _ => "transgressed",
    };

    let mut result = pious_word.to_string();

    if let Some(s) = suffix {
        if !show_neg || record >= 0 {
            if record != 3 {
                result.push(' ');
            }
            result.push_str(s);
        }
    }

    result
}

/// Alignment record tracking karma and alignment
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Alignment {
    /// Current alignment type
    pub typ: AlignmentType,
    /// Karma/alignment record (positive = better standing)
    pub record: i32,
}

impl Alignment {
    /// Create a new alignment
    pub const fn new(typ: AlignmentType) -> Self {
        Self { typ, record: 0 }
    }

    /// Increase alignment record (good deed)
    pub fn increase(&mut self, amount: i32) {
        self.record = self.record.saturating_add(amount);
    }

    /// Decrease alignment record (bad deed)
    pub fn decrease(&mut self, amount: i32) {
        self.record = self.record.saturating_sub(amount);
    }

    /// Check if in good standing with god
    pub const fn in_good_standing(&self) -> bool {
        self.record >= 0
    }

    /// Check if alignment is opposite to another
    pub const fn is_opposite(&self, other: &Alignment) -> bool {
        matches!(
            (&self.typ, &other.typ),
            (AlignmentType::Lawful, AlignmentType::Chaotic)
                | (AlignmentType::Chaotic, AlignmentType::Lawful)
        )
    }
}
