//! Object naming functions (objnam.c)
//!
//! Functions for generating object names from ObjClassDef data.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use super::{ObjClassDef, Object, ObjectClass};

/// Discovery state for a type of object
#[derive(Debug, Clone, Copy, Default)]
pub struct ObjectKnowledge {
    /// Type name is known (object identified)
    pub name_known: bool,
    /// User-assigned name for this type
    pub user_name: Option<&'static str>,
}

/// Get the type name for an object definition.
/// This returns the name to show to the player based on discovery state.
///
/// # Arguments
/// * `def` - The object class definition
/// * `knowledge` - Discovery state for this object type
///
/// # Returns
/// The appropriate name string based on what the player knows
pub fn obj_typename<'a>(def: &'a ObjClassDef, knowledge: &ObjectKnowledge) -> &'a str {
    if knowledge.name_known || def.description.is_empty() {
        // Known or no description = show actual name
        def.name
    } else {
        // Unknown = show description
        def.description
    }
}

/// Get the simple type name (actual name if known, else description).
/// Does not include user-assigned names.
pub fn simple_typename<'a>(def: &'a ObjClassDef, known: bool) -> &'a str {
    if known || def.description.is_empty() {
        def.name
    } else {
        def.description
    }
}

/// Get the base name for an object (like xname but just the base part).
/// This computes the name from the OBJECTS data without any instance-specific modifiers.
///
/// # Arguments
/// * `def` - The object class definition
/// * `knowledge` - Discovery state for this object type
pub fn base_object_name<'a>(def: &'a ObjClassDef, knowledge: &ObjectKnowledge) -> String {
    let typename = obj_typename(def, knowledge);

    match def.class {
        ObjectClass::Amulet => {
            if knowledge.name_known {
                typename.to_string()
            } else if let Some(user_name) = knowledge.user_name {
                format!("amulet called {}", user_name)
            } else if !def.description.is_empty() {
                format!("{} amulet", def.description)
            } else {
                "amulet".to_string()
            }
        }

        ObjectClass::Ring => {
            if knowledge.name_known {
                format!("ring of {}", def.name)
            } else if let Some(user_name) = knowledge.user_name {
                format!("ring called {}", user_name)
            } else if !def.description.is_empty() {
                format!("{} ring", def.description)
            } else {
                "ring".to_string()
            }
        }

        ObjectClass::Potion => {
            if knowledge.name_known {
                format!(
                    "potion of {}",
                    def.name.strip_prefix("potion of ").unwrap_or(def.name)
                )
            } else if let Some(user_name) = knowledge.user_name {
                format!("potion called {}", user_name)
            } else if !def.description.is_empty() {
                format!("{} potion", def.description)
            } else {
                "potion".to_string()
            }
        }

        ObjectClass::Scroll => {
            if knowledge.name_known {
                format!(
                    "scroll of {}",
                    def.name.strip_prefix("scroll of ").unwrap_or(def.name)
                )
            } else if let Some(user_name) = knowledge.user_name {
                format!("scroll called {}", user_name)
            } else if !def.description.is_empty() {
                format!("scroll labeled {}", def.description)
            } else {
                "scroll".to_string()
            }
        }

        ObjectClass::Spellbook => {
            if def.unique {
                // Unique books like Book of the Dead
                typename.to_string()
            } else if knowledge.name_known {
                format!(
                    "spellbook of {}",
                    def.name.strip_prefix("spellbook of ").unwrap_or(def.name)
                )
            } else if let Some(user_name) = knowledge.user_name {
                format!("spellbook called {}", user_name)
            } else if !def.description.is_empty() {
                format!("{} spellbook", def.description)
            } else {
                "spellbook".to_string()
            }
        }

        ObjectClass::Wand => {
            if knowledge.name_known {
                format!(
                    "wand of {}",
                    def.name.strip_prefix("wand of ").unwrap_or(def.name)
                )
            } else if let Some(user_name) = knowledge.user_name {
                format!("wand called {}", user_name)
            } else if !def.description.is_empty() {
                format!("{} wand", def.description)
            } else {
                "wand".to_string()
            }
        }

        ObjectClass::Gem => {
            if knowledge.name_known {
                // Gemstones append "stone" if needed
                let name = def.name;
                if def.material == super::Material::Gemstone && !name.ends_with(" stone") {
                    format!("{} stone", name)
                } else {
                    name.to_string()
                }
            } else if !def.description.is_empty() {
                // Unknown gems show as "blue gem" or "gray stone" etc.
                let suffix = if def.material == super::Material::Mineral {
                    "stone"
                } else {
                    "gem"
                };
                format!("{} {}", def.description, suffix)
            } else {
                "gem".to_string()
            }
        }

        ObjectClass::Weapon | ObjectClass::Tool | ObjectClass::Armor => {
            // These classes use description if unknown, actual name if known
            if knowledge.name_known || def.description.is_empty() {
                typename.to_string()
            } else if let Some(user_name) = knowledge.user_name {
                format!("{} called {}", def.description, user_name)
            } else {
                def.description.to_string()
            }
        }

        ObjectClass::Food => {
            // Food is generally always known
            typename.to_string()
        }

        ObjectClass::Coin => "gold piece".to_string(),

        _ => typename.to_string(),
    }
}

