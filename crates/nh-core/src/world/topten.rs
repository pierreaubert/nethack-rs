//! High score system (topten.c)
//!
//! Handles recording and displaying high scores.
//! Translates NetHack topten.c functions to provide high score tracking,
//! record file management, and score display/filtering.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

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
                self.score,
                self.name,
                self.alignment,
                self.race,
                self.role,
                self.death_reason,
                self.max_dlevel,
                self.turns
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

        let file = File::open(path).map_err(|e| TopTenError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| TopTenError::ParseError(e.to_string()))
    }

    /// Save high scores to file
    pub fn save(&self, path: &Path) -> Result<(), TopTenError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TopTenError::IoError(e.to_string()))?;
        }

        let file = File::create(path).map_err(|e| TopTenError::IoError(e.to_string()))?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self).map_err(|e| TopTenError::IoError(e.to_string()))
    }

    /// Add a new score entry, maintaining sorted order
    /// Returns the rank (1-indexed) if the score made it to the list
    pub fn add_score(&mut self, entry: ScoreEntry) -> Option<usize> {
        // Find insertion position (sorted by score descending)
        let pos = self
            .entries
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
        self.entries.len() < MAX_SCORES
            || self.entries.last().map(|e| score > e.score).unwrap_or(true)
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

/// Main topten function - update high score list with new entry
///
/// Called when the game ends. Adds the player's final score to the high score list,
/// writes it to the record file, and displays the updated rankings.
///
/// # Arguments
/// * `how` - How the game ended (0 = death, 1 = quit, 2 = ascended, etc.)
/// * `when` - Timestamp when the game ended
/// * `entry` - The score entry to add
pub fn topten(
    how: i32,
    when: u64,
    entry: ScoreEntry,
    path: &Path,
) -> Result<Option<usize>, TopTenError> {
    let mut scores = HighScores::load(path).unwrap_or_default();
    let rank = scores.add_score(entry);
    scores.save(path)?;
    Ok(rank)
}

/// Print regular text output (for topten display)
///
/// Equivalent to `topten_print()` - outputs a text line to the topten display.
pub fn topten_print(output: &mut dyn Write, text: &str) -> std::io::Result<()> {
    writeln!(output, "{}", text)
}

/// Print bold text output (for topten display)
///
/// Equivalent to `topten_print_bold()` - outputs a bold/emphasized text line.
pub fn topten_print_bold(output: &mut dyn Write, text: &str) -> std::io::Result<()> {
    writeln!(output, ">>> {} <<<", text)
}

/// Format and print a single score entry with ranking
///
/// Equivalent to `outentry()` - formats a score entry for display with rank number,
/// player name, character class, and death reason/ascension status.
///
/// # Arguments
/// * `rank` - The rank number (1-indexed)
/// * `entry` - The score entry to format
/// * `standout` - Whether to use standout (bold) formatting
pub fn format_score_entry(rank: usize, entry: &ScoreEntry, standout: bool) -> String {
    let prefix = if standout { ">>>" } else { "" };
    let suffix = if standout { "<<<" } else { "" };

    let status = if entry.ascended {
        format!("ascended after {} turns", entry.turns)
    } else {
        format!("{} on level {}", entry.death_reason, entry.max_dlevel)
    };

    let line = format!(
        "{} {:>3}  {:>10} the {} {} {},  {}",
        prefix.trim_end(),
        rank,
        entry.score,
        entry.alignment,
        entry.race,
        entry.role,
        status
    );

    let line = if standout {
        format!("{} {}", line, suffix)
    } else {
        line
    };

    line
}

/// Print formatted score list to output
///
/// Displays a range of scores from the high score list in a nicely formatted table.
pub fn print_score_list(
    output: &mut dyn Write,
    scores: &HighScores,
    start_rank: usize,
    count: usize,
) -> std::io::Result<()> {
    let end = (start_rank + count).min(scores.entries.len());

    writeln!(output, "\n               Top Ten Adventurers\n")?;
    writeln!(
        output,
        "Rank  Score       Character Name              How\n"
    )?;
    writeln!(
        output,
        "----  -------     ----                        ---\n"
    )?;

    for (i, entry) in scores.entries[start_rank..end].iter().enumerate() {
        let rank = start_rank + i + 1;
        let formatted = format_score_entry(rank, entry, false);
        writeln!(output, "{}", formatted)?;
    }

    Ok(())
}

/// Print scores filtered by player name
///
/// Equivalent to `prscore()` - displays scores matching specific filter criteria.
pub fn print_player_scores(
    output: &mut dyn Write,
    scores: &HighScores,
    player_name: Option<&str>,
) -> std::io::Result<()> {
    if let Some(name) = player_name {
        let player_scores = scores.player_scores(name);

        if player_scores.is_empty() {
            writeln!(output, "No scores found for player '{}'", name)?;
        } else {
            writeln!(output, "\nScores for player '{}'\n", name)?;
            for (i, entry) in player_scores.iter().enumerate() {
                let formatted = format_score_entry(i + 1, entry, false);
                writeln!(output, "{}", formatted)?;
            }
        }
    } else {
        print_score_list(output, scores, 0, MAX_SCORES)?;
    }

    Ok(())
}

/// Get a random entry from the high score list
///
/// Equivalent to `get_rnd_toptenentry()` - used for corpses in morgue and statue creation.
/// Returns a random player's name and class from the high scores.
pub fn get_random_topten_entry(scores: &HighScores) -> Option<(String, String)> {
    if scores.entries.is_empty() {
        return None;
    }

    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let random_state = RandomState::new();
    let mut hasher = random_state.build_hasher();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos()
        .hash(&mut hasher);

    let index = (hasher.finish() as usize) % scores.entries.len();
    let entry = &scores.entries[index];

    Some((entry.name.clone(), entry.role.clone()))
}

/// Serialize a score entry to extended log format (xlog format)
///
/// Equivalent to `writexlentry()` - writes entry in tab-separated xlog format
/// used by many NetHack servers for tracking games.
pub fn format_xlog_entry(entry: &ScoreEntry, how: i32) -> String {
    format!(
        "version=3.6\tplayer={}\tscore={}\tlevel={}\tstartime={}\tendtime={}\t\
         role={}\tce={}\talignment={}\tgender={}\trealtime={}\tturn={}\t\
         deathdate={}\tdeath={}",
        entry.name,
        entry.score,
        entry.max_dlevel,
        entry.timestamp,
        entry.timestamp + entry.realtime,
        entry.role,
        entry.race,
        entry.alignment,
        entry.gender,
        entry.realtime,
        entry.turns,
        entry.timestamp,
        entry.death_reason
    )
}

// ============================================================================
// Record File Functions (topten.c equivalents)
// ============================================================================

/// Read a single score entry from a line of text (readentry equivalent)
///
/// Parses a tab-separated or JSON score entry from the record file.
/// Returns None if the line is invalid or empty.
pub fn read_entry(line: &str) -> Option<ScoreEntry> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    // Try JSON format first
    if line.starts_with('{') {
        return serde_json::from_str(line).ok();
    }

    // Try tab-separated xlog format
    let mut entry = ScoreEntry {
        name: String::new(),
        score: 0,
        max_dlevel: 1,
        player_level: 1,
        role: String::new(),
        race: String::new(),
        gender: String::new(),
        alignment: String::new(),
        death_reason: String::new(),
        ascended: false,
        turns: 0,
        realtime: 0,
        timestamp: 0,
        version: String::new(),
    };

    for field in line.split('\t') {
        if let Some((key, value)) = field.split_once('=') {
            match key {
                "player" | "name" => entry.name = value.to_string(),
                "score" | "points" => entry.score = value.parse().unwrap_or(0),
                "level" | "maxlvl" | "deathlev" => entry.max_dlevel = value.parse().unwrap_or(1),
                "hp" | "maxhp" => {} // Ignored for now
                "role" => entry.role = value.to_string(),
                "race" => entry.race = value.to_string(),
                "gender" => entry.gender = value.to_string(),
                "align" | "alignment" => entry.alignment = value.to_string(),
                "death" => entry.death_reason = value.to_string(),
                "turn" | "turns" => entry.turns = value.parse().unwrap_or(0),
                "realtime" => entry.realtime = value.parse().unwrap_or(0),
                "starttime" | "endtime" => entry.timestamp = value.parse().unwrap_or(0),
                "version" => entry.version = value.to_string(),
                "ascended" => entry.ascended = value == "1" || value == "true",
                _ => {}
            }
        }
    }

    if entry.name.is_empty() {
        return None;
    }

    Some(entry)
}

