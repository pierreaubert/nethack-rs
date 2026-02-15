//! Naming system (do_name.c)
//!
//! Handles naming individual objects, naming monsters, and
//! calling (naming) types of objects.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::monster::Monster;
use crate::object::Object;

/// Maximum length for a player-assigned name (PL_PSIZ in C)
pub const MAX_NAME_LEN: usize = 32;

/// Result of a naming attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingResult {
    /// Successfully named the target
    Named(String),
    /// Naming was rejected (artifact resistance, unique monster, etc.)
    Rejected(String),
    /// Nothing to name
    NoTarget,
    /// Player cancelled
    Cancelled,
}

// ============================================================================
// Object naming
// ============================================================================

/// Name an individual object (oname equivalent from do_name.c).
///
/// Assigns a player-given name to a specific object instance.
/// Artifacts cannot be renamed. Names are truncated to MAX_NAME_LEN.
///
/// # Arguments
/// * `obj` - The object to name
/// * `new_name` - The name to assign (empty string removes the name)
pub fn oname(obj: &mut Object, new_name: &str) -> NamingResult {
    // Artifacts resist renaming (matches C line 1313)
    if obj.is_artifact() {
        return NamingResult::Rejected(
            "The artifact seems to resist the attempt.".to_string(),
        );
    }

    // Empty name = remove name
    if new_name.is_empty() {
        obj.name = None;
        return NamingResult::Named(String::new());
    }

    // Truncate to max length (matches C PL_PSIZ limit)
    let truncated = if new_name.len() > MAX_NAME_LEN {
        &new_name[..MAX_NAME_LEN]
    } else {
        new_name
    };

    let name = truncated.to_string();
    obj.name = Some(name.clone());

    // Artifact creation by naming (e.g., naming a sword "Sting") deferred to artifact system

    NamingResult::Named(name)
}

/// Name an individual object interactively (do_oname equivalent).
///
/// Higher-level function that validates and applies a name.
/// Rejects naming of novels, artifacts, and names matching existing artifacts.
pub fn do_oname(obj: &mut Object, proposed_name: &str) -> NamingResult {
    // Novels can't be renamed (matches C line 1236)
    if obj.name.as_ref().is_some_and(|n| n.to_lowercase().contains("novel")) {
        return NamingResult::Rejected(
            "It already has a published name.".to_string(),
        );
    }

    // Check for artifact naming conflicts
    if obj.is_artifact() {
        return NamingResult::Rejected(
            "The artifact seems to resist the attempt.".to_string(),
        );
    }

    // Check if the proposed name matches an existing artifact
    // (matches C lines 1265-1289: hand slips while engraving)
    if is_artifact_name(proposed_name) && !obj.is_artifact() {
        return NamingResult::Rejected(
            "While engraving, your hand slips.".to_string(),
        );
    }

    oname(obj, proposed_name)
}

/// Check if a name matches a known artifact name.
///
/// Simplified check — in C this is artifact_name() + exist_artifact().
fn is_artifact_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    // Check against known artifact names
    matches!(
        lower.as_str(),
        "excalibur"
            | "sting"
            | "orcrist"
            | "stormbringer"
            | "mjollnir"
            | "grayswandir"
            | "frost brand"
            | "fire brand"
            | "dragonbane"
            | "demonbane"
            | "werebane"
            | "trollsbane"
            | "ogresmasher"
            | "sunsword"
            | "vorpal blade"
            | "snickersnee"
            | "magicbane"
            | "cleaver"
    )
}

// ============================================================================
// Monster naming
// ============================================================================

/// Christen (name) a monster (christen_monst equivalent).
///
/// Assigns a player-given name to a specific monster.
/// Names are truncated to MAX_NAME_LEN.
pub fn christen_monst(monster: &mut Monster, new_name: &str) {
    if new_name.is_empty() {
        // Clear the name
        monster.name = String::new();
        return;
    }

    let truncated = if new_name.len() > MAX_NAME_LEN {
        &new_name[..MAX_NAME_LEN]
    } else {
        new_name
    };

    monster.name = truncated.to_string();
}

