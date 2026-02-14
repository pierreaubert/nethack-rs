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

    /// Get the PM_* race ID used in artifact/quest data
    pub const fn race_id(&self) -> i16 {
        match self {
            Race::Human => 100,
            Race::Elf => 101,
            Race::Dwarf => 102,
            Race::Gnome => 103,
            Race::Orc => 104,
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
            (Role::Archeologist, _) => {
                ["Digger", "Field Worker", "Investigator", "Exhumer", "Excavator",
                 "Spelunker", "Speleologist", "Collector", "Curator", "Curator"][idx]
            }
            (Role::Barbarian, Gender::Female) => {
                ["Plunderess", "Pillager", "Bandit", "Brigand", "Raider",
                 "Reaver", "Slayer", "Chieftainess", "Conqueress", "Conqueress"][idx]
            }
            (Role::Barbarian, _) => {
                ["Plunderer", "Pillager", "Bandit", "Brigand", "Raider",
                 "Reaver", "Slayer", "Chieftain", "Conqueror", "Conqueror"][idx]
            }
            (Role::Caveman, _) => {
                ["Troglodyte", "Aborigine", "Wanderer", "Vagrant", "Wayfarer",
                 "Roamer", "Nomad", "Rover", "Pioneer", "Pioneer"][idx]
            }
            (Role::Healer, Gender::Female) => {
                ["Rhizotomist", "Empiric", "Embalmer", "Dresser", "Medica ossium",
                 "Herbalist", "Magistra", "Physician", "Chirurgeon", "Chirurgeon"][idx]
            }
            (Role::Healer, _) => {
                ["Rhizotomist", "Empiric", "Embalmer", "Dresser", "Medicus ossium",
                 "Herbalist", "Magister", "Physician", "Chirurgeon", "Chirurgeon"][idx]
            }
            (Role::Knight, Gender::Female) => {
                ["Gallant", "Esquire", "Bachelor", "Sergeant", "Knight",
                 "Banneret", "Chevaliere", "Dame", "Paladin", "Paladin"][idx]
            }
            (Role::Knight, _) => {
                ["Gallant", "Esquire", "Bachelor", "Sergeant", "Knight",
                 "Banneret", "Chevalier", "Seignieur", "Paladin", "Paladin"][idx]
            }
            (Role::Monk, _) => {
                ["Candidate", "Novice", "Initiate", "Student of Stones", "Student of Waters",
                 "Student of Metals", "Student of Winds", "Student of Fire", "Master", "Master"][idx]
            }
            (Role::Priest, Gender::Female) => {
                ["Aspirant", "Acolyte", "Adept", "Priestess", "Curate",
                 "Canoness", "Lama", "Matriarch", "High Priestess", "High Priestess"][idx]
            }
            (Role::Priest, _) => {
                ["Aspirant", "Acolyte", "Adept", "Priest", "Curate",
                 "Canon", "Lama", "Patriarch", "High Priest", "High Priest"][idx]
            }
            (Role::Ranger, Gender::Female) => {
                ["Tenderfoot", "Lookout", "Trailblazer", "Reconnoiteress", "Scout",
                 "Arbalester", "Archer", "Sharpshooter", "Markswoman", "Markswoman"][idx]
            }
            (Role::Ranger, _) => {
                ["Tenderfoot", "Lookout", "Trailblazer", "Reconnoiterer", "Scout",
                 "Arbalester", "Archer", "Sharpshooter", "Marksman", "Marksman"][idx]
            }
            (Role::Rogue, Gender::Female) => {
                ["Footpad", "Cutpurse", "Rogue", "Pilferer", "Robber",
                 "Burglar", "Filcher", "Magswoman", "Thief", "Thief"][idx]
            }
            (Role::Rogue, _) => {
                ["Footpad", "Cutpurse", "Rogue", "Pilferer", "Robber",
                 "Burglar", "Filcher", "Magsman", "Thief", "Thief"][idx]
            }
            (Role::Samurai, Gender::Female) => {
                ["Hatamoto", "Ronin", "Kunoichi", "Joshu", "Ryoshu",
                 "Kokushu", "Daimyo", "Kuge", "Shogun", "Shogun"][idx]
            }
            (Role::Samurai, _) => {
                ["Hatamoto", "Ronin", "Ninja", "Joshu", "Ryoshu",
                 "Kokushu", "Daimyo", "Kuge", "Shogun", "Shogun"][idx]
            }
            (Role::Tourist, Gender::Female) => {
                ["Rambler", "Sightseer", "Excursionist", "Peregrinatrix", "Traveler",
                 "Journeyer", "Voyager", "Explorer", "Adventurer", "Adventurer"][idx]
            }
            (Role::Tourist, _) => {
                ["Rambler", "Sightseer", "Excursionist", "Peregrinator", "Traveler",
                 "Journeyer", "Voyager", "Explorer", "Adventurer", "Adventurer"][idx]
            }
            (Role::Valkyrie, Gender::Female) => {
                ["Stripling", "Skirmisher", "Fighter", "Woman-at-arms", "Warrior",
                 "Swashbuckler", "Heroine", "Champion", "Lady", "Lady"][idx]
            }
            (Role::Valkyrie, _) => {
                ["Stripling", "Skirmisher", "Fighter", "Man-at-arms", "Warrior",
                 "Swashbuckler", "Hero", "Champion", "Lord", "Lord"][idx]
            }
            (Role::Wizard, Gender::Female) => {
                ["Evoker", "Conjurer", "Thaumaturge", "Magician", "Enchantress",
                 "Sorceress", "Necromancer", "Wizard", "Mage", "Mage"][idx]
            }
            (Role::Wizard, _) => {
                ["Evoker", "Conjurer", "Thaumaturge", "Magician", "Enchanter",
                 "Sorcerer", "Necromancer", "Wizard", "Mage", "Mage"][idx]
            }
        }
    }

    /// Get the PM_* role ID used in artifact/quest data
    pub const fn role_id(&self) -> i16 {
        match self {
            Role::Archeologist => 0,
            Role::Barbarian => 1,
            Role::Caveman => 2,
            Role::Healer => 3,
            Role::Knight => 4,
            Role::Monk => 5,
            Role::Priest => 6,
            Role::Ranger => 7,
            Role::Rogue => 8,
            Role::Samurai => 9,
            Role::Tourist => 10,
            Role::Valkyrie => 11,
            Role::Wizard => 12,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_titles_all_roles_covered() {
        // Ensure all roles have rank titles at all levels
        for role in [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ] {
            for level in 1..=30 {
                let title = role.rank_title(level, Gender::Male);
                assert!(!title.is_empty(), "Role {:?} level {} should have a title", role, level);
            }
        }
    }

    #[test]
    fn test_rank_titles_gender_variants() {
        // Test gender-specific titles
        assert_eq!(Role::Barbarian.rank_title(1, Gender::Male), "Plunderer");
        assert_eq!(Role::Barbarian.rank_title(1, Gender::Female), "Plunderess");

        assert_eq!(Role::Wizard.rank_title(13, Gender::Male), "Enchanter");
        assert_eq!(Role::Wizard.rank_title(13, Gender::Female), "Enchantress");

        assert_eq!(Role::Priest.rank_title(10, Gender::Male), "Priest");
        assert_eq!(Role::Priest.rank_title(10, Gender::Female), "Priestess");
    }

    #[test]
    fn test_rank_titles_level_progression() {
        // Test that ranks progress with level
        let role = Role::Valkyrie;
        assert_eq!(role.rank_title(1, Gender::Female), "Stripling");
        assert_eq!(role.rank_title(4, Gender::Female), "Skirmisher");
        assert_eq!(role.rank_title(7, Gender::Female), "Fighter");
        assert_eq!(role.rank_title(28, Gender::Female), "Lady");
    }

    #[test]
    fn test_monk_rank_titles() {
        assert_eq!(Role::Monk.rank_title(1, Gender::Male), "Candidate");
        assert_eq!(Role::Monk.rank_title(25, Gender::Male), "Master");
    }
}