/// Generate the full display name for an object instance.
/// Combines base name with instance-specific modifiers (quantity, BUC, enchantment, etc.)
///
/// # Arguments
/// * `obj` - The object instance
/// * `def` - The object class definition
/// * `knowledge` - Discovery state for this object type
pub fn full_object_name(obj: &Object, def: &ObjClassDef, knowledge: &ObjectKnowledge) -> String {
    let base = base_object_name(def, knowledge);
    obj.doname(&base)
}

/// Generate a simple name for an object (like xname).
///
/// # Arguments
/// * `obj` - The object instance
/// * `def` - The object class definition
/// * `knowledge` - Discovery state for this object type
pub fn simple_object_name(obj: &Object, def: &ObjClassDef, knowledge: &ObjectKnowledge) -> String {
    let base = base_object_name(def, knowledge);
    obj.xname(&base)
}

/// Pluralize a word using basic English rules.
/// This is a simplified version of NetHack's makeplural().
pub fn makeplural(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    // Special cases
    let word_lower = word.to_lowercase();
    match word_lower.as_str() {
        "gold piece" => return "gold pieces".to_string(),
        "foot" => return word.replace("foot", "feet"),
        "tooth" => return word.replace("tooth", "teeth"),
        "goose" => return word.replace("goose", "geese"),
        "mouse" => return word.replace("mouse", "mice"),
        "louse" => return word.replace("louse", "lice"),
        "knife" => return word.replace("knife", "knives"),
        "staff" => return word.replace("staff", "staves"),
        "loaf" => return word.replace("loaf", "loaves"),
        "leaf" => return word.replace("leaf", "leaves"),
        "wolf" => return word.replace("wolf", "wolves"),
        _ => {}
    }

    // Words ending in certain patterns
    if word.ends_with("us") && !word.ends_with("lotus") {
        return format!("{}i", &word[..word.len() - 2]);
    }

    if word.ends_with("um") {
        return format!("{}a", &word[..word.len() - 2]);
    }

    if word.ends_with("is") {
        return format!("{}es", &word[..word.len() - 2]);
    }

    // Standard English pluralization rules
    if word.ends_with('s')
        || word.ends_with('x')
        || word.ends_with('z')
        || word.ends_with("ch")
        || word.ends_with("sh")
    {
        return format!("{}es", word);
    }

    if word.ends_with('y') && word.len() > 1 {
        let before_y = word.chars().nth(word.len() - 2).unwrap_or('a');
        if !"aeiou".contains(before_y) {
            return format!("{}ies", &word[..word.len() - 1]);
        }
    }

    if word.ends_with('f') && word.len() > 1 {
        return format!("{}ves", &word[..word.len() - 1]);
    }

    if word.ends_with("fe") {
        return format!("{}ves", &word[..word.len() - 2]);
    }

    // Default: just add 's'
    format!("{}s", word)
}

/// Choose "a" or "an" based on the following word (C: an/just_an)
///
/// Matches C NetHack's `just_an()` logic:
/// - Words already starting with "the " pass through unchanged
/// - Vowel-initial words get "an" (with exceptions for "uni", "use", "useful", "uranium")
/// - "x" + non-vowel gets "an" (e.g., "xorn")
/// - Single vowel letters get "an"
pub fn an(word: &str) -> String {
    if word.is_empty() {
        return "a".to_string();
    }

    // Already has "the " prefix → pass through
    let word_lower = word.to_lowercase();
    if word_lower.starts_with("the ") {
        return word.to_string();
    }

    let first_char = word.chars().next().unwrap().to_ascii_lowercase();

    // Single-character words: vowels get "an"
    if word.len() == 1 {
        if "aefhilmnosx".contains(first_char) {
            return format!("an {}", word);
        }
        return format!("a {}", word);
    }

    // Words starting with vowel sounds
    if "aeiou".contains(first_char) {
        // Exceptions: words where the vowel sounds like a consonant
        if word_lower.starts_with("one-")
            || word_lower.starts_with("eucalyptus")
            || word_lower.starts_with("unicorn")
            || word_lower.starts_with("uranium")
            || word_lower.starts_with("useful")
            || word_lower.starts_with("uni")
            || word_lower.starts_with("use")
        {
            return format!("a {}", word);
        }
        return format!("an {}", word);
    }

    // "x" followed by non-vowel sounds like "z" → "an"
    if first_char == 'x'
        && word
            .chars()
            .nth(1)
            .is_some_and(|c| !"aeiou".contains(c.to_ascii_lowercase()))
    {
        return format!("an {}", word);
    }

    format!("a {}", word)
}

/// Get "the" prefix for unique or specific items (C: the/The)
///
/// Matches C NetHack's logic:
/// - Already starts with "the " → return as-is
/// - Proper names (starts with capital, no " of ") → return as-is
/// - Otherwise → prepend "the "
pub fn the(word: &str) -> String {
    if word.is_empty() {
        return "the".to_string();
    }

    let lower = word.to_lowercase();
    if lower.starts_with("the ") {
        return word.to_string();
    }

    // Proper names: starts with uppercase, doesn't contain " of "
    // (items like "Amulet of Yendor" still get "the")
    let first = word.chars().next().unwrap();
    if first.is_ascii_uppercase() && !word.contains(" of ") {
        return word.to_string();
    }

    format!("the {}", word)
}

