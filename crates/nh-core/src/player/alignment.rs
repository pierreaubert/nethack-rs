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
