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
        matches!(self, Race::Elf | Race::Dwarf | Race::Gnome | Race::Orc)
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

    /// Get the lawful god name for this role
    pub const fn lawful_god(&self) -> &'static str {
        match self {
            Role::Archeologist => "Quetzalcoatl",
            Role::Barbarian => "Mitra",
            Role::Caveman => "Anu",
            Role::Healer => "Athena",
            Role::Knight => "Lugh",
            Role::Monk => "Shan Lai Ching",
            Role::Priest => "_Lawful", // depends on alignment
            Role::Ranger => "Mercury",
            Role::Rogue => "Issek",
            Role::Samurai => "Amaterasu Omikami",
            Role::Tourist => "Blind Io",
            Role::Valkyrie => "Tyr",
            Role::Wizard => "Ptah",
        }
    }

    /// Get the neutral god name for this role
    pub const fn neutral_god(&self) -> &'static str {
        match self {
            Role::Archeologist => "Camaxtli",
            Role::Barbarian => "Crom",
            Role::Caveman => "Ishtar",
            Role::Healer => "Hermes",
            Role::Knight => "Brigit",
            Role::Monk => "Chih Sung-tzu",
            Role::Priest => "_Neutral", // depends on alignment
            Role::Ranger => "Venus",
            Role::Rogue => "Mog",
            Role::Samurai => "Raijin",
            Role::Tourist => "The Lady",
            Role::Valkyrie => "Odin",
            Role::Wizard => "Thoth",
        }
    }

    /// Get the chaotic god name for this role
    pub const fn chaotic_god(&self) -> &'static str {
        match self {
            Role::Archeologist => "Huhetotl",
            Role::Barbarian => "Set",
            Role::Caveman => "Anshar",
            Role::Healer => "Poseidon",
            Role::Knight => "Manannan Mac Lir",
            Role::Monk => "Huan Ti",
            Role::Priest => "_Chaotic", // depends on alignment
            Role::Ranger => "Mars",
            Role::Rogue => "Kos",
            Role::Samurai => "Susanowo",
            Role::Tourist => "Offler",
            Role::Valkyrie => "Loki",
            Role::Wizard => "Anhur",
        }
    }

    /// Get the god name for a specific alignment
    pub const fn god_for_alignment(&self, align: super::AlignmentType) -> &'static str {
        match align {
            super::AlignmentType::Lawful => self.lawful_god(),
            super::AlignmentType::Neutral => self.neutral_god(),
            super::AlignmentType::Chaotic => self.chaotic_god(),
        }
    }

    /// Get rank title for given experience level
    pub fn rank_title(&self, level: i32, gender: Gender) -> &'static str {
        let level = level.clamp(1, 30) as usize;
        let idx = (level - 1) / 3; // 0-9 ranks

        match (self, gender) {
            (Role::Archeologist, _) => [
                "Digger",
                "Field Worker",
                "Investigator",
                "Exhumer",
                "Excavator",
                "Spelunker",
                "Speleologist",
                "Collector",
                "Curator",
                "Curator",
            ][idx],
            (Role::Barbarian, Gender::Female) => [
                "Plunderess",
                "Pillager",
                "Bandit",
                "Brigand",
                "Raider",
                "Reaver",
                "Slayer",
                "Chieftainess",
                "Conqueress",
                "Conqueress",
            ][idx],
            (Role::Barbarian, _) => [
                "Plunderer",
                "Pillager",
                "Bandit",
                "Brigand",
                "Raider",
                "Reaver",
                "Slayer",
                "Chieftain",
                "Conqueror",
                "Conqueror",
            ][idx],
            (Role::Caveman, _) => [
                "Troglodyte",
                "Aborigine",
                "Wanderer",
                "Vagrant",
                "Wayfarer",
                "Roamer",
                "Nomad",
                "Rover",
                "Pioneer",
                "Pioneer",
            ][idx],
            (Role::Healer, Gender::Female) => [
                "Rhizotomist",
                "Empiric",
                "Embalmer",
                "Dresser",
                "Medica ossium",
                "Herbalist",
                "Magistra",
                "Physician",
                "Chirurgeon",
                "Chirurgeon",
            ][idx],
            (Role::Healer, _) => [
                "Rhizotomist",
                "Empiric",
                "Embalmer",
                "Dresser",
                "Medicus ossium",
                "Herbalist",
                "Magister",
                "Physician",
                "Chirurgeon",
                "Chirurgeon",
            ][idx],
            (Role::Knight, Gender::Female) => [
                "Gallant",
                "Esquire",
                "Bachelor",
                "Sergeant",
                "Knight",
                "Banneret",
                "Chevaliere",
                "Dame",
                "Paladin",
                "Paladin",
            ][idx],
            (Role::Knight, _) => [
                "Gallant",
                "Esquire",
                "Bachelor",
                "Sergeant",
                "Knight",
                "Banneret",
                "Chevalier",
                "Seignieur",
                "Paladin",
                "Paladin",
            ][idx],
            (Role::Monk, _) => [
                "Candidate",
                "Novice",
                "Initiate",
                "Student of Stones",
                "Student of Waters",
                "Student of Metals",
                "Student of Winds",
                "Student of Fire",
                "Master",
                "Master",
            ][idx],
            (Role::Priest, Gender::Female) => [
                "Aspirant",
                "Acolyte",
                "Adept",
                "Priestess",
                "Curate",
                "Canoness",
                "Lama",
                "Matriarch",
                "High Priestess",
                "High Priestess",
            ][idx],
            (Role::Priest, _) => [
                "Aspirant",
                "Acolyte",
                "Adept",
                "Priest",
                "Curate",
                "Canon",
                "Lama",
                "Patriarch",
                "High Priest",
                "High Priest",
            ][idx],
            (Role::Ranger, Gender::Female) => [
                "Tenderfoot",
                "Lookout",
                "Trailblazer",
                "Reconnoiteress",
                "Scout",
                "Arbalester",
                "Archer",
                "Sharpshooter",
                "Markswoman",
                "Markswoman",
            ][idx],
            (Role::Ranger, _) => [
                "Tenderfoot",
                "Lookout",
                "Trailblazer",
                "Reconnoiterer",
                "Scout",
                "Arbalester",
                "Archer",
                "Sharpshooter",
                "Marksman",
                "Marksman",
            ][idx],
            (Role::Rogue, Gender::Female) => [
                "Footpad",
                "Cutpurse",
                "Rogue",
                "Pilferer",
                "Robber",
                "Burglar",
                "Filcher",
                "Magswoman",
                "Thief",
                "Thief",
            ][idx],
            (Role::Rogue, _) => [
                "Footpad", "Cutpurse", "Rogue", "Pilferer", "Robber", "Burglar", "Filcher",
                "Magsman", "Thief", "Thief",
            ][idx],
            (Role::Samurai, Gender::Female) => [
                "Hatamoto", "Ronin", "Kunoichi", "Joshu", "Ryoshu", "Kokushu", "Daimyo", "Kuge",
                "Shogun", "Shogun",
            ][idx],
            (Role::Samurai, _) => [
                "Hatamoto", "Ronin", "Ninja", "Joshu", "Ryoshu", "Kokushu", "Daimyo", "Kuge",
                "Shogun", "Shogun",
            ][idx],
            (Role::Tourist, Gender::Female) => [
                "Rambler",
                "Sightseer",
                "Excursionist",
                "Peregrinatrix",
                "Traveler",
                "Journeyer",
                "Voyager",
                "Explorer",
                "Adventurer",
                "Adventurer",
            ][idx],
            (Role::Tourist, _) => [
                "Rambler",
                "Sightseer",
                "Excursionist",
                "Peregrinator",
                "Traveler",
                "Journeyer",
                "Voyager",
                "Explorer",
                "Adventurer",
                "Adventurer",
            ][idx],
            (Role::Valkyrie, Gender::Female) => [
                "Stripling",
                "Skirmisher",
                "Fighter",
                "Woman-at-arms",
                "Warrior",
                "Swashbuckler",
                "Heroine",
                "Champion",
                "Lady",
                "Lady",
            ][idx],
            (Role::Valkyrie, _) => [
                "Stripling",
                "Skirmisher",
                "Fighter",
                "Man-at-arms",
                "Warrior",
                "Swashbuckler",
                "Hero",
                "Champion",
                "Lord",
                "Lord",
            ][idx],
            (Role::Wizard, Gender::Female) => [
                "Evoker",
                "Conjurer",
                "Thaumaturge",
                "Magician",
                "Enchantress",
                "Sorceress",
                "Necromancer",
                "Wizard",
                "Mage",
                "Mage",
            ][idx],
            (Role::Wizard, _) => [
                "Evoker",
                "Conjurer",
                "Thaumaturge",
                "Magician",
                "Enchanter",
                "Sorcerer",
                "Necromancer",
                "Wizard",
                "Mage",
                "Mage",
            ][idx],
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

    /// Parse role from string (str2role equivalent)
    ///
    /// # Arguments
    /// * `s` - String to parse (e.g., "valkyrie", "Val", "v")
    ///
    /// # Returns
    /// The parsed role, or None if invalid
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "archeologist" | "arc" | "a" => Some(Role::Archeologist),
            "barbarian" | "bar" | "b" => Some(Role::Barbarian),
            "caveman" | "cavewoman" | "cav" | "c" => Some(Role::Caveman),
            "healer" | "hea" | "h" => Some(Role::Healer),
            "knight" | "kni" | "k" => Some(Role::Knight),
            "monk" | "mon" | "m" => Some(Role::Monk),
            "priest" | "priestess" | "pri" | "p" => Some(Role::Priest),
            "ranger" | "ran" | "r" => Some(Role::Ranger),
            "rogue" | "rog" => Some(Role::Rogue),
            "samurai" | "sam" | "s" => Some(Role::Samurai),
            "tourist" | "tou" | "t" => Some(Role::Tourist),
            "valkyrie" | "val" | "v" => Some(Role::Valkyrie),
            "wizard" | "wiz" | "w" => Some(Role::Wizard),
            _ => None,
        }
    }
}