/// Name a monster interactively (do_mname equivalent).
///
/// Validates that the monster can be named:
/// - Unique monsters refuse names
/// - Shopkeepers refuse names
/// - Priests/minions refuse names
pub fn do_mname(
    monster: &mut Monster,
    proposed_name: &str,
    is_unique: bool,
    is_shopkeeper: bool,
    is_priest: bool,
) -> NamingResult {
    if proposed_name.is_empty() {
        return NamingResult::Cancelled;
    }

    // Unique monsters refuse naming (matches C line 1204)
    if is_unique && !is_priest {
        return NamingResult::Rejected(format!(
            "{} doesn't like being called names!",
            monster.name
        ));
    }

    // Shopkeepers refuse naming (matches C line 1207)
    if is_shopkeeper {
        return NamingResult::Rejected(format!(
            "\"I'm {}, not {}.\"",
            monster.name, proposed_name
        ));
    }

    // Priests and minions refuse naming (matches C line 1212)
    if is_priest {
        return NamingResult::Rejected(format!(
            "{} will not accept the name {}.",
            monster.name, proposed_name
        ));
    }

    christen_monst(monster, proposed_name);
    NamingResult::Named(proposed_name.to_string())
}

// ============================================================================
// Type naming (calling)
// ============================================================================

/// Object classes that can be "called" (given a type name).
///
/// Matches C callable[] array from do_name.c line 1342.
pub fn is_callable_class(class: crate::object::ObjectClass) -> bool {
    use crate::object::ObjectClass;
    matches!(
        class,
        ObjectClass::Scroll
            | ObjectClass::Potion
            | ObjectClass::Wand
            | ObjectClass::Ring
            | ObjectClass::Amulet
            | ObjectClass::Gem
            | ObjectClass::Spellbook
            | ObjectClass::Armor
            | ObjectClass::Tool
    )
}

/// Called names for object types (user-assigned type names).
///
/// In C, this is objects[otyp].oc_uname — a per-type user name.
/// Here we store them in a HashMap keyed by object_type.
#[derive(Debug, Clone, Default)]
pub struct CalledNames {
    /// Map from object_type ID to user-assigned name
    names: hashbrown::HashMap<i16, String>,
}

impl CalledNames {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a "called" name for an object type (docall equivalent).
    ///
    /// This names the *type* of object, not an individual instance.
    /// For example, calling an unidentified potion "healing" so all
    /// potions of that appearance are labeled.
    pub fn set_call(&mut self, object_type: i16, name: &str) {
        if name.is_empty() {
            self.names.remove(&object_type);
        } else {
            let truncated = if name.len() > MAX_NAME_LEN {
                &name[..MAX_NAME_LEN]
            } else {
                name
            };
            self.names.insert(object_type, truncated.to_string());
        }
    }

    /// Get the "called" name for an object type.
    pub fn get_call(&self, object_type: i16) -> Option<&str> {
        self.names.get(&object_type).map(|s| s.as_str())
    }

    /// Remove a "called" name.
    pub fn remove_call(&mut self, object_type: i16) {
        self.names.remove(&object_type);
    }

    /// Check if a type has been called.
    pub fn has_call(&self, object_type: i16) -> bool {
        self.names.contains_key(&object_type)
    }

    /// Number of types that have been called.
    pub fn count(&self) -> usize {
        self.names.len()
    }
}

/// Apply a "call" name to an object type (docall equivalent).
///
/// Matches C docall() from do_name.c. Names the type of object
/// so all objects of the same appearance are labeled.
pub fn docall(called_names: &mut CalledNames, object_type: i16, name: &str) -> NamingResult {
    if name.is_empty() {
        called_names.remove_call(object_type);
        return NamingResult::Named(String::new());
    }

    // Strip leading/trailing whitespace (matches C mungspaces)
    let trimmed = name.trim();
    if trimmed.is_empty() {
        called_names.remove_call(object_type);
        return NamingResult::Named(String::new());
    }

    called_names.set_call(object_type, trimmed);
    NamingResult::Named(trimmed.to_string())
}

// ============================================================================
// Name formatting helpers
// ============================================================================

/// Check if a name is essentially the same (fuzzy match).
///
/// Matches C fuzzymatch() — compares ignoring case and specified characters.
pub fn fuzzy_match(s1: &str, s2: &str) -> bool {
    let clean = |s: &str| -> String {
        s.chars()
            .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
            .flat_map(|c| c.to_lowercase())
            .collect()
    };
    clean(s1) == clean(s2)
}

