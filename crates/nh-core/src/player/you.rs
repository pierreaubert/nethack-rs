//! Main player structure (struct you from you.h)

use serde::{Deserialize, Serialize};

use super::{
    Alignment, AlignmentType, Attributes, Conduct, Gender, HungerState, PropertySet, Race, Role,
    SkillSet,
};
use crate::dungeon::DLevel;
use crate::monster::MonsterId;
use crate::{MAXULEV, NORMAL_SPEED};

/// Position on the map
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    /// Calculate distance squared to another position
    pub const fn distance_sq(&self, other: &Position) -> i32 {
        let dx = (self.x - other.x) as i32;
        let dy = (self.y - other.y) as i32;
        dx * dx + dy * dy
    }

    /// Check if adjacent (including diagonals)
    pub const fn is_adjacent(&self, other: &Position) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx <= 1 && dy <= 1 && (dx > 0 || dy > 0)
    }
}

/// The player character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct You {
    // Identity
    pub name: String,
    pub role: Role,
    pub race: Race,
    pub gender: Gender,

    // Position
    pub pos: Position,
    pub prev_pos: Position,
    pub direction: Position, // dx, dy for movement

    // Dungeon location
    pub level: DLevel,
    pub prev_level: DLevel,
    pub moved: bool,

    // Experience
    pub exp_level: i32,
    pub max_exp_level: i32,
    pub exp: u64,

    // Health
    pub hp: i32,
    pub hp_max: i32,
    pub hp_increases: Vec<i8>,

    // Magic energy
    pub energy: i32,
    pub energy_max: i32,
    pub energy_increases: Vec<i8>,

    // Combat
    pub armor_class: i8,
    pub hit_bonus: i8,
    pub damage_bonus: i8,

    // Attributes
    pub attr_current: Attributes,
    pub attr_max: Attributes,

    // Alignment
    pub alignment: Alignment,
    pub original_alignment: AlignmentType,

    // Luck
    pub luck: i8,
    pub luck_bonus: i8,

    // Hunger
    pub nutrition: i32,
    pub hunger_state: HungerState,

    // Movement
    pub movement_points: i16,

    // Properties (resistances, intrinsics, etc.)
    pub properties: PropertySet,

    // Skills
    pub skills: SkillSet,

    // Conduct
    pub conduct: Conduct,

    // Status effects
    pub confused_timeout: u16,
    pub stunned_timeout: u16,
    pub blinded_timeout: u16,
    pub sleeping_timeout: u16,
    pub hallucinating_timeout: u16,
    pub paralyzed_timeout: u16,

    // Equipment bonuses
    pub weapon_bonus: i8,

    // Wealth
    pub gold: i32,

    // Polymorph
    pub monster_num: Option<i16>,
    pub polymorph_timeout: u32,

    // Encumbrance
    pub carrying_capacity: i32,
    pub current_weight: i32,

    // Special states
    pub swallowed: bool,
    pub underwater: bool,
    pub buried: bool,

    // Monster interactions
    pub grabbed_by: Option<MonsterId>,
    pub steed: Option<MonsterId>,

    // Religion
    pub god_anger: i32,
    pub prayer_timeout: i32,

    // Turns
    pub turns_played: u64,
}