impl Race {
    /// Parse race from string (str2race equivalent)
    ///
    /// # Arguments
    /// * `s` - String to parse (e.g., "human", "H", "hum")
    ///
    /// # Returns
    /// The parsed race, or None if invalid
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "human" | "hum" | "h" => Some(Race::Human),
            "elf" | "elv" | "e" => Some(Race::Elf),
            "dwarf" | "dwa" | "d" => Some(Race::Dwarf),
            "gnome" | "gno" | "g" => Some(Race::Gnome),
            "orc" | "o" => Some(Race::Orc),
            _ => None,
        }
    }
}

impl Gender {
    /// Parse gender from string (str2gend equivalent)
    ///
    /// # Arguments
    /// * `s` - String to parse (e.g., "male", "M", "f")
    ///
    /// # Returns
    /// The parsed gender, or None if invalid
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "male" | "m" => Some(Gender::Male),
            "female" | "f" => Some(Gender::Female),
            "neuter" | "n" => Some(Gender::Neuter),
            _ => None,
        }
    }
}

// ============================================================================
// String conversion functions (str2* from role.c)
// ============================================================================

/// Parse role from string (str2role equivalent)
pub fn str2role(s: &str) -> Option<Role> {
    Role::from_str(s)
}

/// Parse race from string (str2race equivalent)
pub fn str2race(s: &str) -> Option<Race> {
    Race::from_str(s)
}