/// Write a single score entry to a line of text (writeentry equivalent)
///
/// Formats a score entry as a JSON line for the record file.
pub fn write_entry(entry: &ScoreEntry) -> String {
    serde_json::to_string(entry).unwrap_or_else(|_| {
        // Fallback to simple format
        format!(
            "{}|{}|{}|{}|{}|{}",
            entry.name, entry.score, entry.role, entry.race, entry.max_dlevel, entry.death_reason
        )
    })
}

/// Write a score entry in xlog format (writexlentry equivalent)
///
/// Formats a score entry as tab-separated xlog format used by NetHack servers.
pub fn write_xl_entry(entry: &ScoreEntry) -> String {
    format!(
        "version={}\tplayer={}\tscore={}\tlevel={}\trole={}\trace={}\talign={}\tgender={}\tturn={}\trealtime={}\tdeath={}",
        entry.version,
        entry.name,
        entry.score,
        entry.max_dlevel,
        entry.role,
        entry.race,
        entry.alignment,
        entry.gender,
        entry.turns,
        entry.realtime,
        entry.death_reason
    )
}

/// Check if the record file exists and is valid (check_recordfile equivalent)
///
/// Verifies that the high score file exists and can be read.
/// Creates an empty file if it doesn't exist.
pub fn check_record_file(path: &Path) -> Result<bool, TopTenError> {
    if !path.exists() {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TopTenError::IoError(e.to_string()))?;
        }
        // Create empty file
        std::fs::write(path, "{\"entries\":[]}")
            .map_err(|e| TopTenError::IoError(e.to_string()))?;
        return Ok(true);
    }

    // Try to read and parse the file
    match HighScores::load(path) {
        Ok(_) => Ok(true),
        Err(e) => {
            // File exists but is corrupted - back it up and create new
            let backup_path = path.with_extension("json.bak");
            let _ = std::fs::rename(path, &backup_path);
            std::fs::write(path, "{\"entries\":[]}")
                .map_err(|e| TopTenError::IoError(e.to_string()))?;
            Err(e)
        }
    }
}