/// Get "The" prefix (capitalized) for sentence-initial use
#[allow(dead_code)]
pub fn the_upper(word: &str) -> String {
    let result = the(word);
    if let Some(rest) = result.strip_prefix("the ") {
        format!("The {}", rest)
    } else {
        result
    }
}

/// Capitalized "a/an" prefix (An equivalent)
pub fn an_capitalized(word: &str) -> String {
    crate::upstart(&an(word))
}

/// Alias for an_capitalized (matches C NetHack naming)
#[allow(non_snake_case)]
pub fn An(word: &str) -> String {
    an_capitalized(word)
}

/// Capitalized "the" prefix (The equivalent)
pub fn the_capitalized(word: &str) -> String {
    crate::upstart(&the(word))
}

/// Alias for the_capitalized (matches C NetHack naming)
#[allow(non_snake_case)]
pub fn The(word: &str) -> String {
    the_capitalized(word)
}

/// Format quantity with pluralization.
pub fn quantity_name(count: i32, singular: &str) -> String {
    if count == 1 {
        an(singular)
    } else {
        format!("{} {}", count, makeplural(singular))
    }
}

// ============================================================================
// Corpse naming (corpse_xname from objnam.c)
// ============================================================================

/// Generate name for a corpse object.
///
/// Corpses include the monster name, e.g., "kobold corpse" or "newt corpse".
///
/// # Arguments
/// * `obj` - The corpse object
/// * `monster_name` - The name of the monster (from mons[].mname)
/// * `adjective` - Optional adjective like "partly eaten"
///
/// # Returns
/// The formatted corpse name
pub fn corpse_xname(obj: &Object, monster_name: &str, adjective: Option<&str>) -> String {
    let mut parts = Vec::new();

    // Quantity prefix
    if obj.quantity > 1 {
        parts.push(format!("{}", obj.quantity));
    }

    // Adjective (e.g., "partly eaten")
    if let Some(adj) = adjective {
        parts.push(adj.to_string());
    }

    // Monster name + "corpse"
    parts.push(format!("{} corpse", monster_name));

    // Pluralize if needed
    if obj.quantity > 1 {
        let last = parts.pop().unwrap();
        parts.push(makeplural(&last));
    }

    parts.join(" ")
}

/// Generate name for a statue.
///
/// # Arguments
/// * `obj` - The statue object
/// * `monster_name` - The name of the monster
/// * `adjective` - Optional adjective
pub fn statue_xname(obj: &Object, monster_name: &str, adjective: Option<&str>) -> String {
    let mut parts = Vec::new();

    // Quantity prefix
    if obj.quantity > 1 {
        parts.push(format!("{}", obj.quantity));
    }

    // Adjective
    if let Some(adj) = adjective {
        parts.push(adj.to_string());
    }

    // "statue of a/an <monster>"
    parts.push(format!("statue of {}", an(monster_name)));

    // Pluralize if needed
    if obj.quantity > 1 {
        let base = parts.pop().unwrap();
        // "statues of a kobold" not "statue of a kobolds"
        parts.push(base.replace("statue of", "statues of"));
    }

    parts.join(" ")
}

/// Generate name for an egg.
///
/// # Arguments
/// * `obj` - The egg object
/// * `monster_name` - The name of the monster (if known), None for "unknown"
pub fn egg_xname(obj: &Object, monster_name: Option<&str>) -> String {
    let base = match monster_name {
        Some(name) => format!("{} egg", name),
        None => "egg".to_string(),
    };

    if obj.quantity > 1 {
        format!("{} {}", obj.quantity, makeplural(&base))
    } else {
        an(&base)
    }
}

/// Generate name for a figurine.
///
/// # Arguments
/// * `obj` - The figurine object
/// * `monster_name` - The name of the monster
pub fn figurine_xname(obj: &Object, monster_name: &str) -> String {
    let base = format!("figurine of {}", an(monster_name));

    if obj.quantity > 1 {
        format!("{} figurines of {}", obj.quantity, an(monster_name))
    } else {
        an(&base)
    }
}

/// Generate a name suitable for a death message.
///
/// This is used in messages like "killed by a kobold corpse".
/// Similar to xname but formatted for death messages.
///
/// # Arguments
/// * `obj` - The object that caused death
/// * `base_name` - The base object name
pub fn killer_xname(obj: &Object, base_name: &str) -> String {
    // For quantity > 1, use plural
    if obj.quantity > 1 {
        makeplural(base_name)
    } else {
        base_name.to_string()
    }
}

/// Generate a name for the "killed by" message.
///
/// # Arguments
/// * `obj` - The corpse/object
/// * `monster_name` - Monster name if it's a corpse
pub fn killer_corpse_xname(monster_name: &str) -> String {
    format!("{} corpse", monster_name)
}

