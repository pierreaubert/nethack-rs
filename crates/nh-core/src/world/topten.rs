//! High score system (topten.c)
//!
//! Handles recording and displaying high scores.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Maximum number of high scores to keep
pub const MAX_SCORES: usize = 100;

/// A single high score entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    /// Player name
    pub name: String,
    /// Final score
    pub score: i64,
    /// Deepest dungeon level reached
    pub max_dlevel: i32,
    /// Player level at death/ascension
    pub player_level: u8,
    /// Player role (e.g., "Valkyrie")
    pub role: String,
    /// Player race (e.g., "Human")
    pub race: String,
    /// Player gender
    pub gender: String,
    /// Player alignment
    pub alignment: String,
    /// How the game ended
    pub death_reason: String,
    /// Whether player ascended
    pub ascended: bool,
    /// Turn count at end
    pub turns: u64,
    /// Real time played (seconds)
    pub realtime: u64,
    /// Timestamp when game ended
    pub timestamp: u64,
    /// Game version
    pub version: String,
}

impl ScoreEntry {
    /// Create a new score entry
    pub fn new(
        name: String,
        score: i64,
        max_dlevel: i32,
        player_level: u8,
        role: String,
        race: String,
        gender: String,
        alignment: String,
        death_reason: String,
        ascended: bool,
        turns: u64,
    ) -> Self {
        Self {
            name,
            score,
            max_dlevel,
            player_level,
            role,
            race,
            gender,
            alignment,
            death_reason,
            ascended,
            turns,
            realtime: 0,
            timestamp: crate::world::save::current_timestamp(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Format the score entry for display
    pub fn format_short(&self) -> String {
        format!(
            "{:>8} {} the {} {} {} on level {}",
            self.score, self.name, self.alignment, self.race, self.role, self.max_dlevel
        )
    }

    /// Format the score entry with death reason
    pub fn format_full(&self) -> String {
        if self.ascended {
            format!(
                "{:>8} {} the {} {} {}, ascended after {} turns",
                self.score, self.name, self.alignment, self.race, self.role, self.turns
            )
        } else {
            format!(
                "{:>8} {} the {} {} {}, {} on level {} (turn {})",
                self.score, self.name, self.alignment, self.race, self.role,
                self.death_reason, self.max_dlevel, self.turns
            )
        }
    }
}

/// High score table
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HighScores {
    pub entries: Vec<ScoreEntry>,
}

impl HighScores {
    /// Create a new empty high score table
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Load high scores from file
    pub fn load(path: &Path) -> Result<Self, TopTenError> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let file = File::open(path)
            .map_err(|e| TopTenError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader)
            .map_err(|e| TopTenError::ParseError(e.to_string()))
    }

    /// Save high scores to file
    pub fn save(&self, path: &Path) -> Result<(), TopTenError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TopTenError::IoError(e.to_string()))?;
        }

        let file = File::create(path)
            .map_err(|e| TopTenError::IoError(e.to_string()))?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| TopTenError::IoError(e.to_string()))
    }

    /// Add a new score entry, maintaining sorted order
    /// Returns the rank (1-indexed) if the score made it to the list
    pub fn add_score(&mut self, entry: ScoreEntry) -> Option<usize> {
        // Find insertion position (sorted by score descending)
        let pos = self.entries
            .iter()
            .position(|e| entry.score > e.score)
            .unwrap_or(self.entries.len());

        // Check if score makes the cut
        if pos >= MAX_SCORES {
            return None;
        }

        self.entries.insert(pos, entry);

        // Trim to max size
        if self.entries.len() > MAX_SCORES {
            self.entries.truncate(MAX_SCORES);
        }

        Some(pos + 1) // Return 1-indexed rank
    }

    /// Get the top N scores
    pub fn top(&self, n: usize) -> &[ScoreEntry] {
        let end = n.min(self.entries.len());
        &self.entries[..end]
    }

    /// Get scores for a specific player
    pub fn player_scores(&self, name: &str) -> Vec<&ScoreEntry> {
        self.entries
            .iter()
            .filter(|e| e.name.eq_ignore_ascii_case(name))
            .collect()
    }

    /// Get the highest score
    pub fn highest(&self) -> Option<&ScoreEntry> {
        self.entries.first()
    }

    /// Check if a score would make the high score list
    pub fn would_qualify(&self, score: i64) -> bool {
        self.entries.len() < MAX_SCORES || 
            self.entries.last().map(|e| score > e.score).unwrap_or(true)
    }

    /// Get rank for a given score (without adding it)
    pub fn rank_for_score(&self, score: i64) -> usize {
        self.entries
            .iter()
            .position(|e| score > e.score)
            .map(|p| p + 1)
            .unwrap_or(self.entries.len() + 1)
    }
}