/// Sanitize a user-supplied name (strip control chars, truncate).
pub fn sanitize_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_NAME_LEN)
        .collect();
    sanitized.trim().to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::{Monster, MonsterId};

    fn test_object() -> Object {
        Object::default()
    }

    fn test_monster() -> Monster {
        Monster::new(MonsterId(0), 0, 5, 5)
    }

    // ---- oname tests ----

    #[test]
    fn test_oname_basic() {
        let mut obj = test_object();
        let result = oname(&mut obj, "Fluffy");
        assert_eq!(result, NamingResult::Named("Fluffy".to_string()));
        assert_eq!(obj.name, Some("Fluffy".to_string()));
    }

    #[test]
    fn test_oname_empty_removes() {
        let mut obj = test_object();
        obj.name = Some("OldName".to_string());
        let result = oname(&mut obj, "");
        assert_eq!(result, NamingResult::Named(String::new()));
        assert_eq!(obj.name, None);
    }

    #[test]
    fn test_oname_truncates_long_name() {
        let mut obj = test_object();
        let long_name = "A".repeat(100);
        let result = oname(&mut obj, &long_name);
        if let NamingResult::Named(name) = result {
            assert_eq!(name.len(), MAX_NAME_LEN);
        } else {
            panic!("Expected Named result");
        }
    }

    #[test]
    fn test_oname_artifact_rejected() {
        let mut obj = test_object();
        obj.artifact = 1; // Make it an artifact
        let result = oname(&mut obj, "NewName");
        assert!(matches!(result, NamingResult::Rejected(_)));
    }

    // ---- do_oname tests ----

    #[test]
    fn test_do_oname_basic() {
        let mut obj = test_object();
        let result = do_oname(&mut obj, "TestName");
        assert_eq!(result, NamingResult::Named("TestName".to_string()));
    }

    #[test]
    fn test_do_oname_artifact_name_rejected() {
        let mut obj = test_object();
        let result = do_oname(&mut obj, "Excalibur");
        assert!(matches!(result, NamingResult::Rejected(_)));
    }

    #[test]
    fn test_do_oname_artifact_object_rejected() {
        let mut obj = test_object();
        obj.artifact = 1;
        let result = do_oname(&mut obj, "Anything");
        assert!(matches!(result, NamingResult::Rejected(_)));
    }

    // ---- christen_monst tests ----

    #[test]
    fn test_christen_monst_basic() {
        let mut mon = test_monster();
        christen_monst(&mut mon, "Spot");
        assert_eq!(mon.name, "Spot");
    }

    #[test]
    fn test_christen_monst_empty_clears() {
        let mut mon = test_monster();
        mon.name = "OldName".to_string();
        christen_monst(&mut mon, "");
        assert_eq!(mon.name, "");
    }

    #[test]
    fn test_christen_monst_truncates() {
        let mut mon = test_monster();
        let long_name = "B".repeat(100);
        christen_monst(&mut mon, &long_name);
        assert_eq!(mon.name.len(), MAX_NAME_LEN);
    }

    // ---- do_mname tests ----

    #[test]
    fn test_do_mname_regular_monster() {
        let mut mon = test_monster();
        mon.name = "goblin".to_string();
        let result = do_mname(&mut mon, "Snork", false, false, false);
        assert_eq!(result, NamingResult::Named("Snork".to_string()));
        assert_eq!(mon.name, "Snork");
    }

    #[test]
    fn test_do_mname_unique_rejected() {
        let mut mon = test_monster();
        mon.name = "Medusa".to_string();
        let result = do_mname(&mut mon, "Fluffy", true, false, false);
        assert!(matches!(result, NamingResult::Rejected(_)));
    }

    #[test]
    fn test_do_mname_shopkeeper_rejected() {
        let mut mon = test_monster();
        mon.name = "Asidonhopo".to_string();
        let result = do_mname(&mut mon, "Bob", false, true, false);
        if let NamingResult::Rejected(msg) = result {
            assert!(msg.contains("Asidonhopo"));
            assert!(msg.contains("Bob"));
        } else {
            panic!("Expected Rejected");
        }
    }

    #[test]
    fn test_do_mname_priest_rejected() {
        let mut mon = test_monster();
        mon.name = "priest of Anhur".to_string();
        let result = do_mname(&mut mon, "Bob", false, false, true);
        assert!(matches!(result, NamingResult::Rejected(_)));
    }

    #[test]
    fn test_do_mname_empty_cancelled() {
        let mut mon = test_monster();
        let result = do_mname(&mut mon, "", false, false, false);
        assert_eq!(result, NamingResult::Cancelled);
    }

    // ---- CalledNames tests ----

    #[test]
    fn test_called_names_basic() {
        let mut cn = CalledNames::new();
        cn.set_call(42, "healing");
        assert_eq!(cn.get_call(42), Some("healing"));
        assert!(cn.has_call(42));
        assert_eq!(cn.count(), 1);
    }

    #[test]
    fn test_called_names_remove() {
        let mut cn = CalledNames::new();
        cn.set_call(42, "healing");
        cn.remove_call(42);
        assert_eq!(cn.get_call(42), None);
        assert!(!cn.has_call(42));
    }

    #[test]
    fn test_docall_basic() {
        let mut cn = CalledNames::new();
        let result = docall(&mut cn, 42, "healing");
        assert_eq!(result, NamingResult::Named("healing".to_string()));
        assert_eq!(cn.get_call(42), Some("healing"));
    }

    #[test]
    fn test_docall_empty_removes() {
        let mut cn = CalledNames::new();
        cn.set_call(42, "healing");
        docall(&mut cn, 42, "");
        assert_eq!(cn.get_call(42), None);
    }

    #[test]
    fn test_docall_whitespace_stripped() {
        let mut cn = CalledNames::new();
        let result = docall(&mut cn, 42, "  healing  ");
        assert_eq!(result, NamingResult::Named("healing".to_string()));
        assert_eq!(cn.get_call(42), Some("healing"));
    }

    // ---- is_callable_class tests ----

    #[test]
    fn test_callable_classes() {
        use crate::object::ObjectClass;
        assert!(is_callable_class(ObjectClass::Scroll));
        assert!(is_callable_class(ObjectClass::Potion));
        assert!(is_callable_class(ObjectClass::Wand));
        assert!(is_callable_class(ObjectClass::Ring));
        assert!(is_callable_class(ObjectClass::Amulet));
        assert!(is_callable_class(ObjectClass::Gem));
        assert!(is_callable_class(ObjectClass::Spellbook));
        assert!(is_callable_class(ObjectClass::Armor));
        assert!(is_callable_class(ObjectClass::Tool));
        // Non-callable classes
        assert!(!is_callable_class(ObjectClass::Weapon));
        assert!(!is_callable_class(ObjectClass::Food));
        assert!(!is_callable_class(ObjectClass::Coin));
    }

    // ---- Fuzzy match tests ----

    #[test]
    fn test_fuzzy_match_identical() {
        assert!(fuzzy_match("hello", "hello"));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert!(fuzzy_match("Hello", "hello"));
        assert!(fuzzy_match("HELLO", "hello"));
    }

    #[test]
    fn test_fuzzy_match_ignores_separators() {
        assert!(fuzzy_match("long_sword", "long sword"));
        assert!(fuzzy_match("long-sword", "long sword"));
    }

    #[test]
    fn test_fuzzy_match_different() {
        assert!(!fuzzy_match("hello", "world"));
    }

    // ---- sanitize_name tests ----

    #[test]
    fn test_sanitize_name_basic() {
        assert_eq!(sanitize_name("Fluffy"), "Fluffy");
    }

    #[test]
    fn test_sanitize_name_strips_control() {
        assert_eq!(sanitize_name("Flu\x00ffy"), "Fluffy");
    }

    #[test]
    fn test_sanitize_name_truncates() {
        let long = "A".repeat(100);
        let result = sanitize_name(&long);
        assert_eq!(result.len(), MAX_NAME_LEN);
    }

    #[test]
    fn test_sanitize_name_trims_whitespace() {
        assert_eq!(sanitize_name("  Fluffy  "), "Fluffy");
    }

    // ---- is_artifact_name tests ----

    #[test]
    fn test_artifact_names() {
        assert!(is_artifact_name("Excalibur"));
        assert!(is_artifact_name("excalibur"));
        assert!(is_artifact_name("Sting"));
        assert!(is_artifact_name("Mjollnir"));
        assert!(!is_artifact_name("Fluffy"));
        assert!(!is_artifact_name(""));
    }
}