impl Default for You {
    fn default() -> Self {
        Self {
            name: String::new(),
            role: Role::default(),
            race: Race::default(),
            gender: Gender::default(),

            pos: Position::default(),
            prev_pos: Position::default(),
            direction: Position::default(),

            level: DLevel::default(),
            prev_level: DLevel::default(),
            moved: false,

            exp_level: 1,
            max_exp_level: 1,
            exp: 0,

            hp: 10,
            hp_max: 10,
            hp_increases: vec![0; MAXULEV],

            energy: 1,
            energy_max: 1,
            energy_increases: vec![0; MAXULEV],

            armor_class: 10,
            hit_bonus: 0,
            damage_bonus: 0,

            attr_current: Attributes::default(),
            attr_max: Attributes::default(),

            alignment: Alignment::default(),
            original_alignment: AlignmentType::default(),

            luck: 0,
            luck_bonus: 0,

            nutrition: 900,
            hunger_state: HungerState::NotHungry,

            movement_points: NORMAL_SPEED,

            properties: PropertySet::default(),
            skills: SkillSet::default(),
            conduct: Conduct::default(),

            confused_timeout: 0,
            stunned_timeout: 0,
            blinded_timeout: 0,
            sleeping_timeout: 0,
            hallucinating_timeout: 0,
            paralyzed_timeout: 0,

            weapon_bonus: 0,
            gold: 0,

            monster_num: None,
            polymorph_timeout: 0,

            carrying_capacity: 1000,
            current_weight: 0,

            swallowed: false,
            underwater: false,
            buried: false,

            grabbed_by: None,
            steed: None,

            god_anger: 0,
            prayer_timeout: 0,

            turns_played: 0,
        }
    }
}

impl You {
    /// Create a new player with the given identity
    pub fn new(name: String, role: Role, race: Race, gender: Gender) -> Self {
        let mut player = Self {
            name,
            role,
            race,
            gender,
            alignment: Alignment::new(role.default_alignment()),
            original_alignment: role.default_alignment(),
            ..Default::default()
        };

        // Grant racial intrinsics
        if race.has_infravision() {
            player
                .properties
                .grant_intrinsic(super::Property::Infravision);
        }

        player
    }

    /// Check if player is polymorphed
    pub const fn is_polymorphed(&self) -> bool {
        self.monster_num.is_some()
    }

    /// Check if player can move normally
    pub fn can_move(&self) -> bool {
        !self.buried
            && self.sleeping_timeout == 0
            && self.stunned_timeout == 0 // stunned can still move, just randomly
            && !matches!(self.hunger_state, HungerState::Fainted | HungerState::Starved)
    }

    /// Check if player is confused
    pub const fn is_confused(&self) -> bool {
        self.confused_timeout > 0
    }

    /// Check if player is stunned
    pub const fn is_stunned(&self) -> bool {
        self.stunned_timeout > 0
    }

    /// Check if player is blind
    pub const fn is_blind(&self) -> bool {
        self.blinded_timeout > 0
    }

    /// Get player's rank title
    pub fn rank_title(&self) -> &'static str {
        self.role.rank_title(self.exp_level, self.gender)
    }

    /// Calculate encumbrance level
    pub fn encumbrance(&self) -> Encumbrance {
        let capacity = self.carrying_capacity;
        let weight = self.current_weight;

        if weight <= capacity / 2 {
            Encumbrance::Unencumbered
        } else if weight <= (capacity * 3) / 4 {
            Encumbrance::Burdened
        } else if weight <= (capacity * 9) / 10 {
            Encumbrance::Stressed
        } else if weight <= capacity {
            Encumbrance::Strained
        } else if weight <= (capacity * 5) / 4 {
            Encumbrance::Overtaxed
        } else {
            Encumbrance::Overloaded
        }
    }

    /// Update hunger state based on nutrition
    pub fn update_hunger(&mut self) {
        self.hunger_state = HungerState::from_nutrition(self.nutrition);
    }

    /// Decrement nutrition (called each turn)
    pub fn digest(&mut self, amount: i32) {
        self.nutrition = self.nutrition.saturating_sub(amount);
        self.update_hunger();
    }

    /// Gain experience points
    pub fn gain_exp(&mut self, exp: u64) {
        self.exp = self.exp.saturating_add(exp);
        self.check_level_up();
    }

    /// Check if player should level up based on current experience
    pub fn check_level_up(&mut self) {
        use crate::{EXP_THRESHOLDS, MAXULEV};

        while (self.exp_level as usize) < MAXULEV {
            let next_level = self.exp_level as usize; // 0-indexed threshold for next level
            if next_level < EXP_THRESHOLDS.len() && self.exp >= EXP_THRESHOLDS[next_level] {
                self.exp_level += 1;
                if self.exp_level > self.max_exp_level {
                    self.max_exp_level = self.exp_level;
                    // Gain HP on level up (based on constitution)
                    let hp_gain = 1 + (self.attr_current.get(super::Attribute::Constitution) as i32 / 3);
                    self.hp_max += hp_gain;
                    self.hp += hp_gain;
                    // Gain energy on level up
                    let energy_gain = 1 + (self.attr_current.get(super::Attribute::Wisdom) as i32 / 5);
                    self.energy_max += energy_gain;
                    self.energy += energy_gain;
                }
            } else {
                break;
            }
        }
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: i32) {
        self.hp -= damage;
    }

    /// Heal damage
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// Check if player is dead
    pub const fn is_dead(&self) -> bool {
        self.hp <= 0
    }
}