/// High score error types
#[derive(Debug, Clone)]
pub enum TopTenError {
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for TopTenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopTenError::IoError(e) => write!(f, "IO error: {}", e),
            TopTenError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for TopTenError {}

/// Get the default high scores file path
pub fn get_scores_path() -> std::path::PathBuf {
    std::path::PathBuf::from("scores.json")
}

/// Calculate final score based on game state
pub fn calculate_score(
    gold: i64,
    max_dlevel: i32,
    player_level: u8,
    ascended: bool,
    amulet_obtained: bool,
    conducts_kept: u32,
) -> i64 {
    let mut score = gold;

    // Bonus for depth reached
    score += (max_dlevel as i64) * 100;

    // Bonus for player level
    score += (player_level as i64) * 50;

    // Huge bonus for ascension
    if ascended {
        score += 50000;
        // Extra bonus for conducts
        score += (conducts_kept as i64) * 1000;
    }

    // Bonus for obtaining the Amulet
    if amulet_obtained {
        score += 10000;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entry(name: &str, score: i64) -> ScoreEntry {
        ScoreEntry::new(
            name.to_string(),
            score,
            10,
            14,
            "Valkyrie".to_string(),
            "Human".to_string(),
            "female".to_string(),
            "lawful".to_string(),
            "killed by a dragon".to_string(),
            false,
            5000,
        )
    }

    #[test]
    fn test_add_score() {
        let mut scores = HighScores::new();
        
        let rank = scores.add_score(test_entry("Alice", 1000));
        assert_eq!(rank, Some(1));
        
        let rank = scores.add_score(test_entry("Bob", 2000));
        assert_eq!(rank, Some(1)); // Bob has higher score
        
        let rank = scores.add_score(test_entry("Charlie", 500));
        assert_eq!(rank, Some(3)); // Charlie has lowest score
        
        assert_eq!(scores.entries.len(), 3);
        assert_eq!(scores.entries[0].name, "Bob");
        assert_eq!(scores.entries[1].name, "Alice");
        assert_eq!(scores.entries[2].name, "Charlie");
    }

    #[test]
    fn test_top_scores() {
        let mut scores = HighScores::new();
        scores.add_score(test_entry("A", 100));
        scores.add_score(test_entry("B", 200));
        scores.add_score(test_entry("C", 300));
        
        let top2 = scores.top(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].score, 300);
        assert_eq!(top2[1].score, 200);
    }

    #[test]
    fn test_would_qualify() {
        let mut scores = HighScores::new();
        
        // Empty list - any score qualifies
        assert!(scores.would_qualify(0));
        
        scores.add_score(test_entry("A", 100));
        assert!(scores.would_qualify(50)); // Still room
        assert!(scores.would_qualify(200)); // Higher score
    }

    #[test]
    fn test_calculate_score() {
        // Basic score
        let score = calculate_score(1000, 10, 14, false, false, 0);
        assert!(score > 1000); // Should have bonuses
        
        // Ascension bonus
        let ascended_score = calculate_score(1000, 50, 30, true, true, 3);
        assert!(ascended_score > 60000); // Big bonus for ascension
    }

    #[test]
    fn test_format_entry() {
        let entry = test_entry("TestPlayer", 12345);
        let short = entry.format_short();
        assert!(short.contains("12345"));
        assert!(short.contains("TestPlayer"));
        
        let full = entry.format_full();
        assert!(full.contains("dragon"));
    }
}