/// Parse gender from string (str2gend equivalent)
pub fn str2gend(s: &str) -> Option<Gender> {
    Gender::from_str(s)
}

// ============================================================================
// Role/Race/Gender/Alignment Compatibility System
// ============================================================================

/// Tracks which roles/races/genders/alignments are restricted from selection
#[derive(Debug, Clone, Default)]
pub struct RoleFilter {
    /// Excluded roles
    pub excluded_roles: Vec<Role>,
    /// Excluded races
    pub excluded_races: Vec<Race>,
    /// Excluded genders
    pub excluded_genders: Vec<Gender>,
}

impl RoleFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a role to the exclusion list
    pub fn exclude_role(&mut self, role: Role) {
        if !self.excluded_roles.contains(&role) {
            self.excluded_roles.push(role);
        }
    }

    /// Add a race to the exclusion list
    pub fn exclude_race(&mut self, race: Race) {
        if !self.excluded_races.contains(&race) {
            self.excluded_races.push(race);
        }
    }

    /// Add a gender to the exclusion list
    pub fn exclude_gender(&mut self, gender: Gender) {
        if !self.excluded_genders.contains(&gender) {
            self.excluded_genders.push(gender);
        }
    }

    /// Check if a role is excluded
    pub fn is_role_excluded(&self, role: Role) -> bool {
        self.excluded_roles.contains(&role)
    }

    /// Check if a race is excluded
    pub fn is_race_excluded(&self, race: Race) -> bool {
        self.excluded_races.contains(&race)
    }

    /// Check if a gender is excluded
    pub fn is_gender_excluded(&self, gender: Gender) -> bool {
        self.excluded_genders.contains(&gender)
    }

    /// Check if filter has any restrictions
    pub fn is_active(&self) -> bool {
        !self.excluded_roles.is_empty()
            || !self.excluded_races.is_empty()
            || !self.excluded_genders.is_empty()
    }

    /// Clear all restrictions
    pub fn clear(&mut self) {
        self.excluded_roles.clear();
        self.excluded_races.clear();
        self.excluded_genders.clear();
    }
}

// ============================================================================
// Compatibility Checking (validX and okX equivalents)
// ============================================================================

/// Check if a role is valid (validrole equivalent)
pub fn validrole(_role: Role) -> bool {
    // All roles in our enum are valid
    true
}

/// Check if a race is valid for a given role (validrace equivalent)
pub fn validrace(role: Role, race: Race) -> bool {
    is_race_compatible_with_role(role, race)
}