/// Make a word singular (opposite of makeplural).
///
/// This is a simplified version; full implementation would handle more cases.
pub fn makesingular(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let word_lower = word.to_lowercase();

    // Special cases
    match word_lower.as_str() {
        "gold pieces" => return "gold piece".to_string(),
        "feet" => return word.replace("feet", "foot"),
        "teeth" => return word.replace("teeth", "tooth"),
        "geese" => return word.replace("geese", "goose"),
        "mice" => return word.replace("mice", "mouse"),
        "lice" => return word.replace("lice", "louse"),
        "knives" => return word.replace("knives", "knife"),
        "staves" => return word.replace("staves", "staff"),
        "loaves" => return word.replace("loaves", "loaf"),
        "leaves" => return word.replace("leaves", "leaf"),
        "wolves" => return word.replace("wolves", "wolf"),
        _ => {}
    }

    // Words ending in "ies" -> "y"
    if word.ends_with("ies") && word.len() > 3 {
        return format!("{}y", &word[..word.len() - 3]);
    }

    // Words ending in "ves" -> "f" or "fe"
    if word.ends_with("ves") && word.len() > 3 {
        let base = &word[..word.len() - 3];
        // Could be "f" or "fe", default to "f"
        return format!("{}f", base);
    }

    // Words ending in "es" (after ch, sh, s, x, z)
    if word.ends_with("es") && word.len() > 2 {
        let base = &word[..word.len() - 2];
        if base.ends_with("ch")
            || base.ends_with("sh")
            || base.ends_with('s')
            || base.ends_with('x')
            || base.ends_with('z')
        {
            return base.to_string();
        }
    }

    // Words ending in "i" -> "us"
    if word.ends_with('i') && word.len() > 1 {
        return format!("{}us", &word[..word.len() - 1]);
    }

    // Words ending in "a" -> "um"
    if word.ends_with('a') && word.len() > 1 {
        return format!("{}um", &word[..word.len() - 1]);
    }

    // Default: remove trailing 's'
    if word.ends_with('s') && word.len() > 1 {
        return word[..word.len() - 1].to_string();
    }

    word.to_string()
}

// ============================================================================
// Object name with verb (aobjnam, Doname from objnam.c)
// ============================================================================

/// Generate object name with verb action (aobjnam equivalent).
///
/// Creates names like "The +2 sword glows" or "Your armor rusts".
///
/// # Arguments
/// * `obj` - The object
/// * `base_name` - Base name of the object
/// * `verb` - The verb to use (e.g., "glow", "rust")
///
/// # Returns
/// A string like "The sword glows" or "Your swords glow"
pub fn aobjnam(obj: &Object, base_name: &str, verb: &str) -> String {
    let name = if obj.quantity > 1 {
        format!("{} {}", obj.quantity, makeplural(base_name))
    } else {
        the(base_name)
    };

    // Conjugate verb for singular/plural
    let conjugated_verb = if obj.quantity > 1 {
        verb.to_string() // plural verbs don't add 's'
    } else {
        // Simple verb conjugation - add 's' for third person singular
        if verb.ends_with('s')
            || verb.ends_with('x')
            || verb.ends_with("ch")
            || verb.ends_with("sh")
        {
            format!("{}es", verb)
        } else if verb.ends_with('y') {
            let before_y = verb.chars().nth(verb.len() - 2).unwrap_or('a');
            if !"aeiou".contains(before_y) {
                format!("{}ies", &verb[..verb.len() - 1])
            } else {
                format!("{}s", verb)
            }
        } else {
            format!("{}s", verb)
        }
    };

    format!("{} {}", name, conjugated_verb)
}

/// Generate object name with verb action, capitalized (Aobjnam equivalent).
#[allow(non_snake_case)]
pub fn Aobjnam(obj: &Object, base_name: &str, verb: &str) -> String {
    crate::upstart(&aobjnam(obj, base_name, verb))
}

/// Generate a simple object name without modifiers (ansimpleoname equivalent).
///
/// Returns just the base name with a/an prefix, without BUC, enchantment, etc.
///
/// # Arguments
/// * `obj` - The object
/// * `base_name` - Base name of the object
///
/// # Returns
/// Simple name like "a sword" or "5 arrows"
pub fn ansimpleoname(obj: &Object, base_name: &str) -> String {
    if obj.quantity > 1 {
        format!("{} {}", obj.quantity, makeplural(base_name))
    } else {
        an(base_name)
    }
}

/// Generate a name without "a/an/the" prefix (simpleoname equivalent).
pub fn simpleoname(obj: &Object, base_name: &str) -> String {
    if obj.quantity > 1 {
        format!("{} {}", obj.quantity, makeplural(base_name))
    } else {
        base_name.to_string()
    }
}

/// Generate an object name for corpses with monster name (cxname equivalent).
///
/// For corpses/statues/figurines, this includes the monster's name.
/// For other objects, behaves like xname.
///
/// # Arguments
/// * `obj` - The object
/// * `base_name` - Base name of the object
/// * `monster_name` - Name of the corpse's monster type (if applicable)
pub fn cxname(obj: &Object, base_name: &str, monster_name: Option<&str>) -> String {
    if obj.corpse_type >= 0 {
        if let Some(mon_name) = monster_name {
            // It's a corpse/statue/figurine with monster type
            return format!("{} {}", mon_name, base_name);
        }
    }
    // Fall back to regular simple name
    obj.xname(base_name)
}

/// Generate a singular object name for corpses (cxname_singular equivalent).
///
/// Like cxname but always treats quantity as 1 (for display in specific contexts).
pub fn cxname_singular(obj: &Object, base_name: &str, monster_name: Option<&str>) -> String {
    if obj.corpse_type >= 0 {
        if let Some(mon_name) = monster_name {
            return format!("{} {}", mon_name, base_name);
        }
    }
    base_name.to_string()
}