/// Encumbrance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Encumbrance {
    Unencumbered,
    Burdened,
    Stressed,
    Strained,
    Overtaxed,
    Overloaded,
}

impl Encumbrance {
    /// Get movement penalty (subtracted from speed)
    pub const fn movement_penalty(&self) -> i16 {
        match self {
            Encumbrance::Unencumbered => 0,
            Encumbrance::Burdened => 1,
            Encumbrance::Stressed => 3,
            Encumbrance::Strained => 5,
            Encumbrance::Overtaxed => 7,
            Encumbrance::Overloaded => NORMAL_SPEED, // can't move
        }
    }

    /// Get status line display string
    pub const fn status_string(&self) -> Option<&'static str> {
        match self {
            Encumbrance::Unencumbered => None,
            Encumbrance::Burdened => Some("Burdened"),
            Encumbrance::Stressed => Some("Stressed"),
            Encumbrance::Strained => Some("Strained"),
            Encumbrance::Overtaxed => Some("Overtaxed"),
            Encumbrance::Overloaded => Some("Overloaded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Attribute;

    #[test]
    fn test_gain_exp_no_level_up() {
        let mut player = You::default();
        player.exp_level = 1;
        player.exp = 0;
        player.attr_current.set(Attribute::Constitution, 12);
        player.attr_current.set(Attribute::Wisdom, 10);

        // Gain 10 exp - not enough to level up (need 20 for level 2)
        player.gain_exp(10);

        assert_eq!(player.exp, 10);
        assert_eq!(player.exp_level, 1);
    }

    #[test]
    fn test_gain_exp_level_up() {
        let mut player = You::default();
        player.exp_level = 1;
        player.exp = 0;
        player.hp = 10;
        player.hp_max = 10;
        player.energy = 5;
        player.energy_max = 5;
        player.attr_current.set(Attribute::Constitution, 12);
        player.attr_current.set(Attribute::Wisdom, 10);

        // Gain 25 exp - enough to level up to 2 (need 20)
        player.gain_exp(25);

        assert_eq!(player.exp, 25);
        assert_eq!(player.exp_level, 2);
        assert_eq!(player.max_exp_level, 2);
        assert!(player.hp_max > 10, "HP should increase on level up");
        assert!(player.energy_max > 5, "Energy should increase on level up");
    }

    #[test]
    fn test_gain_exp_multiple_levels() {
        let mut player = You::default();
        player.exp_level = 1;
        player.exp = 0;
        player.hp = 10;
        player.hp_max = 10;
        player.energy = 5;
        player.energy_max = 5;
        player.attr_current.set(Attribute::Constitution, 12);
        player.attr_current.set(Attribute::Wisdom, 10);

        // Gain 100 exp - enough to level up to 4 (need 80 for level 4)
        player.gain_exp(100);

        assert_eq!(player.exp, 100);
        assert_eq!(player.exp_level, 4);
        assert_eq!(player.max_exp_level, 4);
    }
}