/// Check if a gender is valid for a given role and race (validgend equivalent)
pub fn validgend(role: Role, race: Race, gender: Gender) -> bool {
    is_race_compatible_with_role(role, race) && is_gender_compatible(role, race, gender)
}

/// Check if an alignment is valid for a given role, race, and gender (validalign equivalent)
pub fn validalign(role: Role, race: Race, gender: Gender, alignment: super::AlignmentType) -> bool {
    is_race_compatible_with_role(role, race)
        && is_gender_compatible(role, race, gender)
        && is_alignment_compatible(role, alignment)
}

/// Check if race is compatible with role (ok_race equivalent)
pub fn ok_race(role: Option<Role>, race: Race, _filter: &RoleFilter) -> bool {
    match role {
        Some(r) => is_race_compatible_with_role(r, race),
        None => true, // Random role is compatible with any race
    }
}

/// Check if gender is compatible with role and race (ok_gend equivalent)
pub fn ok_gend(role: Option<Role>, race: Race, gender: Gender, _filter: &RoleFilter) -> bool {
    match role {
        Some(r) => is_gender_compatible(r, race, gender),
        None => true, // Random role is compatible with any gender
    }
}

/// Check if alignment is compatible with role (ok_align equivalent)
pub fn ok_align(role: Option<Role>, alignment: super::AlignmentType, _filter: &RoleFilter) -> bool {
    match role {
        Some(r) => is_alignment_compatible(r, alignment),
        None => true, // Random role is compatible with any alignment
    }
}

/// Internal helper: Check if race is compatible with role
fn is_race_compatible_with_role(role: Role, race: Race) -> bool {
    match (role, race) {
        // Human is compatible with all roles
        (_, Race::Human) => true,
        // Barbarian favors Orc
        (Role::Barbarian, Race::Orc) => true,
        // Dwarf roles
        (Role::Caveman, Race::Dwarf) => true,
        (Role::Healer, Race::Dwarf) => true,
        (Role::Rogue, Race::Dwarf) => true,
        // Elf roles
        (Role::Archeologist, Race::Elf) => true,
        (Role::Ranger, Race::Elf) => true,
        (Role::Rogue, Race::Elf) => true,
        (Role::Wizard, Race::Elf) => true,
        // Gnome roles
        (Role::Caveman, Race::Gnome) => true,
        (Role::Healer, Race::Gnome) => true,
        (Role::Rogue, Race::Gnome) => true,
        (Role::Wizard, Race::Gnome) => true,
        // Orc as fallback for most roles
        (_, Race::Orc) => true,
        // Most roles accept most races
        _ => true,
    }
}

/// Internal helper: Check if gender is compatible with role and race
fn is_gender_compatible(_role: Role, _race: Race, _gender: Gender) -> bool {
    // All genders are compatible with all role/race combinations in modern NetHack
    true
}

/// Internal helper: Check if alignment is compatible with role
fn is_alignment_compatible(role: Role, alignment: super::AlignmentType) -> bool {
    use super::AlignmentType;
    match (role, alignment) {
        // Lawful roles
        (Role::Archeologist, AlignmentType::Lawful) => true,
        (Role::Knight, AlignmentType::Lawful) => true,
        (Role::Monk, AlignmentType::Lawful) => true,
        (Role::Samurai, AlignmentType::Lawful) => true,
        (Role::Caveman, AlignmentType::Lawful) => true,
        // Chaotic roles
        (Role::Rogue, AlignmentType::Chaotic) => true,
        // Neutral roles or flexible
        (Role::Barbarian, AlignmentType::Neutral) => true,
        (Role::Healer, AlignmentType::Neutral) => true,
        (Role::Ranger, AlignmentType::Neutral) => true,
        (Role::Tourist, AlignmentType::Neutral) => true,
        (Role::Valkyrie, AlignmentType::Neutral) => true,
        (Role::Wizard, AlignmentType::Neutral) => true,
        // Priest can be any alignment (depends on god)
        (Role::Priest, _) => true,
        // Allow any alignment for any role as default
        _ => true,
    }
}

// ============================================================================
// Selection Functions (pickX equivalents)
// ============================================================================

/// Pick a random role respecting given constraints (pick_role equivalent)
pub fn pick_role(
    race: Option<Race>,
    gender: Option<Gender>,
    alignment: Option<super::AlignmentType>,
    filter: &RoleFilter,
) -> Option<Role> {
    let all_roles = [
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
    ];

    // Collect valid roles that match constraints
    let mut valid = Vec::new();
    for &role in &all_roles {
        if !filter.is_role_excluded(role) && validrole(role) {
            // Check other constraints
            let race_ok = race.is_none() || validrace(role, race.unwrap());
            let gender_ok =
                gender.is_none() || validgend(role, race.unwrap_or(Race::Human), gender.unwrap());
            let align_ok = alignment.is_none()
                || validalign(
                    role,
                    race.unwrap_or(Race::Human),
                    gender.unwrap_or(Gender::Male),
                    alignment.unwrap(),
                );

            if race_ok && gender_ok && align_ok {
                valid.push(role);
            }
        }
    }

    if valid.is_empty() {
        None
    } else {
        // Return first valid role (in full game: would use RNG)
        Some(valid[0])
    }
}