// ============================================================================
// Artifact naming (artiname from artifact.c)
// ============================================================================

/// Get the name of an artifact by ID.
///
/// # Arguments
/// * `artifact_id` - The artifact ID (1-based)
///
/// # Returns
/// The artifact name, or None if invalid ID
pub fn artiname(artifact_id: u8) -> Option<&'static str> {
    // Artifact names from NetHack 3.6
    // These are indexed by artifact ID
    match artifact_id {
        1 => Some("Excalibur"),
        2 => Some("Stormbringer"),
        3 => Some("Mjollnir"),
        4 => Some("Cleaver"),
        5 => Some("Grimtooth"),
        6 => Some("Orcrist"),
        7 => Some("Sting"),
        8 => Some("Magicbane"),
        9 => Some("Frost Brand"),
        10 => Some("Fire Brand"),
        11 => Some("Dragonbane"),
        12 => Some("Demonbane"),
        13 => Some("Werebane"),
        14 => Some("Grayswandir"),
        15 => Some("Giantslayer"),
        16 => Some("Ogresmasher"),
        17 => Some("Trollsbane"),
        18 => Some("Vorpal Blade"),
        19 => Some("Snickersnee"),
        20 => Some("Sunsword"),
        21 => Some("Orb of Detection"),
        22 => Some("Heart of Ahriman"),
        23 => Some("Sceptre of Might"),
        24 => Some("Staff of Aesculapius"),
        25 => Some("Magic Mirror of Merlin"),
        26 => Some("Eyes of the Overworld"),
        27 => Some("Mitre of Holiness"),
        28 => Some("Longbow of Diana"),
        29 => Some("Master Key of Thievery"),
        30 => Some("Tsurugi of Muramasa"),
        31 => Some("Platinum Yendorian Express Card"),
        32 => Some("Orb of Fate"),
        33 => Some("Eye of the Aethiopica"),
        34 => Some("Amulet of Yendor"),
        _ => None,
    }
}

/// Check if a string is an artifact name.
pub fn is_artifact_name(name: &str) -> bool {
    for id in 1..=34 {
        if let Some(arti_name) = artiname(id) {
            if arti_name.eq_ignore_ascii_case(name) {
                return true;
            }
        }
    }
    false
}

/// Get artifact ID from name.
pub fn artifact_id_from_name(name: &str) -> Option<u8> {
    for id in 1..=34 {
        if let Some(arti_name) = artiname(id) {
            if arti_name.eq_ignore_ascii_case(name) {
                return Some(id);
            }
        }
    }
    None
}

// ============================================================================
// Artifact utility functions (artifact.c)
// ============================================================================

/// Get the base cost of an artifact (arti_cost equivalent)
///
/// # Arguments
/// * `artifact_id` - The artifact ID (1-based)
///
/// # Returns
/// The artifact's base cost in gold, or 0 for invalid artifacts
pub fn arti_cost(artifact_id: u8) -> u32 {
    // Artifact costs from NetHack 3.6
    match artifact_id {
        1 => 4000,   // Excalibur
        2 => 8000,   // Stormbringer
        3 => 4000,   // Mjollnir
        4 => 1500,   // Cleaver
        5 => 300,    // Grimtooth
        6 => 2000,   // Orcrist
        7 => 800,    // Sting
        8 => 3500,   // Magicbane
        9 => 3000,   // Frost Brand
        10 => 3000,  // Fire Brand
        11 => 500,   // Dragonbane
        12 => 2500,  // Demonbane
        13 => 1500,  // Werebane
        14 => 8000,  // Grayswandir
        15 => 200,   // Giantslayer
        16 => 200,   // Ogresmasher
        17 => 200,   // Trollsbane
        18 => 4000,  // Vorpal Blade
        19 => 1200,  // Snickersnee
        20 => 1500,  // Sunsword
        21 => 2500,  // Orb of Detection
        22 => 2500,  // Heart of Ahriman
        23 => 2500,  // Sceptre of Might
        24 => 5000,  // Staff of Aesculapius
        25 => 1500,  // Magic Mirror of Merlin
        26 => 2500,  // Eyes of the Overworld
        27 => 2000,  // Mitre of Holiness
        28 => 4000,  // Longbow of Diana
        29 => 3500,  // Master Key of Thievery
        30 => 4500,  // Tsurugi of Muramasa
        31 => 7000,  // Platinum Yendorian Express Card
        32 => 3500,  // Orb of Fate
        33 => 4000,  // Eye of the Aethiopica
        34 => 30000, // Amulet of Yendor (priceless really)
        _ => 0,
    }
}

/// Check if an artifact confers immunity to a damage type (arti_immune equivalent)
///
/// Some artifacts provide resistance or immunity to specific damage types
/// when wielded or worn.
///
/// # Arguments
/// * `artifact_id` - The artifact ID (1-based)
/// * `damage_type` - The damage type to check immunity for
///
/// # Returns
/// True if the artifact provides immunity to the damage type
pub fn arti_immune(artifact_id: u8, damage_type: crate::combat::DamageType) -> bool {
    use crate::combat::DamageType;

    match artifact_id {
        1 => {
            // Excalibur - defends against level drain
            matches!(damage_type, DamageType::DrainLife)
        }
        2 => {
            // Stormbringer - defends against level drain
            matches!(damage_type, DamageType::DrainLife)
        }
        9 => {
            // Frost Brand - provides cold resistance when wielded
            matches!(damage_type, DamageType::Cold)
        }
        10 => {
            // Fire Brand - provides fire resistance when wielded
            matches!(damage_type, DamageType::Fire)
        }
        14 => {
            // Grayswandir - defends against curses/level drain
            matches!(damage_type, DamageType::DrainLife | DamageType::Curse)
        }
        27 => {
            // Mitre of Holiness - provides fire resistance
            matches!(damage_type, DamageType::Fire)
        }
        _ => false,
    }
}

