//! Object naming functions (objnam.c)
//!
//! Functions for generating object names from ObjClassDef data.

use super::{Object, ObjectClass, ObjClassDef};

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
                format!("potion of {}", def.name.strip_prefix("potion of ").unwrap_or(def.name))
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
                format!("scroll of {}", def.name.strip_prefix("scroll of ").unwrap_or(def.name))
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
                format!("spellbook of {}", def.name.strip_prefix("spellbook of ").unwrap_or(def.name))
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
                format!("wand of {}", def.name.strip_prefix("wand of ").unwrap_or(def.name))
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

/// Choose "a" or "an" based on the following word.
pub fn an(word: &str) -> String {
    if word.is_empty() {
        return "a".to_string();
    }

    let first_char = word.chars().next().unwrap().to_ascii_lowercase();

    // Words starting with vowel sounds get "an"
    if "aeiou".contains(first_char) {
        // Exception: words starting with "u" that sound like "you"
        if first_char == 'u' {
            let word_lower = word.to_lowercase();
            if word_lower.starts_with("uni") || word_lower.starts_with("use") {
                return format!("a {}", word);
            }
        }
        format!("an {}", word)
    } else {
        format!("a {}", word)
    }
}

/// Get "the" prefix for unique or specific items.
pub fn the(word: &str) -> String {
    format!("the {}", word)
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
        if base.ends_with("ch") || base.ends_with("sh") ||
           base.ends_with('s') || base.ends_with('x') || base.ends_with('z') {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