/// Pick a random race respecting given constraints (pick_race equivalent)
pub fn pick_race(
    role: Option<Role>,
    gender: Option<Gender>,
    alignment: Option<super::AlignmentType>,
    filter: &RoleFilter,
) -> Option<Race> {
    let all_races = [Race::Human, Race::Elf, Race::Dwarf, Race::Gnome, Race::Orc];

    // Collect valid races that match constraints
    let mut valid = Vec::new();
    for &race in &all_races {
        if !filter.is_race_excluded(race) {
            let role_ok = role.is_none() || validrace(role.unwrap(), race);
            let gender_ok = gender.is_none()
                || validgend(role.unwrap_or(Role::Valkyrie), race, gender.unwrap());
            let align_ok = alignment.is_none()
                || validalign(
                    role.unwrap_or(Role::Valkyrie),
                    race,
                    gender.unwrap_or(Gender::Male),
                    alignment.unwrap(),
                );

            if role_ok && gender_ok && align_ok {
                valid.push(race);
            }
        }
    }

    if valid.is_empty() {
        None
    } else {
        // Return first valid race (in full game: would use RNG)
        Some(valid[0])
    }
}

/// Pick a random gender respecting given constraints (pick_gend equivalent)
pub fn pick_gend(
    role: Option<Role>,
    race: Race,
    alignment: Option<super::AlignmentType>,
    filter: &RoleFilter,
) -> Option<Gender> {
    let all_genders = [Gender::Male, Gender::Female, Gender::Neuter];

    // Collect valid genders that match constraints
    let mut valid = Vec::new();
    for &gender in &all_genders {
        if !filter.is_gender_excluded(gender) {
            let role_ok = role.is_none() || validgend(role.unwrap(), race, gender);
            let align_ok = alignment.is_none()
                || validalign(
                    role.unwrap_or(Role::Valkyrie),
                    race,
                    gender,
                    alignment.unwrap(),
                );

            if role_ok && align_ok {
                valid.push(gender);
            }
        }
    }

    if valid.is_empty() {
        None
    } else {
        // Return first valid gender (in full game: would use RNG)
        Some(valid[0])
    }
}

/// Pick a random alignment (pick_align equivalent)
pub fn pick_align(role: Role) -> Option<super::AlignmentType> {
    // Use role's default alignment or random compatible alignment
    Some(role.default_alignment())
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Count number of valid genders for a role and race (role_gendercount equivalent)
pub fn role_gendercount(role: Role) -> usize {
    // Simplified: most roles support 2-3 genders
    match role {
        Role::Monk => 2, // Monks typically have male/female variants
        _ => 2,          // Most roles support at least male/female
    }
}

/// Count valid alignments for a race (race_alignmentcount equivalent)
pub fn race_alignmentcount(_race: Race) -> usize {
    // Most races support all three alignments
    3
}

/// Determine if a role selection is rigid (only one option) (rigid_role_checks equivalent)
pub fn rigid_role_checks(filter: &RoleFilter) -> bool {
    let all_roles = [
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
    ];
    let mut count = 0;
    for role in &all_roles {
        if !filter.is_role_excluded(*role) {
            count += 1;
        }
    }
    count == 1
}

/// Get the pet type for a given role (pet_type equivalent)
pub fn pet_type(role: Role) -> &'static str {
    match role {
        Role::Archeologist => "little dog",
        Role::Barbarian => "orcish hound",
        Role::Caveman => "little dog",
        Role::Healer => "little dog",
        Role::Knight => "warhorse",
        Role::Monk => "little dog",
        Role::Priest => "little dog",
        Role::Ranger => "little dog",
        Role::Rogue => "little dog",
        Role::Samurai => "little dog",
        Role::Tourist => "little dog",
        Role::Valkyrie => "warhorse",
        Role::Wizard => "little dog",
    }
}

/// Get the gender of a polymorphed form (poly_gender equivalent)
pub fn poly_gender(_form: Option<&str>) -> Gender {
    // Simplified: return random gender
    // In full implementation, would depend on specific form
    Gender::Male
}

/// Get gender of a monster (gender equivalent)
pub fn monster_gender(_monster_type: &str) -> Gender {
    // Simplified: return male as default
    // In full implementation, would check monster tables
    Gender::Male
}