/// Check if an artifact reflects (arti_reflects equivalent)
///
/// Some artifacts have the reflection property that reflects
/// ray attacks and gaze attacks.
///
/// # Arguments
/// * `artifact_id` - The artifact ID (1-based)
///
/// # Returns
/// True if the artifact provides reflection
pub fn arti_reflects(artifact_id: u8) -> bool {
    match artifact_id {
        25 => true, // Magic Mirror of Merlin
        32 => true, // Orb of Fate
        _ => false,
    }
}

/// Check if an artifact speaks (arti_speak equivalent)
///
/// Some intelligent artifacts can speak to their wielder.
///
/// # Arguments
/// * `artifact_id` - The artifact ID (1-based)
///
/// # Returns
/// True if the artifact can speak
pub fn arti_speak(artifact_id: u8) -> bool {
    match artifact_id {
        1 => true,  // Excalibur
        2 => true,  // Stormbringer
        8 => true,  // Magicbane
        18 => true, // Vorpal Blade
        _ => false,
    }
}

/// Get artifact light radius (arti_light_radius equivalent)
///
/// Some artifacts emit light when wielded.
///
/// # Arguments
/// * `artifact_id` - The artifact ID (1-based)
///
/// # Returns
/// Light radius in squares, or 0 if artifact doesn't glow
pub fn arti_light_radius(artifact_id: u8) -> u8 {
    match artifact_id {
        20 => 2, // Sunsword
        _ => 0,
    }
}

/// Get artifact light description (arti_light_description equivalent)
///
/// # Arguments
/// * `artifact_id` - The artifact ID (1-based)
///
/// # Returns
/// Description of the light, or None if artifact doesn't glow
pub fn arti_light_description(artifact_id: u8) -> Option<&'static str> {
    match artifact_id {
        20 => Some("shining with a brilliant light"), // Sunsword
        _ => None,
    }
}

/// Generate possessive suffix (s_suffix equivalent).
///
/// Returns "'s" or "'" depending on whether the word ends in 's'.
pub fn s_suffix(word: &str) -> String {
    if word.ends_with('s') || word.ends_with('S') {
        format!("{}'", word)
    } else {
        format!("{}'s", word)
    }
}

/// Generate "Foo's" with proper capitalization.
#[allow(non_snake_case)]
pub fn S_suffix(word: &str) -> String {
    crate::upstart(&s_suffix(word))
}

// ============================================================================
// Advanced naming functions (Phase 4)
// ============================================================================

/// Full object name with all details (doname equivalent)
///
/// Returns complete name including quantity, BUC status, enchantment, erosion, wear status.
/// Example: "blessed +2 plate mail (being worn)"
pub fn doname(obj: &Object, base_name: &str) -> String {
    let mut name = String::new();

    // Add quantity if > 1
    if obj.quantity > 1 {
        name.push_str(&format!("{} ", obj.quantity));
    }

    // Add BUC prefix if known
    if obj.buc_known {
        name.push_str(obj.buc_prefix());
    }

    // Add erosion prefix
    name.push_str(&obj.erosion_prefix());

    // Add enchantment
    name.push_str(&obj.enchantment_str());

    // Add base name
    name.push_str(base_name);

    // Add wear/wield status
    name.push_str(obj.worn_suffix());

    // Add charges for wands
    name.push_str(&obj.charges_suffix());

    name
}

/// Simple object name without quantity (xname equivalent)
///
/// Returns name without quantity prefix, but includes other details.
pub fn xname(obj: &Object, base_name: &str) -> String {
    let mut name = String::new();

    // Add BUC prefix if known
    if obj.buc_known {
        name.push_str(obj.buc_prefix());
    }

    // Add erosion prefix
    name.push_str(&obj.erosion_prefix());

    // Add enchantment
    name.push_str(&obj.enchantment_str());

    // Add base name
    name.push_str(base_name);

    // Add wear/wield status
    name.push_str(obj.worn_suffix());

    // Add charges for wands
    name.push_str(&obj.charges_suffix());

    name
}

/// "Distant" name for far away objects (distant_name equivalent)
///
/// Returns generic "something" name when object is too far to identify clearly.
pub fn distant_name(obj: &Object) -> String {
    match obj.class {
        ObjectClass::Weapon => "something".to_string(),
        ObjectClass::Armor => "something".to_string(),
        ObjectClass::Food => "something edible".to_string(),
        ObjectClass::Potion => "a bottle".to_string(),
        ObjectClass::Scroll => "a scroll".to_string(),
        ObjectClass::Wand => "a wand".to_string(),
        ObjectClass::Ring => "a ring".to_string(),
        ObjectClass::Amulet => "an amulet".to_string(),
        ObjectClass::Gem => "a gem".to_string(),
        ObjectClass::Coin => "some gold".to_string(),
        _ => "something".to_string(),
    }
}