/// Read all entries from a record file
///
/// Reads the entire high score file and returns all entries.
pub fn read_record_file(path: &Path) -> Result<Vec<ScoreEntry>, TopTenError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents =
        std::fs::read_to_string(path).map_err(|e| TopTenError::IoError(e.to_string()))?;

    // Try JSON format first
    if contents.trim().starts_with('{') || contents.trim().starts_with('[') {
        let scores: HighScores =
            serde_json::from_str(&contents).map_err(|e| TopTenError::ParseError(e.to_string()))?;
        return Ok(scores.entries);
    }

    // Try line-by-line xlog format
    let mut entries = Vec::new();
    for line in contents.lines() {
        if let Some(entry) = read_entry(line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Write all entries to a record file
///
/// Writes the entire high score list to the record file.
pub fn write_record_file(path: &Path, entries: &[ScoreEntry]) -> Result<(), TopTenError> {
    let scores = HighScores {
        entries: entries.to_vec(),
    };
    scores.save(path)
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

    // ========================================================================
    // Tests for record file functions (read_entry, write_entry, etc.)
    // ========================================================================

    #[test]
    fn test_read_entry_xlog_format() {
        let line = "player=Alice\tscore=5000\tlevel=10\trole=Valkyrie\trace=Human\talign=lawful\tgender=female\tdeath=killed by a dragon\tturn=1234";
        let entry = read_entry(line);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "Alice");
        assert_eq!(entry.score, 5000);
        assert_eq!(entry.max_dlevel, 10);
        assert_eq!(entry.role, "Valkyrie");
        assert_eq!(entry.race, "Human");
        assert_eq!(entry.alignment, "lawful");
        assert_eq!(entry.death_reason, "killed by a dragon");
        assert_eq!(entry.turns, 1234);
    }

    #[test]
    fn test_read_entry_empty_line() {
        assert!(read_entry("").is_none());
        assert!(read_entry("   ").is_none());
        assert!(read_entry("# comment").is_none());
    }

    #[test]
    fn test_write_entry_json() {
        let entry = test_entry("Bob", 9999);
        let json = write_entry(&entry);
        assert!(json.contains("Bob"));
        assert!(json.contains("9999"));
    }

    #[test]
    fn test_write_xl_entry() {
        let entry = test_entry("Charlie", 7777);
        let xlog = write_xl_entry(&entry);
        assert!(xlog.contains("player=Charlie"));
        assert!(xlog.contains("score=7777"));
        assert!(xlog.contains("role=Valkyrie"));
    }

    #[test]
    fn test_read_write_roundtrip() {
        let original = test_entry("RoundTrip", 12345);
        let json = write_entry(&original);
        let parsed = read_entry(&json);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.score, original.score);
        assert_eq!(parsed.role, original.role);
    }
}