/// Parse role filter from string (setrolefilter equivalent)
pub fn setrolefilter(filter: &mut RoleFilter, s: &str) -> bool {
    if let Some(role) = Role::from_str(s) {
        filter.exclude_role(role);
        return true;
    }
    if let Some(race) = Race::from_str(s) {
        filter.exclude_race(race);
        return true;
    }
    if let Some(gender) = Gender::from_str(s) {
        filter.exclude_gender(gender);
        return true;
    }
    false
}

/// Check if role filter has any restrictions (gotrolefilter equivalent)
pub fn gotrolefilter(filter: &RoleFilter) -> bool {
    filter.is_active()
}

/// Clear all role filters (clearrolefilter equivalent)
pub fn clearrolefilter(filter: &mut RoleFilter) {
    filter.clear()
}

// ============================================================================
// Additional Utility Functions
// ============================================================================

/// Initialize player role and attributes (role_init equivalent)
pub fn role_init(
    player_name: &mut String,
    role: &mut Role,
    race: &mut Race,
    gender: &mut Gender,
) -> bool {
    // 1. Strip role letter from player name if present
    plnamesuffix(player_name);

    // 2. Validate initial role/race/gender combination
    if !validrole(*role) {
        // Try to pick a valid role
        let filter = RoleFilter::new();
        if let Some(picked_role) = pick_role(Some(*race), Some(*gender), None, &filter) {
            *role = picked_role;
        } else {
            return false; // No valid role found
        }
    }

    // Validate race for role
    if !validrace(*role, *race) {
        if let Some(picked_race) = pick_race(Some(*role), Some(*gender), None, &RoleFilter::new()) {
            *race = picked_race;
        } else {
            return false;
        }
    }

    // Validate gender for role and race
    if !validgend(*role, *race, *gender) {
        if let Some(picked_gender) = pick_gend(Some(*role), *race, None, &RoleFilter::new()) {
            *gender = picked_gender;
        } else {
            return false;
        }
    }

    true
}

/// Get innate abilities for a role (role_abil equivalent)
pub fn role_abil(role: Role) -> Vec<&'static str> {
    match role {
        Role::Archeologist => vec!["search"],
        Role::Barbarian => vec!["cleave"],
        Role::Caveman => vec!["kick"],
        Role::Healer => vec!["heal"],
        Role::Knight => vec!["cleave", "bonuses"],
        Role::Monk => vec!["kick", "punch"],
        Role::Priest => vec!["turn undead"],
        Role::Ranger => vec!["shoot"],
        Role::Rogue => vec!["backstab"],
        Role::Samurai => vec!["cleave"],
        Role::Tourist => vec!["charm"],
        Role::Valkyrie => vec!["cleave", "bonuses"],
        Role::Wizard => vec!["spellcasting"],
    }
}

/// Remove the role letter from the player name (plnamesuffix equivalent)
pub fn plnamesuffix(player_name: &mut String) {
    // Strip role letter from end of player name for backwards compatibility
    // Role letters: a=arch, b=bar, c=cave, h=heal, k=knight, m=monk, p=priest
    // r=ranger, o=rogue, s=samurai, t=tourist, v=valkyrie, w=wizard
    if let Some(last_char) = player_name.chars().last() {
        if matches!(
            last_char,
            'a' | 'b' | 'c' | 'h' | 'k' | 'm' | 'p' | 'r' | 'o' | 's' | 't' | 'v' | 'w'
        ) {
            player_name.pop();
        }
    }
}

/// Display role-specific prologue text (role_selection_prolog equivalent)
pub fn role_selection_prolog(role_index: usize, _where: &str) -> String {
    let all_roles = [
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
    ];

    if role_index >= all_roles.len() {
        return String::from("Unknown role.");
    }

    match all_roles[role_index] {
        Role::Archeologist => String::from(
            "Delve into ancient tombs and uncover lost treasures. Skill at searching and good at trap avoidance.",
        ),
        Role::Barbarian => String::from(
            "A warrior of savage strength and rage. Excel at melee combat and cleaving enemies.",
        ),
        Role::Caveman => String::from(
            "A primitive survivor adept at kicking and rough-and-tumble combat. Good with simple weapons.",
        ),
        Role::Healer => String::from(
            "Dedicated to mending wounds and curing ailments. Moderate combat ability with intrinsic healing.",
        ),
        Role::Knight => String::from(
            "A noble warrior bound by honor. Strong combatant with bonuses from blessed armor.",
        ),
        Role::Monk => String::from(
            "Master of martial arts and physical discipline. Excellent bare-handed fighter. Can achieve enlightenment.",
        ),
        Role::Priest => String::from(
            "A servant of the gods. Can turn undead, cast spells, and receive divine intervention in prayer.",
        ),
        Role::Ranger => String::from(
            "A skilled hunter and tracker. Good with ranged weapons and can move stealthily through wilderness.",
        ),
        Role::Rogue => String::from(
            "A master of stealth and backstabs. Can pick locks, disarm traps, and steal items undetected.",
        ),
        Role::Samurai => String::from(
            "An honorable warrior from the Far East. Excellent swordsman with code of honor. Can invoke bushido.",
        ),
        Role::Tourist => String::from(
            "A vacation traveler with camera and credit cards. Weak combat but charming personality.",
        ),
        Role::Valkyrie => String::from(
            "A warrior maiden chosen by fate. Strong melee combatant blessed by the gods.",
        ),
        Role::Wizard => String::from(
            "Master of arcane magic. Can cast powerful spells and enchant objects. Weak in combat.",
        ),
    }
}

