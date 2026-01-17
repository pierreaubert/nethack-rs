//! Player role, race, and gender definitions

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Player gender
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
pub enum Gender {
    #[default]
    Male,
    Female,
    Neuter,
}

impl Gender {
    pub const fn pronoun_subject(&self) -> &'static str {
        match self {
            Gender::Male => "he",
            Gender::Female => "she",
            Gender::Neuter => "it",
        }
    }

    pub const fn pronoun_object(&self) -> &'static str {
        match self {
            Gender::Male => "him",
            Gender::Female => "her",
            Gender::Neuter => "it",
        }
    }

    pub const fn pronoun_possessive(&self) -> &'static str {
        match self {
            Gender::Male => "his",
            Gender::Female => "her",
            Gender::Neuter => "its",
        }
    }
}

/// Player race
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
pub enum Race {
    #[default]
    Human,
    Elf,
    Dwarf,
    Gnome,
    Orc,
}

impl Race {
    /// Get the symbol used for this race
    pub const fn symbol(&self) -> char {
        match self {
            Race::Human => '@',
            Race::Elf => '@',
            Race::Dwarf => 'h',
            Race::Gnome => 'G',
            Race::Orc => 'o',
        }
    }

    /// Get plural form
    pub const fn plural(&self) -> &'static str {
        match self {
            Race::Human => "humans",
            Race::Elf => "elves",
            Race::Dwarf => "dwarves",
            Race::Gnome => "gnomes",
            Race::Orc => "orcs",
        }
    }

    /// Check if this race is inherently infravision-capable
    pub const fn has_infravision(&self) -> bool {
        matches!(
            self,
            Race::Elf | Race::Dwarf | Race::Gnome | Race::Orc
        )
    }
}

/// Player role/class
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
pub enum Role {
    Archeologist,
    Barbarian,
    Caveman,
    Healer,
    Knight,
    Monk,
    Priest,
    Ranger,
    Rogue,
    Samurai,
    Tourist,
    #[default]
    Valkyrie,
    Wizard,
}

impl Role {
    /// Get the file name for quest (e.g., "Val" for Valkyrie)
    pub const fn quest_name(&self) -> &'static str {
        match self {
            Role::Archeologist => "Arc",
            Role::Barbarian => "Bar",
            Role::Caveman => "Cav",
            Role::Healer => "Hea",
            Role::Knight => "Kni",
            Role::Monk => "Mon",
            Role::Priest => "Pri",
            Role::Ranger => "Ran",
            Role::Rogue => "Rog",
            Role::Samurai => "Sam",
            Role::Tourist => "Tou",
            Role::Valkyrie => "Val",
            Role::Wizard => "Wiz",
        }
    }

    /// Get rank title for given experience level
    pub fn rank_title(&self, level: i32, gender: Gender) -> &'static str {
        let level = level.clamp(1, 30) as usize;
        let idx = (level - 1) / 3; // 0-9 ranks

        match (self, gender) {
            (Role::Valkyrie, _) => {
                ["Stripling", "Skirmisher", "Fighter", "Woman-at-arms", "Warrior",
                 "Swashbuckler", "Hero", "Champion", "Lord", "Lady of the Lake"][idx]
            }
            (Role::Wizard, Gender::Female) => {
                ["Evoker", "Conjurer", "Thaumaturge", "Magician", "Enchantress",
                 "Sorceress", "Necromancer", "Wizard", "Mage", "Archmage"][idx]
            }
            (Role::Wizard, _) => {
                ["Evoker", "Conjurer", "Thaumaturge", "Magician", "Enchanter",
                 "Sorcerer", "Necromancer", "Wizard", "Mage", "Archmage"][idx]
            }
            (Role::Priest, Gender::Female) => {
                ["Aspirant", "Acolyte", "Adept", "Priestess", "Curate",
                 "Canoness", "Abbess", "Prioress", "Matriarch", "High Priestess"][idx]
            }
            (Role::Priest, _) => {
                ["Aspirant", "Acolyte", "Adept", "Priest", "Curate",
                 "Canon", "Lama", "Patriarch", "High Priest", "High Priest"][idx]
            }
            // TODO: Add all other roles
            _ => {
                ["Novice", "Apprentice", "Journeyman", "Expert", "Adept",
                 "Master", "Grandmaster", "Champion", "Hero", "Demigod"][idx]
            }
        }
    }

    /// Get starting alignment for this role
    pub const fn default_alignment(&self) -> super::AlignmentType {
        use super::AlignmentType;
        match self {
            Role::Archeologist => AlignmentType::Lawful,
            Role::Barbarian => AlignmentType::Neutral,
            Role::Caveman => AlignmentType::Lawful,
            Role::Healer => AlignmentType::Neutral,
            Role::Knight => AlignmentType::Lawful,
            Role::Monk => AlignmentType::Lawful,
            Role::Priest => AlignmentType::Neutral, // can be any
            Role::Ranger => AlignmentType::Neutral,
            Role::Rogue => AlignmentType::Chaotic,
            Role::Samurai => AlignmentType::Lawful,
            Role::Tourist => AlignmentType::Neutral,
            Role::Valkyrie => AlignmentType::Neutral,
            Role::Wizard => AlignmentType::Neutral,
        }
    }
}