/// Get object name without quantity (singular equivalent)
///
/// Removes quantity prefix from name.
pub fn singular(obj: &Object, base_name: &str) -> String {
    // Just use xname (which doesn't include quantity)
    xname(obj, base_name)
}

/// Object name with "your/the" prefix (yname equivalent)
///
/// Adds appropriate article before name.
pub fn yname(obj: &Object, base_name: &str) -> String {
    let name = xname(obj, base_name);
    if obj.is_worn() {
        format!("your {}", name)
    } else {
        format!("the {}", name)
    }
}

/// "Your" or "The" prefix with capitalization (Yname2 equivalent)
#[allow(non_snake_case)]
pub fn Yname2(obj: &Object, base_name: &str) -> String {
    let name = yname(obj, base_name);
    crate::upstart(&name)
}

/// Your simple name without articles (ysimple_name equivalent)
///
/// Returns "your <name>" for worn items, "<name>" otherwise.
pub fn ysimple_name(obj: &Object, base_name: &str) -> String {
    if obj.is_worn() {
        format!("your {}", base_name)
    } else {
        base_name.to_string()
    }
}

/// Your simple name capitalized (Ysimple_name2 equivalent)
#[allow(non_snake_case)]
pub fn Ysimple_name2(obj: &Object, base_name: &str) -> String {
    let name = ysimple_name(obj, base_name);
    crate::upstart(&name)
}

/// Parse object name from user input (readobjnam equivalent)
///
/// Attempts to parse a string into an object search query.
/// Returns the parsed query or an error message.
pub fn readobjnam(input: &str) -> Result<String, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Please specify an object.".to_string());
    }

    // Check for common patterns
    if trimmed.starts_with("the ") {
        return Ok(trimmed[4..].to_string());
    }

    if trimmed.starts_with("a ") {
        return Ok(trimmed[2..].to_string());
    }

    if trimmed.starts_with("an ") {
        return Ok(trimmed[3..].to_string());
    }

    Ok(trimmed.to_string())
}