/// Get additional role menu text (role_menu_extra equivalent)
pub fn role_menu_extra(role_index: usize, _where: &str) -> Option<String> {
    let all_roles = [
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
    ];

    if role_index >= all_roles.len() {
        return None;
    }

    match all_roles[role_index] {
        Role::Archeologist => Some(String::from("Quest: Find the amulet in the Museum")),
        Role::Barbarian => Some(String::from("Quest: Recover the Heart of Ahriman")),
        Role::Caveman => Some(String::from("Quest: Find the Stone of Gehn")),
        Role::Healer => Some(String::from("Quest: Find the Amulet of Wholeness")),
        Role::Knight => Some(String::from("Quest: Recover the Holy Grail")),
        Role::Monk => Some(String::from("Quest: Achieve Enlightenment in the Temple")),
        Role::Priest => Some(String::from("Quest: Obtain the Orb of Detection")),
        Role::Ranger => Some(String::from("Quest: Recover Sting and Glamdring")),
        Role::Rogue => Some(String::from("Quest: Steal the Amulet from the Arch-Mage")),
        Role::Samurai => Some(String::from("Quest: Recover the Tatsumaki")),
        Role::Tourist => Some(String::from("Quest: Take a photo and return home")),
        Role::Valkyrie => Some(String::from("Quest: Recover the Orb of Fate")),
        Role::Wizard => Some(String::from("Quest: Recover the Amulet of the Sky")),
    }
}

/// Build a prompt for player selection (build_plselection_prompt equivalent)
pub fn build_plselection_prompt(
    role: Option<Role>,
    race: Option<Race>,
    gender: Option<Gender>,
    alignment: Option<super::AlignmentType>,
) -> String {
    let mut prompt = String::from("Your character:\n");

    if let Some(r) = role {
        prompt.push_str(&format!("  Role: {}\n", r));
    } else {
        prompt.push_str("  Role: (random)\n");
    }

    if let Some(r) = race {
        prompt.push_str(&format!("  Race: {}\n", r));
    } else {
        prompt.push_str("  Race: (random)\n");
    }

    if let Some(g) = gender {
        prompt.push_str(&format!("  Gender: {}\n", g));
    } else {
        prompt.push_str("  Gender: (random)\n");
    }

    if let Some(a) = alignment {
        prompt.push_str(&format!("  Alignment: {:?}", a));
    } else {
        prompt.push_str("  Alignment: (role default)");
    }

    prompt
}

/// Build root player selection prompt (root_plselection_prompt equivalent)
pub fn root_plselection_prompt() -> String {
    String::from(
        "Welcome to NetHack!\n\
         \n\
         What would you like to do?\n\
         \n\
         (N)ew Game\n\
         (C)ontinue Saved Game\n\
         (Q)uit\n\
         \n\
         Choose: ",
    )
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
                assert!(
                    !title.is_empty(),
                    "Role {:?} level {} should have a title",
                    role,
                    level
                );
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

    #[test]
    fn test_role_filter() {
        let mut filter = RoleFilter::new();
        assert!(!filter.is_active());

        filter.exclude_role(Role::Wizard);
        assert!(filter.is_active());
        assert!(filter.is_role_excluded(Role::Wizard));
        assert!(!filter.is_role_excluded(Role::Knight));

        filter.clear();
        assert!(!filter.is_active());
    }

    #[test]
    fn test_validrole() {
        assert!(validrole(Role::Wizard));
        assert!(validrole(Role::Knight));
        assert!(validrole(Role::Barbarian));
    }

    #[test]
    fn test_validrace() {
        assert!(validrace(Role::Knight, Race::Human));
        assert!(validrace(Role::Wizard, Race::Human));
        assert!(validrace(Role::Valkyrie, Race::Human));
    }

    #[test]
    fn test_pet_type() {
        assert_eq!(pet_type(Role::Knight), "warhorse");
        assert_eq!(pet_type(Role::Valkyrie), "warhorse");
        assert_eq!(pet_type(Role::Wizard), "little dog");
        assert_eq!(pet_type(Role::Barbarian), "orcish hound");
    }

    #[test]
    fn test_role_abil() {
        let abilities = role_abil(Role::Barbarian);
        assert!(abilities.contains(&"cleave"));

        let abilities = role_abil(Role::Monk);
        assert!(abilities.contains(&"kick") || abilities.contains(&"punch"));
    }

    #[test]
    fn test_rigid_role_checks() {
        let mut filter = RoleFilter::new();
        let mut count = 0;
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
            if !rigid_role_checks(&filter) {
                filter.exclude_role(role);
            } else {
                count += 1;
            }
        }
        // After excluding all but one, should be rigid
        assert!(rigid_role_checks(&filter));
    }

    #[test]
    fn test_pick_functions() {
        let filter = RoleFilter::new();
        let role = pick_role(Some(Race::Human), Some(Gender::Male), None, &filter);
        assert!(role.is_some());

        let race = pick_race(Some(Role::Wizard), Some(Gender::Male), None, &filter);
        assert!(race.is_some());

        let gender = pick_gend(Some(Role::Knight), Race::Human, None, &filter);
        assert!(gender.is_some());
    }

    #[test]
    fn test_filter_functions() {
        let mut filter = RoleFilter::new();
        assert!(!gotrolefilter(&filter));

        assert!(setrolefilter(&mut filter, "wizard"));
        assert!(gotrolefilter(&filter));

        clearrolefilter(&mut filter);
        assert!(!gotrolefilter(&filter));
    }

    #[test]
    fn test_role_init() {
        let mut name = String::from("testw");
        let mut role = Role::Wizard;
        let mut race = Race::Human;
        let mut gender = Gender::Male;

        let result = role_init(&mut name, &mut role, &mut race, &mut gender);
        assert!(result);
        assert_eq!(name, "test"); // Role letter stripped
    }

    #[test]
    fn test_plnamesuffix() {
        let mut name = String::from("testw");
        plnamesuffix(&mut name);
        assert_eq!(name, "test");

        let mut name = String::from("archeob");
        plnamesuffix(&mut name);
        assert_eq!(name, "archeo"); // 'b' is barbarian letter, not at end

        let mut name = String::from("knightk");
        plnamesuffix(&mut name);
        assert_eq!(name, "knight"); // 'k' stripped
    }

    #[test]
    fn test_role_selection_prolog() {
        let prolog = role_selection_prolog(0, "");
        assert!(!prolog.is_empty());
        assert!(prolog.contains("Archeologist") || prolog.contains("ancient"));

        let prolog = role_selection_prolog(12, "");
        assert!(!prolog.is_empty());
        assert!(prolog.contains("Wizard") || prolog.contains("magic"));

        let prolog = role_selection_prolog(999, "");
        assert_eq!(prolog, "Unknown role.");
    }

    #[test]
    fn test_role_menu_extra() {
        let extra = role_menu_extra(0, "");
        assert!(extra.is_some());
        assert!(extra.unwrap().contains("Quest"));

        let extra = role_menu_extra(999, "");
        assert!(extra.is_none());
    }

    #[test]
    fn test_build_plselection_prompt() {
        let prompt = build_plselection_prompt(
            Some(Role::Knight),
            Some(Race::Dwarf),
            Some(Gender::Female),
            None,
        );
        assert!(prompt.contains("Knight"));
        assert!(prompt.contains("Dwarf"));
        assert!(prompt.contains("Female"));

        let prompt = build_plselection_prompt(None, None, None, None);
        assert!(prompt.contains("random"));
    }

    #[test]
    fn test_root_plselection_prompt() {
        let prompt = root_plselection_prompt();
        assert!(prompt.contains("NetHack"));
        assert!(prompt.contains("New") || prompt.contains("new") || prompt.contains("(N)"));
    }

    #[test]
    fn test_pick_role_respects_constraints() {
        let mut filter = RoleFilter::new();
        filter.exclude_role(Role::Wizard);
        filter.exclude_role(Role::Knight);

        let role = pick_role(None, None, None, &filter);
        assert!(role.is_some());
        let r = role.unwrap();
        assert_ne!(r, Role::Wizard);
        assert_ne!(r, Role::Knight);
    }

    #[test]
    #[test]
    fn test_pet_type_all_roles() {
        let warhorse_roles = [Role::Knight, Role::Valkyrie];
        for role in warhorse_roles {
            assert_eq!(pet_type(role), "warhorse");
        }

        let dog_roles = [Role::Wizard, Role::Rogue, Role::Ranger];
        for role in dog_roles {
            assert_eq!(pet_type(role), "little dog");
        }
    }
}