/// Get type name from object type index (obj_typename wrapper)
///
/// Returns the type name based on object definition and knowledge.
pub fn obj_typename_from_obj(
    obj: &Object,
    def: &ObjClassDef,
    knowledge: &ObjectKnowledge,
) -> String {
    obj_typename(def, knowledge).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ObjectId;

    #[test]
    fn test_makeplural() {
        assert_eq!(makeplural("sword"), "swords");
        assert_eq!(makeplural("torch"), "torches");
        assert_eq!(makeplural("box"), "boxes");
        assert_eq!(makeplural("ruby"), "rubies");
        assert_eq!(makeplural("key"), "keys");
        assert_eq!(makeplural("knife"), "knives");
        assert_eq!(makeplural("staff"), "staves");
        assert_eq!(makeplural("gold piece"), "gold pieces");
    }

    #[test]
    fn test_an() {
        assert_eq!(an("sword"), "a sword");
        assert_eq!(an("apple"), "an apple");
        assert_eq!(an("unicorn horn"), "a unicorn horn");
        assert_eq!(an("orange"), "an orange");
        assert_eq!(an("emerald"), "an emerald");
    }

    #[test]
    fn test_an_special_cases() {
        // "the " prefix passes through
        assert_eq!(an("the Amulet of Yendor"), "the Amulet of Yendor");
        // "x" + vowel → normal "a" (xorn: x + o)
        assert_eq!(an("xorn"), "a xorn");
        // "x" + vowel → "a" (normal)
        assert_eq!(an("xenon"), "a xenon");
        // "one-" exception
        assert_eq!(an("one-eyed"), "a one-eyed");
        // "eucalyptus" exception
        assert_eq!(an("eucalyptus leaf"), "a eucalyptus leaf");
        // "useful" exception
        assert_eq!(an("useful item"), "a useful item");
        // Single letter
        assert_eq!(an("a"), "an a");
        assert_eq!(an("b"), "a b");
        assert_eq!(an("x"), "an x");
    }

    #[test]
    fn test_the() {
        // Normal word
        assert_eq!(the("sword"), "the sword");
        // Already has "the "
        assert_eq!(the("the Amulet of Yendor"), "the Amulet of Yendor");
        // Proper name (capital, no " of ")
        assert_eq!(the("Excalibur"), "Excalibur");
        // Capital with " of " → still gets "the"
        assert_eq!(the("Amulet of Yendor"), "the Amulet of Yendor");
    }

    #[test]
    fn test_the_upper() {
        assert_eq!(the_upper("sword"), "The sword");
        assert_eq!(the_upper("Excalibur"), "Excalibur");
    }

    #[test]
    fn test_quantity_name() {
        assert_eq!(quantity_name(1, "arrow"), "an arrow");
        assert_eq!(quantity_name(5, "arrow"), "5 arrows");
        assert_eq!(quantity_name(1, "sword"), "a sword");
        assert_eq!(quantity_name(3, "torch"), "3 torches");
    }

    #[test]
    fn test_base_object_name_known() {
        use super::super::{DirectionType, Material};

        let def = ObjClassDef {
            name: "long sword",
            description: "",
            class: ObjectClass::Weapon,
            material: Material::Iron,
            weight: 40,
            cost: 15,
            probability: 50,
            nutrition: 0,
            w_small_damage: 8,
            w_large_damage: 12,
            bonus: 0,
            skill: 7,
            delay: 0,
            color: 0,
            magical: false,
            merge: false,
            unique: false,
            no_wish: false,
            big: false,
            direction: DirectionType::None,
            armor_category: None,
            property: 0,
        };

        let knowledge = ObjectKnowledge {
            name_known: true,
            user_name: None,
        };

        assert_eq!(base_object_name(&def, &knowledge), "long sword");
    }

    #[test]
    fn test_base_object_name_potion() {
        use super::super::{DirectionType, Material};

        let def = ObjClassDef {
            name: "potion of healing",
            description: "pink",
            class: ObjectClass::Potion,
            material: Material::Glass,
            weight: 20,
            cost: 100,
            probability: 57,
            nutrition: 0,
            w_small_damage: 0,
            w_large_damage: 0,
            bonus: 0,
            skill: 0,
            delay: 0,
            color: 0,
            magical: true,
            merge: true,
            unique: false,
            no_wish: false,
            big: false,
            direction: DirectionType::None,
            armor_category: None,
            property: 0,
        };

        // Unknown potion
        let unknown = ObjectKnowledge {
            name_known: false,
            user_name: None,
        };
        assert_eq!(base_object_name(&def, &unknown), "pink potion");

        // Known potion
        let known = ObjectKnowledge {
            name_known: true,
            user_name: None,
        };
        assert_eq!(base_object_name(&def, &known), "potion of healing");

        // User-named potion
        let named = ObjectKnowledge {
            name_known: false,
            user_name: Some("heal"),
        };
        assert_eq!(base_object_name(&def, &named), "potion called heal");
    }

    #[test]
    fn test_base_object_name_gem() {
        use super::super::{DirectionType, Material};

        let def = ObjClassDef {
            name: "diamond",
            description: "white",
            class: ObjectClass::Gem,
            material: Material::Gemstone,
            weight: 1,
            cost: 4000,
            probability: 3,
            nutrition: 0,
            w_small_damage: 0,
            w_large_damage: 0,
            bonus: 0,
            skill: 0,
            delay: 0,
            color: 0,
            magical: false,
            merge: true,
            unique: false,
            no_wish: false,
            big: false,
            direction: DirectionType::None,
            armor_category: None,
            property: 0,
        };

        // Unknown gem
        let unknown = ObjectKnowledge {
            name_known: false,
            user_name: None,
        };
        assert_eq!(base_object_name(&def, &unknown), "white gem");

        // Known gem
        let known = ObjectKnowledge {
            name_known: true,
            user_name: None,
        };
        assert_eq!(base_object_name(&def, &known), "diamond stone");
    }

    // ========================================================================
    // Phase 4 Tests: Advanced Naming Functions
    // ========================================================================

    #[test]
    fn test_doname() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Coin);
        let name = doname(&obj, "gold coin");

        assert!(name.contains("gold coin"));
        assert!(!name.contains("blessed")); // BUC not known
    }

    #[test]
    fn test_doname_with_quantity() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Coin);
        obj.quantity = 5;
        let name = doname(&obj, "gold coin");

        assert!(name.contains("5"));
        assert!(name.contains("gold coin"));
    }

    #[test]
    fn test_xname() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let name = xname(&obj, "long sword");

        assert!(name.contains("long sword"));
        assert!(!name.contains("1 ")); // No quantity
    }

    #[test]
    fn test_distant_name() {
        let weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let potion = Object::new(ObjectId(2), 1, ObjectClass::Potion);

        assert_eq!(distant_name(&weapon), "something");
        assert_eq!(distant_name(&potion), "a bottle");
    }

    #[test]
    fn test_singular() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let name = singular(&obj, "sword");

        // Should not have quantity
        assert!(!name.contains("1 "));
    }

    #[test]
    fn test_yname() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let name = yname(&obj, "sword");

        assert!(name.contains("the"));
        assert!(name.contains("sword"));
    }

    #[test]
    fn test_yname_worn() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        obj.worn_mask = 1; // Worn

        let name = yname(&obj, "armor");

        assert!(name.contains("your"));
        assert!(name.contains("armor"));
    }

    #[test]
    fn test_ysimple_name() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let name = ysimple_name(&obj, "sword");

        assert_eq!(name, "sword");
    }

    #[test]
    fn test_ysimple_name_worn() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        obj.worn_mask = 1;

        let name = ysimple_name(&obj, "armor");

        assert!(name.contains("your"));
    }

    #[test]
    fn test_readobjnam_empty() {
        let result = readobjnam("");
        assert!(result.is_err());
    }

    #[test]
    fn test_readobjnam_with_article() {
        let result = readobjnam("the sword");
        assert_eq!(result, Ok("sword".to_string()));

        let result = readobjnam("a potion");
        assert_eq!(result, Ok("potion".to_string()));

        let result = readobjnam("an amulet");
        assert_eq!(result, Ok("amulet".to_string()));
    }

    #[test]
    fn test_readobjnam_plain() {
        let result = readobjnam("sword");
        assert_eq!(result, Ok("sword".to_string()));
    }
}
